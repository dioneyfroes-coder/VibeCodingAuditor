//! Leitura declarativa de manifests de outros ecossistemas.
//!
//! Nada aqui executa scripts, instala pacotes ou decide arquitetura. Cada
//! manifesto lido vira uma observação com caminho relativo, origem do metadado
//! e evidência curta; formatos parcialmente suportados viram limitações
//! localizadas em vez de serem descartados em silêncio.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value as JsonValue;

use super::DEFAULT_IGNORED_DIRS;
use super::candidates::{manifest_directory, package_path};

/// Ecossistema que define a gramática do manifesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestEcosystem {
    /// `package.json`, lido como JSON sem executar `scripts`.
    Npm,
    /// `pyproject.toml`, no subconjunto PEP 621 de `[project]`.
    Python,
    /// `go.mod`, no subconjunto formalmente suportado abaixo.
    Go,
}

impl ManifestEcosystem {
    /// Nome estável usado em texto, Markdown e JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Python => "python",
            Self::Go => "go",
        }
    }
}

/// Resultado da leitura de um manifesto presente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestStatus {
    /// O subconjunto suportado foi extraído.
    Parsed,
    /// O conteúdo não pôde ser interpretado; a limitação é localizada.
    Malformed,
    /// O formato presente é conhecido, mas não entra no escopo desta etapa.
    Unsupported,
}

impl ManifestStatus {
    /// Nome estável usado em texto, Markdown e JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parsed => "parsed",
            Self::Malformed => "malformed",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Papel da dependência, preservado sem converter tudo em uma lista só.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyScope {
    /// `dependencies` ou `require` comum.
    Runtime,
    /// `devDependencies`.
    Development,
    /// `peerDependencies`.
    Peer,
    /// `optionalDependencies`.
    Optional,
    /// Grupo de `optional-dependencies` do PEP 621.
    OptionalGroup,
    /// Relação `workspace:` ou `replace` com destino local.
    Workspace,
    /// `require` de `go.mod`.
    Require,
    /// `replace` de `go.mod`.
    Replace,
    /// `exclude` de `go.mod`.
    Exclude,
}

impl DependencyScope {
    /// Nome estável usado em texto, Markdown e JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Development => "development",
            Self::Peer => "peer",
            Self::Optional => "optional",
            Self::OptionalGroup => "optional_group",
            Self::Workspace => "workspace",
            Self::Require => "require",
            Self::Replace => "replace",
            Self::Exclude => "exclude",
        }
    }
}

/// Destino de uma relação declarada no manifesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyDestination {
    /// Pacote externo, sem evidência de que pertença ao projeto.
    External,
    /// Pacote local reconhecido em workspace ou substituição relativa.
    LocalWorkspace,
}

impl DependencyDestination {
    /// Nome estável usado em texto, Markdown e JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::External => "external",
            Self::LocalWorkspace => "local_workspace",
        }
    }
}

/// Pacote local declarado por um manifesto, nunca aprovado automaticamente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalPackage {
    /// Nome declarado no manifesto.
    pub name: String,
    /// Versão declarada, quando existir.
    pub version: Option<String>,
    /// Manifesto de origem, relativo à raiz.
    pub manifest_path: String,
    /// Padrão de caminho sugerido para o código do pacote.
    pub source_path: String,
    /// Trechos curtos que permitem verificar a observação.
    pub evidence: Vec<String>,
}

/// Dependência declarada em manifesto, com papel e destino separados.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestDependency {
    /// Manifesto que declarou a dependência.
    pub source_manifest: String,
    /// Nome normalizado da dependência.
    pub name: String,
    /// Especificação bruta declarada, sem resolução ou download.
    pub version: String,
    /// Papel da dependência no manifesto.
    pub scope: DependencyScope,
    /// Indica se a relação aponta para pacote local conhecido.
    pub destination: DependencyDestination,
    /// Grupo opcional, quando o formato possuir grupos nomeados.
    pub group: Option<String>,
    /// Linha da declaração, quando o formato permitir localizá-la.
    pub line: Option<usize>,
    /// Texto curto da declaração, usado como evidência.
    pub evidence: String,
}

/// Observação produzida por um único manifesto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestObservation {
    /// Ecossistema que interpreta o manifesto.
    pub ecosystem: ManifestEcosystem,
    /// Caminho relativo do manifesto.
    pub path: String,
    /// Estado da leitura.
    pub status: ManifestStatus,
    /// Pacotes locais declarados, inclusive workspaces reconhecidos.
    pub local_packages: Vec<LocalPackage>,
    /// Dependências declaradas, separadas por papel e destino.
    pub dependencies: Vec<ManifestDependency>,
    /// Limitações específicas deste manifesto.
    pub limitations: Vec<String>,
}

/// Resultado agregado da leitura de manifests, antes de virar limitações gerais.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManifestDiscovery {
    /// Observações em ordem estável de caminho.
    pub observations: Vec<ManifestObservation>,
    /// Limitações da busca, também em ordem estável.
    pub limitations: Vec<String>,
}

