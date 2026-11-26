//! Avaliação determinística das regras da Etapa 5.
//!
//! Este módulo transforma os dados de descoberta e análise em achados. Ele não
//! lê arquivos, não altera o projeto auditado e não usa heurísticas: quando a
//! informação é insuficiente, produz um achado `needs_review`.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{Analysis, DependencyKind, ReviewReason};
use crate::contract::Contract;
use crate::discovery::{Inventory, matches_pattern};
use crate::findings::{Finding, Severity};

/// Identificador estável da regra de dependência proibida.
pub const RULE_DEPENDENCY_NOT_ALLOWED: &str = "dep.not_allowed";
/// Identificador estável da regra de dependência não resolvida.
pub const RULE_DEPENDENCY_UNRESOLVED: &str = "dep.unresolved";
/// Identificador estável da regra de arquivo fora dos módulos do contrato.
pub const RULE_FILE_UNMAPPED: &str = "file.unmapped";
/// Identificador estável da regra de arquivo associado a mais de um módulo.
pub const RULE_FILE_AMBIGUOUS_MODULE: &str = "file.ambiguous_module";
/// Identificador estável da regra de ciclo entre módulos.
pub const RULE_DEPENDENCY_CYCLE: &str = "dep.cycle";
/// Identificador estável da regra de módulo sem arquivos correspondentes.
pub const RULE_MODULE_WITHOUT_FILES: &str = "module.no_files";
/// Identificador estável da regra de concentração de dependências.
pub const RULE_MODULE_CONCENTRATION: &str = "module.concentration";
/// Identificador estável da regra de linha de código duplicada.
pub const RULE_CODE_DUPLICATED: &str = "code.duplicated";
/// Identificador estável da regra de módulo sem arestas resolvidas.
pub const RULE_MODULE_UNREFERENCED: &str = "module.unreferenced";

/// Número de módulos distintos que caracteriza concentração de dependências.
const CONCENTRATION_THRESHOLD: usize = 3;

/// Avalia as regras implementadas e devolve achados ordenados por identificador.
pub fn evaluate(contract: &Contract, inventory: &Inventory, analysis: &Analysis) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(forbidden_dependency_findings(contract, analysis));
    findings.extend(unresolved_dependency_findings(analysis));
    findings.extend(review_findings(analysis));
    findings.extend(cycle_findings(analysis));
    findings.extend(module_concentration_findings(contract, analysis));
    findings.extend(duplicated_line_findings(analysis));
    findings.extend(unreferenced_module_findings(contract, inventory, analysis));
    findings.extend(module_without_files_findings(contract, inventory));
    findings.sort_by(|left, right| left.id.cmp(&right.id));
    findings
}

/// Converte arestas proibidas em violações comprovadas.
fn forbidden_dependency_findings(contract: &Contract, analysis: &Analysis) -> Vec<Finding> {
    analysis
        .edges
        .iter()
        .filter(|edge| edge.kind == DependencyKind::Forbidden)
        .map(|edge| {
            let allowed = contract
                .architecture
                .modules
                .iter()
                .find(|module| module.name == edge.from)
                .map(|module| {
                    if module.allowed_dependencies.is_empty() {
                        "nenhuma dependência permitida".to_owned()
                    } else {
                        module.allowed_dependencies.join(", ")
                    }
                })
                .unwrap_or_else(|| "dependências do contrato".to_owned());

            Finding {
                id: format!(
                    "{}:{}:{}:{}:{}",
                    RULE_DEPENDENCY_NOT_ALLOWED,
                    edge.from,
                    edge.to,
                    edge.path,
                    edge.line.unwrap_or(0)
                ),
                rule_id: RULE_DEPENDENCY_NOT_ALLOWED.to_owned(),
                severity: Severity::Error,
                title: format!("Dependência não permitida: {} -> {}", edge.from, edge.to),
                message: format!(
                    "O módulo {} usa {}, mas o contrato não permite essa dependência.",
                    edge.from, edge.to
                ),
                path: edge.path.clone(),
                line_start: edge.line,
                line_end: edge.line,
                evidence: edge.evidence.clone().or_else(|| Some(edge.detail.clone())),
                expected: allowed,
                suggestion: format!(
                    "Revise o uso de {} ou atualize allowed_dependencies de {} no contrato.",
                    edge.to, edge.from
                ),
                confidence: None,
            }
        })
        .collect()
}

