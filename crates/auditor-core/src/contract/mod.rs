//! Leitura, validação e representação do contrato arquitetural (`auditor.toml`).
//!
//! O contrato descreve o projeto auditado, não a configuração do auditor.
//! Nenhuma função desta página modifica o contrato existente.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Versões de schema aceitas por esta versão do auditor.
pub const SUPPORTED_SCHEMA_VERSIONS: [u32; 3] = [1, 2, 3];

/// Rótulo estável das versões aceitas, usado em mensagens de erro.
fn supported_versions_label() -> String {
    let versions: Vec<String> = SUPPORTED_SCHEMA_VERSIONS
        .iter()
        .map(u32::to_string)
        .collect();
    match versions.split_last() {
        None => String::new(),
        Some((only, [])) => only.clone(),
        Some((last, head)) => format!("{} e {}", head.join(", "), last),
    }
}

/// Modelo inicial gerado por `auditor init`; ainda depende de revisão e confirmação do usuário.
pub const EXAMPLE_CONTRACT: &str = include_str!("../../../../auditor.example.toml");

/// Contrato arquitetural completo de um projeto auditado.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Nome legível do projeto.
    pub name: String,
}

/// Padrões de inclusão e exclusão usados na descoberta de arquivos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// Padrões que precisam corresponder para um arquivo entrar no inventário.
    pub include: Vec<String>,
    /// Padrões que removem caminhos mesmo quando incluídos.
    pub exclude: Vec<String>,
}

/// Organização desejada do projeto.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Architecture {
    /// Estilo arquitetural declarado.
    pub style: ArchitectureStyle,
    /// Módulos declarados e suas fronteiras de dependência.
    pub modules: Vec<Module>,
    /// Camadas declaradas no contrato v2; vazia em contratos v1.
    #[serde(default)]
    pub layers: Vec<Layer>,
    /// Domínios ou contextos delimitados do contrato v3.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<Domain>,
    /// Dispensas localizadas do contrato v3.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<Exception>,
    /// Proibições declarativas adicionais do contrato v3.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<CustomRule>,
}

/// Estilos arquiteturais reconhecidos pelo schema v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureStyle {
    /// Monólito modular, único estilo aceito nesta versão.
    ModularMonolith,
}

/// Módulo declarado no contrato.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

/// Domínio ou contexto delimitado declarado no contrato v3.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    /// Identificador único do domínio.
    pub name: String,
    /// Módulos que pertencem a este domínio.
    pub modules: Vec<String>,
    /// Domínios permitidos como destino de dependências.
    #[serde(default)]
    pub allowed_dependencies: Vec<String>,
}

/// Dispensa localizada de um achado no contrato v3.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Exception {
    /// Regra nativa ou `id` da regra personalizada dispensada.
    pub rule: String,
    /// Origem exatamente compatível com o escopo da regra.
    pub from: String,
    /// Destino exatamente compatível com o escopo da regra.
    pub to: String,
    /// Arquivo de origem exato; ausente vale para qualquer arquivo do par.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Motivo humano obrigatório, visível no relatório e no histórico.
    pub justification: String,
    /// Data de revisão humana, sem efeito automático no resultado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_on: Option<String>,
}

/// Regra personalizada declarativa do contrato v3.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomRule {
    /// Identificador único da regra.
    pub id: String,
    /// Escopo onde `from` e `to` são resolvidos.
    pub scope: RuleScope,
    /// Origem proibida no escopo escolhido.
    pub from: String,
    /// Destino proibido no escopo escolhido.
    pub to: String,
    /// Severidade do achado quando a regra é violada.
    #[serde(default)]
    pub severity: RuleSeverity,
    /// Motivo humano obrigatório, visível no relatório.
    pub justification: String,
}

/// Escopo de uma regra personalizada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleScope {
    /// `from` e `to` são nomes de módulos.
    Module,
    /// `from` e `to` são nomes de camadas.
    Layer,
    /// `from` e `to` são nomes de domínios.
    Domain,
}

impl RuleScope {
    /// Valor estável usado em mensagens e na identidade canônica.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Layer => "layer",
            Self::Domain => "domain",
        }
    }
}

/// Severidade declarada de uma regra personalizada.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    /// Violação determinística de uma proibição declarada.
    #[default]
    Error,
    /// Situação proibida que ainda exige revisão humana.
    NeedsReview,
}

