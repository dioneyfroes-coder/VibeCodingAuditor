//! Leitura, validação e representação do contrato arquitetural (`auditor.toml`).
//!
//! O contrato descreve o projeto auditado, não a configuração do auditor.
//! Nenhuma função desta página modifica o contrato existente.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

/// Única versão de schema aceita por esta versão do auditor.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// Modelo inicial gerado por `auditor init`; ainda depende de revisão e confirmação do usuário.
pub const EXAMPLE_CONTRACT: &str = include_str!("../../../../auditor.example.toml");

/// Contrato arquitetural completo de um projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub schema_version: u32,
    pub project: Project,
    pub scan: Scan,
    pub architecture: Architecture,
}

/// Identificação do projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
}

/// Padrões de inclusão e exclusão usados na descoberta de arquivos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// Organização desejada do projeto.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Architecture {
    pub style: ArchitectureStyle,
    pub modules: Vec<Module>,
}

/// Estilos arquiteturais reconhecidos pelo schema v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureStyle {
    ModularMonolith,
}

/// Módulo declarado no contrato.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub name: String,
    pub path: String,
    pub responsibility: String,
    #[serde(default)]
    pub allowed_dependencies: Vec<String>,
}

/// Erros de leitura, parsing e validação do contrato.
#[derive(Debug)]
pub enum ContractError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse(toml::de::Error),
    UnsupportedSchemaVersion {
        found: u32,
    },
    Validation(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    formatter,
                    "não foi possível ler {}: {source}",
                    path.display()
                )
            }
            Self::Parse(source) => write!(formatter, "TOML inválido: {source}"),
            Self::UnsupportedSchemaVersion { found } => write!(
                formatter,
                "schema_version {found} não é suportado; esta versão do auditor aceita apenas {SUPPORTED_SCHEMA_VERSION}"
            ),
            Self::Validation(message) => write!(formatter, "contrato inválido: {message}"),
        }
    }
}

impl Error for ContractError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse(source) => Some(source),
            Self::Validation(_) | Self::UnsupportedSchemaVersion { .. } => None,
        }
    }
}

/// Carrega e valida um `auditor.toml` do disco.
pub fn load_contract(path: &Path) -> Result<Contract, ContractError> {
    let text = fs::read_to_string(path).map_err(|source| ContractError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_contract(&text)
}

/// Carrega e valida um contrato a partir de texto.
pub fn parse_contract(text: &str) -> Result<Contract, ContractError> {
    let document: toml::Value = toml::from_str(text).map_err(ContractError::Parse)?;

    let found = document
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| {
            ContractError::Validation(
                "schema_version é obrigatório e deve ser um número inteiro".to_owned(),
            )
        })?;
    let found = u32::try_from(found).map_err(|_| {
        ContractError::Validation("schema_version deve ser um inteiro não negativo".to_owned())
    })?;

    if found != SUPPORTED_SCHEMA_VERSION {
        return Err(ContractError::UnsupportedSchemaVersion { found });
    }

    let contract: Contract = toml::from_str(text).map_err(ContractError::Parse)?;
    contract.validate()?;
    Ok(contract)
}

/// Retorna o modelo inicial usado por `auditor init`.
pub fn example_contract() -> &'static str {
    EXAMPLE_CONTRACT
}

