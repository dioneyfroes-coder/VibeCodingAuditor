//! Análise inicial de dependências entre os módulos declarados no contrato.
//!
//! A análise é determinística e somente leitura. Ela usa `syn` para interpretar
//! declarações `mod` e `use` em Rust, e scanners conservadores para importações
//! TypeScript/JavaScript e Python. Os manifests Cargo informam os pacotes do
//! workspace. Construções que não podem ser resolvidas com segurança viram itens
//! de revisão ou limitações explícitas, nunca conclusões inventadas.

mod python;
mod typescript;

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use syn::{Item, UseTree};

use crate::contract::{Contract, Module};
use crate::discovery::{Inventory, matches_pattern};

/// Raízes da biblioteca padrão, que não são dependências arquiteturais.
const IGNORED_ROOTS: &[&str] = &["std", "core", "alloc", "test"];

/// Indica se um caractere pode fazer parte de um identificador.
pub(super) fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

/// Encontra uma palavra inteira a partir de `from`.
pub(super) fn find_word(text: &str, word: &str, from: usize) -> Option<usize> {
    let mut cursor = from;
    while let Some(offset) = text.get(cursor..)?.find(word) {
        let index = cursor + offset;
        let before = text[..index].chars().next_back();
        let after = text[index + word.len()..].chars().next();
        if before.is_none_or(|ch| !is_word_char(ch)) && after.is_none_or(|ch| !is_word_char(ch)) {
            return Some(index);
        }
        cursor = index + word.len();
    }
    None
}

/// Tamanho mínimo de uma linha para ser comparada como duplicata.
const DUPLICATE_LINE_MIN_LEN: usize = 40;

/// Indica se a linha é comentário ou separador, que não caracteriza duplicação.
fn is_duplicate_candidate(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.chars().count() < DUPLICATE_LINE_MIN_LEN {
        return false;
    }
    ![
        "//", "/*", "*", "#", "--", "\"\"\"", "import ", "from ", "use ", "export ", "require(",
    ]
    .iter()
    .any(|marker| trimmed.starts_with(marker))
}

/// Registra linhas de código candidatas a duplicação por arquivo.
fn collect_duplicate_lines(
    duplicates: &mut BTreeMap<String, BTreeSet<String>>,
    path: &str,
    text: &str,
) {
    for line in text.lines().filter(|line| is_duplicate_candidate(line)) {
        duplicates
            .entry(line.trim().to_owned())
            .or_default()
            .insert(path.to_owned());
    }
}

/// Extensões reconhecidas como código que ainda não possuem analisador nesta versão.
const UNSUPPORTED_CODE_EXTENSIONS: &[&str] = &[
    "c", "cc", "clj", "cljs", "cpp", "cs", "cxx", "dart", "erl", "ex", "exs", "fs", "fsx", "go",
    "h", "hs", "hpp", "java", "kt", "kts", "lua", "m", "mm", "php", "pl", "ps1", "pyw", "r", "rb",
    "scala", "sh", "swift", "vb",
];

/// Indica se a extensão possui analisador de dependências disponível.
pub fn is_source_file(extension: Option<&str>) -> bool {
    extension == Some("rs")
        || typescript::is_supported_extension(extension)
        || python::is_supported_extension(extension)
}

/// Indica se a extensão é código conhecido sem analisador nesta versão.
pub fn is_unsupported_code_extension(extension: Option<&str>) -> bool {
    let Some(extension) = extension else {
        return false;
    };
    let normalized = extension.to_ascii_lowercase();
    UNSUPPORTED_CODE_EXTENSIONS.contains(&normalized.as_str())
}

/// Extensão de código incluída no inventário que não possui analisador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnsupportedCodeExtension {
    /// Extensão normalizada, sem o ponto inicial (por exemplo, `go`).
    pub extension: String,
    /// Quantidade de arquivos incluídos com essa extensão.
    pub files: usize,
}

/// Cobertura da análise sobre o código incluído no inventário.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AnalysisCoverage {
    /// Arquivos de código incluídos no inventário.
    pub code_files_included: usize,
    /// Arquivos de código processados por um analisador disponível.
    pub code_files_analyzed: usize,
    /// Arquivos de código incluídos que não foram processados.
    pub code_files_not_analyzed: usize,
    /// Extensões de código sem analisador, ordenadas por extensão.
    pub unsupported_code_extensions: Vec<UnsupportedCodeExtension>,
    /// Indica que pelo menos um arquivo de código incluído não foi analisado.
    pub partial: bool,
}

/// Descreve as extensões sem analisador para mensagens e relatórios.
pub fn describe_unsupported_code_extensions(extensions: &[UnsupportedCodeExtension]) -> String {
    extensions
        .iter()
        .map(|entry| format!(".{} ({})", entry.extension, entry.files))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Trecho de código idêntico encontrado em mais de um arquivo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuplicateLine {
    /// Linha normalizada, sem espaços nas extremidades.
    pub text: String,
    /// Arquivos que contêm a linha, em ordem lexicográfica.
    pub files: Vec<String>,
}

/// Resultado da análise do projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Analysis {
    /// Nomes dos módulos declarados no contrato, na ordem do contrato.
    pub modules: Vec<String>,
    /// Pacotes encontrados nos manifests Cargo do workspace.
    pub workspace: Option<WorkspaceInfo>,
    /// Arestas direcionais entre módulos, com origem verificável.
    pub edges: Vec<DependencyEdge>,
    /// Arquivos ou dependências que exigem decisão humana.
    pub reviews: Vec<ReviewItem>,
    /// Declarações `mod` encontradas nos arquivos analisados.
    pub module_declarations: Vec<ModuleDeclaration>,
    /// Construções ou arquivos que a primeira versão não conseguiu analisar.
    pub limitations: Vec<String>,
    /// Linhas idênticas em mais de um arquivo de código incluído.
    pub duplicate_lines: Vec<DuplicateLine>,
    /// Cobertura efetiva da análise sobre o código incluído no inventário.
    pub coverage: AnalysisCoverage,
}

/// Pacotes descobertos no workspace Cargo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceInfo {
    /// Pacotes ordenados por nome e caminho de manifesto.
    pub packages: Vec<WorkspacePackage>,
    /// Crates externas declaradas nos manifests, normalizadas como identificadores Rust.
    pub declared_dependencies: Vec<String>,
}

/// Pacote identificado em um `Cargo.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspacePackage {
    /// Nome declarado em `[package].name`.
    pub name: String,
    /// Manifesto relativo à raiz do projeto auditado.
    pub manifest_path: String,
}

/// Classificação de uma aresta de dependência.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    /// O destino está em `allowed_dependencies`.
    Allowed,
    /// O destino é um módulo do contrato, mas não está em `allowed_dependencies`.
    Forbidden,
    /// O destino não pôde ser associado a um módulo do contrato.
    Unresolved,
}

/// Aresta direcional entre dois módulos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyEdge {
    /// Módulo de origem, conforme o contrato.
    pub from: String,
    /// Nome do destino encontrado na importação.
    pub to: String,
    /// Classificação determinística da aresta.
    pub kind: DependencyKind,
    /// Arquivo de origem, relativo à raiz.
    pub path: String,
    /// Linha da declaração que originou a aresta, quando localizada.
    pub line: Option<usize>,
    /// Texto da linha de código usado como evidência.
    pub evidence: Option<String>,
    /// Explicação curta da classificação.
    pub detail: String,
}

/// Item que o auditor não consegue classificar sozinho.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewItem {
    /// Arquivo que exige decisão humana.
    pub path: String,
    /// Linha relacionada ao item, quando houver.
    pub line: Option<usize>,
    /// Motivo da revisão.
    pub reason: ReviewReason,
    /// Explicação do que não pôde ser concluído.
    pub detail: String,
}

/// Motivo da revisão necessária.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewReason {
    /// Arquivo incluído na varredura, mas fora de todos os módulos do contrato.
    UnmappedFile,
    /// Importação cujo destino não corresponde a um módulo declarado.
    UnresolvedDependency,
    /// Arquivo que corresponde a mais de um módulo do contrato.
    AmbiguousModule,
}

/// Declaração `mod` localizada em um arquivo Rust.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleDeclaration {
    /// Módulo do contrato ao qual o arquivo de origem pertence, quando mapeado.
    pub module_name: Option<String>,
    /// Nome da declaração `mod`.
    pub name: String,
    /// Arquivo de origem, relativo à raiz.
    pub path: String,
    /// Linha da declaração, quando localizada.
    pub line: Option<usize>,
    /// Indica se a declaração abre um bloco `mod { ... }`.
    pub inline: bool,
}