/// Lê manifests presentes na raiz e em subprojetos, sem executar nada.
pub fn read_manifests(root: &Path, files: &BTreeSet<&str>) -> ManifestDiscovery {
    let mut limitations = Vec::new();
    let paths = collect_manifest_paths(root, &mut limitations);
    let mut observations = Vec::new();

    for (relative, absolute, ecosystem) in paths {
        let mut observation = ManifestObservation {
            ecosystem,
            path: relative.clone(),
            status: ManifestStatus::Parsed,
            local_packages: Vec::new(),
            dependencies: Vec::new(),
            limitations: Vec::new(),
        };
        let text = match fs::read_to_string(&absolute) {
            Ok(text) => text,
            Err(error) => {
                observation.status = ManifestStatus::Malformed;
                observation.limitations.push(format!(
                    "manifesto {relative} não pôde ser lido: {}",
                    brief_error(&error)
                ));
                observations.push(observation);
                continue;
            }
        };

        let mut manifest_limitations = Vec::new();
        match ecosystem {
            ManifestEcosystem::Npm => parse_npm(
                root,
                &relative,
                &text,
                files,
                &mut observation,
                &mut manifest_limitations,
            ),
            ManifestEcosystem::Python => parse_python(
                &relative,
                &text,
                files,
                &mut observation,
                &mut manifest_limitations,
            ),
            ManifestEcosystem::Go => {
                parse_go(&relative, &text, files, &mut observation);
                limitations.push(format!(
                    "manifesto {relative} lido; arquivos .go continuam sem analisador determinístico"
                ));
            }
        }
        observation.limitations.extend(manifest_limitations);

        observation.local_packages.sort_by(|left, right| {
            (
                &left.name,
                &left.manifest_path,
                &left.version,
                &left.evidence,
            )
                .cmp(&(
                    &right.name,
                    &right.manifest_path,
                    &right.version,
                    &right.evidence,
                ))
        });
        observation.local_packages.dedup();
        observation.dependencies.sort_by(|left, right| {
            (
                &left.source_manifest,
                &left.name,
                left.scope.as_str(),
                &left.version,
                left.line.unwrap_or_default(),
                left.destination.as_str(),
                &left.group,
                &left.evidence,
            )
                .cmp(&(
                    &right.source_manifest,
                    &right.name,
                    right.scope.as_str(),
                    &right.version,
                    right.line.unwrap_or_default(),
                    right.destination.as_str(),
                    &right.group,
                    &right.evidence,
                ))
        });
        observation.dependencies.dedup();
        observation.limitations.sort();
        observation.limitations.dedup();
        observations.push(observation);
    }

    resolve_workspace_references(&mut observations);
    observations.sort_by(|left, right| {
        (&left.path, left.ecosystem.as_str()).cmp(&(&right.path, right.ecosystem.as_str()))
    });
    limitations.sort();
    limitations.dedup();
    ManifestDiscovery {
        observations,
        limitations,
    }
}

/// Liga relações `workspace:` aos pacotes locais de qualquer manifesto observado.
fn resolve_workspace_references(observations: &mut [ManifestObservation]) {
    let local_packages: Vec<LocalPackage> = observations
        .iter()
        .flat_map(|observation| observation.local_packages.iter().cloned())
        .collect();
    for observation in observations.iter_mut() {
        for dependency in observation
            .dependencies
            .iter_mut()
            .filter(|dependency| dependency.destination == DependencyDestination::LocalWorkspace)
        {
            let specifier = dependency
                .version
                .strip_prefix("workspace:")
                .unwrap_or_default();
            if let Some(target) =
                workspace_dependency_target(&dependency.name, specifier, &local_packages)
            {
                dependency.name = target;
            } else {
                observation.limitations.push(format!(
                    "manifesto {}: relação workspace:{specifier} sem pacote local correspondente",
                    dependency.source_manifest
                ));
            }
        }
        observation.limitations.sort();
        observation.limitations.dedup();
    }
}

/// Caminhos de manifests reconhecidos, ignorando diretórios padrão e links.
fn collect_manifest_paths(
    root: &Path,
    limitations: &mut Vec<String>,
) -> Vec<(String, PathBuf, ManifestEcosystem)> {
    fn visit(
        base: &Path,
        directory: &Path,
        found: &mut Vec<(String, PathBuf, ManifestEcosystem)>,
        limitations: &mut Vec<String>,
    ) {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) => {
                limitations.push(format!(
                    "diretório {} inacessível na busca de manifests: {}",
                    relative_path(base, directory),
                    brief_error(&error)
                ));
                return;
            }
        };
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let relative = relative_path(base, &path);
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    limitations.push(format!(
                        "entrada {relative} não pôde ser classificada: {}",
                        brief_error(&error)
                    ));
                    continue;
                }
            };
            if file_type.is_symlink() {
                limitations.push(format!(
                    "link simbólico ignorado na busca de manifests: {relative}"
                ));
                continue;
            }
            if file_type.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if DEFAULT_IGNORED_DIRS.contains(&name.as_str()) {
                    continue;
                }
                visit(base, &path, found, limitations);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let ecosystem = match entry.file_name().to_string_lossy().as_ref() {
                "package.json" => Some(ManifestEcosystem::Npm),
                "pyproject.toml" => Some(ManifestEcosystem::Python),
                "go.mod" => Some(ManifestEcosystem::Go),
                _ => None,
            };
            if let Some(ecosystem) = ecosystem {
                found.push((relative, path, ecosystem));
            }
        }
    }

    let mut found = Vec::new();
    visit(root, root, &mut found, limitations);
    found.sort_by(|left, right| left.0.cmp(&right.0));
    found
}

