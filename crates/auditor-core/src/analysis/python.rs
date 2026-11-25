//! Analisador determinístico de importações Python.
//!
//! O scanner não executa código do projeto: comentários `#` e strings,
//! inclusive triplas, são mascarados antes da leitura. Nomes absolutos só são
//! considerados quando o primeiro segmento é um módulo do contrato; nomes
//! relativos são resolvidos contra o inventário. O que não pode ser resolvido
//! vira revisão ou limitação, nunca uma conclusão inventada.

use std::collections::HashSet;
use std::path::{Component, Path};

use super::{
    Analysis, DependencyKind, Module, ReviewItem, ReviewReason, find_word, record_dependency,
};
use crate::contract::Contract;
use crate::discovery::{Inventory, matches_pattern};

/// Extensões com analisador disponível.
pub(super) fn is_supported_extension(extension: Option<&str>) -> bool {
    extension == Some("py")
}

/// Sufixos de arquivo tentados para um caminho de módulo Python.
const MODULE_SUFFIXES: &[&str] = &[".py", "/__init__.py"];

/// Estado preservado entre linhas para strings triplas.
#[derive(Default)]
struct ScanState {
    quote: Option<char>,
    triple: bool,
}

/// Substitui comentários e strings por marcadores, preservando o código.
fn mask_line(line: &str, state: &mut ScanState) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut masked = String::with_capacity(line.len());
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];

        if let Some(quote) = state.quote {
            if ch == '\\' {
                index += 2;
                continue;
            }
            if state.triple {
                if ch == quote
                    && index + 2 < chars.len()
                    && chars[index + 1] == quote
                    && chars[index + 2] == quote
                {
                    state.quote = None;
                    state.triple = false;
                    index += 3;
                    continue;
                }
            } else if ch == quote {
                state.quote = None;
            }
            index += 1;
            continue;
        }

        match ch {
            '#' => break,
            '\'' | '"' => {
                masked.push('\0');
                if index + 2 < chars.len() && chars[index + 1] == ch && chars[index + 2] == ch {
                    state.triple = true;
                    index += 3;
                } else {
                    state.triple = false;
                    index += 1;
                }
                state.quote = Some(ch);
            }
            _ => {
                masked.push(ch);
                index += 1;
            }
        }
    }

    masked
}

/// Fim do segmento atual, delimitado por ponto e vírgula.
fn segment_end(masked: &str, from: usize) -> usize {
    masked
        .get(from..)
        .and_then(|rest| rest.find(';'))
        .map(|offset| from + offset)
        .unwrap_or(masked.len())
}

/// Avança sobre espaços ASCII dentro do segmento.
fn skip_spaces(masked: &str, mut position: usize, end: usize) -> usize {
    while position < end
        && masked
            .as_bytes()
            .get(position)
            .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        position += 1;
    }
    position
}

/// Lê um caminho pontuado formado por identificadores ASCII.
fn parse_dotted(text: &str, from: usize, end: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let mut position = from;
    let mut value = String::new();
    let mut after_dot = true;

    while position < end {
        let byte = bytes[position];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            value.push(byte as char);
            after_dot = false;
            position += 1;
        } else if byte == b'.' && !after_dot {
            value.push('.');
            after_dot = true;
            position += 1;
        } else {
            break;
        }
    }

    if value.is_empty() || after_dot {
        None
    } else {
        Some((value, position))
    }
}

/// Extrai o módulo de um `from ... import ...`.
fn from_specifier(masked: &str, start: usize, import_at: usize) -> Option<String> {
    let raw = masked.get(start..import_at)?.trim();
    if raw.is_empty() {
        return None;
    }

    let mut dots = 0;
    while raw.as_bytes().get(dots) == Some(&b'.') {
        dots += 1;
    }
    if dots == raw.len() {
        return Some(".".repeat(dots));
    }
    let (path, _) = parse_dotted(raw, dots, raw.len())?;
    Some(format!("{}{}", ".".repeat(dots), path))
}