/// Erros operacionais da análise.
#[derive(Debug)]
pub enum AnalysisError {
    /// Falha ao ler um manifesto ou arquivo Rust.
    Io {
        /// Caminho que não pôde ser lido.
        path: PathBuf,
        /// Erro de E/S original.
        source: std::io::Error,
    },
    /// Manifesto Cargo com sintaxe inválida.
    Toml {
        /// Caminho do manifesto inválido.
        path: PathBuf,
        /// Erro de parsing original.
        source: toml::de::Error,
    },
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    formatter,
                    "não foi possível ler {}: {source}",
                    path.display()
                )
            }
            Self::Toml { path, source } => {
                write!(formatter, "TOML inválido em {}: {source}", path.display())
            }
        }
    }
}

impl Error for AnalysisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Toml { source, .. } => Some(source),
        }
    }
}

/// Destino resolvido de uma dependência `path` declarada no Cargo.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalDependencyTarget {
    /// O pacote local foi associado sem ambiguidade a um módulo do contrato.
    Module(String),
    /// Não há evidência suficiente; a importação exige revisão explícita.
    NeedsReview(String),
}

/// Dependência Cargo declarada com `path`, antes da resolução para o contrato.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclaredLocalDependency {
    /// Nome usado no código, já normalizado como identificador Rust.
    alias: String,
    /// Nome canônico do pacote declarado, já normalizado.
    package: String,
    /// Diretório do manifesto que declarou o caminho.
    manifest_dir: PathBuf,
    /// Caminho relativo informado em `path`.
    path: String,
}

/// Estado compartilhado durante a visita de uma única unidade Rust.
struct VisitContext<'a> {
    path: &'a str,
    contract_module: Option<&'a Module>,
    lines: &'a [&'a str],
    use_cursor: &'a mut usize,
    mod_cursor: &'a mut usize,
    contract: &'a Contract,
    contract_names: &'a HashSet<String>,
    workspace_names: &'a HashSet<String>,
    declared_dependencies: &'a HashSet<String>,
    local_dependencies: &'a BTreeMap<String, LocalDependencyTarget>,
    inventory_paths: &'a BTreeSet<String>,
    analysis: &'a mut Analysis,
    seen_edges: &'a mut HashSet<String>,
}

/// Analisa os arquivos Rust do inventário e produz o grafo inicial.
pub fn analyze(
    root: &Path,
    contract: &Contract,
    inventory: &Inventory,
) -> Result<Analysis, AnalysisError> {
    let mut workspace_limitations = Vec::new();
    let (workspace, declared_local_dependencies) =
        read_cargo_workspace(root, &mut workspace_limitations)?;
    let workspace_names: HashSet<String> = workspace
        .as_ref()
        .map(|info| {
            info.packages
                .iter()
                .map(|package| package.name.clone())
                .collect()
        })
        .unwrap_or_default();
    let declared_dependencies: HashSet<String> = workspace
        .as_ref()
        .map(|info| info.declared_dependencies.iter().cloned().collect())
        .unwrap_or_default();
    let local_dependencies =
        resolve_local_dependencies(root, contract, &declared_local_dependencies, &workspace);
    let inventory_paths: BTreeSet<String> = inventory
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect();
    let contract_names: HashSet<String> = contract
        .architecture
        .modules
        .iter()
        .map(|module| module.name.clone())
        .collect();

    let mut analysis = Analysis {
        modules: contract
            .architecture
            .modules
            .iter()
            .map(|module| module.name.clone())
            .collect(),
        workspace,
        edges: Vec::new(),
        reviews: Vec::new(),
        module_declarations: Vec::new(),
        limitations: workspace_limitations,
        duplicate_lines: Vec::new(),
        coverage: AnalysisCoverage::default(),
    };

    let mut seen_edges = HashSet::new();
    let mut duplicate_candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut unsupported_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut coverage = AnalysisCoverage::default();

    let mut typescript_files = false;
    let mut python_files = false;

    for file in &inventory.files {
        let extension = file.extension.as_deref();
        if !is_source_file(extension) {
            if let Some(normalized) = extension
                .map(str::to_ascii_lowercase)
                .filter(|value| UNSUPPORTED_CODE_EXTENSIONS.contains(&value.as_str()))
            {
                *unsupported_counts.entry(normalized).or_default() += 1;
                coverage.code_files_not_analyzed += 1;
            }
            continue;
        }
        if typescript::is_supported_extension(extension) {
            typescript_files = true;
        } else if python::is_supported_extension(extension) {
            python_files = true;
        }

        let full_path = root.join(&file.path);
        let text = match fs::read_to_string(&full_path) {
            Ok(text) => text,
            Err(source) => {
                analysis
                    .limitations
                    .push(format!("{} não pôde ser lido: {source}", file.path));
                coverage.code_files_not_analyzed += 1;
                continue;
            }
        };

        collect_duplicate_lines(&mut duplicate_candidates, &file.path, &text);
        let module = module_for_file(contract, &file.path, &mut analysis);

        if extension == Some("rs") {
            let parsed = match syn::parse_file(&text) {
                Ok(parsed) => parsed,
                Err(source) => {
                    analysis
                        .limitations
                        .push(format!("{} não pôde ser analisado: {source}", file.path));
                    coverage.code_files_not_analyzed += 1;
                    continue;
                }
            };

            let lines: Vec<&str> = text.lines().collect();
            let mut use_cursor = 0;
            let mut mod_cursor = 0;

            let mut context = VisitContext {
                path: &file.path,
                contract_module: module,
                lines: &lines,
                use_cursor: &mut use_cursor,
                mod_cursor: &mut mod_cursor,
                contract,
                contract_names: &contract_names,
                workspace_names: &workspace_names,
                declared_dependencies: &declared_dependencies,
                local_dependencies: &local_dependencies,
                inventory_paths: &inventory_paths,
                analysis: &mut analysis,
                seen_edges: &mut seen_edges,
            };
            visit_items(&parsed.items, &mut context);
        } else if typescript::is_supported_extension(extension) {
            typescript::analyze_text(
                contract,
                &contract_names,
                inventory,
                &file.path,
                &text,
                module,
                &mut analysis,
                &mut seen_edges,
            );
        } else {
            python::analyze_text(
                contract,
                &contract_names,
                inventory,
                &file.path,
                &text,
                module,
                &mut analysis,
                &mut seen_edges,
            );
        }
        coverage.code_files_analyzed += 1;
    }

    coverage.code_files_included = coverage.code_files_analyzed + coverage.code_files_not_analyzed;
    coverage.unsupported_code_extensions = unsupported_counts
        .into_iter()
        .map(|(extension, files)| UnsupportedCodeExtension { extension, files })
        .collect();
    coverage.partial = coverage.code_files_not_analyzed > 0;
    if !coverage.unsupported_code_extensions.is_empty() {
        analysis.limitations.push(format!(
            "cobertura parcial: {} arquivo(s) de código incluído(s) não foram analisados; extensões sem analisador observadas: {}",
            coverage.code_files_not_analyzed,
            describe_unsupported_code_extensions(&coverage.unsupported_code_extensions)
        ));
    }
    analysis.coverage = coverage;

    if typescript_files {
        analysis.limitations.push(
            "pacotes npm não são enumerados: o contador de pacotes refere-se apenas ao Cargo"
                .to_owned(),
        );
    }
    if python_files {
        analysis.limitations.push(
            "pacotes pip não são enumerados: o contador de pacotes refere-se apenas ao Cargo"
                .to_owned(),
        );
    }

    analysis.duplicate_lines = duplicate_candidates
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(text, files)| DuplicateLine {
            text,
            files: files.into_iter().collect(),
        })
        .collect();

    analysis.edges.sort_by(|left, right| {
        (&left.from, &left.to, &left.path).cmp(&(&right.from, &right.to, &right.path))
    });
    analysis
        .reviews
        .sort_by(|left, right| (&left.path, left.line).cmp(&(&right.path, right.line)));
    analysis.module_declarations.sort_by(|left, right| {
        (&left.path, left.line, &left.name).cmp(&(&right.path, right.line, &right.name))
    });
    analysis.limitations.sort();

    Ok(analysis)
}

/// Visita itens Rust na ordem do código, registrando `mod` e classificando `use`.
fn visit_items(items: &[Item], context: &mut VisitContext<'_>) {
    for item in items {
        match item {
            Item::Use(item_use) => {
                let (line, evidence) = match locate_source_line(
                    context.lines,
                    context.use_cursor,
                    is_use_declaration_line,
                ) {
                    Some(located) => (Some(located.0), Some(located.1)),
                    None => {
                        context.analysis.limitations.push(format!(
                            "não foi possível localizar a linha de um `use` em {}",
                            context.path
                        ));
                        (None, None)
                    }
                };

                let mut use_path_leaves = Vec::new();
                collect_use_path_leaves(&item_use.tree, &mut Vec::new(), &mut use_path_leaves);

                if context.contract_module.is_some() {
                    for leaf in use_path_leaves {
                        classify_use(context, &leaf, line, evidence.as_deref());
                    }
                }
            }
            Item::Mod(item_mod) => {
                let (line, _) = match locate_source_line(
                    context.lines,
                    context.mod_cursor,
                    is_mod_declaration_line,
                ) {
                    Some(located) => (Some(located.0), located.1),
                    None => {
                        context.analysis.limitations.push(format!(
                            "não foi possível localizar a linha de um `mod` em {}",
                            context.path
                        ));
                        (None, String::new())
                    }
                };

                context
                    .analysis
                    .module_declarations
                    .push(ModuleDeclaration {
                        module_name: context.contract_module.map(|module| module.name.clone()),
                        name: item_mod.ident.to_string(),
                        path: context.path.to_owned(),
                        line,
                        inline: item_mod.content.is_some(),
                    });

                if let Some((_, inner_items)) = &item_mod.content {
                    visit_items(inner_items, context);
                }
            }
            _ => {}
        }
    }
}