/// Caminho relativo estável, usando `/` em qualquer sistema operacional.
fn relative_path(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Primeira linha de um erro, sem trechos de conteúdo do manifesto.
fn brief_error(error: &impl Display) -> String {
    let message = error.to_string();
    message
        .lines()
        .next()
        .unwrap_or("erro desconhecido")
        .trim()
        .chars()
        .take(160)
        .collect()
}

/// Campo de texto de JSON, com limitação localizada quando o tipo é diferente.
fn json_string(
    value: &JsonValue,
    field: &str,
    manifest_path: &str,
    limitations: &mut Vec<String>,
) -> Option<String> {
    match value.get(field) {
        Some(JsonValue::String(text)) => Some(text.clone()),
        Some(_) => {
            limitations.push(format!(
                "manifesto {manifest_path}: campo {field} não é texto"
            ));
            None
        }
        None => None,
    }
}

/// Campo de texto de TOML, com limitação localizada quando o tipo é diferente.
fn toml_string(
    value: &toml::Value,
    field: &str,
    manifest_path: &str,
    limitations: &mut Vec<String>,
) -> Option<String> {
    match value.get(field) {
        Some(toml::Value::String(text)) => Some(text.clone()),
        Some(_) => {
            limitations.push(format!(
                "manifesto {manifest_path}: campo {field} não é texto"
            ));
            None
        }
        None => None,
    }
}

/// Registra um pacote local com caminho de código derivado do inventário.
fn push_local_package(
    observation: &mut ManifestObservation,
    files: &BTreeSet<&str>,
    manifest_path: &str,
    name: String,
    version: Option<String>,
    evidence: Vec<String>,
) {
    if name.trim().is_empty() {
        return;
    }
    let directory = manifest_directory(manifest_path);
    observation.local_packages.push(LocalPackage {
        name,
        version: version.filter(|value| !value.trim().is_empty()),
        manifest_path: manifest_path.to_owned(),
        source_path: package_path(&directory, files),
        evidence,
    });
}

/// Analisa `package.json` sem executar nenhum script declarado.
fn parse_npm(
    root: &Path,
    manifest_path: &str,
    text: &str,
    files: &BTreeSet<&str>,
    observation: &mut ManifestObservation,
    limitations: &mut Vec<String>,
) {
    let value: JsonValue = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            observation.status = ManifestStatus::Malformed;
            limitations.push(format!(
                "manifesto {manifest_path} inválido: {}",
                brief_error(&error)
            ));
            return;
        }
    };
    let Some(table) = value.as_object() else {
        observation.status = ManifestStatus::Malformed;
        limitations.push(format!(
            "manifesto {manifest_path}: a raiz não é um objeto JSON"
        ));
        return;
    };

    let name = json_string(&value, "name", manifest_path, limitations);
    let version = json_string(&value, "version", manifest_path, limitations);
    if let Some(name) = name {
        push_local_package(
            observation,
            files,
            manifest_path,
            name,
            version,
            vec![format!("manifesto: {manifest_path}")],
        );
    }

    let mut workspace_packages = Vec::new();
    for pattern in npm_workspace_patterns(&value, manifest_path, limitations) {
        for directory in workspace_directories(root, manifest_path, &pattern, limitations) {
            let member_manifest = join_relative(&directory, "package.json");
            let Some(package) =
                read_npm_workspace_package(root, &member_manifest, &pattern, files, limitations)
            else {
                continue;
            };
            workspace_packages.push(package.clone());
            observation.local_packages.push(package);
        }
    }

    read_npm_dependencies(
        manifest_path,
        &value,
        "dependencies",
        DependencyScope::Runtime,
        &workspace_packages,
        observation,
        limitations,
    );
    read_npm_dependencies(
        manifest_path,
        &value,
        "devDependencies",
        DependencyScope::Development,
        &workspace_packages,
        observation,
        limitations,
    );
    read_npm_dependencies(
        manifest_path,
        &value,
        "peerDependencies",
        DependencyScope::Peer,
        &workspace_packages,
        observation,
        limitations,
    );
    read_npm_dependencies(
        manifest_path,
        &value,
        "optionalDependencies",
        DependencyScope::Optional,
        &workspace_packages,
        observation,
        limitations,
    );

    if table.contains_key("scripts") {
        limitations.push(format!(
            "manifesto {manifest_path}: scripts declarados não foram executados"
        ));
    }
}

