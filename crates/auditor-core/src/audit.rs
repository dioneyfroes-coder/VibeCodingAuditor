//! Orquestração somente leitura da auditoria completa.
//!
//! Esta camada garante que CLI e futura interface desktop executem o mesmo
//! pipeline: contrato, descoberta, análise, regras e relatório. Ela não escolhe
//! regras nem interpreta resultados: apenas encadeia os módulos do núcleo.

use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::analysis::{self, AnalysisError};
use crate::contract::{self, ContractError};
use crate::discovery::{self, DiscoveryError};
use crate::report::AuditReport;
use crate::rules;

/// Falha operacional em uma etapa do pipeline de auditoria.
#[derive(Debug)]
pub enum AuditError {
    /// O contrato não pôde ser lido ou validado.
    Contract(ContractError),
    /// A varredura da raiz falhou.
    Discovery(DiscoveryError),
    /// A análise estática falhou.
    Analysis(AnalysisError),
}

impl From<ContractError> for AuditError {
    fn from(source: ContractError) -> Self {
        Self::Contract(source)
    }
}

impl From<DiscoveryError> for AuditError {
    fn from(source: DiscoveryError) -> Self {
        Self::Discovery(source)
    }
}

impl From<AnalysisError> for AuditError {
    fn from(source: AnalysisError) -> Self {
        Self::Analysis(source)
    }
}

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(source) => write!(formatter, "contrato: {source}"),
            Self::Discovery(source) => write!(formatter, "descoberta: {source}"),
            Self::Analysis(source) => write!(formatter, "análise: {source}"),
        }
    }
}

impl Error for AuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Contract(source) => Some(source),
            Self::Discovery(source) => Some(source),
            Self::Analysis(source) => Some(source),
        }
    }
}