/// Módulos do contrato cujo `path` corresponde ao caminho informado.
fn modules_for_path<'a>(contract: &'a Contract, path: &str) -> Vec<&'a Module> {
    contract
        .architecture
        .modules
        .iter()
        .filter(|module| matches_pattern(&module.path, path))
        .collect()
}

/// Associa um arquivo a um módulo do contrato e registra revisões de mapeamento.
pub(crate) fn module_for_file<'a>(
    contract: &'a Contract,
    path: &str,
    analysis: &mut Analysis,
) -> Option<&'a Module> {
    let matching_modules = modules_for_path(contract, path);

    match matching_modules.len() {
        0 => {
            analysis.reviews.push(ReviewItem {
                path: path.to_owned(),
                line: None,
                reason: ReviewReason::UnmappedFile,
                detail: "nenhum path de módulo do contrato corresponde ao arquivo".to_owned(),
            });
            None
        }
        1 => matching_modules.first().copied(),
        _ => {
            let mut candidates: Vec<&str> = matching_modules
                .iter()
                .map(|module| module.name.as_str())
                .collect();
            candidates.sort_unstable();
            analysis.reviews.push(ReviewItem {
                path: path.to_owned(),
                line: None,
                reason: ReviewReason::AmbiguousModule,
                detail: format!(
                    "o arquivo corresponde a mais de um módulo do contrato: {}",
                    candidates.join(", ")
                ),
            });
            None
        }
    }
}

/// Registra uma aresta já classificada, sem duplicatas nem revisões repetidas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_dependency(
    module: &Module,
    target: &str,
    kind: DependencyKind,
    detail: String,
    line: Option<usize>,
    evidence: Option<&str>,
    path: &str,
    analysis: &mut Analysis,
    seen_edges: &mut HashSet<String>,
) {
    if kind == DependencyKind::Unresolved {
        analysis.reviews.push(ReviewItem {
            path: path.to_owned(),
            line,
            reason: ReviewReason::UnresolvedDependency,
            detail: format!("{target}: {detail}"),
        });
    }

    let edge_key = format!("{}\0{target}\0{kind:?}\0{path}\0{line:?}", module.name);
    if !seen_edges.insert(edge_key) {
        return;
    }

    analysis.edges.push(DependencyEdge {
        from: module.name.clone(),
        to: target.to_owned(),
        kind,
        path: path.to_owned(),
        line,
        evidence: evidence.map(str::to_owned),
        detail,
    });
}

/// Resultado da resolução de um caminho relativo Rust.
enum RelativeTarget {
    /// O destino pertence ao próprio módulo e não cruza uma fronteira.
    Internal,
    /// O destino foi associado a um módulo do contrato.
    Module(String),
    /// Não há evidência suficiente; o destino precisa de revisão.
    Unknown {
        /// Nome usado como destino na aresta.
        target: String,
        /// Explicação da falta de evidência.
        detail: String,
    },
}

/// Resultado da resolução de um caminho absoluto de crate (`crate::`).
enum CrateTarget {
    /// O destino pertence ao próprio módulo e não cruza uma fronteira.
    Internal,
    /// O destino foi associado a um módulo do contrato.
    Module(String),
    /// O destino existe, mas não pôde ser associado com segurança.
    Unknown {
        /// Nome usado como destino na aresta.
        target: String,
        /// Explicação da falta de evidência.
        detail: String,
    },
    /// Nenhum arquivo do inventário corresponde ao caminho; vale a regra anterior.
    NotFound,
}

/// Classifica uma dependência local do workspace, quando declarada com `path`.
///
/// Retorna `true` quando o destino foi reconhecido como dependência local; nesse
/// caso a classificação externa habitual não deve ser aplicada.
fn classify_local_dependency(
    context: &mut VisitContext<'_>,
    module: &Module,
    target: &str,
    line: Option<usize>,
    evidence: Option<&str>,
) -> bool {
    let Some(resolved) = context.local_dependencies.get(target).cloned() else {
        return false;
    };
    match resolved {
        LocalDependencyTarget::Module(name) => record_classified_target(
            context,
            module,
            &name,
            line,
            evidence,
            "dependência local do workspace associada ao módulo do contrato".to_owned(),
        ),
        LocalDependencyTarget::NeedsReview(detail) => record_dependency(
            module,
            target,
            DependencyKind::Unresolved,
            detail,
            line,
            evidence,
            context.path,
            &mut *context.analysis,
            &mut *context.seen_edges,
        ),
    }
    true
}

/// Classifica uma folha de caminho de `use` e registra a aresta correspondente.
fn classify_use(
    context: &mut VisitContext<'_>,
    leaf: &[String],
    line: Option<usize>,
    evidence: Option<&str>,
) {
    let Some(module) = context.contract_module else {
        return;
    };
    let Some(first) = leaf.first() else {
        return;
    };

    let target = if first == "crate" {
        if leaf.len() < 2 {
            return;
        }
        if leaf[1] == "self" || leaf[1] == "super" {
            classify_relative(context, &leaf[1..], line, evidence, module);
            return;
        }
        match resolve_crate_target(context, &leaf[1..], module) {
            CrateTarget::Internal => return,
            CrateTarget::Module(name) => {
                record_classified_target(
                    context,
                    module,
                    &name,
                    line,
                    evidence,
                    "destino do caminho `crate` associado ao módulo".to_owned(),
                );
                return;
            }
            CrateTarget::Unknown { target, detail } => {
                record_classified_target(context, module, &target, line, evidence, detail);
                return;
            }
            CrateTarget::NotFound => leaf[1].clone(),
        }
    } else if first == "self" || first == "super" {
        classify_relative(context, leaf, line, evidence, module);
        return;
    } else {
        first.clone()
    };

    if IGNORED_ROOTS.contains(&target.as_str()) || target == module.name {
        return;
    }
    if classify_local_dependency(context, module, &target, line, evidence) {
        return;
    }
    if context.declared_dependencies.contains(&target) {
        return;
    }

    let unresolved_detail = if context.workspace_names.contains(&target) {
        "pacote do workspace sem módulo correspondente no contrato".to_owned()
    } else {
        "destino não corresponde a nenhum módulo do contrato".to_owned()
    };
    record_classified_target(context, module, &target, line, evidence, unresolved_detail);
}

/// Registra a classificação final de um destino, com a explicação de quando não resolveu.
fn record_classified_target(
    context: &mut VisitContext<'_>,
    module: &Module,
    target: &str,
    line: Option<usize>,
    evidence: Option<&str>,
    unresolved_detail: String,
) {
    let (kind, detail) = if module
        .allowed_dependencies
        .iter()
        .any(|allowed| allowed == target)
    {
        (
            DependencyKind::Allowed,
            "destino presente em allowed_dependencies".to_owned(),
        )
    } else if context.contract_names.contains(target) {
        (
            DependencyKind::Forbidden,
            "destino declarado no contrato, mas ausente de allowed_dependencies".to_owned(),
        )
    } else {
        (DependencyKind::Unresolved, unresolved_detail)
    };

    record_dependency(
        module,
        target,
        kind,
        detail,
        line,
        evidence,
        context.path,
        &mut *context.analysis,
        &mut *context.seen_edges,
    );
}

/// Classifica `self::`/`super::` a partir do arquivo de origem e do contrato.
fn classify_relative(
    context: &mut VisitContext<'_>,
    leaf: &[String],
    line: Option<usize>,
    evidence: Option<&str>,
    module: &Module,
) {
    let mut prefix = 0;
    let mut super_count = 0;
    while let Some(segment) = leaf.get(prefix) {
        if segment == "super" {
            super_count += 1;
            prefix += 1;
        } else if segment == "self" {
            prefix += 1;
        } else {
            break;
        }
    }
    let Some(segment) = leaf.get(prefix) else {
        return;
    };

    if segment == &module.name || IGNORED_ROOTS.contains(&segment.as_str()) {
        return;
    }
    if classify_local_dependency(context, module, segment, line, evidence) {
        return;
    }
    if context.declared_dependencies.contains(segment) {
        return;
    }

    let (target, unresolved_detail) = if context.contract_names.contains(segment) {
        (segment.clone(), "destino do caminho relativo".to_owned())
    } else {
        match resolve_relative_target(context, segment, super_count, module) {
            RelativeTarget::Internal => return,
            RelativeTarget::Module(name) => (
                name,
                "destino do caminho relativo associado ao módulo".to_owned(),
            ),
            RelativeTarget::Unknown { target, detail } => (target, detail),
        }
    };

    record_classified_target(context, module, &target, line, evidence, unresolved_detail);
}

