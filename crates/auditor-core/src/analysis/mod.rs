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

use syn::{Item, UseTree};

use crate::contract::{Contract, Module};
use crate::discovery::{Inventory, matches_pattern};

/// Raízes da biblioteca padrão, que não são dependências arquiteturais.
const IGNORED_ROOTS: &[&str] = &["std", "core", "alloc", "test"];

/// Resultado da análise do projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceInfo {
    pub packages: Vec<WorkspacePackage>,
}

/// Pacote identificado em um `Cargo.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePackage {
    pub name: String,
    pub manifest_path: String,
}

/// Classificação de uma aresta de dependência.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    /// O destino está em `allowed_dependencies`.
    Allowed,
    /// O destino é um módulo do contrato, mas não está em `allowed_dependencies`.
    Forbidden,
    /// O destino não pôde ser associado a um módulo do contrato.
    Unresolved,
}

/// Aresta direcional entre dois módulos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub kind: DependencyKind,
    pub path: String,
    pub line: Option<usize>,
    pub evidence: Option<String>,
    pub detail: String,
}

/// Item que o auditor não consegue classificar sozinho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewItem {
    pub path: String,
    pub line: Option<usize>,
    pub reason: ReviewReason,
    pub detail: String,
}

/// Motivo da revisão necessária.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewReason {
    /// Arquivo incluído na varredura, mas fora de todos os módulos do contrato.
    UnmappedFile,
    /// Importação cujo destino não corresponde a um módulo declarado.
    UnresolvedDependency,
    /// Arquivo que corresponde a mais de um módulo do contrato.
    AmbiguousModule,
}

/// Declaração `mod` localizada em um arquivo Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDeclaration {
    pub module: Option<String>,
    pub name: String,
    pub path: String,
    pub line: Option<usize>,
    pub inline: bool,
}

/// Erros operacionais da análise.
#[derive(Debug)]
pub enum AnalysisError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Toml {
        path: PathBuf,
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
    module: Option<&'a Module>,
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
    let workspace = read_workspace(root)?;
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

        let matched: Vec<&Module> = contract
            .architecture
            .modules
            .iter()
            .filter(|module| matches_pattern(&module.path, &file.path))
            .collect();

        if matched.len() > 1 {
            analysis.reviews.push(ReviewItem {
                path: file.path.clone(),
                line: None,
                reason: ReviewReason::AmbiguousModule,
                detail: "o arquivo corresponde a mais de um módulo do contrato".to_owned(),
            });
        }

        let module = matched.first().copied();

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
            module,
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

fn visit_items(items: &[Item], context: &mut VisitContext<'_>) {
    for item in items {
        match item {
            Item::Use(item_use) => {
                let (line, evidence) = match locate(context.lines, context.use_cursor, is_use_line)
                {
                    Some(located) => (Some(located.0), Some(located.1)),
                    None => {
                        context.analysis.limitations.push(format!(
                            "não foi possível localizar a linha de um `use` em {}",
                            context.path
                        ));
                        (None, None)
                    }
                };

                let mut leaves = Vec::new();
                collect_use_paths(&item_use.tree, &mut Vec::new(), &mut leaves);

                if context.module.is_some() {
                    for leaf in leaves {
                        classify_use(context, &leaf, line, evidence.as_deref());
                    }
                }
            }
            Item::Mod(item_mod) => {
                let (line, _) = match locate(context.lines, context.mod_cursor, is_mod_line) {
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
                        module: context.module.map(|module| module.name.clone()),
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

fn classify_use(
    context: &mut VisitContext<'_>,
    leaf: &[String],
    line: Option<usize>,
    evidence: Option<&str>,
) {
    let Some(module) = context.module else {
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

    let key = format!("{from}\0{target}\0{kind:?}\0{}\0{line:?}", context.path);
    if !context.seen_edges.insert(key) {
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

fn collect_use_paths(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_paths(&path.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(name) => {
            prefix.push(name.ident.to_string());
            out.push(prefix.clone());
            prefix.pop();
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            out.push(prefix.clone());
            prefix.pop();
        }
        UseTree::Glob(_) => out.push(prefix.clone()),
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use_paths(tree, prefix, out);
            }
        }
    }
}

fn locate(
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

fn is_use_line(line: &str) -> bool {
    let rest = line.trim_start();
    let rest = rest
        .strip_prefix("pub")
        .map(str::trim_start)
        .unwrap_or(rest);
    rest.starts_with("use ") || rest.starts_with("use{")
}

fn is_mod_line(line: &str) -> bool {
    let rest = line.trim_start();
    let rest = rest
        .strip_prefix("pub")
        .map(str::trim_start)
        .unwrap_or(rest);
    rest.starts_with("mod ") || rest.starts_with("mod{")
}

fn read_workspace(root: &Path) -> Result<Option<WorkspaceInfo>, AnalysisError> {
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
    use crate::contract::load_contract;
    use crate::discovery::discover;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/dependencies")
    }

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
        assert_eq!(declaration.module.as_deref(), Some("domain"));
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
        assert_eq!(declaration.module.as_deref(), Some("domain"));
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
}
