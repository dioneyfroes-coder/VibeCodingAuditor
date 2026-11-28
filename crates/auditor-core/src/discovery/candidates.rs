//! Módulos candidatos e classificação das relações observadas.
//!
//! Nada aqui aprova arquitetura. Um candidato vem de um limite explícito em
//! manifesto ou de um agrupamento heurístico por diretório, e sempre exige
//! confirmação humana. Destinos são classificados por evidência verificável,
//! nunca por uma regra de conformidade.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::manifests::{ManifestDiscovery, ManifestObservation};
use super::{Inventory, matches_pattern};
use crate::analysis::{Analysis, DependencyKind};
use crate::resolution::{AliasResolution, ExternalPackageState, ResolutionContext};

/// Origem da evidência que sustenta um módulo candidato.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCategory {
    /// Limite explícito declarado em manifesto Cargo reconhecido pela análise.
    Direct,
    /// Agrupamento heurístico por diretório, sem manifesto que o sustente.
    InferredPath,
    /// Evidências conflitantes; a escolha depende de decisão humana.
    Ambiguous,
}

impl EvidenceCategory {
    /// Nome estável usado em texto, Markdown e JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::InferredPath => "inferred_path",
            Self::Ambiguous => "ambiguous",
        }
    }
}

/// Módulo sugerido a partir de evidências verificáveis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CandidateModule {
    /// Nome sugerido, nunca aprovado automaticamente.
    pub name: String,
    /// Padrão compatível com `architecture.modules[].path`.
    pub path: String,
    /// Origem da evidência que criou o candidato.
    pub category: EvidenceCategory,
    /// Textos curtos que permitem verificar a sugestão.
    pub evidence: Vec<String>,
    /// Sempre `true`: nenhum candidato é uma decisão humana.
    pub needs_confirmation: bool,
}

/// Classificação de uma relação vista no código, sem regras de conformidade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedResolution {
    /// Destino associado a um arquivo ou candidato interno.
    Resolved,
    /// Caminho relativo sem correspondência no inventário.
    Unresolved,
    /// Destino fora dos limites internos conhecidos.
    External,
    /// Mais de um destino interno corresponde ao observado.
    Ambiguous,
}

impl ObservedResolution {
    /// Rótulo legível usado em texto e Markdown, neutro quanto a regras.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolvida",
            Self::Unresolved => "não resolvida",
            Self::External => "externa",
            Self::Ambiguous => "ambígua",
        }
    }
}

/// Relação observada com destino classificado apenas por evidência.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedDependency {
    /// Arquivo de origem, relativo à raiz.
    pub source: String,
    /// Destino bruto encontrado no código.
    pub target: String,
    /// Classificação baseada em evidência, não em contrato.
    pub resolution: ObservedResolution,
    /// Explicação curta da classificação.
    pub detail: String,
    /// Linha da declaração, quando localizada.
    pub line: Option<usize>,
    /// Texto da declaração usado como evidência.
    pub evidence: Option<String>,
}

/// Conflito preservado para decisão humana.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ambiguity {
    /// Arquivo, destino ou nome que originou o conflito.
    pub subject: String,
    /// Explicação do conflito.
    pub detail: String,
    /// Candidatos envolvidos, em ordem estável.
    pub options: Vec<String>,
}

/// Extensão inventariada e analisador disponível para ela.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageCoverage {
    /// Extensão normalizada, sem o ponto inicial.
    pub extension: String,
    /// Quantidade de arquivos inventariados com a extensão.
    pub files: usize,
    /// Analisador determinístico disponível, quando houver.
    pub analyzer: Option<String>,
}

/// Material de descoberta produzido sem contrato.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DiscoveryCandidates {
    /// Módulos sugeridos, todos pendentes de confirmação.
    pub candidate_modules: Vec<CandidateModule>,
    /// Relações observadas com classificação por evidência.
    pub observed_dependencies: Vec<ObservedDependency>,
    /// Conflitos que o usuário precisa resolver.
    pub ambiguities: Vec<Ambiguity>,
    /// Extensões vistas e analisadores disponíveis.
    pub languages: Vec<LanguageCoverage>,
    /// Manifests de outros ecossistemas lidos sem executar ferramentas.
    pub manifests: Vec<ManifestObservation>,
    /// Limitações da descoberta, inclusive ausência de analisador.
    pub limitations: Vec<String>,
}

