//! Leitura, validação e representação do contrato arquitetural (`auditor.toml`).
//!
//! O contrato descreve o projeto auditado, não a configuração do auditor.
//! Nenhuma função desta página modifica o contrato existente.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

/// Versões de schema aceitas por esta versão do auditor.
pub const SUPPORTED_SCHEMA_VERSIONS: [u32; 2] = [1, 2];

/// Rótulo estável das versões aceitas, usado em mensagens de erro.
fn supported_versions_label() -> String {
    SUPPORTED_SCHEMA_VERSIONS
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(" e ")
}

/// Modelo inicial gerado por `auditor init`; ainda depende de revisão e confirmação do usuário.
pub const EXAMPLE_CONTRACT: &str = include_str!("../../../../auditor.example.toml");

/// Contrato arquitetural completo de um projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    /// Versão do schema usada no arquivo.
    pub schema_version: u32,
    /// Identificação do projeto auditado.
    pub project: Project,
    /// Regras de inclusão e exclusão da descoberta.
    pub scan: Scan,
    /// Organização desejada pelo usuário.
    pub architecture: Architecture,
}

/// Identificação do projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Nome legível do projeto.
    pub name: String,
}

/// Padrões de inclusão e exclusão usados na descoberta de arquivos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// Padrões que precisam corresponder para um arquivo entrar no inventário.
    pub include: Vec<String>,
    /// Padrões que removem caminhos mesmo quando incluídos.
    pub exclude: Vec<String>,
}

/// Organização desejada do projeto.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Architecture {
    /// Estilo arquitetural declarado.
    pub style: ArchitectureStyle,
    /// Módulos declarados e suas fronteiras de dependência.
    pub modules: Vec<Module>,
    /// Camadas declaradas no contrato v2; vazia em contratos v1.
    #[serde(default)]
    pub layers: Vec<Layer>,
}

/// Estilos arquiteturais reconhecidos pelo schema v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureStyle {
    /// Monólito modular, único estilo aceito nesta versão.
    ModularMonolith,
}

/// Módulo declarado no contrato.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    /// Nome usado em `allowed_dependencies` e nas arestas do grafo.
    pub name: String,
    /// Padrão de caminho que associa arquivos do inventário a este módulo.
    pub path: String,
    /// Descrição legível da responsabilidade do módulo.
    pub responsibility: String,
    /// Nomes de módulos permitidos como destino de dependências.
    #[serde(default)]
    pub allowed_dependencies: Vec<String>,
}

/// Camada arquitetural declarada no contrato v2.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    /// Nome usado em `allowed_dependencies` das camadas.
    pub name: String,
    /// Módulos que pertencem a esta camada.
    pub modules: Vec<String>,
    /// Camadas permitidas como destino de dependências desta camada.
    #[serde(default)]
    pub allowed_dependencies: Vec<String>,
}

/// Erros de leitura, parsing e validação do contrato.
#[derive(Debug)]
pub enum ContractError {
    /// Falha ao ler o arquivo de contrato.
    Io {
        /// Caminho informado pelo usuário.
        path: PathBuf,
        /// Erro de E/S original.
        source: std::io::Error,
    },
    /// Sintaxe TOML inválida ou campos inesperados.
    Parse(toml::de::Error),
    /// Versão de schema fora das suportadas.
    UnsupportedSchemaVersion {
        /// Versão encontrada no arquivo.
        found: u32,
    },
    /// Contrato sintaticamente válido, mas semanticamente inválido.
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
                "schema_version {found} não é suportado; esta versão do auditor aceita {}",
                supported_versions_label()
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

    if !SUPPORTED_SCHEMA_VERSIONS.contains(&found) {
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
    /// Valida campos obrigatórios, caminhos, referências e camadas.
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

        self.validate_v2_paths()?;
        self.validate_layers(&names)
    }

