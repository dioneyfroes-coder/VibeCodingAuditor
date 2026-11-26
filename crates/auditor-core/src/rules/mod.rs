//! Avaliação determinística das regras da Etapa 5.
//!
//! Este módulo transforma os dados de descoberta e análise em achados. Ele não
//! lê arquivos, não altera o projeto auditado e não usa heurísticas: quando a
//! informação é insuficiente, produz um achado `needs_review`.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{Analysis, DependencyEdge, DependencyKind, ReviewReason};
use crate::contract::{Contract, Layer};
use crate::discovery::{Inventory, matches_pattern};
use crate::findings::{Finding, Severity};

/// Identificador estável da regra de dependência proibida.
pub const RULE_DEPENDENCY_NOT_ALLOWED: &str = "dep.not_allowed";
/// Identificador estável da regra de dependência entre camadas proibida.
pub const RULE_LAYER_NOT_ALLOWED: &str = "dep.layer_not_allowed";
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

/// Limite de arestas exibidas como evidência de um componente cíclico.
const CYCLE_EVIDENCE_LIMIT: usize = 8;

/// Avalia as regras implementadas e devolve achados ordenados por identificador.
pub fn evaluate(contract: &Contract, inventory: &Inventory, analysis: &Analysis) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(forbidden_dependency_findings(contract, analysis));
    findings.extend(layer_dependency_findings(contract, analysis));
    findings.extend(unresolved_dependency_findings(analysis));
    findings.extend(review_findings(analysis));
    findings.extend(cycle_findings(analysis));
    findings.extend(module_concentration_findings(contract, analysis));
    findings.extend(duplicated_line_findings(analysis));
    findings.extend(unreferenced_module_findings(contract, inventory, analysis));
    findings.extend(module_without_files_findings(contract, inventory));
    findings.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line_start.cmp(&right.line_start))
            .then_with(|| left.evidence.cmp(&right.evidence))
    });
    let mut seen_ids = BTreeSet::new();
    findings.retain(|finding| seen_ids.insert(finding.id.clone()));
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
                    "{}:{}:{}:{}",
                    RULE_DEPENDENCY_NOT_ALLOWED, edge.from, edge.to, edge.path
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

