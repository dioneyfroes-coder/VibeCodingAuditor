//! Geração de relatórios consistentes a partir dos achados da auditoria.
//!
//! O núcleo controla os três formatos para que CLI, desktop e futura extensão
//! mostrem os mesmos dados. A escrita em arquivo é responsabilidade do chamador.

use std::error::Error;
use std::fmt;

use serde::Serialize;

use crate::analysis::Analysis;
use crate::contract::Contract;
use crate::discovery::Inventory;
use crate::findings::{Finding, Severity};

/// Formato de saída de um relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Texto legível no terminal.
    Text,
    /// JSON estruturado, estável para consumo por outras interfaces.
    Json,
    /// Markdown para revisão humana e versionamento.
    Markdown,
}

impl ReportFormat {
    /// Nome usado em argumentos de linha de comando e mensagens.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Markdown => "markdown",
        }
    }

    /// Extensão recomendada quando o relatório é gravado em arquivo.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Json => "json",
            Self::Markdown => "md",
        }
    }
}

impl fmt::Display for ReportFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Falha ao converter um relatório para o formato solicitado.
#[derive(Debug)]
pub enum ReportError {
    /// A serialização JSON falhou.
    Json(serde_json::Error),
}

impl fmt::Display for ReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(source) => write!(formatter, "falha ao gerar JSON: {source}"),
        }
    }
}

impl Error for ReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(source) => Some(source),
        }
    }
}

/// Contagem determinística de um relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReportSummary {
    /// Arquivos incluídos na varredura.
    pub files_included: usize,
    /// Entradas não incluídas na varredura.
    pub files_skipped: usize,
    /// Módulos declarados no contrato.
    pub modules: usize,
    /// Pacotes Cargo encontrados no workspace.
    pub packages: usize,
    /// Arestas classificadas na análise.
    pub edges: usize,
    /// Total de achados.
    pub findings: usize,
    /// Achados de severidade `error`.
    pub errors: usize,
    /// Achados de severidade `warning`.
    pub warnings: usize,
    /// Achados de severidade `info`.
    pub infos: usize,
    /// Achados de severidade `needs_review`.
    pub needs_review: usize,
}

/// Relatório completo de uma execução de auditoria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditReport {
    /// Nome do projeto, conforme o contrato.
    pub project: String,
    /// Versão do schema do contrato.
    pub schema_version: u32,
    /// Contagens derivadas dos dados da execução.
    pub summary: ReportSummary,
    /// Achados ordenados por identificador.
    pub findings: Vec<Finding>,
    /// Limitações explícitas da análise, sem conclusões inventadas.
    pub limitations: Vec<String>,
}

impl AuditReport {
    /// Monta o relatório a partir dos resultados já produzidos pelo núcleo.
    pub fn from_pipeline(
        contract: &Contract,
        inventory: &Inventory,
        analysis: &Analysis,
        findings: Vec<Finding>,
    ) -> Self {
        let (errors, warnings, infos, needs_review) = severity_counts(&findings);

        Self {
            project: contract.project.name.clone(),
            schema_version: contract.schema_version,
            summary: ReportSummary {
                files_included: inventory.files.len(),
                files_skipped: inventory.skipped.len(),
                modules: contract.architecture.modules.len(),
                packages: analysis
                    .workspace
                    .as_ref()
                    .map_or(0, |workspace| workspace.packages.len()),
                edges: analysis.edges.len(),
                findings: findings.len(),
                errors,
                warnings,
                infos,
                needs_review,
            },
            findings,
            limitations: analysis.limitations.clone(),
        }
    }

    /// Cria um relatório com um subconjunto de achados e contagens coerentes.
    ///
    /// As contagens de escopo da execução são preservadas; a limitação informa
    /// que o relatório foi recortado.
    pub fn scoped_to(&self, findings: Vec<Finding>, limitation: String) -> Self {
        let (errors, warnings, infos, needs_review) = severity_counts(&findings);
        let mut limitations = self.limitations.clone();
        limitations.push(limitation);

        Self {
            project: self.project.clone(),
            schema_version: self.schema_version,
            summary: ReportSummary {
                files_included: self.summary.files_included,
                files_skipped: self.summary.files_skipped,
                modules: self.summary.modules,
                packages: self.summary.packages,
                edges: self.summary.edges,
                findings: findings.len(),
                errors,
                warnings,
                infos,
                needs_review,
            },
            findings,
            limitations,
        }
    }

    /// Indica se a execução produziu algum achado.
    pub fn has_findings(&self) -> bool {
        self.summary.findings > 0
    }