/// Converte arestas não resolvidas em revisão necessária.
fn unresolved_dependency_findings(analysis: &Analysis) -> Vec<Finding> {
    analysis
        .edges
        .iter()
        .filter(|edge| edge.kind == DependencyKind::Unresolved)
        .map(|edge| Finding {
            id: format!(
                "{}:{}:{}:{}",
                RULE_DEPENDENCY_UNRESOLVED,
                edge.to,
                edge.path,
                edge.line.unwrap_or(0)
            ),
            rule_id: RULE_DEPENDENCY_UNRESOLVED.to_owned(),
            severity: Severity::NeedsReview,
            title: format!("Dependência não mapeada: {}", edge.to),
            message: format!(
                "O módulo {} usa {}, cujo destino não pôde ser associado com segurança a um único módulo: {}.",
                edge.from, edge.to, edge.detail
            ),
            path: edge.path.clone(),
            line_start: edge.line,
            line_end: edge.line,
            evidence: edge.evidence.clone().or_else(|| Some(edge.detail.clone())),
            expected: "Todo destino deve ser associado de forma inequívoca a um módulo declarado no contrato."
                .to_owned(),
            suggestion: format!(
                "Decida qual módulo deve reivindicar {} ou ajuste os paths do contrato e o uso encontrado.",
                edge.to
            ),
            confidence: None,
        })
        .collect()
}

/// Converte revisões da análise que ainda não viraram achado.
fn review_findings(analysis: &Analysis) -> Vec<Finding> {
    analysis
        .reviews
        .iter()
        .filter_map(|review| match review.reason {
            ReviewReason::UnresolvedDependency => None,
            ReviewReason::UnmappedFile => Some(Finding {
                id: format!("{}:{}", RULE_FILE_UNMAPPED, review.path),
                rule_id: RULE_FILE_UNMAPPED.to_owned(),
                severity: Severity::NeedsReview,
                title: "Arquivo de código fora dos módulos declarados".to_owned(),
                message: format!(
                    "O arquivo {} foi incluído na varredura, mas nenhum módulo do contrato o reivindica.",
                    review.path
                ),
                path: review.path.clone(),
                line_start: None,
                line_end: None,
                evidence: Some(review.detail.clone()),
                expected: "Cada arquivo de código incluído deve corresponder a um módulo declarado."
                    .to_owned(),
                suggestion: "Ajuste o path do módulo no contrato ou mova o arquivo após revisão."
                    .to_owned(),
                confidence: None,
            }),
            ReviewReason::AmbiguousModule => Some(Finding {
                id: format!("{}:{}", RULE_FILE_AMBIGUOUS_MODULE, review.path),
                rule_id: RULE_FILE_AMBIGUOUS_MODULE.to_owned(),
                severity: Severity::NeedsReview,
                title: "Arquivo associado a mais de um módulo".to_owned(),
                message: format!(
                    "O arquivo {} corresponde a mais de um path declarado no contrato.",
                    review.path
                ),
                path: review.path.clone(),
                line_start: None,
                line_end: None,
                evidence: Some(review.detail.clone()),
                expected: "Cada arquivo deve corresponder a no máximo um módulo do contrato."
                    .to_owned(),
                suggestion: "Torne os paths dos módulos mutuamente exclusivos no contrato."
                    .to_owned(),
                confidence: None,
            }),
        })
        .collect()
}

/// Detecta ciclos entre módulos a partir das arestas resolvidas.
fn cycle_findings(analysis: &Analysis) -> Vec<Finding> {
    let adjacency = build_adjacency(analysis);
    let cycles = find_cycles(&adjacency);

    cycles
        .into_iter()
        .map(|cycle| {
            let next_module = cycle.get(1).unwrap_or(&cycle[0]);
            let first_edge = analysis
                .edges
                .iter()
                .find(|edge| edge.from == cycle[0] && edge.to == *next_module);
            let path = first_edge
                .map(|edge| edge.path.clone())
                .unwrap_or_else(|| cycle[0].clone());
            let line = first_edge.and_then(|edge| edge.line);

            Finding {
                id: format!("{}:{}", RULE_DEPENDENCY_CYCLE, cycle.join("->")),
                rule_id: RULE_DEPENDENCY_CYCLE.to_owned(),
                severity: Severity::Warning,
                title: format!("Ciclo entre módulos: {}", cycle.join(" -> ")),
                message: format!("O grafo contém um caminho fechado: {}.", cycle.join(" -> ")),
                path,
                line_start: line,
                line_end: line,
                evidence: Some(cycle_evidence(analysis, &cycle)),
                expected: "O grafo de dependências entre módulos não deve conter ciclos."
                    .to_owned(),
                suggestion:
                    "Inverta uma dependência ou extraia uma interface para quebrar o ciclo."
                        .to_owned(),
                confidence: None,
            }
        })
        .collect()
}

