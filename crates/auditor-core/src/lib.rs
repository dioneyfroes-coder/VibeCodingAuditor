//! Núcleo de auditoria do Vibe Coding Auditor.
//!
//! O núcleo contém contrato, descoberta somente leitura, análise de
//! dependências, avaliação determinística de regras e geração de relatórios.
//! Será reutilizado pela CLI e, no futuro, pelo adaptador desktop.
//! Persistência em SQLite pertence a uma etapa posterior.

pub mod analysis;
pub mod audit;
pub mod builder;
pub mod contract;
pub mod diff;
pub mod discovery;
pub mod findings;
pub mod history;
pub mod report;
pub mod resolution;
pub mod rules;

/// Identificador estável do núcleo, usado em metadados e mensagens.
pub const CORE_NAME: &str = "auditor-core";

#[cfg(test)]
mod tests {
    use super::CORE_NAME;

    #[test]
    fn core_name_matches_package_name() {
        assert_eq!(CORE_NAME, env!("CARGO_PKG_NAME"));
    }
}