    /// Rejeita paths sobrepostos apenas em contratos v2.
    fn validate_v2_paths(&self) -> Result<(), ContractError> {
        if self.schema_version < 2 {
            return Ok(());
        }

        let modules = &self.architecture.modules;
        for (index, left) in modules.iter().enumerate() {
            for right in modules.iter().skip(index + 1) {
                if patterns_overlap(&left.path, &right.path) {
                    return Err(ContractError::Validation(format!(
                        "paths de módulos sobrepostos: {} ({}) e {} ({})",
                        left.name, left.path, right.name, right.path
                    )));
                }
            }
        }

        Ok(())
    }

    /// Valida camadas, cobertura de módulos, referências e direção declarada.
    fn validate_layers(&self, module_names: &HashSet<&str>) -> Result<(), ContractError> {
        let layers = &self.architecture.layers;
        if layers.is_empty() {
            return Ok(());
        }

        let mut layer_names = HashSet::new();
        for (index, layer) in layers.iter().enumerate() {
            let label = format!("architecture.layers[{index}]");
            require_non_empty(&format!("{label}.name"), &layer.name)?;
            if !layer_names.insert(layer.name.as_str()) {
                return Err(ContractError::Validation(format!(
                    "nome de camada duplicado: {}",
                    layer.name
                )));
            }
            if layer.modules.is_empty() {
                return Err(ContractError::Validation(format!(
                    "camada {} deve conter ao menos um módulo",
                    layer.name
                )));
            }

            let mut seen_modules = HashSet::new();
            for module in &layer.modules {
                require_non_empty(&format!("{label}.modules"), module)?;
                if !module_names.contains(module.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "camada {} referencia módulo desconhecido: {}",
                        layer.name, module
                    )));
                }
                if !seen_modules.insert(module.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "módulo duplicado na camada {}: {}",
                        layer.name, module
                    )));
                }
            }
        }

        let mut module_layers = BTreeMap::new();
        for layer in layers {
            for module in &layer.modules {
                if let Some(previous) = module_layers.insert(module.as_str(), layer.name.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "módulo {} aparece em duas camadas: {} e {}",
                        module, previous, layer.name
                    )));
                }
            }
        }
        for module in &self.architecture.modules {
            if !module_layers.contains_key(module.name.as_str()) {
                return Err(ContractError::Validation(format!(
                    "módulo {} está sem camada",
                    module.name
                )));
            }
        }

        for layer in layers {
            let mut seen = HashSet::new();
            for dependency in &layer.allowed_dependencies {
                require_non_empty("allowed_dependencies", dependency)?;
                if dependency == &layer.name {
                    return Err(ContractError::Validation(format!(
                        "camada {} não pode depender de si mesma",
                        layer.name
                    )));
                }
                if !layer_names.contains(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "camada {} referencia camada desconhecida: {}",
                        layer.name, dependency
                    )));
                }
                if !seen.insert(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "dependência duplicada na camada {}: {}",
                        layer.name, dependency
                    )));
                }
            }
        }

        let graph: BTreeMap<&str, Vec<&str>> = layers
            .iter()
            .map(|layer| {
                (
                    layer.name.as_str(),
                    layer
                        .allowed_dependencies
                        .iter()
                        .map(String::as_str)
                        .collect(),
                )
            })
            .collect();
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for layer in layers {
            if let Some(cycle) =
                find_layer_cycle(layer.name.as_str(), &graph, &mut visiting, &mut visited)
            {
                return Err(ContractError::Validation(format!(
                    "ciclo entre camadas: {}",
                    cycle.join(" -> ")
                )));
            }
        }

        Ok(())
    }
}

/// Procura um ciclo no grafo de camadas e devolve o caminho encontrado.
fn find_layer_cycle<'a>(
    node: &'a str,
    graph: &BTreeMap<&'a str, Vec<&'a str>>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Option<Vec<&'a str>> {
    if visiting.contains(node) {
        return Some(vec![node]);
    }
    if visited.contains(node) {
        return None;
    }

    visiting.insert(node);
    for next in graph.get(node).into_iter().flatten().copied() {
        if let Some(mut cycle) = find_layer_cycle(next, graph, visiting, visited) {
            cycle.insert(0, node);
            return Some(cycle);
        }
    }
    visiting.remove(node);
    visited.insert(node);
    None
}