impl RuleSeverity {
    /// Valor estável usado em mensagens e na identidade canônica.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::NeedsReview => "needs_review",
        }
    }
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
    /// Falha ao serializar um contrato em memória.
    Serialize(toml::ser::Error),
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
            Self::Serialize(source) => write!(formatter, "falha ao serializar contrato: {source}"),
        }
    }
}

impl Error for ContractError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse(source) => Some(source),
            Self::Validation(_) | Self::UnsupportedSchemaVersion { .. } => None,
            Self::Serialize(source) => Some(source),
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

/// Serializa um contrato em TOML determinístico, na ordem dos campos das structs.
///
/// A saída é uma prévia: esta função não grava arquivos nem altera o projeto.
pub fn render_contract(contract: &Contract) -> Result<String, ContractError> {
    toml::to_string_pretty(contract).map_err(ContractError::Serialize)
}

/// Identidade curta e determinística da forma canônica do contrato.
///
/// Contratos v1 e v2 mantêm o hash legado do TOML renderizado. Contratos v3
/// usam uma projeção semântica ordenada: descrições em `responsibility` não
/// entram na identidade, mas toda política declarada entra.
pub fn contract_identity(contract: &Contract) -> String {
    let text = if contract.schema_version >= 3 {
        canonical_identity_projection(contract)
    } else {
        render_contract(contract).expect("contrato tipado deve serializar")
    };
    format!("{:016x}", stable_identity_hash(&text))
}

/// Projeta a política de um contrato v3 em texto estável e ordenado.
fn canonical_identity_projection(contract: &Contract) -> String {
    let mut lines = vec![
        "schema_version=3".to_owned(),
        format!("project.name={:?}", contract.project.name),
    ];
    for pattern in &contract.scan.include {
        lines.push(format!("scan.include={pattern:?}"));
    }
    for pattern in &contract.scan.exclude {
        lines.push(format!("scan.exclude={pattern:?}"));
    }

    let mut modules: Vec<&Module> = contract.architecture.modules.iter().collect();
    modules.sort_by(|left, right| left.name.cmp(&right.name));
    for module in modules {
        lines.push(format!("module={:?}:{:?}", module.name, module.path));
        let mut dependencies = module.allowed_dependencies.clone();
        dependencies.sort();
        dependencies.dedup();
        for dependency in dependencies {
            lines.push(format!("  module.allow={:?}:{:?}", module.name, dependency));
        }
    }

    let mut layers: Vec<&Layer> = contract.architecture.layers.iter().collect();
    layers.sort_by(|left, right| left.name.cmp(&right.name));
    for layer in layers {
        lines.push(format!("layer={:?}", layer.name));
        let mut members = layer.modules.clone();
        members.sort();
        members.dedup();
        for member in members {
            lines.push(format!("  layer.module={:?}:{:?}", layer.name, member));
        }
        let mut dependencies = layer.allowed_dependencies.clone();
        dependencies.sort();
        dependencies.dedup();
        for dependency in dependencies {
            lines.push(format!("  layer.allow={:?}:{:?}", layer.name, dependency));
        }
    }

    let mut domains: Vec<&Domain> = contract.architecture.domains.iter().collect();
    domains.sort_by(|left, right| left.name.cmp(&right.name));
    for domain in domains {
        lines.push(format!("domain={:?}", domain.name));
        let mut members = domain.modules.clone();
        members.sort();
        members.dedup();
        for member in members {
            lines.push(format!("  domain.module={:?}:{:?}", domain.name, member));
        }
        let mut dependencies = domain.allowed_dependencies.clone();
        dependencies.sort();
        dependencies.dedup();
        for dependency in dependencies {
            lines.push(format!("  domain.allow={:?}:{:?}", domain.name, dependency));
        }
    }

    let mut exceptions: Vec<&Exception> = contract.architecture.exceptions.iter().collect();
    exceptions.sort_by(|left, right| {
        left.rule
            .cmp(&right.rule)
            .then_with(|| left.from.cmp(&right.from))
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.justification.cmp(&right.justification))
            .then_with(|| left.expires_on.cmp(&right.expires_on))
    });
    for exception in exceptions {
        lines.push(format!(
            "exception={:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
            exception.rule,
            exception.from,
            exception.to,
            exception.path,
            exception.justification,
            exception.expires_on
        ));
    }

    let mut rules: Vec<&CustomRule> = contract.architecture.rules.iter().collect();
    rules.sort_by(|left, right| left.id.cmp(&right.id));
    for rule in rules {
        lines.push(format!(
            "rule={:?}:{}:{:?}:{:?}:{}:{:?}",
            rule.id,
            rule.scope.as_str(),
            rule.from,
            rule.to,
            rule.severity.as_str(),
            rule.justification
        ));
    }

    lines.join("\n")
}

