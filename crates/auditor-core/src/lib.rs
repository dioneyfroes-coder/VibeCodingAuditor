//! Núcleo de auditoria do Vibe Coding Auditor.
//!
//! O núcleo já contém o modelo e a validação do contrato e a descoberta
//! somente leitura de arquivos. Ainda não contém regras de arquitetura, grafo
//! de dependências nem persistência; será reutilizado pela CLI e, no futuro,
//! pelo adaptador desktop.

pub mod contract;
pub mod discovery;

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