/// Junta um diretório relativo e um nome de arquivo usando `/` como separador.
fn join_relative(dir: &Path, name: &str) -> String {
    dir.join(name).to_string_lossy().replace('\\', "/")
}

/// Localiza o arquivo `lib.rs`/`mod.rs` que representa um diretório de módulo.
fn module_file_in(context: &VisitContext<'_>, dir: &Path) -> Option<String> {
    ["lib.rs", "mod.rs"]
        .into_iter()
        .map(|name| join_relative(dir, name))
        .find(|candidate| context.inventory_paths.contains(candidate))
}

/// Infere a raiz de fontes do crate a partir do componente `src` do caminho auditado.
fn crate_source_root(path: &str) -> Option<PathBuf> {
    let mut root = PathBuf::new();
    for component in Path::new(path).components() {
        root.push(component.as_os_str());
        if component.as_os_str().to_str() == Some("src") {
            return Some(root);
        }
    }
    None
}

/// Procura um caminho `crate::` no inventário e no contrato.
///
/// Testa os prefixos mais longos primeiro para que `crate::a::b::Item` encontre
/// `src/a/b.rs` antes de `src/a/mod.rs`. Sem arquivo correspondente, o destino
/// continua sendo classificado pelo primeiro segmento, como antes.
fn resolve_crate_target(
    context: &VisitContext<'_>,
    segments: &[String],
    module: &Module,
) -> CrateTarget {
    let Some(root) = crate_source_root(context.path) else {
        return CrateTarget::NotFound;
    };

    let mut modules = BTreeSet::new();
    let mut matched = false;
    for count in (1..=segments.len()).rev() {
        let stem = segments[..count].join("/");
        let candidates = [
            join_relative(&root, &format!("{stem}.rs")),
            join_relative(&root, &format!("{stem}/mod.rs")),
            join_relative(&root, &format!("{stem}/lib.rs")),
        ];
        for candidate in &candidates {
            if context.inventory_paths.contains(candidate) {
                matched = true;
                for found in modules_for_path(context.contract, candidate) {
                    modules.insert(found.name.clone());
                }
            }
        }
        if matched {
            break;
        }
    }

    let first = segments
        .first()
        .cloned()
        .unwrap_or_else(|| "crate".to_owned());
    if modules.len() == 1 {
        let name = modules.into_iter().next().expect("single module");
        if name == module.name {
            CrateTarget::Internal
        } else {
            CrateTarget::Module(name)
        }
    } else if modules.len() > 1 {
        let names: Vec<String> = modules.into_iter().collect();
        let detail = format!(
            "crate::{first}: corresponde a mais de um módulo do contrato: {}",
            names.join(", ")
        );
        CrateTarget::Unknown {
            target: first,
            detail,
        }
    } else if matched {
        CrateTarget::Unknown {
            detail: format!("crate::{first}: não pôde ser associado a um módulo do contrato"),
            target: first,
        }
    } else {
        CrateTarget::NotFound
    }
}

/// Procura o módulo de destino de um caminho relativo no inventário e no contrato.
fn resolve_relative_target(
    context: &VisitContext<'_>,
    segment: &str,
    super_count: usize,
    module: &Module,
) -> RelativeTarget {
    let source_dir = Path::new(context.path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let stem = Path::new(context.path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let ancestor_dir = if stem == "mod" || stem == "lib" {
        source_dir.parent().unwrap_or(source_dir).to_path_buf()
    } else {
        source_dir.to_path_buf()
    };
    let mut base = if super_count == 0 {
        source_dir.to_path_buf()
    } else {
        ancestor_dir.clone()
    };
    for _ in 1..super_count {
        if let Some(parent) = base.parent() {
            base = parent.to_path_buf();
        }
    }

    let prefix = if super_count == 0 {
        "self::".to_owned()
    } else {
        "super::".repeat(super_count)
    };

    let candidates = [
        join_relative(&base, &format!("{segment}.rs")),
        join_relative(&base, &format!("{segment}/mod.rs")),
        join_relative(&base, &format!("{segment}/lib.rs")),
    ];
    let mut modules = BTreeSet::new();
    for candidate in &candidates {
        if context.inventory_paths.contains(candidate) {
            for matched in modules_for_path(context.contract, candidate) {
                modules.insert(matched.name.clone());
            }
        }
    }

    if modules.len() == 1 {
        let name = modules.into_iter().next().expect("single module");
        return if name == module.name {
            RelativeTarget::Internal
        } else {
            RelativeTarget::Module(name)
        };
    }
    if modules.len() > 1 {
        let names: Vec<String> = modules.into_iter().collect();
        return RelativeTarget::Unknown {
            target: segment.to_owned(),
            detail: format!(
                "{prefix}{segment}: corresponde a mais de um módulo do contrato: {}",
                names.join(", ")
            ),
        };
    }

    if super_count == 0 {
        return RelativeTarget::Internal;
    }
    if let Some(file) = module_file_in(context, &ancestor_dir) {
        let ancestors = modules_for_path(context.contract, &file);
        if ancestors.len() == 1 && ancestors[0].name == module.name {
            return RelativeTarget::Internal;
        }
    }

    RelativeTarget::Unknown {
        target: segment.to_owned(),
        detail: format!("{prefix}{segment}: não pôde ser associado a um módulo do contrato"),
    }
}

/// Coleta os caminhos completos das folhas de um `use`, incluindo grupos e renomes.
fn collect_use_path_leaves(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    leaves: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_path_leaves(&path.tree, prefix, leaves);
            prefix.pop();
        }
        UseTree::Name(name) => {
            prefix.push(name.ident.to_string());
            leaves.push(prefix.clone());
            prefix.pop();
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            leaves.push(prefix.clone());
            prefix.pop();
        }
        UseTree::Glob(_) => leaves.push(prefix.clone()),
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use_path_leaves(tree, prefix, leaves);
            }
        }
    }
}

/// Localiza a próxima linha que satisfaz o predicate e devolve número e texto.
fn locate_source_line(
    lines: &[&str],
    cursor: &mut usize,
    predicate: fn(&str) -> bool,
) -> Option<(usize, String)> {
    for (index, line) in lines.iter().enumerate().skip(*cursor) {
        if predicate(line) {
            *cursor = index + 1;
            return Some((index + 1, line.trim().to_owned()));
        }
    }
    None
}

/// Reconhece linhas de declaração `use`, inclusive indentadas e `pub use`.
fn is_use_declaration_line(line: &str) -> bool {
    let rest = line.trim_start();
    let rest = rest
        .strip_prefix("pub")
        .map(str::trim_start)
        .unwrap_or(rest);
    rest.starts_with("use ") || rest.starts_with("use{")
}

/// Reconhece linhas de declaração `mod`, inclusive indentadas e `pub mod`.
fn is_mod_declaration_line(line: &str) -> bool {
    let rest = line.trim_start();
    let rest = rest
        .strip_prefix("pub")
        .map(str::trim_start)
        .unwrap_or(rest);
    rest.starts_with("mod ") || rest.starts_with("mod{")
}

/// Converte um nome de pacote Cargo no identificador usado por `use`.
fn normalize_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

/// Nome canônico do pacote declarado por uma entrada de dependência.
fn declared_dependency_name(
    key: &str,
    entry: &toml::Value,
    workspace_dependencies: Option<&toml::Value>,
) -> String {
    entry
        .get("package")
        .and_then(toml::Value::as_str)
        .or_else(|| {
            if entry.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                workspace_dependencies
                    .and_then(|table| table.get(key))
                    .and_then(|inherited| inherited.get("package"))
                    .and_then(toml::Value::as_str)
            } else {
                None
            }
        })
        .unwrap_or(key)
        .to_owned()
}

/// Caminho `path` herdado de `[workspace.dependencies]`.
fn inherited_dependency_path<'a>(
    key: &str,
    entry: &'a toml::Value,
    workspace_dependencies: Option<&'a toml::Value>,
) -> Option<&'a str> {
    if entry.get("workspace").and_then(toml::Value::as_bool) != Some(true) {
        return None;
    }
    workspace_dependencies
        .and_then(|table| table.get(key))
        .and_then(|inherited| inherited.get("path"))
        .and_then(toml::Value::as_str)
}