impl Contract {
    fn validate(&self) -> Result<(), ContractError> {
        require_non_empty("project.name", &self.project.name)?;

        if self.scan.include.is_empty() {
            return Err(ContractError::Validation(
                "scan.include deve conter ao menos um padrão".to_owned(),
            ));
        }
        for pattern in &self.scan.include {
            validate_pattern("scan.include", pattern)?;
        }
        for pattern in &self.scan.exclude {
            validate_pattern("scan.exclude", pattern)?;
        }

        let mut names = HashSet::new();
        let mut paths = HashSet::new();

        for (index, module) in self.architecture.modules.iter().enumerate() {
            let label = format!("architecture.modules[{index}]");
            require_non_empty(&format!("{label}.name"), &module.name)?;
            require_non_empty(&format!("{label}.path"), &module.path)?;
            require_non_empty(&format!("{label}.responsibility"), &module.responsibility)?;
            validate_pattern(&format!("{label}.path"), &module.path)?;

            if !names.insert(module.name.as_str()) {
                return Err(ContractError::Validation(format!(
                    "nome de módulo duplicado: {}",
                    module.name
                )));
            }
            if !paths.insert(module.path.as_str()) {
                return Err(ContractError::Validation(format!(
                    "path de módulo duplicado: {}",
                    module.path
                )));
            }
        }

        for module in &self.architecture.modules {
            let mut seen = HashSet::new();
            for dependency in &module.allowed_dependencies {
                require_non_empty("allowed_dependencies", dependency)?;
                if dependency == &module.name {
                    return Err(ContractError::Validation(format!(
                        "módulo {} não pode depender de si mesmo",
                        module.name
                    )));
                }
                if !names.contains(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "módulo {} referencia dependência desconhecida: {}",
                        module.name, dependency
                    )));
                }
                if !seen.insert(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "dependência duplicada no módulo {}: {}",
                        module.name, dependency
                    )));
                }
            }
        }

        Ok(())
    }
}

fn require_non_empty(field: &str, value: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::Validation(format!(
            "{field} não pode ser vazio"
        )));
    }
    Ok(())
}

fn validate_pattern(field: &str, pattern: &str) -> Result<(), ContractError> {
    require_non_empty(field, pattern)?;

    if pattern.contains('\\') {
        return Err(ContractError::Validation(format!(
            "{field} deve usar '/' como separador: {pattern}"
        )));
    }

    let path = Path::new(pattern);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_) | Component::RootDir | Component::ParentDir
            )
        })
    {
        return Err(ContractError::Validation(format!(
            "{field} deve ser um caminho relativo sem '..': {pattern}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/contracts")
            .join(name)
    }

    #[test]
    fn valid_contract_is_loaded() {
        let contract =
            load_contract(&fixture("valid/complete.toml")).expect("the valid fixture must load");

        assert_eq!(contract.schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(contract.project.name, "projeto-exemplo");
        assert_eq!(contract.architecture.modules.len(), 2);
        assert_eq!(
            contract.architecture.modules[1].allowed_dependencies,
            ["domain"]
        );
    }

    #[test]
    fn example_contract_is_valid() {
        parse_contract(example_contract()).expect("the generated example must be valid");
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let error = load_contract(&fixture("invalid/unknown_field.toml"))
            .expect_err("unknown fields must be rejected");
        assert!(matches!(error, ContractError::Parse(_)));
    }

    #[test]
    fn unsupported_schema_version_has_clear_message() {
        let error = load_contract(&fixture("invalid/unsupported_version.toml"))
            .expect_err("schema v2 must be rejected");
        let message = error.to_string();

        assert!(matches!(
            error,
            ContractError::UnsupportedSchemaVersion { found: 2 }
        ));
        assert!(message.contains("schema_version"));
        assert!(message.contains('1'));
    }

    #[test]
    fn invalid_contracts_are_rejected() {
        for name in [
            "missing_project.toml",
            "unsupported_style.toml",
            "unknown_dependency.toml",
            "duplicate_module.toml",
            "absolute_path.toml",
            "empty_name.toml",
        ] {
            let error = load_contract(&fixture(&format!("invalid/{name}")))
                .expect_err(&format!("{name} must be rejected"));
            assert!(
                !error.to_string().is_empty(),
                "{name} must explain the error"
            );
        }
    }

    #[test]
    fn missing_file_names_the_path() {
        let missing = fixture("invalid/inexistente.toml");
        let error = load_contract(&missing).expect_err("a missing file must be reported");

        assert!(matches!(error, ContractError::Io { .. }));
        assert!(error.to_string().contains("inexistente.toml"));
    }
}
