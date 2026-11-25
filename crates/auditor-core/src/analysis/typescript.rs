//! Analisador determinístico de importações TypeScript e JavaScript.
//!
//! O scanner não executa código, não usa regex e não lê `tsconfig.json`.
//! Comentários são descartados e literais viram marcadores: assim uma linha
//! como `// import "x"` não gera aresta, mas `import "x"` gera uma.
//! Qualquer construção que não pode ser resolvida com segurança vira
//! limitação ou item de revisão, nunca conclusão inventada.

use std::collections::HashSet;
use std::path::{Component, Path};

use super::{
    Analysis, DependencyKind, Module, ReviewItem, ReviewReason, find_word, record_dependency,
};
use crate::contract::Contract;
use crate::discovery::{Inventory, matches_pattern};

/// Extensões com analisador disponível.
pub(super) fn is_supported_extension(extension: Option<&str>) -> bool {
    matches!(extension, Some("ts" | "tsx" | "js" | "jsx"))
}

/// Sufixos tentados, em ordem, para um especificador relativo sem extensão.
const RESOLUTION_SUFFIXES: &[&str] = &[
    "",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".d.ts",
    "/index.ts",
    "/index.tsx",
    "/index.js",
    "/index.jsx",
    "/index.d.ts",
];

/// Estado preservado entre linhas para comentários de bloco e template literals.
#[derive(Default)]
struct ScanState {
    block_comment: bool,
    quote: Option<char>,
    literal: String,
}

/// Substitui comentários por espaços e literais por `\0`, devolvendo os literais.
fn mask_line(line: &str, state: &mut ScanState) -> (String, Vec<String>) {
    let mut masked = String::with_capacity(line.len());
    let mut literals = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if state.block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                state.block_comment = false;
            }
            continue;
        }

        if let Some(quote) = state.quote {
            if ch == '\\' {
                if let Some(escaped) = chars.next() {
                    state.literal.push(escaped);
                }
                continue;
            }
            if ch == quote {
                literals.push(std::mem::take(&mut state.literal));
                state.quote = None;
                masked.push('\0');
            } else {
                state.literal.push(ch);
            }
            continue;
        }

        match ch {
            '/' if chars.peek() == Some(&'/') => break,
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                state.block_comment = true;
            }
            '\'' | '"' | '`' => {
                state.literal.clear();
                state.quote = Some(ch);
            }
            _ => masked.push(ch),
        }
    }

    (masked, literals)
}

/// Devolve o literal que aparece depois de `from`, usando os marcadores `\0`.
fn literal_after(masked: &str, from: usize, literals: &[String]) -> Option<String> {
    let marker = from + masked.get(from..)?.find('\0')?;
    let index = masked[..marker].matches('\0').count();
    literals.get(index).cloned()
}

/// Fim do segmento atual, delimitado por ponto e vírgula.
fn segment_end(masked: &str, from: usize) -> usize {
    masked
        .get(from..)
        .and_then(|rest| rest.find(';'))
        .map(|offset| from + offset)
        .unwrap_or(masked.len())
}

/// Extrai os especificadores de importação de uma linha já mascarada.
fn parse_specifiers(masked: &str, literals: &[String]) -> Vec<String> {
    let mut specifiers = Vec::new();

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "import", cursor) {
        cursor = index + "import".len();
        if masked
            .get(cursor..)
            .is_some_and(|rest| rest.trim_start().starts_with('('))
        {
            if let Some(value) = literal_after(masked, cursor, literals) {
                specifiers.push(value);
            }
            continue;
        }

        let end = segment_end(masked, cursor);
        if let Some(from) = find_word(&masked[..end], "from", cursor) {
            if let Some(value) = literal_after(masked, from + "from".len(), literals) {
                specifiers.push(value);
            }
            cursor = from + "from".len();
        } else if let Some(value) = literal_after(masked, cursor, literals) {
            specifiers.push(value);
        }
    }

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "export", cursor) {
        cursor = index + "export".len();
        let end = segment_end(masked, cursor);
        match find_word(&masked[..end], "from", cursor) {
            Some(from) => {
                if let Some(value) = literal_after(masked, from + "from".len(), literals) {
                    specifiers.push(value);
                }
                cursor = from + "from".len();
            }
            None => cursor = end.max(cursor + 1),
        }
    }

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "require", cursor) {
        cursor = index + "require".len();
        if masked
            .get(cursor..)
            .is_some_and(|rest| rest.trim_start().starts_with('('))
            && let Some(value) = literal_after(masked, cursor, literals)
        {
            specifiers.push(value);
        }
    }

    specifiers
}