/// Hash FNV-1a de 64 bits, estável entre versões do Rust.
fn stable_identity_hash(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

impl Contract {
    /// Devolve as camadas ativas após a normalização entre versões.
    ///
    /// Contratos v1 não possuem camadas; contratos v2 podem declarar uma lista
    /// vazia quando só usam módulos.
    pub fn normalized_layers(&self) -> Option<&[Layer]> {
        (self.schema_version >= 2).then_some(self.architecture.layers.as_slice())
    }

    /// Devolve os domínios ativos após a normalização entre versões.
    ///
    /// Domínios existem somente no contrato v3; listas vazias significam que a
    /// dimensão não foi declarada.
    pub fn normalized_domains(&self) -> Option<&[Domain]> {
        (self.schema_version >= 3).then_some(self.architecture.domains.as_slice())
    }

    /// Valida campos obrigatórios, caminhos, referências e dimensões declaradas.
    fn validate(&self) -> Result<(), ContractError> {
        if self.schema_version < 2 && !self.architecture.layers.is_empty() {
            return Err(ContractError::Validation(
                "architecture.layers exige schema_version = 2".to_owned(),
            ));
        }
        self.validate_v3_fields()?;

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
        self.validate_layers(&names)?;
        self.validate_domains(&names)?;
        self.validate_custom_rules()?;
        self.validate_exceptions()
    }

    /// Rejeita dimensões do v3 em contratos mais antigos.
    fn validate_v3_fields(&self) -> Result<(), ContractError> {
        if self.schema_version >= 3 {
            return Ok(());
        }
        let fields = [
            ("architecture.domains", self.architecture.domains.len()),
            (
                "architecture.exceptions",
                self.architecture.exceptions.len(),
            ),
            ("architecture.rules", self.architecture.rules.len()),
        ];
        for (field, count) in fields {
            if count > 0 {
                return Err(ContractError::Validation(format!(
                    "{field} exige schema_version = 3"
                )));
            }
        }
        Ok(())
    }

    /// Valida domínios, cobertura opcional e referências.
    fn validate_domains(&self, module_names: &HashSet<&str>) -> Result<(), ContractError> {
        let domains = &self.architecture.domains;
        if domains.is_empty() {
            return Ok(());
        }

        let mut domain_names = HashSet::new();
        for (index, domain) in domains.iter().enumerate() {
            let label = format!("architecture.domains[{index}]");
            require_non_empty(&format!("{label}.name"), &domain.name)?;
            if !domain_names.insert(domain.name.as_str()) {
                return Err(ContractError::Validation(format!(
                    "nome de domínio duplicado em {label}.name: {}",
                    domain.name
                )));
            }
            if domain.modules.is_empty() {
                return Err(ContractError::Validation(format!(
                    "{label}.modules deve conter ao menos um módulo"
                )));
            }

            let mut seen_modules = HashSet::new();
            for module in &domain.modules {
                require_non_empty(&format!("{label}.modules"), module)?;
                if !module_names.contains(module.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label}.modules referencia módulo desconhecido: {module}"
                    )));
                }
                if !seen_modules.insert(module.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label}.modules repete o módulo {module}"
                    )));
                }
            }
        }

        let mut module_domains = BTreeMap::new();
        for domain in domains {
            for module in &domain.modules {
                if let Some(previous) = module_domains.insert(module.as_str(), domain.name.as_str())
                {
                    let module_label = self
                        .architecture
                        .modules
                        .iter()
                        .position(|candidate| candidate.name == *module)
                        .map(|index| format!("architecture.modules[{index}].name"))
                        .unwrap_or_else(|| "architecture.modules[].name".to_owned());
                    return Err(ContractError::Validation(format!(
                        "{module_label} aparece em dois domínios: {previous} e {}",
                        domain.name
                    )));
                }
            }
        }

        for (index, domain) in domains.iter().enumerate() {
            let label = format!("architecture.domains[{index}].allowed_dependencies");
            let mut seen = HashSet::new();
            for dependency in &domain.allowed_dependencies {
                require_non_empty(&label, dependency)?;
                if dependency == &domain.name {
                    return Err(ContractError::Validation(format!(
                        "{label} não pode depender de si mesmo: {}",
                        domain.name
                    )));
                }
                if !domain_names.contains(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label} referencia domínio desconhecido: {dependency}"
                    )));
                }
                if !seen.insert(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label} tem dependência duplicada: {dependency}"
                    )));
                }
            }
        }

        let graph: BTreeMap<&str, Vec<&str>> = domains
            .iter()
            .map(|domain| {
                (
                    domain.name.as_str(),
                    domain
                        .allowed_dependencies
                        .iter()
                        .map(String::as_str)
                        .collect(),
                )
            })
            .collect();
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for (index, domain) in domains.iter().enumerate() {
            if let Some(cycle) =
                find_graph_cycle(domain.name.as_str(), &graph, &mut visiting, &mut visited)
            {
                return Err(ContractError::Validation(format!(
                    "ciclo entre domínios em architecture.domains[{index}].allowed_dependencies: {}",
                    cycle.join(" -> ")
                )));
            }
        }

        Ok(())
    }

    /// Valida regras personalizadas e suas referências de escopo.
    fn validate_custom_rules(&self) -> Result<(), ContractError> {
        let rules = &self.architecture.rules;
        if rules.is_empty() {
            return Ok(());
        }

        let module_names: BTreeSet<&str> = self
            .architecture
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        let layer_names: BTreeSet<&str> = self
            .architecture
            .layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect();
        let domain_names: BTreeSet<&str> = self
            .architecture
            .domains
            .iter()
            .map(|domain| domain.name.as_str())
            .collect();

        let mut ids = HashSet::new();
        let mut pairs = HashSet::new();
        for (index, rule) in rules.iter().enumerate() {
            let label = format!("architecture.rules[{index}]");
            require_non_empty(&format!("{label}.id"), &rule.id)?;
            if !ids.insert(rule.id.as_str()) {
                return Err(ContractError::Validation(format!(
                    "id de regra personalizada duplicado: {}",
                    rule.id
                )));
            }
            require_non_empty(&format!("{label}.from"), &rule.from)?;
            require_non_empty(&format!("{label}.to"), &rule.to)?;
            if rule.from == rule.to {
                return Err(ContractError::Validation(format!(
                    "{label} não pode ter a mesma origem e destino: {}",
                    rule.from
                )));
            }
            require_non_empty(&format!("{label}.justification"), &rule.justification)?;

            let names = match rule.scope {
                RuleScope::Module => &module_names,
                RuleScope::Layer => &layer_names,
                RuleScope::Domain => &domain_names,
            };
            for field in ["from", "to"] {
                let value = if field == "from" {
                    &rule.from
                } else {
                    &rule.to
                };
                if !names.contains(value.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label}.{field} referencia {} desconhecido: {}",
                        rule.scope.as_str(),
                        value
                    )));
                }
            }

            let pair = (rule.scope.as_str(), rule.from.as_str(), rule.to.as_str());
            if !pairs.insert(pair) {
                return Err(ContractError::Validation(format!(
                    "regra personalizada duplicada para {} -> {} no escopo {}",
                    rule.from,
                    rule.to,
                    rule.scope.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Valida exceções, chaves e justificativas.
    fn validate_exceptions(&self) -> Result<(), ContractError> {
        let exceptions = &self.architecture.exceptions;
        if exceptions.is_empty() {
            return Ok(());
        }

        let module_names: BTreeSet<&str> = self
            .architecture
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        let layer_names: BTreeSet<&str> = self
            .architecture
            .layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect();
        let domain_names: BTreeSet<&str> = self
            .architecture
            .domains
            .iter()
            .map(|domain| domain.name.as_str())
            .collect();
        let mut keys = HashSet::new();
        for (index, exception) in exceptions.iter().enumerate() {
            let label = format!("architecture.exceptions[{index}]");
            require_non_empty(&format!("{label}.rule"), &exception.rule)?;
            require_non_empty(&format!("{label}.from"), &exception.from)?;
            require_non_empty(&format!("{label}.to"), &exception.to)?;
            require_non_empty(&format!("{label}.justification"), &exception.justification)?;

            let scope =
                exception_scope(&exception.rule, &self.architecture.rules).ok_or_else(|| {
                    ContractError::Validation(format!(
                        "{label}.rule referencia regra desconhecida: {}",
                        exception.rule
                    ))
                })?;
            let names = match scope {
                RuleScope::Module => &module_names,
                RuleScope::Layer => &layer_names,
                RuleScope::Domain => &domain_names,
            };
            for field in ["from", "to"] {
                let value = if field == "from" {
                    &exception.from
                } else {
                    &exception.to
                };
                if !names.contains(value.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label}.{field} referencia {} desconhecido: {}",
                        scope.as_str(),
                        value
                    )));
                }
            }

            if let Some(path) = &exception.path {
                validate_exception_path(&format!("{label}.path"), path)?;
            }
            if let Some(date) = &exception.expires_on {
                validate_date(&format!("{label}.expires_on"), date)?;
            }

            let key = (
                exception.rule.as_str(),
                exception.from.as_str(),
                exception.to.as_str(),
                exception.path.as_deref(),
            );
            if !keys.insert(key) {
                return Err(ContractError::Validation(format!(
                    "exceção duplicada: {} -> {} pela regra {}",
                    exception.from, exception.to, exception.rule
                )));
            }
        }

        Ok(())
    }

    /// Rejeita paths sobrepostos apenas em contratos v2.
    fn validate_v2_paths(&self) -> Result<(), ContractError> {
        if self.schema_version < 2 {
            return Ok(());
        }

        let modules = &self.architecture.modules;
        for (index, left) in modules.iter().enumerate() {
            for (right_index, right) in modules.iter().enumerate().skip(index + 1) {
                if patterns_overlap(&left.path, &right.path) {
                    return Err(ContractError::Validation(format!(
                        "paths de módulos sobrepostos: {} (architecture.modules[{index}].path = {}) e {} (architecture.modules[{right_index}].path = {})",
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
                    "nome de camada duplicado em {label}.name: {}",
                    layer.name
                )));
            }
            if layer.modules.is_empty() {
                return Err(ContractError::Validation(format!(
                    "{label}.modules deve conter ao menos um módulo"
                )));
            }

            let mut seen_modules = HashSet::new();
            for module in &layer.modules {
                require_non_empty(&format!("{label}.modules"), module)?;
                if !module_names.contains(module.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label}.modules referencia módulo desconhecido: {}",
                        module
                    )));
                }
                if !seen_modules.insert(module.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label}.modules repete o módulo {}",
                        module
                    )));
                }
            }
        }

        let mut module_layers = BTreeMap::new();
        for layer in layers {
            for module in &layer.modules {
                if let Some(previous) = module_layers.insert(module.as_str(), layer.name.as_str()) {
                    let module_label = self
                        .architecture
                        .modules
                        .iter()
                        .position(|candidate| candidate.name.as_str() == module.as_str())
                        .map(|index| format!("architecture.modules[{index}].name"))
                        .unwrap_or_else(|| "architecture.modules[].name".to_owned());
                    return Err(ContractError::Validation(format!(
                        "{module_label} aparece em duas camadas: {} e {}",
                        previous, layer.name
                    )));
                }
            }
        }
        for (index, module) in self.architecture.modules.iter().enumerate() {
            if !module_layers.contains_key(module.name.as_str()) {
                return Err(ContractError::Validation(format!(
                    "architecture.modules[{index}].name está sem camada: {}",
                    module.name
                )));
            }
        }

        for (index, layer) in layers.iter().enumerate() {
            let label = format!("architecture.layers[{index}].allowed_dependencies");
            let mut seen = HashSet::new();
            for dependency in &layer.allowed_dependencies {
                require_non_empty(&label, dependency)?;
                if dependency == &layer.name {
                    return Err(ContractError::Validation(format!(
                        "{label} não pode depender de si mesma: {}",
                        layer.name
                    )));
                }
                if !layer_names.contains(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label} referencia camada desconhecida: {}",
                        dependency
                    )));
                }
                if !seen.insert(dependency.as_str()) {
                    return Err(ContractError::Validation(format!(
                        "{label} tem dependência duplicada: {}",
                        dependency
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
        for (index, layer) in layers.iter().enumerate() {
            if let Some(cycle) =
                find_graph_cycle(layer.name.as_str(), &graph, &mut visiting, &mut visited)
            {
                return Err(ContractError::Validation(format!(
                    "ciclo entre camadas em architecture.layers[{index}].allowed_dependencies: {}",
                    cycle.join(" -> ")
                )));
            }
        }

        Ok(())
    }
}

/// Procura um ciclo em um grafo de dimensões e devolve o caminho encontrado.
fn find_graph_cycle<'a>(
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
        if let Some(mut cycle) = find_graph_cycle(next, graph, visiting, visited) {
            cycle.insert(0, node);
            return Some(cycle);
        }
    }
    visiting.remove(node);
    visited.insert(node);
    None
}

