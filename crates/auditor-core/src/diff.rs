//! Interpreta diffs unificados do Git sem executar `git` nem tocar arquivos.
//!
//! A entrada é o texto produzido por `git diff`. O módulo devolve apenas os
//! caminhos e intervalos de linhas adicionadas, usados para limitar achados a
//! uma revisão incremental. Nenhuma regra de auditoria vive aqui.

use std::error::Error;
use std::fmt;

use crate::discovery::matches_pattern;
use crate::findings::Finding;
use crate::report::AuditReport;

/// Intervalo inclusivo de linhas no arquivo depois do diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    /// Primeira linha do intervalo.
    pub start: usize,
    /// Última linha do intervalo.
    pub end: usize,
}

impl LineRange {
    /// Indica se o intervalo contém a linha informada.
    fn contains(self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }
}

/// Arquivo alterado por um diff e linhas adicionadas nele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    /// Caminho relativo à raiz, sem os prefixos `a/` e `b/`.
    pub path: String,
    /// Intervalos de linhas adicionadas, fundidos e ordenados.
    pub lines: Vec<LineRange>,
}

/// Conjunto de arquivos alterados por um diff.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChangedFiles {
    /// Arquivos alterados, na ordem em que aparecem no diff.
    pub files: Vec<ChangedFile>,
}

impl ChangedFiles {
    /// Número de arquivos alterados.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Indica que o diff não contém arquivos alterados.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Indica se um achado pertence a algum arquivo ou linha alterada.
    fn matches_finding(&self, finding: &Finding) -> bool {
        self.files.iter().any(|changed| {
            let path_matches = changed.path == finding.path
                || (finding.path.contains('*') && matches_pattern(&finding.path, &changed.path));
            if !path_matches {
                return false;
            }

            // Achados sem linha não podem ser recortados com precisão.
            let Some(start) = finding.line_start else {
                return true;
            };
            changed.lines.iter().any(|range| range.contains(start))
        })
    }
}

/// Falha ao interpretar um diff que não segue o formato unificado esperado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffError {
    line: usize,
    detail: String,
}

impl DiffError {
    /// Cria um erro associado a uma linha do patch.
    fn new(line: usize, detail: impl Into<String>) -> Self {
        Self {
            line,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for DiffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "diff inválido na linha {}: {}",
            self.line, self.detail
        )
    }
}

impl Error for DiffError {}

/// Estado de um bloco `@@ ... @@` enquanto o diff é lido.
struct HunkState {
    file_index: usize,
    new_cursor: usize,
    /// Linhas da nova versão que ainda pertencem a este bloco.
    new_remaining: usize,
}

/// Interpreta um patch unificado e reúne os arquivos e linhas alteradas.
pub fn parse_unified_diff(text: &str) -> Result<ChangedFiles, DiffError> {
    let mut files: Vec<ChangedFile> = Vec::new();
    let mut current_index: Option<usize> = None;
    let mut old_path: Option<String> = None;
    let mut hunk: Option<HunkState> = None;

    for (offset, raw_line) in text.lines().enumerate() {
        let line_number = offset + 1;
        let line = raw_line.trim_end_matches('\r');

        if line.starts_with("diff --git ") {
            hunk = None;
            current_index = None;
            old_path = None;
            continue;
        }

        if let Some(target) = line.strip_prefix("rename to ") {
            hunk = None;
            let path = normalize_path(target);
            if path.is_empty() {
                return Err(DiffError::new(line_number, "caminho renomeado vazio"));
            }
            current_index = Some(ensure_file(&mut files, path));
            continue;
        }

        if let Some(rest) = line.strip_prefix("@@ ") {
            let file_index = current_index.ok_or_else(|| {
                DiffError::new(
                    line_number,
                    "bloco encontrado antes do cabeçalho de arquivo",
                )
            })?;
            let header = rest.split(" @@").next().unwrap_or(rest);
            hunk = parse_hunk_header(header, line_number, file_index)?;
            continue;
        }

        if line.starts_with("@@") {
            return Err(DiffError::new(
                line_number,
                "formato de bloco não suportado",
            ));
        }

        // `--- ` e `+++ ` só são cabeçalhos fora de um bloco: dentro dele podem
        // ser linhas de código removidas ou adicionadas começando com `-`/`+`.
        if hunk.is_none() {
            if let Some(source) = line.strip_prefix("--- ") {
                old_path = Some(normalize_path(source));
                continue;
            }

            if let Some(target) = line.strip_prefix("+++ ") {
                let candidate = normalize_path(target);
                let path = if candidate == "/dev/null" {
                    old_path.take().ok_or_else(|| {
                        DiffError::new(line_number, "+++ /dev/null sem cabeçalho --- anterior")
                    })?
                } else {
                    candidate
                };
                if path.is_empty() {
                    return Err(DiffError::new(line_number, "caminho de arquivo vazio"));
                }
                current_index = Some(ensure_file(&mut files, path));
                continue;
            }
        }

        let hunk_finished = if let Some(state) = hunk.as_mut() {
            consume_hunk_line(&mut files[state.file_index], state, line);
            state.new_remaining == 0
        } else {
            false
        };
        if hunk_finished {
            hunk = None;
        }
    }

    for file in &mut files {
        file.lines = merge_ranges(std::mem::take(&mut file.lines));
    }

    Ok(ChangedFiles { files })
}