/// Sinaliza módulos com três ou mais origens ou destinos distintos no grafo.
fn module_concentration_findings(contract: &Contract, analysis: &Analysis) -> Vec<Finding> {
    let adjacency = build_adjacency(analysis);
    let mut incoming: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (from, targets) in &adjacency {
        for to in targets {
            incoming
                .entry(to.as_str())
                .or_default()
                .insert(from.as_str());
        }
    }

    contract
        .architecture
        .modules
        .iter()
        .filter_map(|module| {
            let targets = adjacency.get(&module.name);
            let sources = incoming.get(module.name.as_str());
            let outgoing_count = targets.map_or(0, BTreeSet::len);
            let incoming_count = sources.map_or(0, BTreeSet::len);
            if outgoing_count < CONCENTRATION_THRESHOLD
                && incoming_count < CONCENTRATION_THRESHOLD
            {
                return None;
            }

            let mut parts = Vec::new();
            if incoming_count > 0 {
                let names: Vec<&str> = sources
                    .expect("non-zero incoming count must have sources")
                    .iter()
                    .copied()
                    .collect();
                parts.push(format!("origens ({incoming_count}): {}", names.join(", ")));
            }
            if outgoing_count > 0 {
                let names: Vec<&str> = targets
                    .expect("non-zero outgoing count must have targets")
                    .iter()
                    .map(String::as_str)
                    .collect();
                parts.push(format!("destinos ({outgoing_count}): {}", names.join(", ")));
            }

            Some(Finding {
                id: format!("{}:{}", RULE_MODULE_CONCENTRATION, module.name),
                rule_id: RULE_MODULE_CONCENTRATION.to_owned(),
                severity: Severity::NeedsReview,
                title: format!("Concentração de dependências em {}", module.name),
                message: format!(
                    "O módulo {} se relaciona com {} origens e {} destinos distintos no grafo.",
                    module.name, incoming_count, outgoing_count
                ),
                path: module.path.clone(),
                line_start: None,
                line_end: None,
                evidence: Some(parts.join("; ")),
                expected: format!(
                    "Nenhum módulo deve concentrar {CONCENTRATION_THRESHOLD} ou mais dependências diretas distintas."
                ),
                suggestion: "Revise se parte das responsabilidades pode ser movida para outro módulo antes de aceitar esse acoplamento."
                    .to_owned(),
                confidence: None,
            })
        })
        .collect()
}