    /// Renderiza o relatório no formato solicitado.
    pub fn render(&self, format: ReportFormat) -> Result<String, ReportError> {
        match format {
            ReportFormat::Text => Ok(self.render_text()),
            ReportFormat::Json => self.render_json(),
            ReportFormat::Markdown => Ok(self.render_markdown()),
        }
    }

    /// Renderiza texto legível para o terminal.
    fn render_text(&self) -> String {
        let mut lines = vec![
            format!(
                "Auditoria: {} (schema v{})",
                self.project, self.schema_version
            ),
            format!(
                "Arquivos: {} incluídos, {} não incluídos",
                self.summary.files_included, self.summary.files_skipped
            ),
            format!(
                "Módulos: {} | Pacotes Cargo: {} | Arestas: {}",
                self.summary.modules, self.summary.packages, self.summary.edges
            ),
            format!(
                "Achados: {} ({} erros, {} avisos, {} informações, {} revisões)",
                self.summary.findings,
                self.summary.errors,
                self.summary.warnings,
                self.summary.infos,
                self.summary.needs_review
            ),
            String::new(),
        ];

        if self.findings.is_empty() {
            lines.push("Nenhum achado.".to_owned());
        } else {
            lines.push("Achados:".to_owned());
            for finding in &self.findings {
                let location = finding_location(finding);
                lines.push(format!(
                    "  [{}] {} — {}",
                    finding.severity.label(),
                    finding.rule_id,
                    finding.title
                ));
                lines.push(format!("    id: {}", finding.id));
                lines.push(format!("    local: {location}"));
                lines.push(format!("    mensagem: {}", finding.message));
                if let Some(evidence) = &finding.evidence {
                    lines.push(format!("    evidência: {evidence}"));
                }
                lines.push(format!("    esperado: {}", finding.expected));
                lines.push(format!("    sugestão: {}", finding.suggestion));
            }
        }

        if !self.limitations.is_empty() {
            lines.push(String::new());
            lines.push("Limitações:".to_owned());
            for limitation in &self.limitations {
                lines.push(format!("  - {limitation}"));
            }
        }

        lines.join("\n")
    }

    /// Serializa o relatório em JSON formatado.
    fn render_json(&self) -> Result<String, ReportError> {
        serde_json::to_string_pretty(self).map_err(ReportError::Json)
    }

    /// Renderiza o relatório em Markdown com uma seção por achado.
    fn render_markdown(&self) -> String {
        let mut lines = vec![
            format!("# Auditoria: {}", self.project),
            String::new(),
            format!("- Schema: v{}", self.schema_version),
            format!("- Arquivos incluídos: {}", self.summary.files_included),
            format!("- Arquivos não incluídos: {}", self.summary.files_skipped),
            format!("- Módulos: {}", self.summary.modules),
            format!("- Pacotes Cargo: {}", self.summary.packages),
            format!("- Arestas: {}", self.summary.edges),
            format!(
                "- Achados: {} ({} erros, {} avisos, {} informações, {} revisões)",
                self.summary.findings,
                self.summary.errors,
                self.summary.warnings,
                self.summary.infos,
                self.summary.needs_review
            ),
            String::new(),
            "## Achados".to_owned(),
        ];

        if self.findings.is_empty() {
            lines.push(String::new());
            lines.push("Nenhum achado.".to_owned());
        }

        for finding in &self.findings {
            lines.push(String::new());
            lines.push(format!(
                "### [{}] {}",
                finding.severity.label(),
                markdown_text(&finding.title)
            ));
            lines.push(String::new());
            lines.push(format!("- **ID:** `{}`", finding.id));
            lines.push(format!("- **Regra:** `{}`", finding.rule_id));
            lines.push(format!("- **Local:** `{}`", finding_location(finding)));
            lines.push(format!(
                "- **Mensagem:** {}",
                markdown_text(&finding.message)
            ));
            if let Some(evidence) = &finding.evidence {
                lines.push(format!("- **Evidência:** `{}`", markdown_text(evidence)));
            }
            lines.push(format!(
                "- **Esperado:** {}",
                markdown_text(&finding.expected)
            ));
            lines.push(format!(
                "- **Sugestão:** {}",
                markdown_text(&finding.suggestion)
            ));
        }

        if !self.limitations.is_empty() {
            lines.push(String::new());
            lines.push("## Limitações".to_owned());
            for limitation in &self.limitations {
                lines.push(format!("- {}", markdown_text(limitation)));
            }
        }

        format!("{}\n", lines.join("\n"))
    }
}

/// Conta achados por severidade, usada em relatórios completos e recortados.
fn severity_counts(findings: &[Finding]) -> (usize, usize, usize, usize) {
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();
    let infos = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Info)
        .count();
    let needs_review = findings
        .iter()
        .filter(|finding| finding.severity == Severity::NeedsReview)
        .count();
    (errors, warnings, infos, needs_review)
}