/// Coleta os nomes e as dependências `path` de uma tabela de dependências.
fn collect_dependency_table(
    table: Option<&toml::Value>,
    workspace_dependencies: Option<&toml::Value>,
    names: &mut BTreeSet<String>,
    manifest_dir: &Path,
    workspace_dir: &Path,
    local_dependencies: &mut Vec<DeclaredLocalDependency>,
) {
    let Some(entries) = table.and_then(toml::Value::as_table) else {
        return;
    };
    for (key, entry) in entries {
        let declared = declared_dependency_name(key, entry, workspace_dependencies);
        for name in [normalize_crate_name(key), normalize_crate_name(&declared)] {
            names.insert(name);
        }
        let (path, base_dir) = match entry.get("path").and_then(toml::Value::as_str) {
            Some(path) => (Some(path), manifest_dir),
            None => (
                inherited_dependency_path(key, entry, workspace_dependencies),
                workspace_dir,
            ),
        };
        if let Some(path) = path {
            local_dependencies.push(DeclaredLocalDependency {
                alias: normalize_crate_name(key),
                package: normalize_crate_name(&declared),
                manifest_dir: base_dir.to_path_buf(),
                path: path.to_owned(),
            });
        }
    }
}

/// Coleta as tabelas de dependências relevantes de um manifesto.
fn collect_manifest_dependencies(
    document: &toml::Value,
    workspace_dependencies: Option<&toml::Value>,
    names: &mut BTreeSet<String>,
    manifest_dir: &Path,
    workspace_dir: &Path,
    local_dependencies: &mut Vec<DeclaredLocalDependency>,
) {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        collect_dependency_table(
            document.get(section),
            workspace_dependencies,
            names,
            manifest_dir,
            workspace_dir,
            local_dependencies,
        );
    }
    if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                collect_dependency_table(
                    target.get(section),
                    workspace_dependencies,
                    names,
                    manifest_dir,
                    workspace_dir,
                    local_dependencies,
                );
            }
        }
    }
}

/// Indica se um caminho de membro do workspace contém curinga.
fn is_member_pattern(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

/// Expande um membro do workspace, registrando limitação para globs não suportados.
fn expand_member_pattern(
    root: &Path,
    pattern: &str,
    excludes: &[String],
    limitations: &mut Vec<String>,
) -> Vec<String> {
    let trimmed = pattern.trim_matches('/');
    let components: Vec<&str> = trimmed.split('/').filter(|part| !part.is_empty()).collect();
    let Some(last) = components.last() else {
        return Vec::new();
    };

    let unsupported = || {
        format!(
            "padrão de membro do workspace '{pattern}' não é suportado; use caminhos explícitos ou '*' no último componente"
        )
    };
    let no_matches =
        |reason: &str| format!("padrão de membro do workspace '{pattern}' não encontrou {reason}");

    if !last.contains('*') {
        if is_member_pattern(trimmed) {
            limitations.push(unsupported());
            return Vec::new();
        }
        return vec![trimmed.to_owned()];
    }

    let prefix_parts = &components[..components.len() - 1];
    let star_count = last.chars().filter(|character| *character == '*').count();
    let other_wildcards = last
        .chars()
        .any(|character| matches!(character, '?' | '[' | ']'));
    if prefix_parts.is_empty()
        || prefix_parts
            .iter()
            .any(|part| is_member_pattern(part) || matches!(*part, "." | ".."))
        || star_count != 1
        || other_wildcards
    {
        limitations.push(unsupported());
        return Vec::new();
    }

    let prefix = prefix_parts.join("/");
    let prefix_path = root.join(&prefix);
    let Ok(prefix_metadata) = fs::symlink_metadata(&prefix_path) else {
        limitations.push(no_matches("nenhum diretório com Cargo.toml"));
        return Vec::new();
    };
    if !prefix_metadata.is_dir() || prefix_metadata.file_type().is_symlink() {
        limitations.push(no_matches("nenhum diretório com Cargo.toml"));
        return Vec::new();
    }
    let entries = match fs::read_dir(&prefix_path) {
        Ok(entries) => entries,
        Err(_) => {
            limitations.push(no_matches("nenhum diretório com Cargo.toml"));
            return Vec::new();
        }
    };

    let mut members = Vec::new();
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() || !kind.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !matches_pattern(last, &name) {
            continue;
        }
        let member = format!("{prefix}/{name}");
        if root.join(&member).join("Cargo.toml").is_file() {
            members.push(member);
        }
    }
    members.sort();
    members.dedup();

    let before_excludes = members.len();
    members.retain(|member| {
        !excludes
            .iter()
            .any(|exclude| matches_pattern(exclude, member))
    });
    if members.is_empty() {
        let reason = if before_excludes == 0 {
            "nenhum diretório com Cargo.toml"
        } else {
            "membros após aplicar workspace.exclude"
        };
        limitations.push(no_matches(reason));
    }
    members
}

/// Resolve lexicalmente um caminho `path` sem seguir links nem sair da raiz.
fn resolve_dependency_directory(root: &Path, manifest_dir: &Path, path: &str) -> Option<PathBuf> {
    let mut resolved = manifest_dir.to_path_buf();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            Component::Normal(part) => resolved.push(part),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    resolved.starts_with(root).then_some(resolved)
}

/// Prefixo literal de um padrão de caminho do contrato.
fn literal_path_prefix(pattern: &str) -> String {
    pattern
        .split('/')
        .take_while(|part| !part.contains('*') && !part.contains('?') && !part.contains('['))
        .collect::<Vec<_>>()
        .join("/")
}

/// Indica se os arquivos de um módulo ficam dentro do diretório do pacote local.
fn module_is_inside_package(module_path: &str, package_dir: &str) -> bool {
    let prefix = literal_path_prefix(module_path);
    prefix == package_dir || prefix.starts_with(&format!("{package_dir}/"))
}

/// Associa uma dependência `path` a um módulo do contrato, quando possível.
fn resolve_one_local_dependency(
    root: &Path,
    contract: &Contract,
    dependency: &DeclaredLocalDependency,
    workspace: &WorkspaceInfo,
) -> LocalDependencyTarget {
    let Some(directory) =
        resolve_dependency_directory(root, &dependency.manifest_dir, &dependency.path)
    else {
        return LocalDependencyTarget::NeedsReview(format!(
            "dependência path '{}' não pôde ser resolvida dentro da raiz",
            dependency.path
        ));
    };
    let relative = directory
        .strip_prefix(root)
        .expect("caminho verificado dentro da raiz")
        .to_string_lossy()
        .replace('\\', "/");
    let manifest_path = format!("{relative}/Cargo.toml");
    let Some(package) = workspace
        .packages
        .iter()
        .find(|candidate| candidate.manifest_path == manifest_path)
    else {
        return LocalDependencyTarget::NeedsReview(format!(
            "dependência path '{}' não corresponde a um membro do workspace",
            dependency.path
        ));
    };

    let mut candidates: Vec<&str> = contract
        .architecture
        .modules
        .iter()
        .filter(|module| module_is_inside_package(&module.path, &relative))
        .map(|module| module.name.as_str())
        .collect();
    candidates.sort_unstable();
    candidates.dedup();

    match candidates.as_slice() {
        [name] => LocalDependencyTarget::Module((*name).to_owned()),
        [] => LocalDependencyTarget::NeedsReview(format!(
            "pacote local '{}' não tem módulo correspondente no contrato",
            package.name
        )),
        names => LocalDependencyTarget::NeedsReview(format!(
            "pacote local '{}' corresponde a múltiplos módulos do contrato: {}",
            package.name,
            names.join(", ")
        )),
    }
}

/// Registra um destino local, transformando conflitos de alias em revisão.
fn insert_local_target(
    resolved: &mut BTreeMap<String, LocalDependencyTarget>,
    alias: String,
    target: LocalDependencyTarget,
) {
    match resolved.get(&alias) {
        Some(existing) if existing != &target => {
            let detail = format!("'{alias}' foi declarado com destinos locais diferentes");
            resolved.insert(alias, LocalDependencyTarget::NeedsReview(detail));
        }
        Some(_) => {}
        None => {
            resolved.insert(alias, target);
        }
    }
}

/// Resolve todas as dependências `path` declaradas no Cargo.
fn resolve_local_dependencies(
    root: &Path,
    contract: &Contract,
    declared: &[DeclaredLocalDependency],
    workspace: &Option<WorkspaceInfo>,
) -> BTreeMap<String, LocalDependencyTarget> {
    let mut resolved = BTreeMap::new();
    let Some(workspace) = workspace else {
        return resolved;
    };
    for dependency in declared {
        let target = resolve_one_local_dependency(root, contract, dependency, workspace);
        insert_local_target(&mut resolved, dependency.alias.clone(), target.clone());
        insert_local_target(&mut resolved, dependency.package.clone(), target);
    }
    resolved
}