/// Constrói candidatos e classifica relações a partir do inventário e da análise.
pub fn build_candidates(inventory: &Inventory, analysis: &Analysis) -> DiscoveryCandidates {
    let files: BTreeSet<&str> = inventory
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    let mut limitations = Vec::new();
    let mut candidates = direct_candidates(analysis, &files);
    let inferred = inferred_candidates(&candidates, &files, &mut limitations);
    candidates.extend(inferred);
    candidates.sort_by(|left, right| (&left.name, &left.path).cmp(&(&right.name, &right.path)));

    let context = ResolutionContext::for_inventory(inventory);
    let mut ambiguities = mark_ambiguous(&mut candidates, &files);
    let observed_dependencies =
        classify_observations(analysis, &candidates, &files, &context, &mut ambiguities);
    ambiguities.sort_by(|left, right| {
        (&left.subject, &left.detail, &left.options).cmp(&(
            &right.subject,
            &right.detail,
            &right.options,
        ))
    });

    let manifests: ManifestDiscovery = super::manifests::read_manifests(&inventory.root, &files);
    for observation in &manifests.observations {
        limitations.extend(observation.limitations.iter().cloned());
    }
    limitations.extend(manifests.limitations.iter().cloned());
    limitations.extend(context.limitations.iter().cloned());
    if analysis.workspace.is_none() {
        limitations.push("sem manifesto Cargo reconhecido: não há candidatos direct".to_owned());
    }
    limitations.push(
        "descoberta sem contrato inventaria somente extensões com analisador conhecido; outras linguagens exigem scan.include explícito no contrato"
            .to_owned(),
    );
    limitations.sort();
    limitations.dedup();

    DiscoveryCandidates {
        candidate_modules: candidates,
        observed_dependencies,
        ambiguities,
        languages: language_coverage(inventory),
        manifests: manifests.observations,
        limitations,
    }
}

/// Candidatos apoiados em pacotes de manifestos reconhecidos.
fn direct_candidates(analysis: &Analysis, files: &BTreeSet<&str>) -> Vec<CandidateModule> {
    let Some(workspace) = analysis.workspace.as_ref() else {
        return Vec::new();
    };

    workspace
        .packages
        .iter()
        .map(|package| {
            let directory = manifest_directory(&package.manifest_path);
            CandidateModule {
                name: package.name.clone(),
                path: package_path(&directory, files),
                category: EvidenceCategory::Direct,
                evidence: vec![format!("manifesto: {}", package.manifest_path)],
                needs_confirmation: true,
            }
        })
        .collect()
}