/// Executa o pipeline completo na raiz informada e devolve o relatório.
pub fn audit_project(root: &Path) -> Result<AuditReport, AuditError> {
    let contract = contract::load_contract(&root.join("auditor.toml"))?;
    let inventory = discovery::discover(root, &contract)?;
    let analysis = analysis::analyze(root, &contract, &inventory)?;
    let findings = rules::evaluate(&contract, &inventory, &analysis);
    Ok(AuditReport::from_pipeline(
        &contract, &inventory, &analysis, findings,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::Severity;
    use crate::rules::{
        RULE_DEPENDENCY_CYCLE, RULE_DEPENDENCY_NOT_ALLOWED, RULE_DEPENDENCY_UNRESOLVED,
        RULE_FILE_AMBIGUOUS_MODULE, RULE_FILE_UNMAPPED, RULE_MODULE_WITHOUT_FILES,
    };

    /// Raiz de uma fixture de projeto, relativa ao crate.
    fn project_fixture(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name)
    }

    /// Raiz da fixture de dependências, relativa ao crate.
    fn project_root() -> std::path::PathBuf {
        project_fixture("dependencies")
    }

    #[test]
    fn fixture_report_is_complete_and_deterministic() {
        let root = project_root();
        let first = audit_project(&root).expect("audit must succeed");
        let second = audit_project(&root).expect("audit must succeed");

        assert_eq!(first, second);
        assert_eq!(first.project, "projeto-dependencias");
        assert_eq!(first.schema_version, 1);
        assert!(first.has_findings());
        assert_eq!(first.summary.findings, first.findings.len());
        assert_eq!(first.summary.errors, 2);
        assert_eq!(first.summary.warnings, 3);
        assert_eq!(first.summary.needs_review, 2);
        assert_eq!(first.limitations.len(), 1);
    }

    #[test]
    fn fixture_report_traces_rule_ids() {
        let report = audit_project(&project_root()).expect("audit must succeed");
        let rule_ids: Vec<&str> = report
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();

        assert!(rule_ids.contains(&RULE_DEPENDENCY_NOT_ALLOWED));
        assert!(rule_ids.contains(&RULE_DEPENDENCY_CYCLE));
    }

    /// Projeto sem violações, sem revisões pendentes e sem limitações.
    #[test]
    fn clean_fixture_has_no_findings_or_limitations() {
        let root = project_fixture("clean");
        let report = audit_project(&root).expect("audit must succeed");

        assert_eq!(report, audit_project(&root).expect("audit must succeed"));
        assert_eq!(report.project, "projeto-correto");
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.summary.modules, 3);
        assert_eq!(report.summary.packages, 3);
        assert_eq!(report.summary.edges, 3);
        assert_eq!(report.summary.findings, 0);
        assert!(!report.has_findings());
        assert!(report.limitations.is_empty());
    }

    /// Estrutura parcialmente desconhecida gera revisões, nunca conclusões inventadas.
    #[test]
    fn partial_fixture_reports_unknown_structure_without_inventing_conclusions() {
        let root = project_fixture("partial");
        let report = audit_project(&root).expect("audit must succeed");

        assert_eq!(report, audit_project(&root).expect("audit must succeed"));
        assert_eq!(report.project, "projeto-parcial");
        assert_eq!(report.summary.findings, 4);
        assert_eq!(report.summary.errors, 0);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.needs_review, 3);

        let rule_ids: Vec<&str> = report
            .findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();
        assert!(rule_ids.contains(&RULE_DEPENDENCY_UNRESOLVED));
        assert!(rule_ids.contains(&RULE_FILE_AMBIGUOUS_MODULE));
        assert!(rule_ids.contains(&RULE_FILE_UNMAPPED));
        assert!(rule_ids.contains(&RULE_MODULE_WITHOUT_FILES));

        let has_id = |id: &str| report.findings.iter().any(|finding| finding.id == id);
        assert!(has_id(
            "dep.unresolved:external_sdk:src/application/lib.rs:4"
        ));
        assert!(has_id("file.ambiguous_module:src/domain/overlap.rs"));
        assert!(has_id("file.unmapped:src/stray.rs"));
        assert!(has_id("module.no_files:infrastructure"));

        let unresolved = report
            .findings
            .iter()
            .find(|finding| finding.rule_id == RULE_DEPENDENCY_UNRESOLVED)
            .expect("unresolved dependency must be reported");
        assert_eq!(unresolved.severity, Severity::NeedsReview);
        assert_eq!(
            unresolved.evidence.as_deref(),
            Some("use external_sdk::Client;")
        );

        assert_eq!(report.limitations.len(), 1);
        assert!(report.limitations[0].contains("src/domain/broken.rs"));
    }

    /// Projeto TypeScript/JavaScript com arestas permitidas, proibidas e ambíguas.
    #[test]
    fn typescript_fixture_reports_resolved_edges_without_inventing_aliases() {
        let root = project_fixture("typescript");
        let report = audit_project(&root).expect("audit must succeed");

        assert_eq!(report, audit_project(&root).expect("audit must succeed"));
        assert_eq!(report.project, "projeto-typescript");
        assert_eq!(report.summary.modules, 3);
        assert_eq!(report.summary.edges, 8);
        assert_eq!(report.summary.findings, 4);
        assert_eq!(report.summary.errors, 2);
        assert_eq!(report.summary.warnings, 0);
        assert_eq!(report.summary.needs_review, 2);
        assert_eq!(report.limitations.len(), 1);
        assert!(report.limitations[0].contains("pacotes npm"));

        let forbidden: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_NOT_ALLOWED)
            .collect();
        assert_eq!(forbidden.len(), 2);
        assert!(forbidden.iter().any(|finding| {
            finding.path == "src/application/use-case.ts" && finding.line_start == Some(2)
        }));
        assert!(forbidden.iter().any(|finding| {
            finding.path == "src/application/legacy.js" && finding.line_start == Some(1)
        }));
        assert!(forbidden.iter().all(|finding| {
            finding
                .evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("infrastructure"))
        }));

        let unresolved = report
            .findings
            .iter()
            .find(|finding| {
                finding.rule_id == RULE_DEPENDENCY_UNRESOLVED
                    && finding.path == "src/application/ambiguous.ts"
            })
            .expect("bare specifier must be a review");
        assert_eq!(unresolved.severity, Severity::NeedsReview);
        assert_eq!(unresolved.line_start, Some(2));
        assert_eq!(unresolved.evidence.as_deref(), Some("import \"domain\";"));
    }

    /// Comentários e strings não podem virar dependências inventadas.
    #[test]
    fn typescript_comments_and_strings_do_not_create_findings() {
        let root = project_fixture("typescript");
        let report = audit_project(&root).expect("audit must succeed");

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.path != "src/application/comments.ts")
        );
    }

    /// Projeto Python com imports permitidos, proibidos, relativos e ruído.
    #[test]
    fn python_fixture_reports_resolved_edges_without_inventing_aliases() {
        let root = project_fixture("python");
        let report = audit_project(&root).expect("audit must succeed");

        assert_eq!(report, audit_project(&root).expect("audit must succeed"));
        assert_eq!(report.project, "projeto-python");
        assert_eq!(report.summary.modules, 3);
        assert_eq!(report.summary.edges, 4);
        assert_eq!(report.summary.findings, 3);
        assert_eq!(report.summary.errors, 2);
        assert_eq!(report.summary.warnings, 0);
        assert_eq!(report.summary.needs_review, 1);
        assert_eq!(report.limitations.len(), 1);
        assert!(
            report
                .limitations
                .iter()
                .any(|item| item.contains("pacotes pip"))
        );

        let forbidden: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| finding.rule_id == RULE_DEPENDENCY_NOT_ALLOWED)
            .collect();
        assert_eq!(forbidden.len(), 2);
        assert!(
            forbidden
                .iter()
                .any(|finding| finding.path == "src/application/use_case.py")
        );
        assert!(
            forbidden
                .iter()
                .any(|finding| finding.path == "src/application/legacy.py")
        );

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.path != "src/application/noise.py")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.path == "src/unmapped.py")
        );
    }

    /// Concentração de dependências é revisão, não violação comprovada.
    #[test]
    fn concentration_fixture_flags_hub_as_needs_review() {
        let root = project_fixture("concentration");
        let report = audit_project(&root).expect("audit must succeed");

        assert_eq!(report, audit_project(&root).expect("audit must succeed"));
        assert_eq!(report.project, "projeto-concentracao");
        assert_eq!(report.summary.modules, 4);
        assert_eq!(report.summary.edges, 3);
        assert_eq!(report.summary.findings, 1);
        assert_eq!(report.summary.errors, 0);
        assert_eq!(report.summary.warnings, 0);
        assert_eq!(report.summary.needs_review, 1);
        assert!(report.limitations.is_empty());

        let finding = report
            .findings
            .first()
            .expect("concentration finding must exist");
        assert_eq!(finding.id, "module.concentration:application");
        assert_eq!(finding.rule_id, "module.concentration");
        assert_eq!(finding.severity, Severity::NeedsReview);
        assert_eq!(finding.path, "crates/application/**");
        assert_eq!(
            finding.evidence.as_deref(),
            Some("destinos (3): domain, infrastructure, presentation")
        );
        assert!(finding.confidence.is_none());
    }

    #[test]
    fn missing_root_is_an_audit_error() {
        let missing = project_root().join("nao-existe");
        let error = audit_project(&missing).expect_err("missing root must fail");

        assert!(matches!(
            error,
            AuditError::Contract(ContractError::Io { .. })
        ));
        assert!(error.to_string().contains("contrato"));
        assert!(Error::source(&error).is_some());
    }
}
