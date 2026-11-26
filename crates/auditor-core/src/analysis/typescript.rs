//! Analisador determinístico de importações TypeScript e JavaScript.
//!
//! O scanner não executa código, não usa crate de regex, não lê `tsconfig.json`
//! e não adiciona dependências. Comentários, strings, template literals e regex
//! são mascarados em um documento contínuo; declarações são limitadas pela
//! profundidade de parênteses, colchetes e chaves. Assim um `import` multilinha
//! é uma única declaração, enquanto `import.meta` não consome literais de outras
//! instruções. Qualquer construção que não pode ser resolvida com segurança
//! vira limitação ou item de revisão, nunca conclusão inventada.

use std::collections::HashSet;
use std::path::{Component, Path};

use super::{
    Analysis, DependencyKind, Module, ReviewItem, ReviewReason, find_word, is_word_char,
    record_dependency,
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

/// Literal encontrado no texto mascarado.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Literal {
    value: String,
    dynamic: bool,
}

/// Estado preservado entre linhas para comentários de bloco e template literals.
#[derive(Default)]
struct ScanState {
    block_comment: bool,
    quote: Option<char>,
    literal: String,
    literal_dynamic: bool,
}

/// Contexto mínimo para distinguir regex de divisão.
#[derive(Default)]
struct Previous {
    significant: Option<char>,
    word: String,
}

impl Previous {
    /// Registra um caractere já classificado como código.
    fn note(&mut self, ch: char) {
        if is_word_char(ch) {
            self.word.push(ch);
            return;
        }
        if !self.word.is_empty() {
            let keyword = matches!(
                self.word.as_str(),
                "return"
                    | "throw"
                    | "case"
                    | "typeof"
                    | "instanceof"
                    | "in"
                    | "of"
                    | "yield"
                    | "await"
            );
            self.significant = if keyword { Some('=') } else { Some('a') };
            self.word.clear();
        }
        if !ch.is_whitespace() {
            self.significant = Some(ch);
        }
    }

    /// Indica se um `/` neste ponto provavelmente abre uma regex.
    fn allows_regex(&self) -> bool {
        if !self.word.is_empty() {
            return matches!(
                self.word.as_str(),
                "return"
                    | "throw"
                    | "case"
                    | "typeof"
                    | "instanceof"
                    | "in"
                    | "of"
                    | "yield"
                    | "await"
            );
        }
        matches!(
            self.significant,
            None | Some('=')
                | Some('(')
                | Some('[')
                | Some('{')
                | Some(',')
                | Some(';')
                | Some(':')
                | Some('!')
                | Some('?')
                | Some(')')
                | Some('}')
        )
    }
}

/// Fim de uma regex literal iniciada em `from`, sem usar expressões regulares.
fn regex_end(chars: &[char], from: usize) -> Option<usize> {
    let mut index = from;
    let mut in_class = false;
    while index < chars.len() {
        match chars[index] {
            '\\' => index += 2,
            '[' => {
                in_class = true;
                index += 1;
            }
            ']' => {
                in_class = false;
                index += 1;
            }
            '/' if !in_class => return Some(index),
            '\n' => return None,
            _ => index += 1,
        }
    }
    None
}

/// Substitui comentários, strings, template literals e regex por espaços ou
/// marcadores `\0`, devolvendo os literais na ordem em que aparecem.
fn mask_line(line: &str, state: &mut ScanState) -> (String, Vec<Literal>) {
    let chars: Vec<char> = line.chars().collect();
    let mut masked = String::with_capacity(line.len());
    let mut literals = Vec::new();
    let mut previous = Previous::default();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];

        if state.block_comment {
            if ch == '*' && chars.get(index + 1) == Some(&'/') {
                state.block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if let Some(quote) = state.quote {
            if ch == '\\' {
                if let Some(next) = chars.get(index + 1) {
                    state.literal.push(*next);
                    index += 2;
                } else {
                    index += 1;
                }
                continue;
            }
            if ch == quote {
                literals.push(Literal {
                    value: std::mem::take(&mut state.literal),
                    dynamic: state.literal_dynamic,
                });
                state.quote = None;
                state.literal_dynamic = false;
                masked.push('\0');
            } else {
                state.literal.push(ch);
            }
            index += 1;
            continue;
        }

        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            break;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'*') {
            state.block_comment = true;
            index += 2;
            continue;
        }
        if matches!(ch, '\'' | '"' | '`') {
            state.literal.clear();
            state.literal_dynamic = ch == '`';
            state.quote = Some(ch);
            index += 1;
            continue;
        }
        if ch == '/'
            && previous.allows_regex()
            && let Some(end) = regex_end(&chars, index + 1)
        {
            masked.push(' ');
            previous.note('/');
            index = end + 1;
            continue;
        }

        masked.push(ch);
        previous.note(ch);
        index += 1;
    }

    (masked, literals)
}

/// Devolve o literal seguinte dentro do intervalo da declaração.
fn literal_within(masked: &str, literals: &[Literal], from: usize, end: usize) -> Option<Literal> {
    let rest = masked.get(from..end)?;
    let marker = from + rest.find('\0')?;
    let index = masked[..marker].matches('\0').count();
    literals.get(index).cloned()
}