/// Lê o manifesto raiz e os manifestos dos membros declarados no workspace.
fn read_cargo_workspace(
    root: &Path,
    limitations: &mut Vec<String>,
) -> Result<(Option<WorkspaceInfo>, Vec<DeclaredLocalDependency>), AnalysisError> {
    let manifest = root.join("Cargo.toml");
    if !manifest.exists() {
        return Ok((None, Vec::new()));
    }

    let text = fs::read_to_string(&manifest).map_err(|source| AnalysisError::Io {
        path: manifest.clone(),
        source,
    })?;
    let document: toml::Value = toml::from_str(&text).map_err(|source| AnalysisError::Toml {
        path: manifest.clone(),
        source,
    })?;

    let workspace = document.get("workspace");
    let workspace_dependencies = workspace.and_then(|table| table.get("dependencies"));
    let mut declared = BTreeSet::new();
    let mut local_dependencies = Vec::new();
    collect_dependency_table(
        workspace_dependencies,
        workspace_dependencies,
        &mut declared,
        root,
        root,
        &mut local_dependencies,
    );
    if document.get("package").is_some() {
        collect_manifest_dependencies(
            &document,
            workspace_dependencies,
            &mut declared,
            root,
            root,
            &mut local_dependencies,
        );
    }

    let mut packages = Vec::new();

    if let Some(name) = document
        .get("package")
        .and_then(|table| table.get("name"))
        .and_then(toml::Value::as_str)
    {
        packages.push(WorkspacePackage {
            name: name.to_owned(),
            manifest_path: "Cargo.toml".to_owned(),
        });
    }

    let excludes: Vec<String> = workspace
        .and_then(|workspace| workspace.get("exclude"))
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    if let Some(members) = workspace
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
    {
        let mut expanded: Vec<String> = members
            .iter()
            .filter_map(toml::Value::as_str)
            .flat_map(|member| expand_member_pattern(root, member, &excludes, limitations))
            .collect();
        expanded.sort();
        expanded.dedup();

        for member in expanded {
            let member_manifest = root.join(&member).join("Cargo.toml");
            let member_text =
                fs::read_to_string(&member_manifest).map_err(|source| AnalysisError::Io {
                    path: member_manifest.clone(),
                    source,
                })?;
            let member_document: toml::Value =
                toml::from_str(&member_text).map_err(|source| AnalysisError::Toml {
                    path: member_manifest.clone(),
                    source,
                })?;
            collect_manifest_dependencies(
                &member_document,
                workspace_dependencies,
                &mut declared,
                &root.join(&member),
                root,
                &mut local_dependencies,
            );

            if let Some(name) = member_document
                .get("package")
                .and_then(|table| table.get("name"))
                .and_then(toml::Value::as_str)
            {
                packages.push(WorkspacePackage {
                    name: name.to_owned(),
                    manifest_path: format!("{member}/Cargo.toml"),
                });
            }
        }
    }

    packages.sort_by(|left, right| {
        (&left.name, &left.manifest_path).cmp(&(&right.name, &right.manifest_path))
    });

    Ok((
        Some(WorkspaceInfo {
            packages,
            declared_dependencies: declared.into_iter().collect(),
        }),
        local_dependencies,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::contract::{
        Architecture, ArchitectureStyle, Contract, Project, Scan, load_contract,
    };
    use crate::discovery::discover;
    use syn::Item;

    /// Raiz da fixture de dependências, relativa ao crate.
    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/dependencies")
    }

    /// Análise completa da fixture de dependências.
    fn analyze_fixture() -> Analysis {
        let root = project_root();
        let contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        let inventory = discover(&root, &contract).expect("discovery must succeed");
        analyze(&root, &contract, &inventory).expect("analysis must succeed")
    }

    #[test]
    fn workspace_packages_are_read_from_cargo_manifests() {
        let analysis = analyze_fixture();
        let workspace = analysis.workspace.expect("workspace must be read");
        let names: Vec<&str> = workspace
            .packages
            .iter()
            .map(|package| package.name.as_str())
            .collect();

        assert_eq!(names, ["application", "domain", "infrastructure"]);
    }

    #[test]
    fn allowed_and_forbidden_edges_are_classified() {
        let analysis = analyze_fixture();

        let application_domain = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "application" && edge.to == "domain")
            .expect("allowed edge must exist");
        assert_eq!(application_domain.kind, DependencyKind::Allowed);
        assert_eq!(application_domain.path, "crates/application/src/lib.rs");
        assert_eq!(application_domain.line, Some(1));
        assert_eq!(
            application_domain.evidence.as_deref(),
            Some("use domain::Entity;")
        );

        let application_infrastructure = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "application" && edge.to == "infrastructure")
            .expect("forbidden edge must exist");
        assert_eq!(application_infrastructure.kind, DependencyKind::Forbidden);
        assert_eq!(
            application_infrastructure.path,
            "crates/application/src/forbidden.rs"
        );

        let infrastructure_application = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "infrastructure" && edge.to == "application")
            .expect("allowed infrastructure edge must exist");
        assert_eq!(infrastructure_application.kind, DependencyKind::Allowed);
    }

    #[test]
    fn unresolved_dependency_becomes_review() {
        let analysis = analyze_fixture();

        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.to == "unknown_crate")
            .expect("unresolved edge must exist");
        assert_eq!(edge.kind, DependencyKind::Unresolved);

        let review = analysis
            .reviews
            .iter()
            .find(|review| review.reason == ReviewReason::UnresolvedDependency)
            .expect("unresolved dependency must require review");
        assert_eq!(review.path, "infra/src/unresolved.rs");
        assert_eq!(review.line, Some(1));
    }

    #[test]
    fn unmapped_file_becomes_review() {
        let analysis = analyze_fixture();

        let review = analysis
            .reviews
            .iter()
            .find(|review| review.reason == ReviewReason::UnmappedFile)
            .expect("unmapped file must require review");
        assert_eq!(review.path, "tools/helper.rs");
    }

    #[test]
    fn module_declaration_keeps_origin() {
        let analysis = analyze_fixture();

        let declaration = analysis
            .module_declarations
            .iter()
            .find(|declaration| declaration.name == "internal")
            .expect("mod declaration must be recorded");
        assert_eq!(declaration.module_name.as_deref(), Some("domain"));
        assert_eq!(declaration.path, "crates/domain/src/lib.rs");
        assert_eq!(declaration.line, Some(1));
        assert!(!declaration.inline);
    }

    #[test]
    fn inline_module_items_are_visited() {
        let analysis = analyze_fixture();

        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "domain" && edge.to == "application")
            .expect("inline use must produce an edge");
        assert_eq!(edge.kind, DependencyKind::Forbidden);
        assert_eq!(edge.path, "crates/domain/src/inline.rs");
        assert_eq!(edge.line, Some(2));
        assert_eq!(edge.evidence.as_deref(), Some("use application::Service;"));

        let declaration = analysis
            .module_declarations
            .iter()
            .find(|declaration| declaration.name == "nested")
            .expect("inline mod declaration must be recorded");
        assert_eq!(declaration.module_name.as_deref(), Some("domain"));
        assert_eq!(declaration.path, "crates/domain/src/inline.rs");
        assert_eq!(declaration.line, Some(1));
        assert!(declaration.inline);
    }

    #[test]
    fn local_and_standard_imports_are_ignored() {
        let analysis = analyze_fixture();

        assert!(
            analysis
                .edges
                .iter()
                .all(|edge| edge.to != "std" && edge.to != "super")
        );
    }

    #[test]
    fn unparseable_file_is_a_limitation_not_a_failure() {
        let analysis = analyze_fixture();

        assert!(
            analysis
                .limitations
                .iter()
                .any(|limitation| limitation.contains("crates/domain/src/broken.rs"))
        );
    }

    #[test]
    fn unsupported_extension_descriptions_are_stable() {
        let extensions = vec![
            UnsupportedCodeExtension {
                extension: "go".to_owned(),
                files: 2,
            },
            UnsupportedCodeExtension {
                extension: "rb".to_owned(),
                files: 1,
            },
        ];

        assert_eq!(
            describe_unsupported_code_extensions(&extensions),
            ".go (2), .rb (1)"
        );
    }

    #[test]
    fn unsupported_code_extensions_are_recognized_without_classifying_documents() {
        assert!(is_unsupported_code_extension(Some("go")));
        assert!(is_unsupported_code_extension(Some("RB")));
        assert!(!is_unsupported_code_extension(Some("rs")));
        assert!(!is_unsupported_code_extension(Some("md")));
        assert!(!is_unsupported_code_extension(Some("json")));
        assert!(!is_unsupported_code_extension(None));
    }

    #[test]
    fn declaration_lines_are_recognized() {
        assert!(is_use_declaration_line("use domain::Entity;"));
        assert!(is_use_declaration_line("    pub use domain::Entity;"));
        assert!(!is_use_declaration_line("// use domain::Entity;"));
        assert!(is_mod_declaration_line("mod internal;"));
        assert!(is_mod_declaration_line("    pub mod nested {"));
        assert!(!is_mod_declaration_line("let valor = 1;"));
    }

    #[test]
    fn locate_source_line_advances_the_cursor() {
        let lines = ["use a::A;", "let valor = 1;", "    use b::B;"];
        let mut cursor = 0;

        assert_eq!(
            locate_source_line(&lines, &mut cursor, is_use_declaration_line),
            Some((1, "use a::A;".to_owned()))
        );
        assert_eq!(
            locate_source_line(&lines, &mut cursor, is_use_declaration_line),
            Some((3, "use b::B;".to_owned()))
        );
        assert_eq!(
            locate_source_line(&lines, &mut cursor, is_use_declaration_line),
            None
        );
    }

    #[test]
    fn use_path_leaves_cover_groups_and_renames() {
        let parsed = syn::parse_file("use crate::{domain::Entity, infrastructure as infra};")
            .expect("use statement must parse");
        let Item::Use(item_use) = &parsed.items[0] else {
            panic!("first item must be a use declaration");
        };
        let mut leaves = Vec::new();

        collect_use_path_leaves(&item_use.tree, &mut Vec::new(), &mut leaves);

        let expected = vec![
            vec!["crate".to_owned(), "domain".to_owned(), "Entity".to_owned()],
            vec!["crate".to_owned(), "infrastructure".to_owned()],
        ];
        assert_eq!(leaves, expected);
    }

    #[test]
    fn duplicate_line_candidates_ignore_declarations_and_comments() {
        assert!(is_duplicate_candidate(
            "pub fn calcular_total_com_desconto_e_imposto(valor: f64) -> f64 { valor }"
        ));
        assert!(!is_duplicate_candidate(
            "import { Entity } from \"../domain/entity\";"
        ));
        assert!(!is_duplicate_candidate(
            "// comentário longo o suficiente para ser ignorado"
        ));
        assert!(!is_duplicate_candidate("curto"));
    }

    #[test]
    fn analysis_errors_explain_the_operation() {
        let io_error = AnalysisError::Io {
            path: PathBuf::from("Cargo.toml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "ausente"),
        };
        assert!(io_error.to_string().contains("Cargo.toml"));
        assert!(Error::source(&io_error).is_some());

        let toml_error = AnalysisError::Toml {
            path: PathBuf::from("Cargo.toml"),
            source: toml::from_str::<toml::Value>("invalid =").expect_err("invalid TOML"),
        };
        assert!(toml_error.to_string().contains("TOML inválido"));
        assert!(Error::source(&toml_error).is_some());
    }

    #[test]
    fn ambiguous_module_becomes_review_without_definitive_mapping() {
        let root = project_root();
        let module = |name: &str, path: &str| Module {
            name: name.to_owned(),
            path: path.to_owned(),
            responsibility: "Módulo usado no teste de ambiguidade".to_owned(),
            allowed_dependencies: Vec::new(),
        };
        let geral = module("geral", "crates/**/*.rs");
        let dominio = module("dominio", "crates/domain/src/**");

        for modules in [
            vec![geral.clone(), dominio.clone()],
            vec![dominio.clone(), geral.clone()],
        ] {
            let overlapping_contract = Contract {
                schema_version: 1,
                project: Project {
                    name: "sobreposicao".to_owned(),
                },
                scan: Scan {
                    include: vec!["**/*.rs".to_owned()],
                    exclude: Vec::new(),
                },
                architecture: Architecture {
                    style: ArchitectureStyle::ModularMonolith,
                    modules,
                },
            };

            let mut direct = Analysis {
                modules: Vec::new(),
                workspace: None,
                edges: Vec::new(),
                reviews: Vec::new(),
                module_declarations: Vec::new(),
                limitations: Vec::new(),
                duplicate_lines: Vec::new(),
                coverage: AnalysisCoverage::default(),
            };
            assert!(
                module_for_file(
                    &overlapping_contract,
                    "crates/domain/src/lib.rs",
                    &mut direct
                )
                .is_none(),
                "ambiguous file must not have a definitive module"
            );
            let review = direct
                .reviews
                .iter()
                .find(|review| review.reason == ReviewReason::AmbiguousModule)
                .expect("overlapping module paths must require review");
            assert_eq!(
                review.detail,
                "o arquivo corresponde a mais de um módulo do contrato: dominio, geral"
            );

            let inventory = discover(&root, &overlapping_contract).expect("discovery must succeed");
            let analysis =
                analyze(&root, &overlapping_contract, &inventory).expect("analysis must succeed");
            assert!(
                analysis
                    .module_declarations
                    .iter()
                    .filter(|declaration| declaration.path.starts_with("crates/domain/src/"))
                    .all(|declaration| declaration.module_name.is_none())
            );
            assert!(
                analysis
                    .edges
                    .iter()
                    .all(|edge| !edge.path.starts_with("crates/domain/src/"))
            );
        }
    }

    /// Raiz da fixture de dependências externas e caminhos relativos Rust.
    fn rust_dependencies_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/rust_dependencies")
    }

    /// Análise completa da fixture de dependências externas Rust.
    fn analyze_rust_dependencies() -> Analysis {
        let root = rust_dependencies_root();
        let contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        let inventory = discover(&root, &contract).expect("discovery must succeed");
        analyze(&root, &contract, &inventory).expect("analysis must succeed")
    }

    #[test]
    fn declared_cargo_dependencies_are_read_from_all_tables() {
        let analysis = analyze_rust_dependencies();
        let workspace = analysis.workspace.expect("workspace must be read");

        assert_eq!(
            workspace.declared_dependencies,
            [
                "alias_api",
                "build_tool",
                "dev_helper",
                "json_alias",
                "json_real",
                "real_api",
                "serde",
                "unix_only"
            ]
        );
    }

    #[test]
    fn declared_external_imports_do_not_create_dependencies_or_reviews() {
        let analysis = analyze_rust_dependencies();

        for target in [
            "serde",
            "json_alias",
            "alias_api",
            "dev_helper",
            "build_tool",
            "unix_only",
        ] {
            assert!(
                analysis.edges.iter().all(|edge| edge.to != target),
                "{target} must not become an architectural edge"
            );
            assert!(
                analysis
                    .reviews
                    .iter()
                    .all(|review| !review.detail.contains(target)),
                "{target} must not require review when declared in Cargo"
            );
        }
    }

    #[test]
    fn relative_paths_crossing_contract_boundaries_are_classified() {
        let analysis = analyze_rust_dependencies();

        let forbidden = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "domain" && edge.to == "application")
            .expect("super crossing into application must produce an edge");
        assert_eq!(forbidden.kind, DependencyKind::Forbidden);
        assert_eq!(forbidden.path, "src/domain/mod.rs");
        assert_eq!(forbidden.line, Some(3));
        assert_eq!(
            forbidden.evidence.as_deref(),
            Some("use super::application::Service;")
        );

        let allowed = analysis
            .edges
            .iter()
            .find(|edge| {
                edge.from == "application"
                    && edge.to == "domain"
                    && edge.path == "src/application/mod.rs"
            })
            .expect("super crossing into domain must produce an edge");
        assert_eq!(allowed.kind, DependencyKind::Allowed);
        assert_eq!(allowed.line, Some(3));
    }

    #[test]
    fn unresolvable_relative_path_requires_explicit_review() {
        let analysis = analyze_rust_dependencies();

        let review = analysis
            .reviews
            .iter()
            .find(|review| review.path == "src/application/mod.rs" && review.line == Some(4))
            .expect("unknown relative target must require review");
        assert_eq!(review.reason, ReviewReason::UnresolvedDependency);
        assert!(review.detail.contains("missing_module"));
    }

    #[test]
    fn undeclared_symbol_remains_unresolved() {
        let analysis = analyze_rust_dependencies();

        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.to == "unknown_symbol")
            .expect("undeclared root must remain unresolved");
        assert_eq!(edge.kind, DependencyKind::Unresolved);
        assert!(
            analysis
                .reviews
                .iter()
                .any(|review| review.detail.contains("unknown_symbol"))
        );
    }

    #[test]
    fn crate_self_alias_does_not_become_a_dependency() {
        let analysis = analyze_rust_dependencies();

        assert!(
            analysis
                .edges
                .iter()
                .all(|edge| edge.to != "self" && edge.to != "app_self")
        );
        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.path == "src/application/crate_self.rs")
            .expect("grouped crate use must still classify domain");
        assert_eq!(edge.to, "domain");
        assert_eq!(edge.kind, DependencyKind::Allowed);
    }

    #[test]
    fn relative_imports_inside_the_same_module_stay_internal() {
        let analysis = analyze_rust_dependencies();

        assert!(analysis.edges.iter().all(|edge| edge.to != "Entity"));
        assert!(
            analysis
                .reviews
                .iter()
                .all(|review| !review.detail.contains("Entity"))
        );
    }

    /// Raiz da fixture de resolução de `crate::`, relativa ao crate.
    fn crate_module_resolution_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/crate_module_resolution")
    }

    /// Análise completa da fixture de resolução de `crate::`.
    fn analyze_crate_module_resolution() -> Analysis {
        let root = crate_module_resolution_root();
        let contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        let inventory = discover(&root, &contract).expect("discovery must succeed");
        analyze(&root, &contract, &inventory).expect("analysis must succeed")
    }

    #[test]
    fn crate_paths_to_internal_modules_stay_internal() {
        let analysis = analyze_crate_module_resolution();

        assert!(
            analysis.edges.iter().all(|edge| edge.to != "analysis"),
            "internal crate paths must not become architectural edges"
        );
        assert!(
            analysis
                .reviews
                .iter()
                .all(|review| !review.detail.contains("analysis")),
            "internal crate paths must not require review"
        );
    }

    #[test]
    fn unresolved_crate_paths_still_require_review() {
        let analysis = analyze_crate_module_resolution();

        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.to == "unknown")
            .expect("unknown crate path must remain visible");
        assert_eq!(edge.kind, DependencyKind::Unresolved);
        assert_eq!(edge.path, "src/lib.rs");
        assert_eq!(edge.line, Some(3));
        assert_eq!(edge.evidence.as_deref(), Some("use crate::unknown::Thing;"));
        assert!(analysis.reviews.iter().any(|review| {
            review.reason == ReviewReason::UnresolvedDependency && review.detail.contains("unknown")
        }));
    }

    #[test]
    fn crate_source_root_is_inferred_from_the_src_component() {
        assert_eq!(
            crate_source_root("crates/auditor-core/src/analysis/mod.rs"),
            Some(PathBuf::from("crates/auditor-core/src"))
        );
        assert_eq!(crate_source_root("src/lib.rs"), Some(PathBuf::from("src")));
        assert_eq!(crate_source_root("tools/helper.rs"), None);
    }

    /// Raiz da fixture de dependências locais do workspace, relativa ao crate.
    fn workspace_local_dependencies_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/workspace_local_dependencies")
    }

    /// Analisa a fixture de dependências locais com um contrato explícito.
    fn analyze_workspace_local_dependencies_with(contract: &Contract) -> Analysis {
        let root = workspace_local_dependencies_root();
        let inventory = discover(&root, contract).expect("discovery must succeed");
        analyze(&root, contract, &inventory).expect("analysis must succeed")
    }

    /// Analisa a fixture de dependências locais com o contrato versionado.
    fn analyze_workspace_local_dependencies() -> Analysis {
        let root = workspace_local_dependencies_root();
        let contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        analyze_workspace_local_dependencies_with(&contract)
    }

    #[test]
    fn workspace_path_dependency_becomes_an_architectural_edge() {
        let analysis = analyze_workspace_local_dependencies();

        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "cli" && edge.to == "core")
            .expect("local path dependency must become an architectural edge");
        assert_eq!(edge.kind, DependencyKind::Allowed);
        assert_eq!(edge.path, "crates/cli/src/main.rs");
        assert_eq!(edge.line, Some(1));
        assert_eq!(edge.evidence.as_deref(), Some("use core_api::api;"));
    }

    #[test]
    fn external_dependency_declaration_continues_to_be_ignored() {
        let analysis = analyze_workspace_local_dependencies();

        assert!(analysis.edges.iter().all(|edge| edge.to != "serde"));
        assert!(
            analysis
                .reviews
                .iter()
                .all(|review| !review.detail.contains("serde"))
        );
        assert_eq!(
            analysis
                .workspace
                .expect("workspace must be read")
                .declared_dependencies,
            ["core_api", "core_lib", "serde", "tools", "tools_lib"]
        );
    }

    #[test]
    fn unmapped_local_workspace_dependency_requires_review() {
        let analysis = analyze_workspace_local_dependencies();

        let edge = analysis
            .edges
            .iter()
            .find(|edge| edge.from == "cli" && edge.to == "tools")
            .expect("local package without a contract module must remain visible");
        assert_eq!(edge.kind, DependencyKind::Unresolved);
        assert!(analysis.reviews.iter().any(|review| {
            review.reason == ReviewReason::UnresolvedDependency && review.detail.contains("tools")
        }));
    }

    #[test]
    fn inherited_workspace_path_is_resolved_from_the_workspace_root() {
        let analysis = analyze_workspace_local_dependencies();

        assert!(analysis.reviews.iter().any(|review| {
            review
                .detail
                .contains("pacote local 'tools-lib' não tem módulo correspondente no contrato")
        }));
        assert!(
            analysis
                .reviews
                .iter()
                .all(|review| !review.detail.contains("destinos locais diferentes"))
        );
    }

    #[test]
    fn local_workspace_analysis_ignores_contract_module_order() {
        let root = workspace_local_dependencies_root();
        let mut contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        let mut first = analyze_workspace_local_dependencies_with(&contract);
        contract.architecture.modules.reverse();
        let mut second = analyze_workspace_local_dependencies_with(&contract);
        // A listagem de módulos segue a ordem do contrato; a classificação não pode seguir.
        first.modules.sort();
        second.modules.sort();
        assert_eq!(first, second);
    }

    #[test]
    fn dependency_paths_are_resolved_lexically_inside_the_root() {
        let root = Path::new("/projeto");
        let member = root.join("crates/cli");

        assert_eq!(
            resolve_dependency_directory(root, &member, "../core"),
            Some(root.join("crates/core"))
        );
        assert_eq!(
            resolve_dependency_directory(root, &member, "./../core/src"),
            Some(root.join("crates/core/src"))
        );
        assert_eq!(
            resolve_dependency_directory(root, &member, "../../../fora"),
            None
        );
        assert_eq!(
            resolve_dependency_directory(root, &member, "/absoluto"),
            None
        );
    }

    #[test]
    fn literal_path_prefix_stops_before_the_first_wildcard() {
        assert_eq!(literal_path_prefix("crates/core/src/**"), "crates/core/src");
        assert_eq!(literal_path_prefix("src/**/*.rs"), "src");
        assert_eq!(literal_path_prefix("crates/*/src/**"), "crates");
    }

    #[test]
    fn module_mapping_requires_a_literal_package_directory() {
        assert!(module_is_inside_package(
            "crates/core/src/**",
            "crates/core"
        ));
        assert!(!module_is_inside_package("crates/*/src/**", "crates/core"));
        assert!(!module_is_inside_package("src/domain/**", "crates/core"));
    }

    #[test]
    fn conflicting_local_alias_becomes_an_explicit_review() {
        let mut resolved = BTreeMap::new();
        insert_local_target(
            &mut resolved,
            "api".to_owned(),
            LocalDependencyTarget::Module("core".to_owned()),
        );
        insert_local_target(
            &mut resolved,
            "api".to_owned(),
            LocalDependencyTarget::Module("cli".to_owned()),
        );

        assert_eq!(
            resolved["api"],
            LocalDependencyTarget::NeedsReview(
                "'api' foi declarado com destinos locais diferentes".to_owned()
            )
        );
    }

    /// Raiz de uma fixture de workspace com glob, relativa ao crate.
    fn workspace_glob_root(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name)
    }

    /// Análise completa de uma fixture de workspace com glob.
    fn analyze_workspace_glob(name: &str) -> Analysis {
        let root = workspace_glob_root(name);
        let contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        let inventory = discover(&root, &contract).expect("discovery must succeed");
        analyze(&root, &contract, &inventory).expect("analysis must succeed")
    }

    #[test]
    fn workspace_glob_expands_members_and_respects_exclude() {
        let analysis = analyze_workspace_glob("workspace_glob");
        let workspace = analysis.workspace.expect("workspace must be read");

        let names: Vec<&str> = workspace
            .packages
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        let manifests: Vec<&str> = workspace
            .packages
            .iter()
            .map(|package| package.manifest_path.as_str())
            .collect();
        assert_eq!(names, ["alpha", "beta", "legacy"]);
        assert_eq!(
            manifests,
            [
                "crates/alpha/Cargo.toml",
                "crates/beta/Cargo.toml",
                "legacy/Cargo.toml"
            ]
        );
        assert_eq!(workspace.declared_dependencies, ["serde"]);
        assert!(analysis.limitations.is_empty());
    }

    #[test]
    fn workspace_glob_analysis_is_deterministic() {
        let first = analyze_workspace_glob("workspace_glob");
        let second = analyze_workspace_glob("workspace_glob");

        assert_eq!(first, second);
    }

    #[test]
    fn unsupported_workspace_member_patterns_become_limitations() {
        let analysis = analyze_workspace_glob("workspace_glob_unsupported");

        assert!(
            analysis
                .workspace
                .expect("workspace must be read")
                .packages
                .is_empty()
        );
        assert!(analysis.limitations.iter().any(|limitation| {
            limitation.contains("crates/**") && limitation.contains("não é suportado")
        }));
        assert!(analysis.limitations.iter().any(|limitation| {
            limitation.contains("empty/*") && limitation.contains("Cargo.toml")
        }));
        assert!(analysis.limitations.iter().any(|limitation| {
            limitation.contains("crates/?lpha") && limitation.contains("não é suportado")
        }));
        assert!(analysis.limitations.iter().any(|limitation| {
            limitation.contains("../outside/*") && limitation.contains("não é suportado")
        }));
    }
}