/// Extrai os módulos referenciados por `import` e `from ... import`.
fn parse_specifiers(masked: &str) -> Vec<String> {
    let mut specifiers = Vec::new();
    let mut consumed = Vec::new();

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "from", cursor) {
        let start = index + "from".len();
        let end = segment_end(masked, start);
        let Some(import_at) = masked
            .get(start..end)
            .and_then(|segment| find_word(segment, "import", 0))
            .map(|at| start + at)
        else {
            cursor = start.max(index + 1);
            continue;
        };
        if let Some(specifier) = from_specifier(masked, start, import_at) {
            specifiers.push(specifier);
        }
        consumed.push((index, import_at + "import".len()));
        cursor = import_at + "import".len();
    }

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "import", cursor) {
        cursor = index + "import".len();
        if consumed
            .iter()
            .any(|(start, end)| index >= *start && index < *end)
        {
            continue;
        }

        let end = segment_end(masked, cursor);
        let mut position = skip_spaces(masked, cursor, end);
        while position < end {
            let Some((path, next)) = parse_dotted(masked, position, end) else {
                break;
            };
            specifiers.push(path);
            position = skip_spaces(masked, next, end);

            while position < end && masked.as_bytes()[position] != b',' {
                position += 1;
            }

            if position < end && masked.as_bytes()[position] == b',' {
                position = skip_spaces(masked, position + 1, end);
                continue;
            }
            break;
        }
        cursor = end.max(cursor + 1);
    }

    specifiers
}