/// Padrões de workspace em array ou em `packages` de objeto.
fn npm_workspace_patterns(
    value: &JsonValue,
    manifest_path: &str,
    limitations: &mut Vec<String>,
) -> Vec<String> {
    let Some(workspaces) = value.get("workspaces") else {
        return Vec::new();
    };
    let mut patterns = Vec::new();
    match workspaces {
        JsonValue::Array(values) => {
            for item in values {
                if let Some(pattern) = item.as_str() {
                    patterns.push(pattern.to_owned());
                } else {
                    limitations.push(format!(
                        "manifesto {manifest_path}: entrada de workspaces não é texto"
                    ));
                }
            }
        }
        JsonValue::Object(table) => {
            match table.get("packages") {
                Some(JsonValue::Array(values)) => {
                    for item in values {
                        if let Some(pattern) = item.as_str() {
                            patterns.push(pattern.to_owned());
                        } else {
                            limitations.push(format!(
                                "manifesto {manifest_path}: entrada de workspaces.packages não é texto"
                            ));
                        }
                    }
                }
                Some(_) => limitations.push(format!(
                    "manifesto {manifest_path}: workspaces.packages não é uma lista"
                )),
                None => limitations.push(format!(
                    "manifesto {manifest_path}: objeto workspaces sem packages"
                )),
            }
            for key in table.keys().filter(|key| key.as_str() != "packages") {
                limitations.push(format!(
                    "manifesto {manifest_path}: chave de workspaces não suportada: {key}"
                ));
            }
        }
        _ => limitations.push(format!(
            "manifesto {manifest_path}: campo workspaces não é lista nem objeto"
        )),
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

/// Diretórios sustentados por um padrão de workspace.
fn workspace_directories(
    root: &Path,
    manifest_path: &str,
    pattern: &str,
    limitations: &mut Vec<String>,
) -> Vec<String> {
    let normalized = normalize_relative_pattern(pattern);
    let stars = normalized.matches('*').count();
    let supported = !normalized.contains("..")
        && !normalized.contains("**")
        && !normalized.contains('?')
        && (stars == 0 || (stars == 1 && normalized.ends_with("/*")));
    if !supported {
        limitations.push(format!(
            "manifesto {manifest_path}: padrão de workspace não suportado: {pattern}"
        ));
        return Vec::new();
    }

    let base = manifest_directory(manifest_path);
    if let Some(prefix) = normalized.strip_suffix("/*") {
        let absolute = root.join(join_relative(&base, prefix));
        let entries = match fs::read_dir(&absolute) {
            Ok(entries) => entries,
            Err(error) => {
                limitations.push(format!(
                    "manifesto {manifest_path}: diretório de workspace {} inacessível: {}",
                    join_relative(&base, prefix),
                    brief_error(&error)
                ));
                return Vec::new();
            }
        };
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.file_name());
        let mut directories = Vec::new();
        for entry in entries {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            directories.push(join_relative(&join_relative(&base, prefix), &name));
        }
        directories
    } else {
        vec![join_relative(&base, &normalized)]
    }
}

/// Normaliza barras e remove `./` sem transformar o padrão em outro glob.
fn normalize_relative_pattern(pattern: &str) -> String {
    pattern
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_owned()
}

/// Junta dois caminhos relativos já normalizados.
fn join_relative(base: &str, child: &str) -> String {
    let base = base.trim_end_matches('/');
    let child = child.trim_start_matches("./");
    if base.is_empty() {
        child.to_owned()
    } else {
        format!("{base}/{child}")
    }
}

/// Lê nome e versão de um pacote local de workspace.
fn read_npm_workspace_package(
    root: &Path,
    manifest_path: &str,
    pattern: &str,
    files: &BTreeSet<&str>,
    limitations: &mut Vec<String>,
) -> Option<LocalPackage> {
    if !is_regular_file(&root.join(manifest_path)) {
        limitations.push(format!(
            "manifesto {manifest_path}: workspace declarado sem package.json"
        ));
        return None;
    }
    let text = match fs::read_to_string(root.join(manifest_path)) {
        Ok(text) => text,
        Err(error) => {
            limitations.push(format!(
                "manifesto {manifest_path} não pôde ser lido: {}",
                brief_error(&error)
            ));
            return None;
        }
    };
    let value: JsonValue = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            limitations.push(format!(
                "manifesto {manifest_path} inválido: {}",
                brief_error(&error)
            ));
            return None;
        }
    };
    let name = json_string(&value, "name", manifest_path, limitations)?;
    let version = json_string(&value, "version", manifest_path, limitations);
    let directory = manifest_directory(manifest_path);
    Some(LocalPackage {
        name,
        version: version.filter(|value| !value.trim().is_empty()),
        manifest_path: manifest_path.to_owned(),
        source_path: package_path(&directory, files),
        evidence: vec![
            format!("manifesto: {manifest_path}"),
            format!("workspace: {pattern}"),
        ],
    })
}

