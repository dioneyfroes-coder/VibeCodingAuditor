//! Núcleo de auditoria do Vibe Coding Auditor.
//!
//! Nesta etapa o núcleo ainda não contém regras, analisadores nem persistência:
//! ele apenas estabelece a biblioteca que será reutilizada pela CLI e, no
//! futuro, pelo adaptador desktop.

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