/// Converte `..pacote.modulo` em um caminho relativo à raiz, sem tocar o disco.
fn resolve_relative(path: &str, specifier: &str) -> Option<String> {
    let dots = specifier.chars().take_while(|ch| *ch == '.').count();
    if dots == 0 {
        return None;
    }

    let mut parts: Vec<String> = Path::new(path)
        .parent()?
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    if dots > parts.len() {
        return None;
    }
    for _ in 1..dots {
        parts.pop()?;
    }

    let rest = &specifier[dots..];
    if !rest.is_empty() {
        for component in rest.split('.') {
            if component.is_empty() {
                return None;
            }
            parts.push(component.to_owned());
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// Caminhos de arquivo possíveis para uma base de módulo Python.
fn candidate_files(base: &str) -> Vec<String> {
    MODULE_SUFFIXES
        .iter()
        .map(|suffix| format!("{base}{suffix}"))
        .collect()
}

/// Encontra arquivos do inventário que correspondem a um módulo absoluto.
fn absolute_files(files: &HashSet<&str>, dotted: &str) -> Vec<String> {
    let stem = dotted.replace('.', "/");
    let mut found = Vec::new();

    for suffix in MODULE_SUFFIXES {
        let candidate = format!("{stem}{suffix}");
        for file in files {
            if *file == candidate || file.ends_with(&format!("/{candidate}")) {
                found.push((*file).to_owned());
            }
        }
    }

    found.sort();
    found.dedup();
    found
}

/// Classifica um especificador e registra arestas, revisões ou limitações.
#[allow(clippy::too_many_arguments)]
fn classify_specifier(
    contract: &Contract,
    contract_names: &HashSet<String>,
    files: &HashSet<&str>,
    module: &Module,
    path: &str,
    line: usize,
    evidence: &str,
    specifier: &str,
    analysis: &mut Analysis,
    seen_edges: &mut HashSet<String>,
) {
    let relative = specifier.starts_with('.');
    let dots = specifier.chars().take_while(|ch| *ch == '.').count();

    if relative && dots == specifier.len() {
        analysis.limitations.push(format!(
            "{path}:{line} import relativo sem módulo nomeado não gera aresta"
        ));
        return;
    }

    let found: Vec<String> = if relative {
        let Some(base) = resolve_relative(path, specifier) else {
            analysis.limitations.push(format!(
                "{path}:{line} import relativo fora da raiz: {specifier}"
            ));
            return;
        };
        candidate_files(&base)
            .into_iter()
            .filter(|candidate| files.contains(candidate.as_str()))
            .collect()
    } else {
        let Some(first) = specifier.split('.').next() else {
            return;
        };
        if !contract_names.contains(first) {
            return;
        }
        absolute_files(files, specifier)
    };

    if found.is_empty() {
        if relative {
            analysis.limitations.push(format!(
                "{path}:{line} import relativo não resolvido no inventário: {specifier}"
            ));
        } else {
            analysis.reviews.push(ReviewItem {
                path: path.to_owned(),
                line: Some(line),
                reason: ReviewReason::UnresolvedDependency,
                detail: format!(
                    "{specifier}: módulo do contrato sem arquivo Python correspondente"
                ),
            });
        }
        return;
    }

    if found.len() > 1 {
        analysis.reviews.push(ReviewItem {
            path: path.to_owned(),
            line: Some(line),
            reason: ReviewReason::UnresolvedDependency,
            detail: format!("{specifier}: destino ambíguo ({})", found.join(", ")),
        });
        return;
    }

    let resolved_path = &found[0];
    let targets: Vec<&Module> = contract
        .architecture
        .modules
        .iter()
        .filter(|candidate| matches_pattern(&candidate.path, resolved_path))
        .collect();

    let Some(target_module) = targets.first().copied() else {
        analysis.reviews.push(ReviewItem {
            path: path.to_owned(),
            line: Some(line),
            reason: ReviewReason::UnresolvedDependency,
            detail: format!(
                "{specifier}: arquivo resolvido {resolved_path} não pertence a nenhum módulo do contrato"
            ),
        });
        return;
    };

    let target = target_module.name.as_str();
    if target == module.name {
        return;
    }

    let (kind, detail) = if module
        .allowed_dependencies
        .iter()
        .any(|allowed| allowed == target)
    {
        (
            DependencyKind::Allowed,
            "destino Python resolvido e presente em allowed_dependencies".to_owned(),
        )
    } else {
        (
            DependencyKind::Forbidden,
            "destino Python resolvido no contrato, mas ausente de allowed_dependencies".to_owned(),
        )
    };

    record_dependency(
        module,
        target,
        kind,
        detail,
        Some(line),
        Some(evidence),
        path,
        analysis,
        seen_edges,
    );
}

/// Analisa um arquivo Python e acrescenta arestas, revisões e limitações.
#[allow(clippy::too_many_arguments)]
pub(super) fn analyze_text(
    contract: &Contract,
    contract_names: &HashSet<String>,
    inventory: &Inventory,
    path: &str,
    text: &str,
    module: Option<&Module>,
    analysis: &mut Analysis,
    seen_edges: &mut HashSet<String>,
) {
    let Some(module) = module else {
        return;
    };

    let files: HashSet<&str> = inventory
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    let mut state = ScanState::default();

    for (index, raw_line) in text.lines().enumerate() {
        let masked = mask_line(raw_line, &mut state);
        for specifier in parse_specifiers(&masked) {
            classify_specifier(
                contract,
                contract_names,
                &files,
                module,
                path,
                index + 1,
                raw_line.trim(),
                &specifier,
                analysis,
                seen_edges,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_and_triple_strings_are_masked() {
        let mut state = ScanState::default();
        assert_eq!(mask_line("# from domain.entity import X", &mut state), "");
        assert_eq!(mask_line("texto = \"\"\"", &mut state), "texto = \0");
        assert_eq!(mask_line("from domain.entity import X", &mut state), "");
        assert_eq!(mask_line("\"\"\"", &mut state), "");
        assert_eq!(
            mask_line("from infra.client import Y", &mut state),
            "from infra.client import Y"
        );
    }

    #[test]
    fn absolute_and_relative_imports_are_recognized() {
        let masked = mask_line(
            "import domain.entity, infra.client as client",
            &mut ScanState::default(),
        );
        assert_eq!(
            parse_specifiers(&masked),
            vec!["domain.entity", "infra.client"]
        );

        let masked = mask_line(
            "from ..domain.entity import Entity",
            &mut ScanState::default(),
        );
        assert_eq!(parse_specifiers(&masked), vec!["..domain.entity"]);

        let masked = mask_line("from . import local", &mut ScanState::default());
        assert_eq!(parse_specifiers(&masked), vec!["."]);
    }

    #[test]
    fn from_import_does_not_create_a_second_specifier() {
        let masked = mask_line(
            "from domain.entity import Entity",
            &mut ScanState::default(),
        );
        assert_eq!(parse_specifiers(&masked), vec!["domain.entity"]);
    }

    #[test]
    fn relative_paths_are_normalized_without_touching_the_disk() {
        assert_eq!(
            resolve_relative("src/application/use_case.py", "..domain.entity"),
            Some("src/domain/entity".to_owned())
        );
        assert_eq!(
            resolve_relative("src/application/use_case.py", ".dominio"),
            Some("src/application/dominio".to_owned())
        );
        assert_eq!(resolve_relative("src/use_case.py", "..fora"), None);
    }
}