/// Arquivo comum, sem seguir links simbólicos.
fn is_regular_file(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

/// Lê um grupo de dependências npm preservando seu papel.
fn read_npm_dependencies(
    manifest_path: &str,
    value: &JsonValue,
    field: &str,
    scope: DependencyScope,
    workspace_packages: &[LocalPackage],
    observation: &mut ManifestObservation,
    limitations: &mut Vec<String>,
) {
    let Some(field_value) = value.get(field) else {
        return;
    };
    let Some(table) = field_value.as_object() else {
        limitations.push(format!(
            "manifesto {manifest_path}: campo {field} não é um objeto"
        ));
        return;
    };
    let entries: BTreeMap<&String, &JsonValue> = table.iter().collect();
    for (name, spec) in entries {
        let Some(raw) = spec.as_str() else {
            limitations.push(format!(
                "manifesto {manifest_path}: {field}.{name} não é texto"
            ));
            continue;
        };
        let (target, destination) = match raw.strip_prefix("workspace:") {
            Some(specifier) => (
                workspace_dependency_target(name, specifier, workspace_packages)
                    .unwrap_or_else(|| name.clone()),
                DependencyDestination::LocalWorkspace,
            ),
            None => (name.clone(), DependencyDestination::External),
        };
        observation.dependencies.push(ManifestDependency {
            source_manifest: manifest_path.to_owned(),
            name: target,
            version: raw.to_owned(),
            scope,
            destination,
            group: None,
            line: None,
            evidence: format!("{manifest_path}:{field}.{name} = {raw}"),
        });
    }
}

/// Associa uma especificação `workspace:` a um pacote local conhecido.
fn workspace_dependency_target(
    dependency_name: &str,
    specifier: &str,
    workspace_packages: &[LocalPackage],
) -> Option<String> {
    let trimmed = specifier.trim();
    if trimmed.is_empty() || matches!(trimmed, "*" | "^" | "~") {
        return workspace_packages
            .iter()
            .find(|package| package.name == dependency_name)
            .map(|package| package.name.clone());
    }
    let normalized = normalize_relative_pattern(trimmed);
    workspace_packages
        .iter()
        .find(|package| {
            package.manifest_path.strip_suffix("/package.json") == Some(normalized.as_str())
                || package
                    .source_path
                    .trim_end_matches("/**")
                    .starts_with(normalized.as_str())
        })
        .map(|package| package.name.clone())
}

/// Analisa somente `[project]` no padrão PEP 621.
fn parse_python(
    manifest_path: &str,
    text: &str,
    files: &BTreeSet<&str>,
    observation: &mut ManifestObservation,
    limitations: &mut Vec<String>,
) {
    let value: toml::Value = match toml::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            observation.status = ManifestStatus::Malformed;
            limitations.push(format!(
                "manifesto {manifest_path} inválido: {}",
                brief_error(&error)
            ));
            return;
        }
    };
    let manager = value
        .get("tool")
        .and_then(|tool| tool.get("poetry").or_else(|| tool.get("uv")));
    let Some(project) = value.get("project") else {
        observation.status = ManifestStatus::Unsupported;
        if manager.is_some() {
            limitations.push(format!(
                "manifesto {manifest_path}: formato de gerenciador não suportado; apenas PEP 621 [project] é extraído"
            ));
        } else {
            limitations.push(format!(
                "manifesto {manifest_path}: ausente [project] no padrão PEP 621"
            ));
        }
        return;
    };
    let Some(table) = project.as_table() else {
        observation.status = ManifestStatus::Malformed;
        limitations.push(format!(
            "manifesto {manifest_path}: [project] não é uma tabela"
        ));
        return;
    };

    let name = toml_string(project, "name", manifest_path, limitations);
    let version = toml_string(project, "version", manifest_path, limitations);
    if let Some(name) = name {
        push_local_package(
            observation,
            files,
            manifest_path,
            name,
            version.clone(),
            vec![format!("manifesto: {manifest_path}")],
        );
    }
    if version.is_none() && table.contains_key("dynamic") {
        limitations.push(format!(
            "manifesto {manifest_path}: versão dinâmica não resolvida"
        ));
    }
    if let Some(dependencies) = project.get("dependencies") {
        read_python_requirements(
            manifest_path,
            dependencies,
            DependencyScope::Runtime,
            None,
            "dependencies",
            observation,
            limitations,
        );
    }
    if let Some(optional) = project.get("optional-dependencies") {
        let Some(groups) = optional.as_table() else {
            limitations.push(format!(
                "manifesto {manifest_path}: optional-dependencies não é uma tabela"
            ));
            return;
        };
        for (group, requirements) in groups {
            read_python_requirements(
                manifest_path,
                requirements,
                DependencyScope::OptionalGroup,
                Some(group.clone()),
                &format!("optional-dependencies.{group}"),
                observation,
                limitations,
            );
        }
    }
    if manager.is_some() {
        limitations.push(format!(
            "manifesto {manifest_path}: seções de gerenciador além de [project] não extraídas"
        ));
    }
}