/// Compara dois paths de forma conservadora para detectar sobreposição.
fn patterns_overlap(left: &str, right: &str) -> bool {
    fn components(pattern: &str) -> Vec<&str> {
        let mut parts: Vec<&str> = pattern.split('/').collect();
        while parts.last() == Some(&"**") {
            parts.pop();
        }
        parts
    }

    let left = components(left);
    let right = components(right);
    let shared = left.len().min(right.len());

    for index in 0..shared {
        if left[index] == right[index] {
            continue;
        }
        let has_wildcard = |part: &str| part.contains('*') || part.contains('?');
        if has_wildcard(left[index]) || has_wildcard(right[index]) {
            return true;
        }
        return false;
    }

    true
}

/// Exige que um campo obrigatório não contenha apenas espaços.
fn require_non_empty(field: &str, value: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::Validation(format!(
            "{field} não pode ser vazio"
        )));
    }
    Ok(())
}

/// Valida que um padrão seja relativo, use `/` e não contenha `..`.
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

    /// Caminho absoluto de um contrato de teste, relativo ao crate.
    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/contracts")
            .join(name)
    }

    #[test]
    fn valid_contract_is_loaded() {
        let contract = load_contract(&fixture_path("valid/complete.toml"))
            .expect("the valid fixture must load");

        assert_eq!(contract.schema_version, 1);
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
        let error = load_contract(&fixture_path("invalid/unknown_field.toml"))
            .expect_err("unknown fields must be rejected");
        assert!(matches!(error, ContractError::Parse(_)));
    }

    #[test]
    fn unsupported_schema_version_has_clear_message() {
        let error = load_contract(&fixture_path("invalid/unsupported_version.toml"))
            .expect_err("schema v3 must be rejected");
        let message = error.to_string();

        assert!(matches!(
            error,
            ContractError::UnsupportedSchemaVersion { found: 3 }
        ));
        assert!(message.contains("schema_version"));
        assert!(message.contains('1'));
        assert!(message.contains('2'));
    }

    /// Contrato v2 mínimo com duas camadas válidas.
    fn v2_contract() -> &'static str {
        r#"
schema_version = 2

[project]
name = "contrato-v2"

[scan]
include = ["src/**/*.rs"]
exclude = []

[architecture]
style = "modular_monolith"

[[architecture.modules]]
name = "domain"
path = "src/domain/**"
responsibility = "Domínio"
allowed_dependencies = []

[[architecture.modules]]
name = "application"
path = "src/application/**"
responsibility = "Aplicação"
allowed_dependencies = ["domain"]

[[architecture.layers]]
name = "domain"
modules = ["domain"]
allowed_dependencies = []

