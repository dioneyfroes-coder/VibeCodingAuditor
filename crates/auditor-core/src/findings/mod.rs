//! Modelo estruturado de achados da auditoria.
//!
//! CLI, futura interface desktop e extensão devem mostrar os mesmos campos.
//! Nenhum achado modifica o projeto auditado: sugestões são apenas texto.

use serde::Serialize;

/// Severidade de um achado, conforme o modelo do plano.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Violação determinística de uma regra do contrato.
    Error,
    /// Situação suspeita que ainda não foi classificada como violação.
    Warning,
    /// Informação útil sobre a execução.
    Info,
    /// O auditor não tem informação suficiente para decidir.
    NeedsReview,
}

impl Severity {
    /// Valor estável usado em JSON, banco de dados e comparações.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::NeedsReview => "needs_review",
        }
    }

    /// Rótulo curto usado na saída legível.
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "erro",
            Self::Warning => "aviso",
            Self::Info => "informação",
            Self::NeedsReview => "revisão necessária",
        }
    }
}

/// Dispensa aplicada a um achado que deixou de ser contado.
///
/// Ela não apaga o histórico: a linha de limitação derivada deste registro é
/// persistida pelo módulo de histórico já existente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppliedException {
    /// Nome da regra declarada na exceção, que pode ser um `id` personalizado.
    pub rule: String,
    /// Identificador da regra que produziria o achado.
    pub rule_id: String,
    /// Origem no escopo da regra.
    pub from: String,
    /// Destino no escopo da regra.
    pub to: String,
    /// Caminho exato da exceção, quando declarado.
    pub path: Option<String>,
    /// Motivo humano da dispensa.
    pub justification: String,
    /// Data de revisão humana, sem aplicação automática.
    pub expires_on: Option<String>,
    /// Identificador do achado que teria sido emitido.
    pub finding_id: String,
    /// Arquivo do achado dispensado.
    pub finding_path: String,
    /// Linha do achado dispensado, quando conhecida.
    pub finding_line: Option<usize>,
}

impl AppliedException {
    /// Prefixo estável usado para identificar dispensas nas limitações gravadas.
    pub const PREFIX: &'static str = "exceção aplicada:";

    /// Linha determinística gravada em `limitations` e exibida no histórico.
    pub fn limitation_line(&self) -> String {
        let path = self.path.as_deref().unwrap_or("-");
        let expires = self.expires_on.as_deref().unwrap_or("-");
        let line = self
            .finding_line
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_owned());
        format!(
            "{} regra={} | origem={} | destino={} | arquivo={} | achado={} | emissao={} | linha={} | expira_em={} | justificativa={}",
            Self::PREFIX,
            self.rule,
            self.from,
            self.to,
            path,
            self.finding_id,
            self.rule_id,
            line,
            expires,
            self.justification
        )
    }
}

/// Achado estruturado com regra, evidência e sugestão rastreáveis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// Identificador estável da ocorrência lógica; não inclui linha nem evidência volátil.
    pub id: String,
    /// Identificador estável da regra que produziu o achado.
    pub rule_id: String,
    /// Classificação de severidade.
    pub severity: Severity,
    /// Descrição curta do problema.
    pub title: String,
    /// Explicação objetiva do que foi observado.
    pub message: String,
    /// Arquivo ou caminho envolvido, relativo à raiz quando aplicável.
    pub path: String,
    /// Primeira linha envolvida, quando conhecida.
    pub line_start: Option<usize>,
    /// Última linha envolvida, quando conhecida.
    pub line_end: Option<usize>,
    /// Trecho ou descrição do que foi efetivamente observado.
    pub evidence: Option<String>,
    /// O que o contrato ou a regra exige.
    pub expected: String,
    /// Possível correção, nunca aplicada automaticamente.
    pub suggestion: String,
    /// Confiança declarada apenas para análises heurísticas; nulo nas regras determinísticas.
    pub confidence: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_values_are_stable() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Info.as_str(), "info");
        assert_eq!(Severity::NeedsReview.as_str(), "needs_review");
    }

    #[test]
    fn severity_labels_are_stable() {
        assert_eq!(Severity::Error.label(), "erro");
        assert_eq!(Severity::Warning.label(), "aviso");
        assert_eq!(Severity::Info.label(), "informação");
        assert_eq!(Severity::NeedsReview.label(), "revisão necessária");
    }
}
