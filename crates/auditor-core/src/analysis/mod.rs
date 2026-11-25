//! Análise inicial de dependências entre os módulos declarados no contrato.
//!
//! A análise é determinística e somente leitura. Ela usa `syn` para interpretar
//! declarações `mod` e `use`, e os manifests Cargo para conhecer os pacotes do
//! workspace. Construções que não podem ser resolvidas com segurança viram
//! itens de revisão ou limitações explícitas, nunca conclusões inventadas.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use syn::{Item, UseTree};

use crate::contract::{Contract, Module};
use crate::discovery::{Inventory, matches_pattern};

/// Raízes da biblioteca padrão, que não são dependências arquiteturais.
const IGNORED_ROOTS: &[&str] = &["std", "core", "alloc", "test"];

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
}

/// Pacotes descobertos no workspace Cargo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceInfo {
    /// Pacotes ordenados por nome e caminho de manifesto.
    pub packages: Vec<WorkspacePackage>,
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

/// Estado compartilhado durante a visita de uma única unidade Rust.
struct VisitContext<'a> {
    path: &'a str,
    contract_module: Option<&'a Module>,
    lines: &'a [&'a str],
    use_cursor: &'a mut usize,
    mod_cursor: &'a mut usize,
    contract_names: &'a HashSet<String>,
    workspace_names: &'a HashSet<String>,
    analysis: &'a mut Analysis,
    seen_edges: &'a mut HashSet<String>,
}

/// Analisa os arquivos Rust do inventário e produz o grafo inicial.
pub fn analyze(
    root: &Path,
    contract: &Contract,
    inventory: &Inventory,
) -> Result<Analysis, AnalysisError> {
    let workspace = read_cargo_workspace(root)?;
    let workspace_names: HashSet<String> = workspace
        .as_ref()
        .map(|info| {
            info.packages
                .iter()
                .map(|package| package.name.clone())
                .collect()
        })
        .unwrap_or_default();
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
        limitations: Vec::new(),
    };

    let mut seen_edges = HashSet::new();

    for file in &inventory.files {
        if file.extension.as_deref() != Some("rs") {
            continue;
        }

        let full_path = root.join(&file.path);
        let text = match fs::read_to_string(&full_path) {
            Ok(text) => text,
            Err(source) => {
                analysis
                    .limitations
                    .push(format!("{} não pôde ser lido: {source}", file.path));
                continue;
            }
        };

        let parsed = match syn::parse_file(&text) {
            Ok(parsed) => parsed,
            Err(source) => {
                analysis
                    .limitations
                    .push(format!("{} não pôde ser analisado: {source}", file.path));
                continue;
            }
        };

        let matching_modules: Vec<&Module> = contract
            .architecture
            .modules
            .iter()
            .filter(|module| matches_pattern(&module.path, &file.path))
            .collect();

        if matching_modules.len() > 1 {
            analysis.reviews.push(ReviewItem {
                path: file.path.clone(),
                line: None,
                reason: ReviewReason::AmbiguousModule,
                detail: "o arquivo corresponde a mais de um módulo do contrato".to_owned(),
            });
        }

        let module = matching_modules.first().copied();

        if module.is_none() {
            analysis.reviews.push(ReviewItem {
                path: file.path.clone(),
                line: None,
                reason: ReviewReason::UnmappedFile,
                detail: "nenhum path de módulo do contrato corresponde ao arquivo".to_owned(),
            });
        }

        let lines: Vec<&str> = text.lines().collect();
        let mut use_cursor = 0;
        let mut mod_cursor = 0;

        let mut context = VisitContext {
            path: &file.path,
            contract_module: module,
            lines: &lines,
            use_cursor: &mut use_cursor,
            mod_cursor: &mut mod_cursor,
            contract_names: &contract_names,
            workspace_names: &workspace_names,
            analysis: &mut analysis,
            seen_edges: &mut seen_edges,
        };
        visit_items(&parsed.items, &mut context);
    }

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
        leaf[1].clone()
    } else if first == "self" || first == "super" {
        // Dependências internas ao mesmo módulo não cruzam a fronteira do contrato.
        return;
    } else {
        first.clone()
    };

    let from = &module.name;
    if IGNORED_ROOTS.contains(&target.as_str()) || &target == from {
        return;
    }

    let (kind, detail) = if module.allowed_dependencies.contains(&target) {
        (
            DependencyKind::Allowed,
            "destino presente em allowed_dependencies".to_owned(),
        )
    } else if context.contract_names.contains(&target) {
        (
            DependencyKind::Forbidden,
            "destino declarado no contrato, mas ausente de allowed_dependencies".to_owned(),
        )
    } else {
        let detail = if context.workspace_names.contains(&target) {
            "pacote do workspace sem módulo correspondente no contrato".to_owned()
        } else {
            "destino não corresponde a nenhum módulo do contrato".to_owned()
        };
        (DependencyKind::Unresolved, detail)
    };

    if kind == DependencyKind::Unresolved {
        context.analysis.reviews.push(ReviewItem {
            path: context.path.to_owned(),
            line,
            reason: ReviewReason::UnresolvedDependency,
            detail: format!("{target}: {detail}"),
        });
    }

    let edge_key = format!("{from}\0{target}\0{kind:?}\0{}\0{line:?}", context.path);
    if !context.seen_edges.insert(edge_key) {
        return;
    }

    context.analysis.edges.push(DependencyEdge {
        from: from.clone(),
        to: target,
        kind,
        path: context.path.to_owned(),
        line,
        evidence: evidence.map(str::to_owned),
        detail,
    });
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

/// Lê o manifesto raiz e os manifestos dos membros declarados no workspace.
fn read_cargo_workspace(root: &Path) -> Result<Option<WorkspaceInfo>, AnalysisError> {
    let manifest = root.join("Cargo.toml");
    if !manifest.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(&manifest).map_err(|source| AnalysisError::Io {
        path: manifest.clone(),
        source,
    })?;
    let document: toml::Value = toml::from_str(&text).map_err(|source| AnalysisError::Toml {
        path: manifest.clone(),
        source,
    })?;

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

    if let Some(members) = document
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
    {
        for member in members.iter().filter_map(toml::Value::as_str) {
            let member_manifest = root.join(member).join("Cargo.toml");
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

    Ok(Some(WorkspaceInfo { packages }))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::contract::{Architecture, ArchitectureStyle, Project, Scan, load_contract};
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
    fn ambiguous_module_becomes_review() {
        let root = project_root();
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
                modules: vec![
                    Module {
                        name: "geral".to_owned(),
                        path: "crates/**/*.rs".to_owned(),
                        responsibility: "Todos os arquivos Rust".to_owned(),
                        allowed_dependencies: Vec::new(),
                    },
                    Module {
                        name: "dominio".to_owned(),
                        path: "crates/domain/src/**".to_owned(),
                        responsibility: "Arquivos de domínio".to_owned(),
                        allowed_dependencies: Vec::new(),
                    },
                ],
            },
        };
        let inventory = discover(&root, &overlapping_contract).expect("discovery must succeed");
        let analysis =
            analyze(&root, &overlapping_contract, &inventory).expect("analysis must succeed");

        let review = analysis
            .reviews
            .iter()
            .find(|review| review.reason == ReviewReason::AmbiguousModule)
            .expect("overlapping module paths must require review");
        assert!(
            review.path.starts_with("crates/domain/src/"),
            "review must point to a domain source file"
        );
    }
}