[[architecture.layers]]
name = "application"
modules = ["application"]
allowed_dependencies = ["domain"]
"#
    }

    #[test]
    fn v2_contract_with_layers_is_accepted() {
        let contract = parse_contract(v2_contract()).expect("valid v2 contract must load");

        assert_eq!(contract.schema_version, 2);
        assert_eq!(contract.project.name, "contrato-v2");
        assert_eq!(contract.architecture.modules.len(), 2);
        assert_eq!(contract.architecture.layers.len(), 2);
        assert_eq!(contract.architecture.layers[1].name, "application");
        assert_eq!(contract.architecture.layers[1].modules, ["application"]);
        assert_eq!(
            contract.architecture.layers[1].allowed_dependencies,
            ["domain"]
        );
    }

    #[test]
    fn v2_contract_without_layers_is_accepted() {
        let text = v2_contract()
            .split("[[architecture.layers]]")
            .next()
            .expect("prefix must exist");
        let contract = parse_contract(text).expect("v2 without layers must remain valid");

        assert_eq!(contract.schema_version, 2);
        assert!(contract.architecture.layers.is_empty());
    }

    #[test]
    fn v2_rejects_unknown_layer_field() {
        let text = v2_contract().replace(
            "modules = [\"domain\"]",
            "modules = [\"domain\"]\npriority = 1",
        );
        let error = parse_contract(&text).expect_err("unknown layer field must be rejected");

        assert!(error.to_string().contains("priority"));
    }

    #[test]
    fn v2_rejects_duplicate_layer_name() {
        let text = v2_contract().replace(
            "name = \"application\"\nmodules = [\"application\"]",
            "name = \"domain\"\nmodules = [\"application\"]",
        );
        let error = parse_contract(&text).expect_err("duplicate layer name must be rejected");

        assert!(error.to_string().contains("camada duplicado"));
    }

    #[test]
    fn v2_rejects_unknown_layer_module() {
        let text = v2_contract().replace("modules = [\"domain\"]", "modules = [\"missing\"]");
        let error = parse_contract(&text).expect_err("unknown module must be rejected");

        assert!(error.to_string().contains("módulo desconhecido"));
    }

    #[test]
    fn v2_rejects_module_in_two_layers() {
        let text = format!(
            "{}\n[[architecture.layers]]\nname = \"shared\"\nmodules = [\"domain\", \"application\"]\nallowed_dependencies = []\n",
            v2_contract()
        );
        let error = parse_contract(&text).expect_err("module in two layers must be rejected");

        assert!(error.to_string().contains("duas camadas"));
    }

    #[test]
    fn v2_rejects_module_without_layer() {
        let marker = "[[architecture.layers]]\nname = \"application\"";
        let index = v2_contract()
            .find(marker)
            .expect("second layer marker must exist");
        let text = &v2_contract()[..index];
        let error = parse_contract(text).expect_err("module without layer must be rejected");

        assert!(error.to_string().contains("sem camada"));
    }

    #[test]
    fn v2_rejects_layer_cycle() {
        let text = v2_contract().replace(
            "name = \"domain\"\nmodules = [\"domain\"]\nallowed_dependencies = []",
            "name = \"domain\"\nmodules = [\"domain\"]\nallowed_dependencies = [\"application\"]",
        );
        let error = parse_contract(&text).expect_err("layer cycle must be rejected");

        assert!(error.to_string().contains("ciclo entre camadas"));
    }

    #[test]
    fn v2_rejects_self_layer_dependency() {
        let text = v2_contract().replace(
            "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\"]",
            "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"application\"]",
        );
        let error = parse_contract(&text).expect_err("self layer dependency must be rejected");

        assert!(error.to_string().contains("si mesma"));
    }

    #[test]
    fn v2_rejects_duplicate_layer_dependency() {
        let text = v2_contract().replace(
            "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\"]",
            "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\", \"domain\"]",
        );
        let error = parse_contract(&text).expect_err("duplicate layer dependency must be rejected");

        assert!(error.to_string().contains("duplicada"));
    }

    #[test]
    fn v2_rejects_overlapping_module_paths() {
        let text = v2_contract().replace("path = \"src/application/**\"", "path = \"src/**\"");
        let error = parse_contract(&text).expect_err("overlapping paths must be rejected");

        assert!(error.to_string().contains("sobrepostos"));
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
            let error = load_contract(&fixture_path(&format!("invalid/{name}")))
                .expect_err(&format!("{name} must be rejected"));
            assert!(
                !error.to_string().is_empty(),
                "{name} must explain the error"
            );
        }
    }

    #[test]
    fn missing_file_names_the_path() {
        let missing = fixture_path("invalid/inexistente.toml");
        let error = load_contract(&missing).expect_err("a missing file must be reported");

        assert!(matches!(error, ContractError::Io { .. }));
        assert!(error.to_string().contains("inexistente.toml"));
    }

    #[test]
    fn required_fields_reject_blank_values() {
        assert!(require_non_empty("project.name", "   ").is_err());
        assert!(require_non_empty("project.name", "projeto").is_ok());
    }

    #[test]
    fn patterns_reject_unsafe_paths() {
        assert!(validate_pattern("scan.include", "crates/**/*.rs").is_ok());
        assert!(validate_pattern("scan.include", "crates\\app.rs").is_err());
        assert!(validate_pattern("scan.include", "/abs/root.rs").is_err());
        assert!(validate_pattern("scan.include", "../root.rs").is_err());
    }
}