/// Reduz o relatório aos achados que tocam arquivos ou linhas alteradas.
pub fn scope_report(report: &AuditReport, changes: &ChangedFiles) -> AuditReport {
    let findings = report
        .findings
        .iter()
        .filter(|finding| changes.matches_finding(finding))
        .cloned()
        .collect();

    let limitation = if changes.is_empty() {
        "relatório filtrado por diff vazio: nenhum achado exibido".to_owned()
    } else {
        format!(
            "relatório filtrado por diff: {} arquivo(s) alterado(s); achados fora do diff não exibidos",
            changes.len()
        )
    };

    report.scoped_to(findings, limitation)
}

/// Obtém ou cria a entrada de um arquivo no resultado.
fn ensure_file(files: &mut Vec<ChangedFile>, path: String) -> usize {
    if let Some(index) = files.iter().position(|file| file.path == path) {
        return index;
    }
    files.push(ChangedFile {
        path,
        lines: Vec::new(),
    });
    files.len() - 1
}

/// Remove os prefixos de caminho usados por `git diff`.
fn normalize_path(raw: &str) -> String {
    let mut path = raw.trim().to_owned();
    if path.len() >= 2 && path.starts_with('"') && path.ends_with('"') {
        path = path[1..path.len() - 1].to_owned();
    }
    path.strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(&path)
        .to_owned()
}

/// Lê os números de linha do cabeçalho de um bloco.
fn parse_hunk_header(
    header: &str,
    line_number: usize,
    file_index: usize,
) -> Result<Option<HunkState>, DiffError> {
    let mut parts = header.split_whitespace();
    let old_range = parts
        .next()
        .ok_or_else(|| DiffError::new(line_number, "bloco sem intervalo antigo"))?;
    let new_range = parts
        .next()
        .ok_or_else(|| DiffError::new(line_number, "bloco sem intervalo novo"))?;
    parse_range(old_range, line_number)?;
    let (new_start, new_count) = parse_range(new_range, line_number)?;

    if new_count == 0 {
        return Ok(None);
    }

    Ok(Some(HunkState {
        file_index,
        new_cursor: new_start,
        new_remaining: new_count,
    }))
}

/// Converte `-1,3` ou `+1,3` em início e contagem de linhas.
fn parse_range(value: &str, line_number: usize) -> Result<(usize, usize), DiffError> {
    let value = value
        .strip_prefix('-')
        .or_else(|| value.strip_prefix('+'))
        .unwrap_or(value);
    let (start_text, count_text) = match value.split_once(',') {
        Some((start, count)) => (start, count),
        None => (value, "1"),
    };
    let start = start_text
        .parse::<usize>()
        .map_err(|_| DiffError::new(line_number, "número de linha inválido"))?;
    let count = count_text
        .parse::<usize>()
        .map_err(|_| DiffError::new(line_number, "contagem de linhas inválida"))?;
    if count > 0 && start == 0 {
        return Err(DiffError::new(
            line_number,
            "bloco começa na linha zero com linhas preservadas",
        ));
    }
    Ok((start, count))
}