/// Hash FNV-1a estável para identificar um trecho duplicado sem depender da versão do Rust.
fn stable_hash(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Converte linhas idênticas em revisões, nunca em violações comprovadas.
fn duplicated_line_findings(analysis: &Analysis) -> Vec<Finding> {
    analysis
        .duplicate_lines
        .iter()
        .map(|duplicate| Finding {
            id: format!(
                "{}:{:016x}:{}",
                RULE_CODE_DUPLICATED,
                stable_hash(&duplicate.text),
                duplicate.files.join("|")
            ),
            rule_id: RULE_CODE_DUPLICATED.to_owned(),
            severity: Severity::NeedsReview,
            title: "Trecho de código duplicado".to_owned(),
            message: format!(
                "A mesma linha de código aparece em {} arquivos incluídos.",
                duplicate.files.len()
            ),
            path: duplicate.files[0].clone(),
            line_start: None,
            line_end: None,
            evidence: Some(format!(
                "trecho repetido em {}: {}",
                duplicate.files.join(", "),
                duplicate.text
            )),
            expected: "Trechos idênticos devem ser revisados antes de serem aceitos como duplicação intencional."
                .to_owned(),
            suggestion: "Extraia o trecho para uma função compartilhada ou confirme que a repetição é intencional."
                .to_owned(),
            confidence: None,
        })
        .collect()
}

/// Sinaliza módulos com arquivos, mas sem arestas resolvidas incidentes.
fn unreferenced_module_findings(
    contract: &Contract,
    inventory: &Inventory,
    analysis: &Analysis,
) -> Vec<Finding> {
    let adjacency = build_adjacency(analysis);
    let mut modules_with_edges: BTreeSet<&str> = BTreeSet::new();
    for (from, targets) in &adjacency {
        modules_with_edges.insert(from.as_str());
        for to in targets {
            modules_with_edges.insert(to.as_str());
        }
    }

    let source_files: Vec<&str> = inventory
        .files
        .iter()
        .filter(|file| crate::analysis::is_source_file(file.extension.as_deref()))
        .map(|file| file.path.as_str())
        .collect();

    contract
        .architecture
        .modules
        .iter()
        .filter(|module| {
            !modules_with_edges.contains(module.name.as_str())
                && source_files
                    .iter()
                    .any(|path| matches_pattern(&module.path, path))
        })
        .map(|module| Finding {
            id: format!("{}:{}", RULE_MODULE_UNREFERENCED, module.name),
            rule_id: RULE_MODULE_UNREFERENCED.to_owned(),
            severity: Severity::NeedsReview,
            title: format!("Módulo sem referências no grafo: {}", module.name),
            message: format!(
                "O módulo {} possui arquivos incluídos, mas nenhuma aresta resolvida começa ou termina nele.",
                module.name
            ),
            path: module.path.clone(),
            line_start: None,
            line_end: None,
            evidence: Some("0 arestas resolvidas incidentes".to_owned()),
            expected: "Módulos com arquivos devem participar do grafo ou ter sua exclusão decidida explicitamente."
                .to_owned(),
            suggestion: format!(
                "Revise se {} deve ser integrado ao grafo, removido ou documentado como exceção.",
                module.name
            ),
            confidence: None,
        })
        .collect()
}

/// Identifica módulos declarados sem arquivos de código correspondentes.
fn module_without_files_findings(contract: &Contract, inventory: &Inventory) -> Vec<Finding> {
    let source_files: Vec<&str> = inventory
        .files
        .iter()
        .filter(|file| crate::analysis::is_source_file(file.extension.as_deref()))
        .map(|file| file.path.as_str())
        .collect();

    contract
        .architecture
        .modules
        .iter()
        .filter(|module| {
            !source_files
                .iter()
                .any(|path| matches_pattern(&module.path, path))
        })
        .map(|module| Finding {
            id: format!("{}:{}", RULE_MODULE_WITHOUT_FILES, module.name),
            rule_id: RULE_MODULE_WITHOUT_FILES.to_owned(),
            severity: Severity::Warning,
            title: format!("Módulo sem arquivos: {}", module.name),
            message: format!(
                "Nenhum arquivo de código corresponde ao path declarado por {}.",
                module.name
            ),
            path: module.path.clone(),
            line_start: None,
            line_end: None,
            evidence: Some("0 arquivos de código corresponderam ao path".to_owned()),
            expected:
                "Cada módulo declarado deve ter ao menos um arquivo de código correspondente."
                    .to_owned(),
            suggestion: format!(
                "Revise o path de {} ou crie arquivos dentro da pasta declarada.",
                module.name
            ),
            confidence: None,
        })
        .collect()
}

/// Constrói adjacência ordenada entre módulos, ignorando destinos não resolvidos.
fn build_adjacency(analysis: &Analysis) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in &analysis.edges {
        if edge.kind == DependencyKind::Unresolved {
            continue;
        }
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .insert(edge.to.clone());
    }
    adjacency
}

/// Encontra ciclos simples e remove rotações duplicadas.
fn find_cycles(adjacency: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<Vec<String>> {
    let mut cycles = BTreeSet::new();

    for start in adjacency.keys() {
        let mut stack = vec![start.clone()];
        let mut on_stack = BTreeSet::new();
        on_stack.insert(start.clone());
        visit_cycles(start, adjacency, &mut stack, &mut on_stack, &mut cycles);
    }

    cycles
}

/// Visita caminhos simples a partir de um módulo, registrando fechamentos de ciclo.
fn visit_cycles(
    current: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    stack: &mut Vec<String>,
    on_stack: &mut BTreeSet<String>,
    cycles: &mut BTreeSet<Vec<String>>,
) {
    let Some(next_nodes) = adjacency.get(current) else {
        return;
    };

    for next in next_nodes {
        if on_stack.contains(next) {
            let start_index = stack
                .iter()
                .position(|node| node == next)
                .expect("node in stack must have a position");
            cycles.insert(canonical_cycle(&stack[start_index..]));
            continue;
        }

        on_stack.insert(next.clone());
        stack.push(next.clone());
        visit_cycles(next, adjacency, stack, on_stack, cycles);
        stack.pop();
        on_stack.remove(next);
    }
}

/// Rotaciona um ciclo para começar no menor módulo, tornando-o estável.
fn canonical_cycle(cycle: &[String]) -> Vec<String> {
    let Some(start_index) = cycle
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(index, _)| index)
    else {
        return Vec::new();
    };

    cycle[start_index..]
        .iter()
        .chain(cycle[..start_index].iter())
        .cloned()
        .collect()
}