/// Resolve o escopo de uma regra citada por uma exceção.
fn exception_scope(rule: &str, custom_rules: &[CustomRule]) -> Option<RuleScope> {
    match rule {
        "dep.not_allowed" => Some(RuleScope::Module),
        "dep.layer_not_allowed" => Some(RuleScope::Layer),
        "dep.domain_not_allowed" => Some(RuleScope::Domain),
        _ => custom_rules
            .iter()
            .find(|candidate| candidate.id == rule)
            .map(|candidate| candidate.scope),
    }
}

/// Valida um caminho exato de exceção, sem curingas e sem travessia para fora da raiz.
fn validate_exception_path(field: &str, path: &str) -> Result<(), ContractError> {
    validate_pattern(field, path)?;
    if path.contains('*') || path.contains('?') {
        return Err(ContractError::Validation(format!(
            "{field} deve ser um caminho exato, sem curingas: {path}"
        )));
    }
    Ok(())
}

/// Valida uma data `YYYY-MM-DD` sem depender de relógio ou biblioteca externa.
fn validate_date(field: &str, value: &str) -> Result<(), ContractError> {
    let invalid = || {
        ContractError::Validation(format!(
            "{field} deve ser uma data YYYY-MM-DD válida: {value}"
        ))
    };
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return Err(invalid());
    }

    let year = value[0..4].parse::<i32>().map_err(|_| invalid())?;
    let month = value[5..7].parse::<u32>().map_err(|_| invalid())?;
    let day = value[8..10].parse::<u32>().map_err(|_| invalid())?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err(invalid()),
    };
    if day == 0 || day > max_day {
        return Err(invalid());
    }
    Ok(())
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
    fn rendered_contract_round_trips_through_the_parser() {
        for source in [
            example_contract().to_owned(),
            v2_contract().to_owned(),
            v3_contract_text(),
        ] {
            let contract = parse_contract(&source).expect("fixture must be valid");
            let text = render_contract(&contract).expect("contract must serialize");
            assert_eq!(
                parse_contract(&text).expect("rendered TOML must be valid"),
                contract
            );
            assert_eq!(render_contract(&contract).expect("must re-render"), text);
        }
    }

    #[test]
    fn contract_identity_is_stable_and_canonical() {
        let first = parse_contract(v2_contract()).expect("first contract must load");
        let reordered = v2_contract()
            .replace(
                "[project]\nname = \"contrato-v2\"\n\n[scan]\ninclude = [\"src/**/*.rs\"]\nexclude = []",
                "[scan]\ninclude = [\"src/**/*.rs\"]\nexclude = []\n\n[project]\nname = \"contrato-v2\"",
            );
        let second = parse_contract(&reordered).expect("reordered contract must load");

        assert_eq!(contract_identity(&first), contract_identity(&second));
        assert_eq!(contract_identity(&first).len(), 16);
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
            .expect_err("schema v4 must be rejected");
        let message = error.to_string();

        assert!(matches!(
            error,
            ContractError::UnsupportedSchemaVersion { found: 4 }
        ));
        assert!(message.contains("schema_version"));
        assert!(message.contains('1'));
        assert!(message.contains('2'));
        assert!(message.contains('3'));
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
    fn v2_rejects_schema_zero_with_field_name() {
        let text = v2_contract().replace("schema_version = 2", "schema_version = 0");
        let error = parse_contract(&text).expect_err("schema v0 must be rejected");

        assert!(matches!(
            error,
            ContractError::UnsupportedSchemaVersion { found: 0 }
        ));
        assert!(error.to_string().contains("schema_version"));
    }

    #[test]
    fn v1_contract_cannot_declare_layers() {
        let text = v2_contract().replace("schema_version = 2", "schema_version = 1");
        let error = parse_contract(&text).expect_err("v1 must not declare layers");

        assert!(error.to_string().contains("architecture.layers"));
        assert!(error.to_string().contains("schema_version = 2"));
    }

    #[test]
    fn normalized_layers_distinguish_v1_from_v2() {
        let v2 = parse_contract(v2_contract()).expect("v2 contract must load");
        let v1_text = v2_contract()
            .split("[[architecture.layers]]")
            .next()
            .expect("prefix must exist")
            .replace("schema_version = 2", "schema_version = 1");
        let v1 = parse_contract(&v1_text).expect("v1 contract must load");

        assert!(v1.normalized_layers().is_none());
        assert_eq!(v2.normalized_layers().map(<[Layer]>::len), Some(2));
    }

    #[test]
    fn v2_field_order_does_not_change_the_normalized_contract() {
        let reordered = r#"
schema_version = 2

[scan]
include = ["src/**/*.rs"]
exclude = []

[project]
name = "contrato-v2"

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
"#;
        let first = parse_contract(v2_contract()).expect("canonical contract must load");
        let second = parse_contract(reordered).expect("reordered contract must load");

        assert_eq!(first, second);
    }

    #[test]
    fn v2_layer_errors_name_the_field_path() {
        let cases = [
            (
                v2_contract().replace(
                    "name = \"domain\"\nmodules = [\"domain\"]",
                    "name = \"\"\nmodules = [\"domain\"]",
                ),
                "architecture.layers[0].name",
            ),
            (
                v2_contract().replace(
                    "name = \"application\"\nmodules = [\"application\"]",
                    "name = \"domain\"\nmodules = [\"application\"]",
                ),
                "architecture.layers[1].name",
            ),
            (
                v2_contract().replace("modules = [\"domain\"]", "modules = [\"missing\"]"),
                "architecture.layers[0].modules",
            ),
            (
                v2_contract().replace(
                    "modules = [\"domain\"]",
                    "modules = [\"domain\", \"domain\"]",
                ),
                "architecture.layers[0].modules",
            ),
            (
                v2_contract()
                    .split("[[architecture.layers]]\nname = \"application\"")
                    .next()
                    .expect("prefix must exist")
                    .to_owned(),
                "architecture.modules[1].name",
            ),
            (
                v2_contract().replace(
                    "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\"]",
                    "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"application\"]",
                ),
                "architecture.layers[1].allowed_dependencies",
            ),
            (
                v2_contract().replace(
                    "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\"]",
                    "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"missing\"]",
                ),
                "architecture.layers[1].allowed_dependencies",
            ),
            (
                v2_contract().replace(
                    "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\"]",
                    "name = \"application\"\nmodules = [\"application\"]\nallowed_dependencies = [\"domain\", \"domain\"]",
                ),
                "architecture.layers[1].allowed_dependencies",
            ),
            (
                v2_contract().replace(
                    "name = \"domain\"\nmodules = [\"domain\"]\nallowed_dependencies = []",
                    "name = \"domain\"\nmodules = [\"domain\"]\nallowed_dependencies = [\"application\"]",
                ),
                "architecture.layers[0].allowed_dependencies",
            ),
            (
                v2_contract().replace("path = \"src/application/**\"", "path = \"src/**\""),
                "architecture.modules[1].path",
            ),
        ];

        for (text, field) in cases {
            let error = parse_contract(&text).expect_err("invalid v2 contract must be rejected");
            assert!(
                error.to_string().contains(field),
                "{field} missing from: {error}"
            );
        }
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

    /// Texto do contrato v3 completo usado nos testes de compatibilidade.
    fn v3_contract_text() -> String {
        std::fs::read_to_string(fixture_path("valid/contract_v3.toml"))
            .expect("a fixture v3 deve ser legível")
    }

    #[test]
    fn v1_and_v2_keep_their_legacy_identities() {
        let v1 = load_contract(&fixture_path("valid/complete.toml")).expect("v1 deve carregar");
        assert_eq!(contract_identity(&v1), "b345bf2885de375d");

        let v2_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/layer_boundaries/auditor.toml");
        let v2 = load_contract(&v2_path).expect("v2 deve carregar");
        assert_eq!(contract_identity(&v2), "4ea90b714bd2a819");
    }

    #[test]
    fn v3_contract_with_domains_exceptions_and_rules_is_accepted() {
        let contract = parse_contract(&v3_contract_text()).expect("contrato v3 válido");

        assert_eq!(contract.schema_version, 3);
        assert_eq!(contract.architecture.domains.len(), 3);
        assert_eq!(contract.architecture.exceptions.len(), 2);
        assert_eq!(contract.architecture.rules.len(), 1);
        assert_eq!(contract.normalized_domains().map(<[Domain]>::len), Some(3));
        assert_eq!(contract.architecture.rules[0].scope, RuleScope::Module);
    }

    #[test]
    fn v3_rejects_unknown_fields_with_the_field_name() {
        let text = v3_contract_text().replace(
            "justification = \"A aplicação deve ser acionada por um caso de uso explícito\"",
            "script = \"executar()\"",
        );
        let error = parse_contract(&text).expect_err("script não é campo do schema");

        assert!(matches!(error, ContractError::Parse(_)));
        assert!(error.to_string().contains("script"), "{error}");
    }

    #[test]
    fn v3_domain_validation_names_the_problem() {
        let cases = [
            (
                format!(
                    "{}\n[[architecture.domains]]\nname = \"duplicado\"\nmodules = [\"domain\"]\nallowed_dependencies = []\n",
                    v3_contract_text()
                ),
                "aparece em dois domínios",
            ),
            (
                format!(
                    "{}\n[[architecture.domains]]\nname = \"vazio\"\nmodules = []\nallowed_dependencies = []\n",
                    v3_contract_text()
                ),
                "deve conter ao menos um módulo",
            ),
            (
                v3_contract_text().replace(
                    "name = \"sales\"\nmodules = [\"application\"]\nallowed_dependencies = []",
                    "name = \"sales\"\nmodules = [\"application\"]\nallowed_dependencies = [\"core\"]",
                ),
                "ciclo entre domínios",
            ),
            (
                v3_contract_text().replace(
                    "name = \"sales\"\nmodules = [\"application\"]\nallowed_dependencies = []",
                    "name = \"sales\"\nmodules = [\"application\"]\nallowed_dependencies = [\"sales\"]",
                ),
                "si mesmo",
            ),
        ];

        for (text, expected) in cases {
            let error = parse_contract(&text).expect_err("domínio inválido deve falhar");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn v3_exception_validation_names_the_problem() {
        let cases = [
            (
                v3_contract_text().replace(
                    "justification = \"Integração temporária revisada pelo time\"\n",
                    "",
                ),
                "justification",
            ),
            (
                v3_contract_text().replace(
                    "rule = \"dep.domain_not_allowed\"",
                    "rule = \"dep.desconhecida\"",
                ),
                "regra desconhecida",
            ),
            (
                v3_contract_text().replace(
                    "path = \"src/application/service.rs\"",
                    "path = \"../application/service.rs\"",
                ),
                "sem '..'",
            ),
            (
                v3_contract_text().replace(
                    "path = \"src/application/service.rs\"",
                    "path = \"src/application/*.rs\"",
                ),
                "sem curingas",
            ),
            (
                v3_contract_text()
                    .replace("expires_on = \"2026-12-31\"", "expires_on = \"2026-02-30\""),
                "data YYYY-MM-DD válida",
            ),
        ];

        for (text, expected) in cases {
            let error = parse_contract(&text).expect_err("exceção inválida deve falhar");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn v3_rejects_duplicate_exception_keys() {
        let text = format!(
            "{}\n[[architecture.exceptions]]\nrule = \"dep.domain_not_allowed\"\nfrom = \"sales\"\nto = \"billing\"\npath = \"src/application/service.rs\"\njustification = \"Segunda decisão\"\n",
            v3_contract_text()
        );
        let error = parse_contract(&text).expect_err("chave duplicada deve falhar");

        assert!(error.to_string().contains("exceção duplicada"), "{error}");
    }

    #[test]
    fn older_versions_reject_v3_only_fields() {
        let v2 = v3_contract_text().replace("schema_version = 3", "schema_version = 2");
        let error = parse_contract(&v2).expect_err("v2 não admite campos v3");
        assert!(
            error.to_string().contains("architecture.domains"),
            "{error}"
        );
        assert!(error.to_string().contains("schema_version = 3"), "{error}");

        let v1 = format!(
            "{}\n[[architecture.domains]]\nname = \"dominio\"\nmodules = [\"domain\"]\nallowed_dependencies = []\n",
            std::fs::read_to_string(fixture_path("valid/complete.toml"))
                .expect("fixture v1 deve ser legível")
        );
        let error = parse_contract(&v1).expect_err("v1 não admite campos v3");
        assert!(
            error.to_string().contains("architecture.domains"),
            "{error}"
        );
        assert!(error.to_string().contains("schema_version = 3"), "{error}");
    }

    #[test]
    fn v3_identity_is_canonical_and_policy_sensitive() {
        let first =
            load_contract(&fixture_path("valid/contract_v3.toml")).expect("v3 deve carregar");
        let reordered = load_contract(&fixture_path("valid/contract_v3_reordered.toml"))
            .expect("v3 reordenado deve carregar");
        assert_eq!(contract_identity(&first), contract_identity(&reordered));

        let changed_rule =
            parse_contract(&v3_contract_text().replace("no-direct-application", "regra-diferente"))
                .expect("regra alterada deve ser válida");
        assert_ne!(contract_identity(&first), contract_identity(&changed_rule));

        let changed_description = parse_contract(
            &v3_contract_text().replace("Regras de negócio", "Outra descrição legível"),
        )
        .expect("descrição alterada deve ser válida");
        assert_eq!(
            contract_identity(&first),
            contract_identity(&changed_description)
        );
    }
}