/// Diretório que contém o manifesto informado.
pub(super) fn manifest_directory(manifest_path: &str) -> String {
    Path::new(manifest_path)
        .parent()
        .map(|parent| parent.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

/// Prefixo de caminho sustentado pelos arquivos do pacote no inventário.
pub(super) fn package_path(directory: &str, files: &BTreeSet<&str>) -> String {
    let source = if directory.is_empty() {
        "src".to_owned()
    } else {
        format!("{directory}/src")
    };
    let prefix = format!("{source}/");
    if files.iter().any(|file| file.starts_with(prefix.as_str())) {
        format!("{source}/**")
    } else if directory.is_empty() {
        "**".to_owned()
    } else {
        format!("{directory}/**")
    }
}

/// Candidatos heurísticos para código fora dos limites declarados em manifestos.
fn inferred_candidates(
    direct: &[CandidateModule],
    files: &BTreeSet<&str>,
    limitations: &mut Vec<String>,
) -> Vec<CandidateModule> {
    let mut roots: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for &file in files.iter() {
        if direct
            .iter()
            .any(|candidate| matches_pattern(&candidate.path, file))
        {
            continue;
        }
        let Some((root, rest)) = file.split_once('/') else {
            limitations.push(format!(
                "arquivo na raiz não gera candidato por diretório: {file}"
            ));
            continue;
        };
        if !rest.is_empty() {
            roots.entry(root.to_owned()).or_default().push(file);
        }
    }

    roots
        .into_iter()
        .map(|(root, paths)| {
            let mut evidence: Vec<String> = paths
                .iter()
                .take(3)
                .map(|path| format!("arquivo: {path}"))
                .collect();
            if paths.len() > 3 {
                evidence.push(format!("+{} outros arquivos", paths.len() - 3));
            }
            CandidateModule {
                name: root.clone(),
                path: format!("{root}/**"),
                category: EvidenceCategory::InferredPath,
                evidence,
                needs_confirmation: true,
            }
        })
        .collect()
}

/// Registra conflitos de nome e de cobertura, marcando os candidatos envolvidos.
fn mark_ambiguous(candidates: &mut [CandidateModule], files: &BTreeSet<&str>) -> Vec<Ambiguity> {
    let mut by_name: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        by_name
            .entry(candidate.name.clone())
            .or_default()
            .push(index);
    }

    let mut ambiguities = Vec::new();
    for (name, indexes) in by_name {
        if indexes.len() <= 1 {
            continue;
        }
        let options: Vec<String> = indexes
            .iter()
            .map(|index| candidates[*index].path.clone())
            .collect();
        ambiguities.push(Ambiguity {
            subject: name,
            detail: "mais de um candidato usa o mesmo nome; a escolha exige decisão humana"
                .to_owned(),
            options,
        });
        for index in indexes {
            candidates[index].category = EvidenceCategory::Ambiguous;
        }
    }

    for &file in files.iter() {
        let options: Vec<String> = candidates
            .iter()
            .filter(|candidate| matches_pattern(&candidate.path, file))
            .map(|candidate| candidate.name.clone())
            .collect();
        if options.len() > 1 {
            ambiguities.push(Ambiguity {
                subject: file.to_owned(),
                detail: "arquivo corresponde a mais de um candidato; nenhum módulo foi escolhido"
                    .to_owned(),
                options,
            });
        }
    }

    ambiguities
}

/// Classifica cada relação observada usando manifestos, inventário e candidatos.
fn classify_observations(
    analysis: &Analysis,
    candidates: &[CandidateModule],
    files: &BTreeSet<&str>,
    context: &ResolutionContext,
    ambiguities: &mut Vec<Ambiguity>,
) -> Vec<ObservedDependency> {
    let workspace_packages: Vec<&str> = analysis
        .workspace
        .as_ref()
        .map(|workspace| {
            workspace
                .packages
                .iter()
                .map(|package| package.name.as_str())
                .collect()
        })
        .unwrap_or_default();
    let declared: Vec<&str> = analysis
        .workspace
        .as_ref()
        .map(|workspace| {
            workspace
                .declared_dependencies
                .iter()
                .map(String::as_str)
                .filter(|name| {
                    !workspace_packages
                        .iter()
                        .any(|package| names_match(package, name))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut observed = Vec::new();
    for edge in analysis
        .edges
        .iter()
        .filter(|edge| edge.kind == DependencyKind::Observed)
    {
        let root = target_root(&edge.to);
        let relation = analysis
            .external_packages
            .iter()
            .find(|relation| relation.path == edge.path && relation.specifier == edge.to);
        let (resolution, detail, options) = if declared
            .iter()
            .any(|name| names_match(name, root) || names_match(name, &edge.to))
        {
            (
                ObservedResolution::External,
                "declarada em manifesto Cargo; não é módulo interno".to_owned(),
                Vec::new(),
            )
        } else if edge.to.starts_with('.') {
            let matches = resolve_relative_target(&edge.path, &edge.to, files);
            match matches.len() {
                0 => (
                    ObservedResolution::Unresolved,
                    "caminho relativo não resolvido no inventário".to_owned(),
                    Vec::new(),
                ),
                1 => (
                    ObservedResolution::Resolved,
                    format!("resolvido para {}", matches[0]),
                    Vec::new(),
                ),
                _ => (
                    ObservedResolution::Ambiguous,
                    "mais de um arquivo corresponde ao caminho relativo".to_owned(),
                    matches,
                ),
            }
        } else {
            match context.resolve_alias(&edge.to, files) {
                AliasResolution::Resolved(path) => (
                    ObservedResolution::Resolved,
                    format!("alias `{}` resolvido para {}", edge.to, path),
                    Vec::new(),
                ),
                AliasResolution::Ambiguous(options) => (
                    ObservedResolution::Ambiguous,
                    format!("alias `{}` com destinos ambíguos", edge.to),
                    options,
                ),
                AliasResolution::Unresolved(detail) => {
                    (ObservedResolution::Unresolved, detail, Vec::new())
                }
                AliasResolution::NoMatch => {
                    let python_files = if Path::new(&edge.path)
                        .extension()
                        .and_then(|extension| extension.to_str())
                        == Some("py")
                        && edge.to.contains('.')
                    {
                        crate::resolution::resolve_python_module(&edge.to, files)
                    } else {
                        Vec::new()
                    };
                    if python_files.len() == 1 {
                        (
                            ObservedResolution::Resolved,
                            format!("módulo Python resolvido para {}", python_files[0]),
                            Vec::new(),
                        )
                    } else if python_files.len() > 1 {
                        (
                            ObservedResolution::Ambiguous,
                            "mais de um arquivo Python corresponde ao módulo".to_owned(),
                            python_files,
                        )
                    } else {
                        let matches: Vec<&CandidateModule> = candidates
                            .iter()
                            .filter(|candidate| names_match(&candidate.name, root))
                            .collect();
                        match matches.len() {
                            0 => {
                                let (state, origin) = relation.map_or(
                                    (
                                        ExternalPackageState::Unconfirmed,
                                        "sem declaração verificável",
                                    ),
                                    |relation| (relation.state, relation.origin.as_str()),
                                );
                                let detail = match state {
                                    ExternalPackageState::Builtin => {
                                        format!("embutida ({origin}); não é módulo interno")
                                    }
                                    ExternalPackageState::Declared => format!(
                                        "externa declarada; origem: {origin}; não é módulo interno"
                                    ),
                                    ExternalPackageState::Unconfirmed => format!(
                                        "externa não confirmada; origem: {origin}; não é módulo interno"
                                    ),
                                    ExternalPackageState::WorkspaceLocal => format!(
                                        "local de workspace; origem: {origin}; não é módulo interno"
                                    ),
                                };
                                (ObservedResolution::External, detail, Vec::new())
                            }
                            1 => (
                                ObservedResolution::Resolved,
                                format!(
                                    "associado ao candidato `{}` ({})",
                                    matches[0].name, matches[0].path
                                ),
                                Vec::new(),
                            ),
                            _ => (
                                ObservedResolution::Ambiguous,
                                "destino corresponde a mais de um candidato".to_owned(),
                                matches
                                    .iter()
                                    .map(|candidate| {
                                        format!("{} ({})", candidate.name, candidate.path)
                                    })
                                    .collect(),
                            ),
                        }
                    }
                }
            }
        };

        if resolution == ObservedResolution::Ambiguous {
            ambiguities.push(Ambiguity {
                subject: edge.to.clone(),
                detail: detail.clone(),
                options: options.clone(),
            });
        }
        observed.push(ObservedDependency {
            source: edge.path.clone(),
            target: edge.to.clone(),
            resolution,
            detail,
            line: edge.line,
            evidence: edge.evidence.clone(),
        });
    }

    observed.sort_by(|left, right| {
        (
            &left.source,
            &left.target,
            left.line.unwrap_or_default(),
            &left.detail,
        )
            .cmp(&(
                &right.source,
                &right.target,
                right.line.unwrap_or_default(),
                &right.detail,
            ))
    });
    observed
}

/// Primeiro segmento nomeado de um destino bruto.
fn target_root(target: &str) -> &str {
    let trimmed = target.trim_start_matches(['.', '/']);
    trimmed
        .split([':', '.', '/'])
        .find(|part| !part.is_empty())
        .unwrap_or(trimmed)
}

/// Nomes de pacote Rust e Cargo podem alternar hífen e sublinhado.
fn names_match(left: &str, right: &str) -> bool {
    left == right || left.replace('-', "_") == right.replace('-', "_")
}

/// Resolve um especificador relativo contra os caminhos do inventário.
fn resolve_relative_target(source: &str, target: &str, files: &BTreeSet<&str>) -> Vec<String> {
    let directory = source
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("");
    let mut parts: Vec<&str> = directory
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            named => parts.push(named),
        }
    }

    let base = parts.join("/");
    let suffixes = [
        "",
        ".rs",
        ".ts",
        ".tsx",
        ".js",
        ".jsx",
        ".py",
        "/mod.rs",
        "/__init__.py",
    ];
    let mut found: Vec<String> = suffixes
        .iter()
        .map(|suffix| format!("{base}{suffix}"))
        .filter(|candidate| files.contains(candidate.as_str()))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// Extensões inventariadas com o analisador determinístico correspondente.
fn language_coverage(inventory: &Inventory) -> Vec<LanguageCoverage> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for file in &inventory.files {
        let extension = file
            .extension
            .clone()
            .unwrap_or_else(|| "unknown".to_owned());
        *counts.entry(extension).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(extension, files)| LanguageCoverage {
            analyzer: analyzer_for(&extension).map(str::to_owned),
            extension,
            files,
        })
        .collect()
}

/// Analisador disponível para uma extensão, quando houver.
pub(super) fn analyzer_for(extension: &str) -> Option<&'static str> {
    match extension {
        "rs" => Some("rust"),
        "ts" | "tsx" | "js" | "jsx" => Some("typescript_javascript"),
        "py" => Some("python"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::analysis::analyze_observations;
    use crate::discovery::discover_without_contract;

    /// Descoberta completa de uma fixture pelo nome do diretório.
    fn candidates_for(name: &str) -> DiscoveryCandidates {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name);
        let inventory = discover_without_contract(&root).expect("discovery must succeed");
        let analysis = analyze_observations(&root, &inventory).expect("analysis must succeed");
        build_candidates(&inventory, &analysis)
    }

    #[test]
    fn manifests_and_directories_produce_stable_candidates() {
        let candidates = candidates_for("discovery_candidates");
        let described: Vec<(&str, &str, EvidenceCategory)> = candidates
            .candidate_modules
            .iter()
            .map(|candidate| {
                (
                    candidate.name.as_str(),
                    candidate.path.as_str(),
                    candidate.category,
                )
            })
            .collect();

        assert_eq!(
            described,
            vec![
                (
                    "application",
                    "crates/application/src/**",
                    EvidenceCategory::Direct
                ),
                ("domain", "crates/domain/src/**", EvidenceCategory::Direct),
                ("web", "web/**", EvidenceCategory::InferredPath),
            ]
        );
        assert!(
            candidates
                .candidate_modules
                .iter()
                .all(|candidate| candidate.needs_confirmation)
        );
        assert!(candidates.ambiguities.is_empty());
    }

    #[test]
    fn observed_dependencies_are_distinguished_by_evidence() {
        let candidates = candidates_for("discovery_candidates");
        let resolution = |source: &str, target: &str| {
            candidates
                .observed_dependencies
                .iter()
                .find(|dependency| dependency.source == source && dependency.target == target)
                .map(|dependency| dependency.resolution)
                .expect("dependency must be observed")
        };

        assert_eq!(
            resolution("crates/application/src/lib.rs", "domain"),
            ObservedResolution::Resolved
        );
        assert_eq!(
            resolution("crates/application/src/lib.rs", "serde"),
            ObservedResolution::External
        );
        assert_eq!(
            resolution("web/app.ts", "./util"),
            ObservedResolution::Resolved
        );
        assert_eq!(
            resolution("web/broken.ts", "./absent"),
            ObservedResolution::Unresolved
        );
    }

    #[test]
    fn aliases_and_package_states_are_classified_without_contract() {
        let candidates = candidates_for("resolution_typescript");
        let observed = |target: &str| {
            candidates
                .observed_dependencies
                .iter()
                .find(|dependency| dependency.target == target)
                .unwrap_or_else(|| panic!("dependência observada ausente: {target}"))
        };

        let alias = observed("@domain/entity");
        assert_eq!(alias.resolution, ObservedResolution::Resolved);
        assert!(alias.detail.contains("src/domain/entity.ts"));
        assert_eq!(
            observed("@local/duplo").resolution,
            ObservedResolution::Ambiguous
        );
        assert_eq!(
            observed("@local/ausente").resolution,
            ObservedResolution::Unresolved
        );

        let builtin = observed("node:fs");
        assert_eq!(builtin.resolution, ObservedResolution::External);
        assert!(builtin.detail.contains("embutida"));
        let declared = observed("react");
        assert_eq!(declared.resolution, ObservedResolution::External);
        assert!(declared.detail.contains("externa declarada"));
        assert!(declared.detail.contains("package.json"));
        let unconfirmed = observed("desconhecido/nested");
        assert_eq!(unconfirmed.resolution, ObservedResolution::External);
        assert!(unconfirmed.detail.contains("externa não confirmada"));
        let workspace = observed("@demo/api");
        assert_eq!(workspace.resolution, ObservedResolution::External);
        assert!(workspace.detail.contains("local de workspace"));
    }

    #[test]
    fn python_discovery_keeps_declared_and_unconfirmed_distributions_separate() {
        let candidates = candidates_for("resolution_python");
        let observed = |target: &str| {
            candidates
                .observed_dependencies
                .iter()
                .find(|dependency| dependency.target == target)
                .unwrap_or_else(|| panic!("dependência observada ausente: {target}"))
        };

        assert!(observed("httpx").detail.contains("externa declarada"));
        assert!(
            observed("PIL.Image")
                .detail
                .contains("externa não confirmada")
        );
        assert!(observed("sys").detail.contains("embutida"));
        assert_eq!(
            observed("domain.entity").resolution,
            ObservedResolution::Resolved
        );
    }

    #[test]
    fn duplicate_manifest_names_are_preserved_as_ambiguity() {
        let candidates = candidates_for("discovery_candidates_ambiguous");

        assert!(
            candidates
                .ambiguities
                .iter()
                .any(|ambiguity| { ambiguity.subject == "shared" && ambiguity.options.len() == 2 })
        );
        assert!(
            candidates
                .candidate_modules
                .iter()
                .any(|candidate| candidate.category == EvidenceCategory::Ambiguous)
        );
        let ambiguous = candidates
            .observed_dependencies
            .iter()
            .find(|dependency| {
                dependency.target == "shared"
                    && dependency.resolution == ObservedResolution::Ambiguous
            })
            .expect("duplicate package name must keep the destination ambiguous");
        let name_ambiguity = candidates
            .ambiguities
            .iter()
            .find(|ambiguity| ambiguity.subject == "shared")
            .expect("duplicate package name must be explained");
        assert_eq!(name_ambiguity.options.len(), 2);
        assert!(
            name_ambiguity
                .options
                .iter()
                .all(|option| option.contains("/src/**"))
        );
        assert_eq!(ambiguous.target, "shared");
    }

    #[test]
    fn present_manifests_are_observed_instead_of_reported_as_unsupported() {
        let candidates = candidates_for("discovery_candidates");

        assert!(
            candidates
                .manifests
                .iter()
                .any(|manifest| manifest.path == "package.json")
        );
        assert!(
            candidates
                .limitations
                .iter()
                .all(|limitation| !limitation.contains("ainda não extrai"))
        );
    }

    #[test]
    fn candidate_building_is_deterministic() {
        assert_eq!(
            candidates_for("discovery_candidates"),
            candidates_for("discovery_candidates")
        );
    }

    #[test]
    fn language_coverage_names_the_available_analyzers() {
        let candidates = candidates_for("discovery_candidates");

        assert_eq!(
            candidates.languages,
            vec![
                LanguageCoverage {
                    extension: "rs".to_owned(),
                    files: 2,
                    analyzer: Some("rust".to_owned()),
                },
                LanguageCoverage {
                    extension: "ts".to_owned(),
                    files: 3,
                    analyzer: Some("typescript_javascript".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn limitations_explain_uncovered_files_and_default_scope() {
        let candidates = candidates_for("discovery_no_contract");

        assert!(
            candidates
                .limitations
                .iter()
                .any(|limitation| limitation.contains("script.py"))
        );
        assert!(
            candidates
                .limitations
                .iter()
                .any(|limitation| limitation.contains("scan.include"))
        );
    }

    #[test]
    fn analyzers_are_named_only_for_supported_extensions() {
        assert_eq!(analyzer_for("rs"), Some("rust"));
        assert_eq!(analyzer_for("tsx"), Some("typescript_javascript"));
        assert_eq!(analyzer_for("py"), Some("python"));
        assert_eq!(analyzer_for("go"), None);
    }
}
