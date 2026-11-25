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
    use crate::rules::{RULE_DEPENDENCY_CYCLE, RULE_DEPENDENCY_NOT_ALLOWED};

    /// Raiz da fixture de dependências, relativa ao crate.
    fn project_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/dependencies")
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