/// Normaliza `./` e `../` lexicalmente, sem acessar o disco.
fn normalize_relative(path: &str, specifier: &str) -> Option<String> {
    let mut parts: Vec<String> = Path::new(path)
        .parent()?
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    for component in Path::new(specifier).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            _ => return None,
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// Classifica um especificador e registra a aresta ou a incerteza correspondente.
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
    if specifier.starts_with('.') {
        let Some(base) = normalize_relative(path, specifier) else {
            analysis.limitations.push(format!(
                "{path}:{line} especificador relativo fora da raiz: {specifier}"
            ));
            return;
        };

        let resolved = RESOLUTION_SUFFIXES
            .iter()
            .map(|suffix| format!("{base}{suffix}"))
            .find(|candidate| files.contains(candidate.as_str()));

        let Some(resolved_path) = resolved else {
            analysis.limitations.push(format!(
                "{path}:{line} import relativo não resolvido no inventário: {specifier}"
            ));
            return;
        };

        let targets: Vec<&Module> = contract
            .architecture
            .modules
            .iter()
            .filter(|candidate| matches_pattern(&candidate.path, &resolved_path))
            .collect();

        if targets.len() > 1 {
            analysis.reviews.push(ReviewItem {
                path: path.to_owned(),
                line: Some(line),
                reason: ReviewReason::UnresolvedDependency,
                detail: format!("{specifier}: destino ambíguo ({resolved_path})"),
            });
            return;
        }

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
                "destino relativo resolvido e presente em allowed_dependencies".to_owned(),
            )
        } else {
            (
                DependencyKind::Forbidden,
                "destino relativo resolvido no contrato, mas ausente de allowed_dependencies"
                    .to_owned(),
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
        return;
    }

    if contract_names.contains(specifier) {
        record_dependency(
            module,
            specifier,
            DependencyKind::Unresolved,
            "especificador sem caminho relativo pode ser pacote externo ou alias; exige decisão humana"
                .to_owned(),
            Some(line),
            Some(evidence),
            path,
            analysis,
            seen_edges,
        );
    }
}

/// Analisa um arquivo TypeScript/JavaScript e acrescenta arestas e revisões.
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
        let (masked, literals) = mask_line(raw_line, &mut state);
        for specifier in parse_specifiers(&masked, &literals) {
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
    fn comments_and_literals_are_masked_without_losing_specifiers() {
        let mut state = ScanState::default();
        let (masked, literals) = mask_line("// import \"velho\"", &mut state);
        assert!(masked.is_empty());
        assert!(literals.is_empty());

        let (masked, literals) = mask_line("import { X } from \"novo\";", &mut state);
        assert_eq!(masked, "import { X } from \0;");
        assert_eq!(literals, vec!["novo"]);
    }

    #[test]
    fn block_comments_span_lines() {
        let mut state = ScanState::default();
        let (masked, _) = mask_line("/* inicio", &mut state);
        assert!(masked.is_empty());
        let (masked, literals) = mask_line("import \"x\" */ import \"y\";", &mut state);
        assert_eq!(masked.trim_start(), "import \0;");
        assert_eq!(literals, vec!["y"]);
    }

    #[test]
    fn every_import_form_is_recognized() {
        let mut state = ScanState::default();
        let forms = [
            "import X from \"estatico\";",
            "import { Y } from \"grupo\";",
            "import \"efeito\";",
            "export { Z } from \"reexporte\";",
            "const w = require(\"comum\");",
            "const lazy = await import(\"dinamico\");",
            "import \"a\"; import \"b\";",
            "export const valor = 1; export { outro } from \"segundo\";",
        ];
        let mut found = Vec::new();
        for form in forms {
            let (masked, literals) = mask_line(form, &mut state);
            found.extend(parse_specifiers(&masked, &literals));
        }
        assert_eq!(
            found,
            vec![
                "estatico",
                "grupo",
                "efeito",
                "reexporte",
                "comum",
                "dinamico",
                "a",
                "b",
                "segundo"
            ]
        );
    }

    #[test]
    fn relative_paths_are_normalized_without_touching_the_disk() {
        assert_eq!(
            normalize_relative("src/app/use-case.ts", "../domain/entity"),
            Some("src/domain/entity".to_owned())
        );
        assert_eq!(
            normalize_relative("src/app/use-case.ts", "./local"),
            Some("src/app/local".to_owned())
        );
        assert_eq!(normalize_relative("src/app.ts", "../../fora"), None);
    }
}