/// Avança o cursor da nova versão e registra linhas adicionadas.
fn consume_hunk_line(file: &mut ChangedFile, state: &mut HunkState, line: &str) {
    match line.as_bytes().first() {
        Some(b'+') => {
            file.lines.push(LineRange {
                start: state.new_cursor,
                end: state.new_cursor,
            });
            state.new_cursor = state.new_cursor.saturating_add(1);
            state.new_remaining = state.new_remaining.saturating_sub(1);
        }
        Some(b' ') => {
            state.new_cursor = state.new_cursor.saturating_add(1);
            state.new_remaining = state.new_remaining.saturating_sub(1);
        }
        Some(b'-') | Some(b'\\') | None => {}
        Some(_) => {}
    }
}

/// Fundiciona intervalos adjacentes ou sobrepostos.
fn merge_ranges(ranges: Vec<LineRange>) -> Vec<LineRange> {
    let mut sorted = ranges;
    sorted.sort_by_key(|range| range.start);
    let mut merged: Vec<LineRange> = Vec::new();
    for range in sorted {
        match merged.last_mut() {
            Some(last) if range.start <= last.end.saturating_add(1) => {
                last.end = last.end.max(range.end);
            }
            _ => merged.push(range),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::AnalysisCoverage;
    use crate::audit::{ContractStatus, ExecutionMode};
    use crate::discovery::{SkipReason, SkippedEntry};
    use crate::findings::Severity;
    use crate::report::SkipReasonSummary;
    use crate::report::{AuditReport, ReportSummary};

    /// Patch mínimo com duas linhas adicionadas consecutivas.
    const PATCH: &str = concat!(
        "diff --git a/src/lib.rs b/src/lib.rs\n",
        "--- a/src/lib.rs\n",
        "+++ b/src/lib.rs\n",
        "@@ -1,2 +1,4 @@\n",
        " contexto\n",
        "+novo\n",
        "+outro\n",
        " contexto\n",
    );

    #[test]
    fn added_lines_become_merged_ranges() {
        let changes = parse_unified_diff(PATCH).expect("patch must parse");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes.files[0].path, "src/lib.rs");
        assert_eq!(changes.files[0].lines, vec![LineRange { start: 2, end: 3 }]);
    }

    #[test]
    fn header_like_code_inside_a_hunk_is_not_a_file_header() {
        let patch = concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -1,2 +1,2 @@\n",
            " contexto\n",
            "--- valor\n",
            "+++ valor\n",
        );
        let changes = parse_unified_diff(patch).expect("patch must parse");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes.files[0].path, "src/lib.rs");
        assert_eq!(changes.files[0].lines, vec![LineRange { start: 2, end: 2 }]);
    }

    #[test]
    fn deleted_file_keeps_the_old_path_without_lines() {
        let patch = "--- a/src/old.rs\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-conteudo\n";
        let changes = parse_unified_diff(patch).expect("patch must parse");
        assert_eq!(changes.files[0].path, "src/old.rs");
        assert!(changes.files[0].lines.is_empty());
    }

    #[test]
    fn rename_target_is_used_as_changed_path() {
        let patch = "diff --git a/src/old.rs b/src/new.rs\n\
similarity index 100%\n\
rename from src/old.rs\n\
rename to src/new.rs\n";
        let changes = parse_unified_diff(patch).expect("patch must parse");
        assert_eq!(changes.files[0].path, "src/new.rs");
    }

    #[test]
    fn malformed_hunk_reports_its_line() {
        let error = parse_unified_diff("--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -x +1 @@\n")
            .expect_err("malformed hunk must fail");
        assert!(error.to_string().contains("linha 3"));
    }

    /// Achado sintético para testar o recorte do relatório.
    fn finding(id: &str, path: &str, line: Option<usize>, severity: Severity) -> Finding {
        Finding {
            id: id.to_owned(),
            rule_id: "regra.teste".to_owned(),
            severity,
            title: "Achado de teste".to_owned(),
            message: "Mensagem".to_owned(),
            path: path.to_owned(),
            line_start: line,
            line_end: line,
            evidence: None,
            expected: "Esperado".to_owned(),
            suggestion: "Sugestão".to_owned(),
            confidence: None,
        }
    }

    #[test]
    fn scope_report_keeps_only_changed_lines_and_recomputes_summary() {
        let report = AuditReport {
            project: "projeto".to_owned(),
            schema_version: 1,
            summary: ReportSummary {
                files_included: 3,
                files_skipped: 1,
                modules: 2,
                packages: 1,
                edges: 4,
                findings: 4,
                errors: 2,
                warnings: 0,
                infos: 0,
                needs_review: 2,
            },
            skip_summary: SkipReasonSummary::from_entries(&[SkippedEntry {
                path: "vendor/lib.rs".to_owned(),
                reason: SkipReason::Excluded,
            }]),
            findings: vec![
                finding("linha-2", "src/lib.rs", Some(2), Severity::Error),
                finding("linha-9", "src/lib.rs", Some(9), Severity::Warning),
                finding("sem-linha", "src/lib.rs", None, Severity::NeedsReview),
                finding("outro", "src/outro.rs", Some(2), Severity::Info),
            ],
            exceptions_applied: Vec::new(),
            limitations: Vec::new(),
            coverage: AnalysisCoverage::default(),
            external_packages: Vec::new(),
            mode: ExecutionMode::Compliance,
            contract: ContractStatus::Present { schema_version: 1 },
        };
        let changes = parse_unified_diff(PATCH).expect("patch must parse");

        let scoped = scope_report(&report, &changes);
        let ids: Vec<&str> = scoped
            .findings
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        assert_eq!(ids, vec!["linha-2", "sem-linha"]);
        assert_eq!(scoped.summary.findings, 2);
        assert_eq!(scoped.summary.errors, 1);
        assert_eq!(scoped.summary.warnings, 0);
        assert_eq!(scoped.summary.infos, 0);
        assert_eq!(scoped.summary.needs_review, 1);
        assert_eq!(scoped.summary.files_included, 3);
        assert_eq!(scoped.skip_summary.total, 1);
        assert_eq!(scoped.skip_summary.reasons[0].reason, SkipReason::Excluded);
        assert_eq!(scoped.limitations.len(), 1);
        assert!(scoped.limitations[0].contains("1 arquivo(s) alterado(s)"));
    }

    #[test]
    fn deletion_only_diff_keeps_only_findings_without_a_line() {
        let report = AuditReport {
            project: "projeto".to_owned(),
            schema_version: 1,
            summary: ReportSummary {
                files_included: 1,
                files_skipped: 0,
                modules: 1,
                packages: 0,
                edges: 0,
                findings: 2,
                errors: 1,
                warnings: 0,
                infos: 0,
                needs_review: 1,
            },
            skip_summary: SkipReasonSummary::default(),
            findings: vec![
                finding("linha-1", "src/old.rs", Some(1), Severity::Error),
                finding("sem-linha", "src/old.rs", None, Severity::NeedsReview),
            ],
            exceptions_applied: Vec::new(),
            limitations: Vec::new(),
            coverage: AnalysisCoverage::default(),
            external_packages: Vec::new(),
            mode: ExecutionMode::Compliance,
            contract: ContractStatus::Present { schema_version: 1 },
        };
        let changes =
            parse_unified_diff("--- a/src/old.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-conteudo\n")
                .expect("patch must parse");

        let scoped = scope_report(&report, &changes);
        let ids: Vec<&str> = scoped
            .findings
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        assert_eq!(ids, vec!["sem-linha"]);
        assert_eq!(scoped.summary.findings, 1);
        assert_eq!(scoped.summary.errors, 0);
        assert_eq!(scoped.summary.needs_review, 1);
    }

    #[test]
    fn scope_report_matches_module_patterns() {
        let report = AuditReport {
            project: "projeto".to_owned(),
            schema_version: 1,
            summary: ReportSummary {
                files_included: 1,
                files_skipped: 0,
                modules: 1,
                packages: 0,
                edges: 0,
                findings: 1,
                errors: 0,
                warnings: 0,
                infos: 0,
                needs_review: 1,
            },
            skip_summary: SkipReasonSummary::default(),
            findings: vec![finding(
                "modulo",
                "crates/domain/**",
                None,
                Severity::NeedsReview,
            )],
            exceptions_applied: Vec::new(),
            limitations: Vec::new(),
            coverage: AnalysisCoverage::default(),
            external_packages: Vec::new(),
            mode: ExecutionMode::Compliance,
            contract: ContractStatus::Present { schema_version: 1 },
        };
        let changes = parse_unified_diff(
            "--- a/crates/domain/src/lib.rs\n\
+++ b/crates/domain/src/lib.rs\n\
@@ -1 +1 @@\n\
-conteudo\n\
+conteudo novo\n",
        )
        .expect("patch must parse");

        let scoped = scope_report(&report, &changes);
        assert_eq!(scoped.findings.len(), 1);
    }
}