/// Converte arestas permitidas pelo módulo, mas proibidas pela camada.
fn layer_dependency_findings(contract: &Contract, analysis: &Analysis) -> Vec<Finding> {
    let layers = &contract.architecture.layers;
    if layers.is_empty() {
        return Vec::new();
    }

    let module_layers: BTreeMap<&str, &Layer> = layers
        .iter()
        .flat_map(|layer| {
            layer
                .modules
                .iter()
                .map(move |module| (module.as_str(), layer))
        })
        .collect();

    analysis
        .edges
        .iter()
        .filter(|edge| edge.kind == DependencyKind::Allowed)
        .filter_map(|edge| {
            let source_layer = module_layers.get(edge.from.as_str()).copied()?;
            let target_layer = module_layers.get(edge.to.as_str()).copied()?;
            if source_layer
                .allowed_dependencies
                .iter()
                .any(|name| name == &target_layer.name)
            {
                return None;
            }

            let allowed = if source_layer.allowed_dependencies.is_empty() {
                "nenhuma dependência permitida".to_owned()
            } else {
                source_layer.allowed_dependencies.join(", ")
            };

            Some(Finding {
                id: format!(
                    "{}:{}:{}:{}",
                    RULE_LAYER_NOT_ALLOWED, edge.from, edge.to, edge.path
                ),
                rule_id: RULE_LAYER_NOT_ALLOWED.to_owned(),
                severity: Severity::Error,
                title: format!(
                    "Dependência entre camadas não permitida: {} -> {}",
                    source_layer.name, target_layer.name
                ),
                message: format!(
                    "A camada {} usa {}, mas a camada não permite essa dependência.",
                    source_layer.name, target_layer.name
                ),
                path: edge.path.clone(),
                line_start: edge.line,
                line_end: edge.line,
                evidence: edge.evidence.clone().or_else(|| Some(edge.detail.clone())),
                expected: allowed,
                suggestion: format!(
                    "Revise o uso de {} ou atualize allowed_dependencies da camada {} no contrato.",
                    edge.to, source_layer.name
                ),
                confidence: None,
            })
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
            id: format!("{}:{}:{}", RULE_DEPENDENCY_UNRESOLVED, edge.to, edge.path),
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

/// Detecta componentes fortemente conectados a partir das arestas resolvidas.
fn cycle_findings(analysis: &Analysis) -> Vec<Finding> {
    let adjacency = build_adjacency(analysis);
    let components = find_cyclic_components(&adjacency);

    let mut component_by_node: BTreeMap<String, String> = BTreeMap::new();
    for component in &components {
        let key = component[0].clone();
        for node in component {
            component_by_node.insert(node.clone(), key.clone());
        }
    }

    let mut edges_by_component: BTreeMap<String, Vec<&DependencyEdge>> = BTreeMap::new();
    for edge in &analysis.edges {
        if edge.kind == DependencyKind::Unresolved {
            continue;
        }
        let (Some(from), Some(to)) = (
            component_by_node.get(&edge.from),
            component_by_node.get(&edge.to),
        ) else {
            continue;
        };
        if from == to {
            edges_by_component
                .entry(from.clone())
                .or_default()
                .push(edge);
        }
    }

    components
        .into_iter()
        .map(|component| {
            let mut edges = edges_by_component.remove(&component[0]).unwrap_or_default();
            sort_component_edges(&mut edges);
            let first = edges.first().copied();
            let names = component.join(", ");

            Finding {
                id: format!("{}:{}", RULE_DEPENDENCY_CYCLE, component.join("->")),
                rule_id: RULE_DEPENDENCY_CYCLE.to_owned(),
                severity: Severity::Warning,
                title: format!("Ciclo entre módulos: {names}"),
                message: format!("O grafo contém um componente fortemente conectado: {names}."),
                path: first
                    .map(|edge| edge.path.clone())
                    .unwrap_or_else(|| component[0].clone()),
                line_start: first.and_then(|edge| edge.line),
                line_end: first.and_then(|edge| edge.line),
                evidence: Some(cycle_evidence(&edges)),
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

/// Registra a entrada de um módulo na busca de Tarjan.
fn discover(
    node: &str,
    next_index: &mut usize,
    indices: &mut BTreeMap<String, usize>,
    lowlinks: &mut BTreeMap<String, usize>,
    stack: &mut Vec<String>,
    on_stack: &mut BTreeSet<String>,
) {
    indices.insert(node.to_owned(), *next_index);
    lowlinks.insert(node.to_owned(), *next_index);
    *next_index += 1;
    stack.push(node.to_owned());
    on_stack.insert(node.to_owned());
}

/// Destinos ordenados de um módulo; módulos sem saída têm vizinhos vazios.
fn neighbors(node: &str, adjacency: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    adjacency
        .get(node)
        .map(|targets| targets.iter().cloned().collect())
        .unwrap_or_default()
}

/// Encontra componentes fortemente conectados com Tarjan iterativo.
///
/// A pilha explícita evita estouro em grafos profundos; a ordem dos vértices e
/// das arestas vem de `BTreeMap`/`BTreeSet`, tornando o resultado estável.
fn find_cyclic_components(adjacency: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    let mut nodes: BTreeSet<String> = adjacency.keys().cloned().collect();
    for targets in adjacency.values() {
        nodes.extend(targets.iter().cloned());
    }

    let mut indices: BTreeMap<String, usize> = BTreeMap::new();
    let mut lowlinks: BTreeMap<String, usize> = BTreeMap::new();
    let mut stack: Vec<String> = Vec::new();
    let mut on_stack: BTreeSet<String> = BTreeSet::new();
    let mut components: Vec<Vec<String>> = Vec::new();
    let mut next_index = 0_usize;

    for root in nodes {
        if indices.contains_key(&root) {
            continue;
        }

        discover(
            &root,
            &mut next_index,
            &mut indices,
            &mut lowlinks,
            &mut stack,
            &mut on_stack,
        );
        let mut frames = vec![(root.clone(), neighbors(&root, adjacency), 0_usize)];

        while let Some(frame) = frames.last_mut() {
            let current = frame.0.clone();
            if frame.2 < frame.1.len() {
                let target = frame.1[frame.2].clone();
                frame.2 += 1;

                if !indices.contains_key(&target) {
                    discover(
                        &target,
                        &mut next_index,
                        &mut indices,
                        &mut lowlinks,
                        &mut stack,
                        &mut on_stack,
                    );
                    frames.push((target.clone(), neighbors(&target, adjacency), 0));
                } else if on_stack.contains(&target) {
                    let target_index = indices[&target];
                    let current_low = lowlinks
                        .get_mut(&current)
                        .expect("visited node must have a lowlink");
                    if target_index < *current_low {
                        *current_low = target_index;
                    }
                }
                continue;
            }

            let (finished, _, _) = frames.pop().expect("frame must exist");
            if lowlinks[&finished] == indices[&finished] {
                let mut component = Vec::new();
                loop {
                    let member = stack.pop().expect("stack must contain the component root");
                    on_stack.remove(&member);
                    let is_root = member == finished;
                    component.push(member);
                    if is_root {
                        break;
                    }
                }
                component.sort();
                components.push(component);
            }

            if let Some(parent_frame) = frames.last_mut() {
                let parent = parent_frame.0.clone();
                let child_low = lowlinks[&finished];
                let parent_low = lowlinks
                    .get_mut(&parent)
                    .expect("parent must have a lowlink");
                if child_low < *parent_low {
                    *parent_low = child_low;
                }
            }
        }
    }

    components.retain(|component| {
        component.len() > 1
            || adjacency
                .get(&component[0])
                .is_some_and(|targets| targets.contains(&component[0]))
    });
    components.sort();
    components
}

/// Ordena as arestas internas de um componente e remove duplicatas.
fn sort_component_edges(edges: &mut Vec<&DependencyEdge>) {
    edges.sort_by(|left, right| {
        (
            left.from.as_str(),
            left.to.as_str(),
            left.path.as_str(),
            left.line.unwrap_or(0),
            left.detail.as_str(),
        )
            .cmp(&(
                right.from.as_str(),
                right.to.as_str(),
                right.path.as_str(),
                right.line.unwrap_or(0),
                right.detail.as_str(),
            ))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from
            && left.to == right.to
            && left.path == right.path
            && left.line == right.line
    });
}

/// Evidência limitada das arestas internas de um componente.
fn cycle_evidence(edges: &[&DependencyEdge]) -> String {
    let shown = edges.len().min(CYCLE_EVIDENCE_LIMIT);
    let mut parts: Vec<String> = edges[..shown]
        .iter()
        .map(|edge| {
            let origin = match edge.line {
                Some(line) => format!("{}:{}", edge.path, line),
                None => edge.path.clone(),
            };
            format!("{} -> {} ({})", edge.from, edge.to, origin)
        })
        .collect();

    if edges.len() > shown {
        parts.push(format!(
            "+{} aresta(s) adicionais no componente",
            edges.len() - shown
        ));
    }
    parts.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{DependencyEdge, ReviewItem, analyze};
    use crate::audit::audit_project;
    use crate::contract::{Module, load_contract, parse_contract};
    use crate::discovery::discover;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    /// Aresta sintética usada nos testes de grafo.
    fn synthetic_edge(from: &str, to: &str) -> DependencyEdge {
        DependencyEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            kind: DependencyKind::Allowed,
            path: format!("src/{from}-{to}.rs"),
            line: Some(1),
            evidence: None,
            detail: "aresta sintética de teste".to_owned(),
        }
    }

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

    /// Aresta proibida sintética com linha e evidência configuráveis.
    fn forbidden_edge(line: usize, evidence: &str) -> DependencyEdge {
        DependencyEdge {
            from: "application".to_owned(),
            to: "infrastructure".to_owned(),
            kind: DependencyKind::Forbidden,
            path: "src/lib.rs".to_owned(),
            line: Some(line),
            evidence: Some(evidence.to_owned()),
            detail: "destino declarado no contrato, mas ausente de allowed_dependencies".to_owned(),
        }
    }

    /// Análise com apenas as arestas informadas, para isolar a regra testada.
    fn analysis_with_edges(edges: Vec<DependencyEdge>) -> Analysis {
        let (_, _, mut analysis) = fixture_data();
        analysis.edges = edges;
        analysis.reviews.clear();
        analysis.duplicate_lines.clear();
        analysis
    }

    /// Raiz de uma variante da fixture de identidade de achados.
    fn identity_root(variant: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/finding_identity")
            .join(variant)
    }

    /// Único achado proibido de um relatório real da fixture de identidade.
    fn only_forbidden(root: &Path) -> Finding {
        let report = audit_project(root).expect("identity fixture audit must succeed");
        let mut findings = report
            .findings
            .into_iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_NOT_ALLOWED);
        let finding = findings
            .next()
            .expect("identity fixture must have a forbidden finding");
        assert!(
            findings.next().is_none(),
            "identity fixture must have exactly one forbidden finding"
        );
        finding
    }

    /// Contrato v2 em memória com quatro módulos e quatro camadas.
    fn layer_contract() -> Contract {
        parse_contract(
            r#"
schema_version = 2

[project]
name = "camadas"

[scan]
include = ["src/**/*.rs", "web/src/**/*.ts"]
exclude = []

[architecture]
style = "modular_monolith"

[[architecture.modules]]
name = "domain"
path = "src/domain/**"
responsibility = "Domínio"
allowed_dependencies = []

[[architecture.modules]]
name = "application"
path = "src/application/**"
responsibility = "Aplicação"
allowed_dependencies = ["domain"]

[[architecture.modules]]
name = "infrastructure"
path = "src/infrastructure/**"
responsibility = "Infraestrutura"
allowed_dependencies = ["domain", "application"]

[[architecture.modules]]
name = "web"
path = "web/src/**"
responsibility = "Cliente"
allowed_dependencies = ["application", "infrastructure", "domain"]

[[architecture.layers]]
name = "domain"
modules = ["domain"]
allowed_dependencies = []

[[architecture.layers]]
name = "application"
modules = ["application"]
allowed_dependencies = ["domain"]

[[architecture.layers]]
name = "infrastructure"
modules = ["infrastructure"]
allowed_dependencies = ["domain", "application"]

[[architecture.layers]]
name = "presentation"
modules = ["web"]
allowed_dependencies = ["application"]
"#,
        )
        .expect("layer contract must be valid")
    }

    /// Inventário vazio para isolar as regras que não dependem de arquivos.
    fn contract_inventory(_contract: &Contract) -> Inventory {
        Inventory {
            root: PathBuf::new(),
            files: Vec::new(),
            skipped: Vec::new(),
        }
    }

    /// Aresta sintética com origem, destino, tipo e arquivo configuráveis.
    fn typed_edge(from: &str, to: &str, kind: DependencyKind) -> DependencyEdge {
        DependencyEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            kind,
            path: format!("src/{from}-{to}.rs"),
            line: Some(3),
            evidence: Some(format!("use {to}::Client;")),
            detail: "aresta sintética de teste".to_owned(),
        }
    }

    #[test]
    fn layer_rule_flags_allowed_module_but_forbidden_layer() {
        let contract = layer_contract();
        let analysis = analysis_with_edges(vec![typed_edge(
            "web",
            "infrastructure",
            DependencyKind::Allowed,
        )]);
        let findings = evaluate(&contract, &contract_inventory(&contract), &analysis);
        let layer: Vec<&Finding> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_LAYER_NOT_ALLOWED)
            .collect();

        assert_eq!(layer.len(), 1);
        assert_eq!(
            layer[0].id,
            "dep.layer_not_allowed:web:infrastructure:src/web-infrastructure.rs"
        );
        assert_eq!(layer[0].severity, Severity::Error);
        assert_eq!(layer[0].line_start, Some(3));
        assert_eq!(
            layer[0].evidence.as_deref(),
            Some("use infrastructure::Client;")
        );
        assert!(layer[0].message.contains("presentation"));
        assert!(layer[0].message.contains("infrastructure"));
        assert_eq!(layer[0].expected, "application");
    }

    #[test]
    fn forbidden_module_does_not_get_a_second_layer_error() {
        let contract = layer_contract();
        let analysis =
            analysis_with_edges(vec![typed_edge("web", "domain", DependencyKind::Forbidden)]);
        let findings = evaluate(&contract, &contract_inventory(&contract), &analysis);
        let layer_count = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_LAYER_NOT_ALLOWED)
            .count();
        let module_count = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_NOT_ALLOWED)
            .count();

        assert_eq!(layer_count, 0);
        assert_eq!(module_count, 1);
    }

    #[test]
    fn allowed_module_and_layer_produce_no_layer_finding() {
        let contract = layer_contract();
        let analysis = analysis_with_edges(vec![typed_edge(
            "web",
            "application",
            DependencyKind::Allowed,
        )]);
        let findings = evaluate(&contract, &contract_inventory(&contract), &analysis);

        assert!(
            findings
                .iter()
                .all(|finding| finding.rule_id != RULE_LAYER_NOT_ALLOWED)
        );
    }

    #[test]
    fn unresolved_edge_does_not_produce_a_layer_finding() {
        let contract = layer_contract();
        let analysis = analysis_with_edges(vec![typed_edge(
            "web",
            "externo",
            DependencyKind::Unresolved,
        )]);
        let findings = evaluate(&contract, &contract_inventory(&contract), &analysis);
        let unresolved = findings
            .iter()
            .any(|finding| finding.rule_id == RULE_DEPENDENCY_UNRESOLVED);
        let layer = findings
            .iter()
            .any(|finding| finding.rule_id == RULE_LAYER_NOT_ALLOWED);

        assert!(unresolved);
        assert!(!layer);
    }

    #[test]
    fn v1_contract_ignores_layer_rule() {
        let (contract, inventory, _) = fixture_data();
        let analysis = analysis_with_edges(vec![typed_edge(
            "application",
            "infrastructure",
            DependencyKind::Allowed,
        )]);
        let findings = evaluate(&contract, &inventory, &analysis);

        assert!(
            findings
                .iter()
                .all(|finding| finding.rule_id != RULE_LAYER_NOT_ALLOWED)
        );
    }

    #[test]
    fn layer_findings_are_deterministic() {
        let contract = layer_contract();
        let inventory = contract_inventory(&contract);
        let analysis = analysis_with_edges(vec![
            typed_edge("web", "infrastructure", DependencyKind::Allowed),
            typed_edge("web", "domain", DependencyKind::Allowed),
        ]);
        let first = evaluate(&contract, &inventory, &analysis);
        let second = evaluate(&contract, &inventory, &analysis);

        assert_eq!(first, second);
        let ids: Vec<&str> = first
            .iter()
            .filter(|finding| finding.rule_id == RULE_LAYER_NOT_ALLOWED)
            .map(|finding| finding.id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "dep.layer_not_allowed:web:domain:src/web-domain.rs",
                "dep.layer_not_allowed:web:infrastructure:src/web-infrastructure.rs",
            ]
        );
    }

    #[test]
    fn stable_rule_ids_are_declared() {
        assert_eq!(RULE_DEPENDENCY_NOT_ALLOWED, "dep.not_allowed");
        assert_eq!(RULE_LAYER_NOT_ALLOWED, "dep.layer_not_allowed");
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
        assert_eq!(count(RULE_DEPENDENCY_CYCLE), 1);
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
    fn inserted_comment_lines_do_not_create_a_new_finding() {
        let before = only_forbidden(&identity_root("before"));
        let after = only_forbidden(&identity_root("after"));

        assert_eq!(before.id, after.id);
        assert_eq!(before.path, after.path);
        assert_eq!(before.line_start, Some(1));
        assert_eq!(after.line_start, Some(4));
    }

    #[test]
    fn changed_evidence_without_new_binding_keeps_identity() {
        let before = only_forbidden(&identity_root("before"));
        let changed = only_forbidden(&identity_root("evidence_changed"));

        assert_eq!(before.id, changed.id);
        assert_eq!(before.line_start, changed.line_start);
        assert_ne!(before.evidence, changed.evidence);
    }

    #[test]
    fn identity_fixture_audit_is_deterministic() {
        let root = identity_root("before");
        let first = audit_project(&root).expect("first audit must succeed");
        let second = audit_project(&root).expect("second audit must succeed");
        let first_ids: Vec<&str> = first
            .findings
            .iter()
            .map(|finding| finding.id.as_str())
            .collect();
        let second_ids: Vec<&str> = second
            .findings
            .iter()
            .map(|finding| finding.id.as_str())
            .collect();

        assert_eq!(first_ids, second_ids);
    }

    #[test]
    fn repeated_forbidden_binding_is_reported_once() {
        let (contract, inventory, _) = fixture_data();
        let analysis = analysis_with_edges(vec![
            forbidden_edge(1, "use infrastructure::Client;"),
            forbidden_edge(9, "use infrastructure::Client as Other;"),
        ]);
        let findings = evaluate(&contract, &inventory, &analysis);
        let forbidden: Vec<&Finding> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_NOT_ALLOWED)
            .collect();

        assert_eq!(forbidden.len(), 1);
        assert_eq!(forbidden[0].line_start, Some(1));
        assert_eq!(
            forbidden[0].evidence.as_deref(),
            Some("use infrastructure::Client;")
        );
    }

    #[test]
    fn identity_changes_with_source_target_or_rule() {
        let (contract, _, _) = fixture_data();
        let base = forbidden_dependency_findings(
            &contract,
            &analysis_with_edges(vec![forbidden_edge(1, "use infrastructure::Client;")]),
        )
        .remove(0)
        .id;

        let mut changed_source = forbidden_edge(1, "use infrastructure::Client;");
        changed_source.from = "domain".to_owned();
        let changed_source_id =
            forbidden_dependency_findings(&contract, &analysis_with_edges(vec![changed_source]))
                .remove(0)
                .id;

        let mut changed_target = forbidden_edge(1, "use infrastructure::Client;");
        changed_target.to = "domain".to_owned();
        let changed_target_id =
            forbidden_dependency_findings(&contract, &analysis_with_edges(vec![changed_target]))
                .remove(0)
                .id;

        let mut unresolved = forbidden_edge(1, "use infrastructure::Client;");
        unresolved.kind = DependencyKind::Unresolved;
        unresolved.to = "external_sdk".to_owned();
        let changed_rule_id =
            unresolved_dependency_findings(&analysis_with_edges(vec![unresolved]))
                .remove(0)
                .id;

        assert_ne!(base, changed_source_id);
        assert_ne!(base, changed_target_id);
        assert_ne!(base, changed_rule_id);
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
    fn cycle_evidence_traces_component_edges() {
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
        assert_eq!(finding.id, "dep.cycle:application->domain->infrastructure");
        assert_eq!(finding.path, "crates/application/src/lib.rs");
        assert_eq!(finding.line_start, Some(1));
        for expected in [
            "application -> domain",
            "application -> infrastructure",
            "domain -> application",
            "infrastructure -> application",
            "infrastructure -> domain",
            "crates/application/src/lib.rs:1",
        ] {
            assert!(evidence.contains(expected), "evidence faltou: {expected}");
        }
    }

    #[test]
    fn one_finding_per_cyclic_component() {
        let (contract, inventory, analysis) = fixture_data();
        let findings = evaluate(&contract, &inventory, &analysis);
        let cycles: Vec<&str> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .map(|finding| finding.id.as_str())
            .collect();

        assert_eq!(cycles, ["dep.cycle:application->domain->infrastructure"]);
    }

    #[test]
    fn acyclic_and_disconnected_graphs_have_no_cycle_findings() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.edges.clear();
        for (from, to) in [("a", "b"), ("c", "d"), ("e", "f")] {
            analysis.edges.push(synthetic_edge(from, to));
        }

        let findings = evaluate(&contract, &inventory, &analysis);
        assert!(
            findings
                .iter()
                .all(|finding| finding.rule_id != RULE_DEPENDENCY_CYCLE)
        );
    }

    #[test]
    fn disconnected_cyclic_components_are_reported_separately() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.edges.clear();
        for (from, to) in [
            ("b", "a"),
            ("a", "b"),
            ("d", "c"),
            ("c", "d"),
            ("g", "g"),
            ("e", "f"),
        ] {
            analysis.edges.push(synthetic_edge(from, to));
        }

        let findings = evaluate(&contract, &inventory, &analysis);
        let cycles: Vec<&str> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .map(|finding| finding.id.as_str())
            .collect();

        assert_eq!(cycles, ["dep.cycle:a->b", "dep.cycle:c->d", "dep.cycle:g"]);
    }

    #[test]
    fn dense_graph_cycle_detection_is_bounded() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.edges.clear();
        let nodes: Vec<String> = (0..80).map(|index| format!("m{index:03}")).collect();
        for from in &nodes {
            for to in &nodes {
                analysis.edges.push(synthetic_edge(from, to));
            }
        }

        let started = Instant::now();
        let findings = evaluate(&contract, &inventory, &analysis);
        let elapsed = started.elapsed();
        let cycle = findings
            .iter()
            .find(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .expect("dense graph must have one cyclic component");
        let cycles: Vec<&str> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .map(|finding| finding.id.as_str())
            .collect();

        assert_eq!(cycles, [format!("dep.cycle:{}", nodes.join("->")).as_str()]);
        assert!(
            elapsed < Duration::from_secs(5),
            "SCC demorou {elapsed:?} em grafo denso"
        );
        let evidence = cycle.evidence.as_deref().expect("cycle must have evidence");
        assert!(evidence.contains("+6392 aresta(s) adicionais no componente"));
    }

    #[test]
    fn many_cyclic_components_are_indexed_once_in_bounded_time() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.edges.clear();
        for index in 0..600 {
            let left = format!("par{index:03}a");
            let right = format!("par{index:03}b");
            analysis.edges.push(synthetic_edge(&left, &right));
            analysis.edges.push(synthetic_edge(&right, &left));
        }

        let started = Instant::now();
        let findings = evaluate(&contract, &inventory, &analysis);
        let elapsed = started.elapsed();
        let cycles: Vec<&str> = findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_CYCLE)
            .map(|finding| finding.id.as_str())
            .collect();

        assert_eq!(cycles.len(), 600);
        assert_eq!(cycles.first(), Some(&"dep.cycle:par000a->par000b"));
        assert_eq!(cycles.last(), Some(&"dep.cycle:par599a->par599b"));
        assert!(
            elapsed < Duration::from_secs(10),
            "SCC demorou {elapsed:?} em 600 componentes"
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
    fn self_loop_is_reported_without_panicking() {
        let (contract, inventory, mut analysis) = fixture_data();
        analysis.edges.clear();
        analysis.edges.push(synthetic_edge("isolado", "isolado"));

        let findings = evaluate(&contract, &inventory, &analysis);
        let finding = findings
            .iter()
            .find(|finding| finding.id == "dep.cycle:isolado")
            .expect("self loop must become a cycle finding");

        assert_eq!(finding.path, "src/isolado-isolado.rs");
        assert_eq!(finding.line_start, Some(1));
        assert_eq!(
            finding.evidence.as_deref(),
            Some("isolado -> isolado (src/isolado-isolado.rs:1)")
        );
    }
}