/// Lê uma lista de requisitos PEP 508 sem resolver ou instalar nada.
fn read_python_requirements(
    manifest_path: &str,
    value: &toml::Value,
    scope: DependencyScope,
    group: Option<String>,
    field: &str,
    observation: &mut ManifestObservation,
    limitations: &mut Vec<String>,
) {
    let Some(requirements) = value.as_array() else {
        limitations.push(format!(
            "manifesto {manifest_path}: campo {field} não é uma lista"
        ));
        return;
    };
    for (index, requirement) in requirements.iter().enumerate() {
        let Some(raw) = requirement.as_str() else {
            limitations.push(format!(
                "manifesto {manifest_path}: {field}[{index}] não é texto"
            ));
            continue;
        };
        let name = python_requirement_name(raw);
        if name == "desconhecido" {
            limitations.push(format!(
                "manifesto {manifest_path}: requisito {field}[{index}] sem nome reconhecível"
            ));
        }
        observation.dependencies.push(ManifestDependency {
            source_manifest: manifest_path.to_owned(),
            name,
            version: raw.to_owned(),
            scope,
            destination: DependencyDestination::External,
            group: group.clone(),
            line: None,
            evidence: format!("{manifest_path}:{field}[{index}] = {raw}"),
        });
    }
}

/// Primeiro identificador de um requisito PEP 508, normalizado para comparação.
fn python_requirement_name(requirement: &str) -> String {
    let trimmed = requirement.trim();
    let end = trimmed
        .find(|character: char| {
            !(character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                || character == '.')
        })
        .unwrap_or(trimmed.len());
    let name = &trimmed[..end];
    if name.is_empty() {
        "desconhecido".to_owned()
    } else {
        name.to_ascii_lowercase()
    }
}

/// Diretivas `go.mod` com extração implementada.
#[derive(Debug, Clone, Copy)]
enum GoDirective {
    Require,
    Replace,
    Exclude,
}

impl GoDirective {
    /// Palavra-chave que inicia a diretiva.
    fn keyword(self) -> &'static str {
        match self {
            Self::Require => "require",
            Self::Replace => "replace",
            Self::Exclude => "exclude",
        }
    }

    /// Papel atribuído às dependências da diretiva.
    fn scope(self) -> DependencyScope {
        match self {
            Self::Require => DependencyScope::Require,
            Self::Replace => DependencyScope::Replace,
            Self::Exclude => DependencyScope::Exclude,
        }
    }
}

/// Lê o subconjunto `module`/`require`/`replace`/`exclude` de `go.mod`.
fn parse_go(
    manifest_path: &str,
    text: &str,
    files: &BTreeSet<&str>,
    observation: &mut ManifestObservation,
) {
    let mut module_path: Option<String> = None;
    let mut block: Option<GoDirective> = None;
    let mut limitations = Vec::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = strip_go_comment(raw_line);
        if line.is_empty() {
            continue;
        }
        if let Some(directive) = block {
            if line == ")" {
                block = None;
            } else {
                parse_go_entry(
                    manifest_path,
                    directive,
                    line,
                    line_number,
                    observation,
                    &mut limitations,
                );
            }
            continue;
        }

        let (keyword, rest) = match line.split_once(char::is_whitespace) {
            Some((keyword, rest)) => (keyword, rest.trim()),
            None => (line, ""),
        };
        match keyword {
            "module" => {
                if rest.is_empty() || rest.contains(char::is_whitespace) {
                    limitations.push(format!(
                        "manifesto {manifest_path}: diretiva module inválida na linha {line_number}"
                    ));
                } else if module_path.is_some() {
                    limitations.push(format!(
                        "manifesto {manifest_path}: diretiva module repetida na linha {line_number}"
                    ));
                } else {
                    module_path = Some(rest.to_owned());
                }
            }
            "require" | "replace" | "exclude" => {
                let directive = match keyword {
                    "require" => GoDirective::Require,
                    "replace" => GoDirective::Replace,
                    _ => GoDirective::Exclude,
                };
                if rest.starts_with('(') {
                    if rest != "(" {
                        limitations.push(format!(
                            "manifesto {manifest_path}: bloco {keyword} com conteúdo inesperado na linha {line_number}"
                        ));
                    }
                    block = Some(directive);
                } else {
                    parse_go_entry(
                        manifest_path,
                        directive,
                        rest,
                        line_number,
                        observation,
                        &mut limitations,
                    );
                }
            }
            "go" => limitations.push(format!(
                "manifesto {manifest_path}: diretiva go informativa não extraída na linha {line_number}"
            )),
            _ => limitations.push(format!(
                "manifesto {manifest_path}: diretiva não suportada `{keyword}` na linha {line_number}"
            )),
        }
    }
    if block.is_some() {
        limitations.push(format!(
            "manifesto {manifest_path}: bloco de dependências não fechado"
        ));
    }
    if let Some(module_path) = module_path {
        push_local_package(
            observation,
            files,
            manifest_path,
            module_path.clone(),
            None,
            vec![
                format!("manifesto: {manifest_path}"),
                format!("module: {module_path}"),
            ],
        );
    } else {
        limitations.push(format!(
            "manifesto {manifest_path}: ausente diretiva module"
        ));
    }
    observation.limitations.extend(limitations);
}