/// Avança sobre espaços e quebras de linha.
fn skip_whitespace(text: &str, from: usize) -> usize {
    text.get(from..)
        .map(|rest| from + rest.len() - rest.trim_start().len())
        .unwrap_or(from)
}

/// Fim da declaração corrente, respeitando parênteses, colchetes e chaves.
fn statement_end(masked: &str, from: usize) -> usize {
    let mut depth = 0_i32;
    for (offset, ch) in masked.char_indices() {
        if offset < from {
            continue;
        }
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = (depth - 1).max(0),
            ';' | '\n' if depth == 0 => return offset,
            _ => {}
        }
    }
    masked.len()
}

/// Especificador reconhecido em uma declaração.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ParsedSpecifier {
    Static(String),
    DynamicLiteral(String),
    DynamicExpression,
}

/// Declaração encontrada, com deslocamento de início e fim no texto mascarado.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedStatement {
    offset: usize,
    end: usize,
    specifier: ParsedSpecifier,
}

/// Converte um literal em especificador estático ou dinâmico.
fn specifier_from_literal(literal: Literal) -> ParsedSpecifier {
    if literal.dynamic {
        ParsedSpecifier::DynamicLiteral(literal.value)
    } else {
        ParsedSpecifier::Static(literal.value)
    }
}

/// Extrai as declarações de importação/exportação de um documento mascarado.
fn parse_specifiers(masked: &str, literals: &[Literal]) -> Vec<ParsedStatement> {
    let mut found = Vec::new();

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "import", cursor) {
        cursor = index + "import".len();
        let call = skip_whitespace(masked, cursor);
        let next = masked.get(call..).and_then(|rest| rest.chars().next());
        if next == Some('.') {
            continue;
        }
        let end = statement_end(masked, cursor);
        if next == Some('(') {
            let specifier = match literal_within(masked, literals, call + 1, end) {
                Some(literal) => specifier_from_literal(literal),
                None => ParsedSpecifier::DynamicExpression,
            };
            found.push(ParsedStatement {
                offset: index,
                end,
                specifier,
            });
            cursor = end.max(cursor + 1);
            continue;
        }
        if let Some(from) = find_word(&masked[..end], "from", cursor) {
            if let Some(literal) = literal_within(masked, literals, from + "from".len(), end) {
                found.push(ParsedStatement {
                    offset: index,
                    end,
                    specifier: specifier_from_literal(literal),
                });
            }
            cursor = (from + "from".len()).max(cursor + 1);
        } else if let Some(literal) = literal_within(masked, literals, cursor, end) {
            found.push(ParsedStatement {
                offset: index,
                end,
                specifier: specifier_from_literal(literal),
            });
            cursor = end.max(cursor + 1);
        } else {
            cursor = end.max(cursor + 1);
        }
    }

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "export", cursor) {
        cursor = index + "export".len();
        let end = statement_end(masked, cursor);
        if let Some(from) = find_word(&masked[..end], "from", cursor) {
            if let Some(literal) = literal_within(masked, literals, from + "from".len(), end) {
                found.push(ParsedStatement {
                    offset: index,
                    end,
                    specifier: specifier_from_literal(literal),
                });
            }
            cursor = (from + "from".len()).max(cursor + 1);
        } else {
            cursor = end.max(cursor + 1);
        }
    }

    let mut cursor = 0;
    while let Some(index) = find_word(masked, "require", cursor) {
        cursor = index + "require".len();
        let call = skip_whitespace(masked, cursor);
        if masked
            .get(call..)
            .and_then(|rest| rest.chars().next())
            .is_none_or(|ch| ch != '(')
        {
            cursor = cursor.max(index + "require".len() + 1);
            continue;
        }
        let end = statement_end(masked, cursor);
        let specifier = match literal_within(masked, literals, call + 1, end) {
            Some(literal) => specifier_from_literal(literal),
            None => ParsedSpecifier::DynamicExpression,
        };
        found.push(ParsedStatement {
            offset: index,
            end,
            specifier,
        });
        cursor = end.max(cursor + 1);
    }

    found
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