/// Formata o caminho e as linhas de um achado para saída legível.
pub fn finding_location(finding: &Finding) -> String {
    match (finding.line_start, finding.line_end) {
        (Some(start), Some(end)) if start == end => format!("{}:{}", finding.path, start),
        (Some(start), Some(end)) => format!("{}:{}-{}", finding.path, start, end),
        (Some(start), None) => format!("{}:{}", finding.path, start),
        (None, Some(end)) => format!("{}:~{}", finding.path, end),
        (None, None) => finding.path.clone(),
    }
}

/// Escapa caracteres que alterariam a formatação Markdown.
fn markdown_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::Severity;

    /// Achado mínimo usado para testar a renderização.
    fn sample_finding() -> Finding {
        Finding {
            id: "dep.not_allowed:domain:application:src/lib.rs:3".to_owned(),
            rule_id: "dep.not_allowed".to_owned(),
            severity: Severity::Error,
            title: "Dependência não permitida: domain -> application".to_owned(),
            message: "O uso não está no contrato.".to_owned(),
            path: "src/lib.rs".to_owned(),
            line_start: Some(3),
            line_end: Some(3),
            evidence: Some("use application::Service;".to_owned()),
            expected: "nenhuma dependência permitida".to_owned(),
            suggestion: "Revise o uso ou o contrato.".to_owned(),
            confidence: None,
        }
    }

    /// Relatório em memória com um achado e uma limitação.
    fn sample_report() -> AuditReport {
        AuditReport {
            project: "projeto-teste".to_owned(),
            schema_version: 1,
            summary: ReportSummary {
                files_included: 4,
                files_skipped: 2,
                modules: 3,
                packages: 2,
                edges: 5,
                findings: 1,
                errors: 1,
                warnings: 0,
                infos: 0,
                needs_review: 0,
            },
            findings: vec![sample_finding()],
            limitations: vec!["arquivo.rs não pôde ser analisado".to_owned()],
        }
    }

    #[test]
    fn formats_have_stable_names_and_extensions() {
        assert_eq!(ReportFormat::Text.as_str(), "text");
        assert_eq!(ReportFormat::Json.as_str(), "json");
        assert_eq!(ReportFormat::Markdown.as_str(), "markdown");
        assert_eq!(ReportFormat::Text.extension(), "txt");
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Markdown.extension(), "md");
        assert_eq!(ReportFormat::Markdown.to_string(), "markdown");
    }

    #[test]
    fn text_report_contains_actionable_fields() {
        let output = sample_report()
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");

        assert!(output.contains("Auditoria: projeto-teste (schema v1)"));
        assert!(output.contains("[erro] dep.not_allowed"));
        assert!(output.contains("id: dep.not_allowed:domain:application:src/lib.rs:3"));
        assert!(output.contains("local: src/lib.rs:3"));
        assert!(output.contains("evidência: use application::Service;"));
        assert!(output.contains("Limitações:"));
    }

    #[test]
    fn json_report_exposes_the_full_finding_model() {
        let output = sample_report()
            .render(ReportFormat::Json)
            .expect("JSON rendering must succeed");
        let value: serde_json::Value = serde_json::from_str(&output).expect("JSON must be valid");

        assert_eq!(value["project"], "projeto-teste");
        assert_eq!(value["summary"]["findings"], 1);
        assert_eq!(value["findings"][0]["severity"], "error");
        assert_eq!(value["findings"][0]["line_start"], 3);
        assert_eq!(
            value["findings"][0]["evidence"],
            "use application::Service;"
        );
        assert!(value["findings"][0]["confidence"].is_null());
    }

    #[test]
    fn markdown_report_escapes_pipes_and_newlines() {
        let mut report = sample_report();
        report.findings[0].title = "canal | linha\nnova".to_owned();
        let output = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");

        assert!(output.contains("# Auditoria: projeto-teste"));
        assert!(output.contains("canal \\| linha<br>nova"));
        assert!(output.contains("**Evidência:** `use application::Service;`"));
        assert!(output.contains("## Limitações"));
    }

    #[test]
    fn empty_report_is_explicit() {
        let mut report = sample_report();
        report.findings.clear();
        report.summary.findings = 0;
        report.summary.errors = 0;

        assert!(!report.has_findings());
        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        assert!(text.contains("Nenhum achado."));
    }

    #[test]
    fn finding_location_handles_missing_and_ranged_lines() {
        let mut finding = sample_finding();
        finding.line_end = Some(5);
        assert_eq!(finding_location(&finding), "src/lib.rs:3-5");

        finding.line_start = None;
        finding.line_end = None;
        assert_eq!(finding_location(&finding), "src/lib.rs");
    }
}