/// Remove comentários e espaços de uma linha de `go.mod`.
fn strip_go_comment(line: &str) -> &str {
    line.split_once("//")
        .map(|(before, _)| before)
        .unwrap_or(line)
        .trim()
}

/// Interpreta uma entrada simples de `require`, `replace` ou `exclude`.
fn parse_go_entry(
    manifest_path: &str,
    directive: GoDirective,
    line: &str,
    line_number: usize,
    observation: &mut ManifestObservation,
    limitations: &mut Vec<String>,
) {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    match directive {
        GoDirective::Require | GoDirective::Exclude => {
            if tokens.len() < 2 {
                limitations.push(format!(
                    "manifesto {manifest_path}: entrada {} inválida na linha {line_number}",
                    directive.keyword()
                ));
                return;
            }
            observation.dependencies.push(ManifestDependency {
                source_manifest: manifest_path.to_owned(),
                name: tokens[0].to_owned(),
                version: tokens[1..].join(" "),
                scope: directive.scope(),
                destination: DependencyDestination::External,
                group: None,
                line: Some(line_number),
                evidence: format!(
                    "{manifest_path}:{line_number} = {} {line}",
                    directive.keyword()
                ),
            });
        }
        GoDirective::Replace => {
            let Some(arrow) = tokens.iter().position(|token| *token == "=>") else {
                limitations.push(format!(
                    "manifesto {manifest_path}: replace sem `=>` na linha {line_number}"
                ));
                return;
            };
            if arrow == 0 || arrow + 1 >= tokens.len() {
                limitations.push(format!(
                    "manifesto {manifest_path}: replace incompleto na linha {line_number}"
                ));
                return;
            }
            let replacement = tokens[arrow + 1];
            let destination = if replacement.starts_with('.') || replacement.starts_with('/') {
                DependencyDestination::LocalWorkspace
            } else {
                DependencyDestination::External
            };
            observation.dependencies.push(ManifestDependency {
                source_manifest: manifest_path.to_owned(),
                name: tokens[0].to_owned(),
                version: tokens[arrow + 1..].join(" "),
                scope: DependencyScope::Replace,
                destination,
                group: None,
                line: Some(line_number),
                evidence: format!(
                    "{manifest_path}:{line_number} = {} {line}",
                    directive.keyword()
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::*;
    use crate::discovery::ScanOptions;
    use crate::discovery::discover_with_options;
    use crate::discovery::discover_without_contract;

    /// Lê manifests de uma fixture e devolve o resultado agregado.
    fn manifests_for(name: &str) -> ManifestDiscovery {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name);
        let inventory = discover_without_contract(&root).expect("discovery must succeed");
        let files: BTreeSet<&str> = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        read_manifests(&inventory.root, &files)
    }

    /// Descoberta com um include explícito, para testar linguagens sem parser.
    fn inventory_with_go(name: &str) -> crate::discovery::Inventory {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name);
        discover_with_options(
            &root,
            &ScanOptions {
                include: vec!["**/*.go".to_owned(), "**/*.py".to_owned()],
                exclude: Vec::new(),
            },
        )
        .expect("discovery must succeed")
    }

    #[test]
    fn npm_manifest_extracts_scopes_without_mixing_them() {
        let result = manifests_for("discovery_manifest_npm");
        let observation = result
            .observations
            .iter()
            .find(|item| item.path == "package.json")
            .expect("npm manifest must be observed");

        assert_eq!(observation.status, ManifestStatus::Parsed);
        assert_eq!(observation.ecosystem, ManifestEcosystem::Npm);
        assert_eq!(observation.local_packages[0].name, "loja-web");
        assert_eq!(
            observation.local_packages[0].version.as_deref(),
            Some("1.4.0")
        );
        let scopes: Vec<(&str, &str)> = observation
            .dependencies
            .iter()
            .map(|dependency| (dependency.name.as_str(), dependency.scope.as_str()))
            .collect();
        assert_eq!(
            scopes,
            vec![
                ("@types/node", "development"),
                ("fsevents", "optional"),
                ("react", "runtime"),
                ("react-dom", "peer"),
            ]
        );
        assert!(
            observation
                .dependencies
                .iter()
                .all(|dependency| dependency.destination == DependencyDestination::External)
        );
    }

    #[test]
    fn npm_workspaces_separate_local_packages_and_workspace_relations() {
        let result = manifests_for("discovery_manifest_npm_workspace");
        let root = result
            .observations
            .iter()
            .find(|item| item.path == "package.json")
            .expect("root manifest must be observed");
        let names: Vec<&str> = root
            .local_packages
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        assert_eq!(names, vec!["@demo/api", "@demo/web", "demo-root"]);
        let web = result
            .observations
            .iter()
            .find(|item| item.path == "packages/web/package.json")
            .expect("nested manifest must be observed");
        let local = web
            .dependencies
            .iter()
            .find(|dependency| dependency.name == "@demo/api")
            .expect("workspace relation must be recorded");
        assert_eq!(local.scope, DependencyScope::Runtime);
        assert_eq!(local.destination, DependencyDestination::LocalWorkspace);
        assert!(local.version.starts_with("workspace:"));
        let external = web
            .dependencies
            .iter()
            .find(|dependency| dependency.name == "zod")
            .expect("external dependency must be recorded");
        assert_eq!(external.destination, DependencyDestination::External);
        assert_eq!(external.scope, DependencyScope::Development);
        assert!(
            web.limitations
                .iter()
                .all(|limitation| !limitation.contains("sem pacote local correspondente")),
            "workspace declarado no pacote raiz deve ser resolvido"
        );
        assert!(root.limitations.iter().any(|limitation| {
            limitation.contains("padrão de workspace não suportado")
                && limitation.contains("packages/**/src/*")
        }));
    }

    #[test]
    fn malformed_npm_manifest_is_a_localized_limitation() {
        let result = manifests_for("discovery_manifest_npm_invalid");
        let observation = &result.observations[0];
        assert_eq!(observation.status, ManifestStatus::Malformed);
        assert!(observation.local_packages.is_empty());
        assert!(observation.dependencies.is_empty());
        assert!(observation.limitations.iter().any(
            |limitation| limitation.contains("package.json") && limitation.contains("inválido")
        ));
    }

    #[test]
    fn python_pep621_extracts_runtime_and_optional_dependencies() {
        let result = manifests_for("discovery_manifest_python");
        let observation = &result.observations[0];
        assert_eq!(observation.ecosystem, ManifestEcosystem::Python);
        assert_eq!(observation.local_packages[0].name, "pagamentos");
        assert_eq!(
            observation.local_packages[0].version.as_deref(),
            Some("0.3.1")
        );
        let dependencies: Vec<(&str, &str, Option<&str>)> = observation
            .dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.name.as_str(),
                    dependency.scope.as_str(),
                    dependency.group.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            dependencies,
            vec![
                ("httpx", "runtime", None),
                ("mypy", "optional_group", Some("dev")),
                ("pydantic", "runtime", None),
                ("pytest", "optional_group", Some("test")),
            ]
        );
    }

    #[test]
    fn unsupported_python_manager_is_not_treated_as_pep621() {
        let result = manifests_for("discovery_manifest_python_unsupported");
        let observation = &result.observations[0];
        assert_eq!(observation.status, ManifestStatus::Unsupported);
        assert!(observation.local_packages.is_empty());
        assert!(observation.dependencies.is_empty());
        assert!(
            observation
                .limitations
                .iter()
                .any(|limitation| limitation.contains("gerenciador não suportado"))
        );
    }

    #[test]
    fn go_mod_extracts_supported_directives_and_reports_unknown_ones() {
        let result = manifests_for("discovery_manifest_go");
        let observation = &result.observations[0];
        assert_eq!(observation.ecosystem, ManifestEcosystem::Go);
        assert_eq!(observation.local_packages[0].name, "example.com/servico");
        let kinds: Vec<&str> = observation
            .dependencies
            .iter()
            .map(|dependency| dependency.scope.as_str())
            .collect();
        assert_eq!(
            kinds,
            vec!["replace", "exclude", "require", "require", "require"]
        );
        let local_replace = observation
            .dependencies
            .iter()
            .find(|dependency| dependency.scope == DependencyScope::Replace)
            .expect("replace must be recorded");
        assert_eq!(
            local_replace.destination,
            DependencyDestination::LocalWorkspace
        );
        assert!(
            observation
                .limitations
                .iter()
                .any(|limitation| limitation.contains("diretiva não suportada `toolchain`"))
        );
        assert!(result.limitations.iter().any(|limitation| {
            limitation.contains("go.mod") && limitation.contains("sem analisador")
        }));
    }

    #[test]
    fn mixed_manifests_are_discovered_in_subprojects() {
        let result = manifests_for("discovery_manifest_mixed");
        let paths: Vec<&str> = result
            .observations
            .iter()
            .map(|observation| observation.path.as_str())
            .collect();
        assert_eq!(
            paths,
            vec![
                "go.mod",
                "package.json",
                "services/api/pyproject.toml",
                "services/worker/go.mod",
            ]
        );
    }

    #[test]
    fn absent_manifests_produce_no_observations() {
        let result = manifests_for("discovery_no_contract");
        assert!(result.observations.is_empty());
        assert!(result.limitations.is_empty());
    }

    #[test]
    fn manifest_reading_is_deterministic() {
        assert_eq!(
            manifests_for("discovery_manifest_mixed"),
            manifests_for("discovery_manifest_mixed")
        );
    }

    #[test]
    fn files_without_analyzer_are_still_counted_without_a_verdict() {
        let inventory = inventory_with_go("discovery_manifest_go");
        let extensions: Vec<(&str, Option<&str>)> = inventory
            .files
            .iter()
            .map(|file| {
                (
                    file.extension.as_deref().unwrap_or("unknown"),
                    super::super::candidates::analyzer_for(
                        file.extension.as_deref().unwrap_or("unknown"),
                    ),
                )
            })
            .collect();
        assert_eq!(extensions, vec![("go", None)]);
    }
}