/// Converte um deslocamento no documento mascarado em número de linha 1-based.
fn line_for_offset(line_starts: &[usize], offset: usize) -> usize {
    let mut line = 1;
    for &start in line_starts.iter().skip(1) {
        if offset < start {
            break;
        }
        line += 1;
    }
    line.min(line_starts.len().saturating_sub(1))
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
    let mut masked_document = String::new();
    let mut literals = Vec::new();
    let mut line_starts = Vec::new();
    let mut source_lines = Vec::new();

    for raw_line in text.lines() {
        line_starts.push(masked_document.len());
        source_lines.push(raw_line.trim());
        let (masked, mut line_literals) = mask_line(raw_line, &mut state);
        masked_document.push_str(&masked);
        masked_document.push('\n');
        literals.append(&mut line_literals);
    }
    line_starts.push(masked_document.len());

    for statement in parse_specifiers(&masked_document, &literals) {
        let line = line_for_offset(&line_starts, statement.offset);
        let end_line = line_for_offset(&line_starts, statement.end.saturating_sub(1));
        let first = line.saturating_sub(1);
        let last = end_line.max(line).min(source_lines.len());
        let evidence = source_lines[first..last].join(" ");

        match statement.specifier {
            ParsedSpecifier::Static(specifier) => classify_specifier(
                contract,
                contract_names,
                &files,
                module,
                path,
                line,
                &evidence,
                &specifier,
                analysis,
                seen_edges,
            ),
            ParsedSpecifier::DynamicLiteral(specifier) => {
                analysis.limitations.push(format!(
                    "{path}:{line} especificador de template literal não é resolvido: {specifier}"
                ));
            }
            ParsedSpecifier::DynamicExpression => {
                analysis.limitations.push(format!(
                    "{path}:{line} importação dinâmica sem literal estático não é resolvida"
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Especificadores estáticos de um conjunto de declarações.
    fn static_values(statements: &[ParsedStatement]) -> Vec<&str> {
        statements
            .iter()
            .filter_map(|statement| match &statement.specifier {
                ParsedSpecifier::Static(value) => Some(value.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn comments_and_literals_are_masked_without_losing_specifiers() {
        let mut state = ScanState::default();
        let (masked, literals) = mask_line("// import \"velho\"", &mut state);
        assert!(masked.is_empty());
        assert!(literals.is_empty());

        let (masked, literals) = mask_line("import { X } from \"novo\";", &mut state);
        assert_eq!(masked, "import { X } from \0;");
        assert_eq!(
            literals,
            vec![Literal {
                value: "novo".to_owned(),
                dynamic: false
            }]
        );
    }

    #[test]
    fn block_comments_span_lines() {
        let mut state = ScanState::default();
        let (masked, _) = mask_line("/* inicio", &mut state);
        assert!(masked.is_empty());
        let (masked, literals) = mask_line("import \"x\" */ import \"y\";", &mut state);
        assert_eq!(masked.trim_start(), "import \0;");
        assert_eq!(
            literals,
            vec![Literal {
                value: "y".to_owned(),
                dynamic: false
            }]
        );
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
        let mut statements = Vec::new();
        for form in forms {
            let (masked, literals) = mask_line(form, &mut state);
            statements.extend(parse_specifiers(&masked, &literals));
        }
        assert_eq!(
            static_values(&statements),
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
        assert!(
            statements
                .iter()
                .all(|statement| matches!(statement.specifier, ParsedSpecifier::Static(_)))
        );
    }

    #[test]
    fn multiline_static_import_is_a_single_statement() {
        let mut state = ScanState::default();
        let mut masked_document = String::new();
        let mut literals = Vec::new();
        for line in [
            "import {",
            "  Entity,",
            "  Repository",
            "} from \"./domain\";",
        ] {
            let (masked, mut line_literals) = mask_line(line, &mut state);
            masked_document.push_str(&masked);
            masked_document.push('\n');
            literals.append(&mut line_literals);
        }

        let statements = parse_specifiers(&masked_document, &literals);
        assert_eq!(statements.len(), 1);
        assert_eq!(static_values(&statements), vec!["./domain"]);
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

    #[test]
    fn import_meta_does_not_consume_a_later_literal() {
        let mut state = ScanState::default();
        let (masked, literals) = mask_line(
            "const url = import.meta.url; import \"domain\";",
            &mut state,
        );

        let statements = parse_specifiers(&masked, &literals);
        assert_eq!(statements.len(), 1);
        assert_eq!(static_values(&statements), vec!["domain"]);
    }

    #[test]
    fn regex_literal_is_not_an_import() {
        let mut state = ScanState::default();
        let (masked, literals) = mask_line("const re = /import \"domain\"/;", &mut state);

        assert!(parse_specifiers(&masked, &literals).is_empty());
    }

    #[test]
    fn template_literal_content_is_not_an_import() {
        let mut state = ScanState::default();
        let (masked, literals) = mask_line("const msg = `import \"domain\"`;", &mut state);

        assert!(parse_specifiers(&masked, &literals).is_empty());
    }

    #[test]
    fn dynamic_specifiers_are_distinguished_from_static_ones() {
        let mut state = ScanState::default();
        let (masked, literals) = mask_line("const a = import(`modelo`);", &mut state);
        let statements = parse_specifiers(&masked, &literals);
        assert_eq!(
            statements
                .iter()
                .map(|statement| &statement.specifier)
                .collect::<Vec<_>>(),
            vec![&ParsedSpecifier::DynamicLiteral("modelo".to_owned())]
        );

        let mut state = ScanState::default();
        let (masked, literals) = mask_line("const b = require(alvo);", &mut state);
        let statements = parse_specifiers(&masked, &literals);
        assert_eq!(
            statements
                .iter()
                .map(|statement| &statement.specifier)
                .collect::<Vec<_>>(),
            vec![&ParsedSpecifier::DynamicExpression]
        );
    }
}