/// Lista as arestas de um ciclo com arquivo e linha de cada uma.
fn cycle_evidence(analysis: &Analysis, cycle: &[String]) -> String {
    let mut parts = Vec::new();
    for (from, to) in cycle.iter().zip(cycle.iter().cycle().skip(1)) {
        let edge = analysis.edges.iter().find(|edge| {
            edge.kind != DependencyKind::Unresolved && edge.from == *from && edge.to == *to
        });
        let origin = match edge {
            Some(edge) => match edge.line {
                Some(line) => format!("{}:{}", edge.path, line),
                None => edge.path.clone(),
            },
            None => "origem indisponível".to_owned(),
        };
        parts.push(format!("{from} -> {to} ({origin})"));
    }
    parts.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{DependencyEdge, ReviewItem, analyze};
    use crate::contract::{Module, load_contract};
    use crate::discovery::discover;
    use std::path::PathBuf;

    /// Raiz da fixture de dependências, relativa ao crate.
    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/dependencies")
    }

    /// Contrato, inventário e análise da fixture de dependências.
    fn fixture_data() -> (Contract, Inventory, Analysis) {
        let root = project_root();
        let contract =
            load_contract(&root.join("auditor.toml")).expect("fixture contract must load");
        let inventory = discover(&root, &contract).expect("discovery must succeed");
        let analysis = analyze(&root, &contract, &inventory).expect("analysis must succeed");
        (contract, inventory, analysis)
    }

    #[test]
    fn stable_rule_ids_are_declared() {
        assert_eq!(RULE_DEPENDENCY_NOT_ALLOWED, "dep.not_allowed");
        assert_eq!(RULE_DEPENDENCY_UNRESOLVED, "dep.unresolved");
        assert_eq!(RULE_FILE_UNMAPPED, "file.unmapped");
        assert_eq!(RULE_FILE_AMBIGUOUS_MODULE, "file.ambiguous_module");
        assert_eq!(RULE_DEPENDENCY_CYCLE, "dep.cycle");
        assert_eq!(RULE_MODULE_WITHOUT_FILES, "module.no_files");
        assert_eq!(RULE_MODULE_CONCENTRATION, "module.concentration");
        assert_eq!(RULE_CODE_DUPLICATED, "code.duplicated");
        assert_eq!(RULE_MODULE_UNREFERENCED, "module.unreferenced");
    }

    #[test]
    fn fixture_findings_are_classified() {
        let (contract, inventory, analysis) = fixture_data();
        let findings = evaluate(&contract, &inventory, &analysis);

        let count = |rule_id: &str| {
            findings
                .iter()
                .filter(|finding| finding.rule_id == rule_id)
                .count()
        };
        assert_eq!(count(RULE_DEPENDENCY_NOT_ALLOWED), 2);
        assert_eq!(count(RULE_DEPENDENCY_UNRESOLVED), 1);
        assert_eq!(count(RULE_FILE_UNMAPPED), 1);
        assert_eq!(count(RULE_DEPENDENCY_CYCLE), 3);
        assert_eq!(count(RULE_MODULE_WITHOUT_FILES), 0);
        assert_eq!(count(RULE_MODULE_CONCENTRATION), 0);
        assert_eq!(count(RULE_CODE_DUPLICATED), 0);
        assert_eq!(count(RULE_MODULE_UNREFERENCED), 0);
    }

    #[test]
    fn finding_ids_are_unique_and_deterministic() {
        let (contract, inventory, analysis) = fixture_data();
        let first = evaluate(&contract, &inventory, &analysis);
        let second = evaluate(&contract, &inventory, &analysis);
        let ids: Vec<&str> = first.iter().map(|finding| finding.id.as_str()).collect();

        assert_eq!(
            ids,
            second
                .iter()
                .map(|finding| finding.id.as_str())
                .collect::<Vec<_>>()
        );
        assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), ids.len());
    }

    #[test]
    fn forbidden_finding_points_to_verifiable_evidence() {
        let (contract, inventory, analysis) = fixture_data();
        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| {
                finding.rule_id == RULE_DEPENDENCY_NOT_ALLOWED
                    && finding.path == "crates/application/src/forbidden.rs"
            })
            .expect("forbidden finding must exist");

        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.line_start, Some(1));
        assert_eq!(finding.line_end, Some(1));
        assert_eq!(
            finding.evidence.as_deref(),
            Some("use infrastructure::Client;")
        );
        assert!(finding.expected.contains("domain"));
    }

    #[test]
    fn unresolved_dependency_is_a_review_finding() {
        let (contract, inventory, analysis) = fixture_data();
        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| finding.rule_id == RULE_DEPENDENCY_UNRESOLVED)
            .expect("unresolved finding must exist");

        assert_eq!(finding.severity, Severity::NeedsReview);
        assert_eq!(finding.path, "infra/src/unresolved.rs");
        assert_eq!(finding.line_start, Some(1));
        assert!(finding.confidence.is_none());
    }

    #[test]
    fn ambiguous_review_becomes_a_finding() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.reviews.push(ReviewItem {
            path: "crates/domain/src/lib.rs".to_owned(),
            line: None,
            reason: ReviewReason::AmbiguousModule,
            detail: "o arquivo corresponde a mais de um módulo do contrato".to_owned(),
        });

        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| finding.rule_id == RULE_FILE_AMBIGUOUS_MODULE)
            .expect("ambiguous finding must exist");

        assert_eq!(finding.severity, Severity::NeedsReview);
        assert_eq!(finding.path, "crates/domain/src/lib.rs");
    }

    #[test]
    fn cycle_evidence_traces_every_edge() {
        let (contract, inventory, analysis) = fixture_data();
        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .expect("cycle finding must exist");
        let evidence = finding
            .evidence
            .as_deref()
            .expect("cycle must have evidence");

        assert_eq!(finding.severity, Severity::Warning);
        assert!(finding.id.starts_with("dep.cycle:application->domain"));
        assert!(evidence.contains("application -> domain"));
        assert!(evidence.contains("domain -> application"));
        assert!(evidence.contains("crates/application/src/lib.rs:1"));
    }

    #[test]
    fn every_simple_cycle_is_detected_once() {
        let (contract, inventory, analysis) = fixture_data();
        let findings = evaluate(&contract, &inventory, &analysis);
        let cycles: BTreeSet<&str> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .map(|finding| finding.id.as_str())
            .collect();

        assert_eq!(
            cycles,
            [
                "dep.cycle:application->domain",
                "dep.cycle:application->infrastructure",
                "dep.cycle:application->infrastructure->domain",
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn module_without_files_is_reported() {
        let (mut contract, inventory, analysis) = fixture_data();
        contract.architecture.modules.push(Module {
            name: "fantasma".to_owned(),
            path: "nao_existe/**".to_owned(),
            responsibility: "Módulo de teste".to_owned(),
            allowed_dependencies: Vec::new(),
        });

        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| finding.rule_id == RULE_MODULE_WITHOUT_FILES)
            .expect("empty module finding must exist");

        assert_eq!(finding.id, "module.no_files:fantasma");
        assert_eq!(finding.path, "nao_existe/**");
    }

    #[test]
    fn canonical_cycle_starts_at_the_smallest_module() {
        assert_eq!(
            canonical_cycle(&["z".to_owned(), "a".to_owned(), "m".to_owned()]),
            ["a", "m", "z"]
        );
    }

    #[test]
    fn self_loop_is_reported_without_panicking() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.edges.push(DependencyEdge {
            from: "domain".to_owned(),
            to: "domain".to_owned(),
            kind: DependencyKind::Forbidden,
            path: "crates/domain/src/lib.rs".to_owned(),
            line: Some(4),
            evidence: Some("use domain::Entity;".to_owned()),
            detail: "autoimportação".to_owned(),
        });

        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| finding.id == "dep.cycle:domain")
            .expect("self loop must become a cycle finding");

        assert_eq!(finding.path, "crates/domain/src/lib.rs");
        assert_eq!(
            finding.evidence.as_deref(),
            Some("domain -> domain (crates/domain/src/lib.rs:4)")
        );
    }
}
