//! Resolução determinística de especificadores, sem rede e sem dependências instaladas.
//!
//! Este módulo não toma decisão arquitetural: ele apenas reúne evidência de
//! `tsconfig.json` local e dos manifests lidos na Etapa 6.2 para dizer se um
//! especificador é relativo, alias local, pacote de workspace, embutido,
//! externo declarado ou externo não confirmado. Nada aqui executa comandos,
//! consulta registries ou lê `node_modules`/site-packages.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::discovery::manifests::{
    DependencyDestination, ManifestEcosystem, ManifestObservation, ManifestStatus,
};
use crate::discovery::{DEFAULT_IGNORED_DIRS, Inventory};

/// Sufixos provados contra o inventário, na ordem em que são testados.
const RESOLUTION_SUFFIXES: &[&str] = &[
    "",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".d.ts",
    "/index.ts",
    "/index.tsx",
    "/index.js",
    "/index.jsx",
    "/index.d.ts",
];

/// Sufixos de arquivo de um módulo Python, na ordem em que são testados.
const PYTHON_MODULE_SUFFIXES: &[&str] = &[".py", "/__init__.py"];

/// Bases Node consideradas embutidas, sem consultar a instalação do Node.
const NODE_BUILTINS: &[&str] = &[
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "sqlite",
    "stream",
    "string_decoder",
    "sys",
    "test",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// Raízes da biblioteca padrão Python usadas apenas para não rotular built-in como externo.
const PYTHON_BUILTINS: &[&str] = &[
    "abc",
    "argparse",
    "array",
    "asyncio",
    "base64",
    "bisect",
    "builtins",
    "calendar",
    "collections",
    "concurrent",
    "configparser",
    "contextlib",
    "copy",
    "csv",
    "ctypes",
    "dataclasses",
    "datetime",
    "decimal",
    "difflib",
    "enum",
    "errno",
    "functools",
    "getpass",
    "glob",
    "gzip",
    "hashlib",
    "heapq",
    "hmac",
    "html",
    "http",
    "importlib",
    "inspect",
    "io",
    "itertools",
    "json",
    "logging",
    "math",
    "multiprocessing",
    "os",
    "pathlib",
    "pickle",
    "platform",
    "pprint",
    "random",
    "re",
    "secrets",
    "shutil",
    "signal",
    "socket",
    "sqlite3",
    "stat",
    "statistics",
    "string",
    "subprocess",
    "sys",
    "tempfile",
    "textwrap",
    "threading",
    "time",
    "timeit",
    "types",
    "typing",
    "unittest",
    "urllib",
    "uuid",
    "warnings",
    "weakref",
    "xml",
    "zipfile",
];

/// Estado de uma relação com pacote, sem implicar regra de conformidade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalPackageState {
    /// Built-in conhecido, sem manifesto e sem aresta arquitetural.
    Builtin,
    /// Pacote declarado em manifesto lido da raiz ou de subprojeto.
    Declared,
    /// Pacote sem declaração verificável no projeto auditado.
    Unconfirmed,
    /// Pacote local reconhecido em workspace, não externo.
    WorkspaceLocal,
}

impl ExternalPackageState {
    /// Rótulo estável usado em texto, Markdown e JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Builtin => "embutida",
            Self::Declared => "externa declarada",
            Self::Unconfirmed => "externa não confirmada",
            Self::WorkspaceLocal => "local de workspace",
        }
    }
}

/// Relação com pacote, com evidência e origem explícitas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalPackageRelation {
    /// Especificador exato encontrado no código.
    pub specifier: String,
    /// Nome-base do pacote, sem subpath.
    pub base_name: String,
    /// Estado determinístico da relação.
    pub state: ExternalPackageState,
    /// Origem da declaração ou do reconhecimento do built-in.
    pub origin: String,
    /// Evidência curta da classificação.
    pub evidence: String,
    /// Arquivo de origem, relativo à raiz.
    pub path: String,
    /// Linha da declaração, quando localizada.
    pub line: Option<usize>,
}

/// Resultado de um alias de `tsconfig.json` contra o inventário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasResolution {
    /// Exatamente um arquivo do inventário corresponde ao alias.
    Resolved(String),
    /// Mais de um arquivo corresponde; a escolha não é silenciosa.
    Ambiguous(Vec<String>),
    /// O alias correspondeu, mas nenhum arquivo do inventário foi encontrado.
    Unresolved(String),
    /// Nenhum padrão de alias corresponde ao especificador.
    NoMatch,
}

/// Regra `paths` de um `tsconfig.json`, já convertida para caminhos relativos à raiz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasRule {
    /// Padrão original, com zero ou um `*`.
    pub pattern: String,
    /// Destinos normalizados, relativos à raiz.
    pub targets: Vec<String>,
    /// Configuração que declarou a regra.
    pub config_path: String,
}

/// Pacote local declarado em manifesto, usado para não chamar workspace de externo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalPackageRef {
    /// Nome declarado no manifesto.
    pub name: String,
    /// Manifesto de origem, relativo à raiz.
    pub manifest_path: String,
    /// Prefixo de código sugerido pelo manifesto.
    pub source_path: String,
}

/// Dependência externa declarada, com papel e origem preservados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredExternalRef {
    /// Nome-base declarado.
    pub name: String,
    /// Ecossistema que declarou a dependência.
    pub ecosystem: &'static str,
    /// Papel da dependência no manifesto.
    pub scope: &'static str,
    /// Manifesto de origem, relativo à raiz.
    pub origin: String,
}

/// Contexto determinístico usado por todas as linguagens com analisador.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolutionContext {
    /// Regras de alias ordenadas da mais específica para a mais genérica.
    pub aliases: Vec<AliasRule>,
    /// Pacotes locais reconhecidos em manifests, indexados por nome.
    pub local_packages: BTreeMap<String, LocalPackageRef>,
    /// Dependências externas declaradas, indexadas pelo nome-base.
    pub declared_external: BTreeMap<String, DeclaredExternalRef>,
    /// Nomes importáveis com correspondência determinística à distribuição Python.
    pub python_equivalent: BTreeMap<String, String>,
    /// Configurações ausentes, parciais ou fora do subconjunto suportado.
    pub limitations: Vec<String>,
}

/// Encontra arquivos do inventário que correspondem a um módulo Python absoluto.
pub fn resolve_python_module(specifier: &str, files: &BTreeSet<&str>) -> Vec<String> {
    let stem = specifier.replace('.', "/");
    let mut found = Vec::new();
    for suffix in PYTHON_MODULE_SUFFIXES {
        let candidate = format!("{stem}{suffix}");
        for file in files {
            if *file == candidate || file.ends_with(&format!("/{candidate}")) {
                found.push((*file).to_owned());
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

impl ResolutionContext {
    /// Monta o contexto a partir do inventário, sem sair da raiz.
    pub fn for_inventory(inventory: &Inventory) -> Self {
        let files: BTreeSet<&str> = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        Self::from_project(&inventory.root, &files)
    }

    /// Monta o contexto lendo manifests e `tsconfig.json` dentro da raiz.
    pub fn from_project(root: &Path, files: &BTreeSet<&str>) -> Self {
        let mut context = Self::default();
        let manifests = crate::discovery::manifests::read_manifests(root, files);
        for observation in &manifests.observations {
            add_manifest(root, observation, &mut context);
        }
        context
            .limitations
            .extend(manifests.limitations.iter().cloned());
        for observation in &manifests.observations {
            context
                .limitations
                .extend(observation.limitations.iter().cloned());
        }

        let configs = read_tsconfigs(root, &mut context.limitations);
        context.aliases = build_aliases(&configs, &mut context.limitations);
        context.limitations.sort();
        context.limitations.dedup();
        context
    }

    /// Tenta resolver um especificador por alias local, em ordem determinística.
    pub fn resolve_alias(&self, specifier: &str, files: &BTreeSet<&str>) -> AliasResolution {
        for rule in &self.aliases {
            let Some(captured) = alias_capture(&rule.pattern, specifier) else {
                continue;
            };
            let mut found = BTreeSet::new();
            for target in &rule.targets {
                let candidate = target.replace('*', captured);
                for suffix in RESOLUTION_SUFFIXES {
                    let path = format!("{candidate}{suffix}");
                    if files.contains(path.as_str()) {
                        found.insert(path);
                    }
                }
            }
            return match found.len() {
                0 => AliasResolution::Unresolved(format!(
                    "alias `{}` de {} sem arquivo correspondente no inventário",
                    rule.pattern, rule.config_path
                )),
                1 => AliasResolution::Resolved(found.into_iter().next().expect("um arquivo")),
                _ => AliasResolution::Ambiguous(found.into_iter().collect()),
            };
        }
        AliasResolution::NoMatch
    }

    /// Classifica um especificador TypeScript/JavaScript sem olhar módulos do contrato.
    pub fn classify_external(&self, specifier: &str) -> ExternalClassification {
        let (base_name, _) = package_base(specifier);
        if is_node_builtin(specifier) {
            return ExternalClassification {
                state: ExternalPackageState::Builtin,
                base_name: base_name.to_owned(),
                origin: "node embutido".to_owned(),
            };
        }
        if let Some(package) = self
            .local_packages
            .get(specifier)
            .or_else(|| self.local_packages.get(base_name))
        {
            return ExternalClassification {
                state: ExternalPackageState::WorkspaceLocal,
                base_name: base_name.to_owned(),
                origin: format!("manifesto {}", package.manifest_path),
            };
        }
        if let Some(declared) = self
            .declared_external
            .get(specifier)
            .or_else(|| self.declared_external.get(base_name))
        {
            return ExternalClassification {
                state: ExternalPackageState::Declared,
                base_name: base_name.to_owned(),
                origin: format!("manifesto {} ({})", declared.origin, declared.scope),
            };
        }
        ExternalClassification {
            state: ExternalPackageState::Unconfirmed,
            base_name: base_name.to_owned(),
            origin: "sem declaração verificável".to_owned(),
        }
    }

    /// Classifica um nome Python absoluto fora dos módulos do contrato.
    pub fn classify_python(&self, name: &str) -> ExternalClassification {
        let normalized = normalize_distribution(name);
        if is_python_builtin(name) {
            return ExternalClassification {
                state: ExternalPackageState::Builtin,
                base_name: name.to_owned(),
                origin: "biblioteca padrão Python".to_owned(),
            };
        }
        if let Some(distribution) = self.python_equivalent.get(&normalized) {
            return ExternalClassification {
                state: ExternalPackageState::Declared,
                base_name: name.to_owned(),
                origin: format!("distribuição `{distribution}` declarada em manifesto Python"),
            };
        }
        ExternalClassification {
            state: ExternalPackageState::Unconfirmed,
            base_name: name.to_owned(),
            origin: "sem mapeamento determinístico entre módulo e distribuição".to_owned(),
        }
    }
}

/// Classificação de pacote sem arquivo de origem, completada pelo analisador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalClassification {
    /// Estado determinístico da relação.
    pub state: ExternalPackageState,
    /// Nome-base do pacote, sem subpath.
    pub base_name: String,
    /// Origem da declaração ou do reconhecimento.
    pub origin: String,
}

/// Nome-base de um especificador e seu subpath, quando houver.
pub fn package_base(specifier: &str) -> (&str, Option<&str>) {
    if specifier.starts_with("node:") {
        return (specifier, None);
    }
    if let Some(rest) = specifier.strip_prefix('@') {
        let mut parts = rest.splitn(2, '/');
        let _scope = parts.next().unwrap_or(rest);
        if let Some(remainder) = parts.next() {
            let mut name_and_path = remainder.splitn(2, '/');
            let name = name_and_path.next().unwrap_or(remainder);
            let base_end = specifier.len() - remainder.len() + name.len();
            return (
                &specifier[..base_end],
                name_and_path.next().filter(|sub| !sub.is_empty()),
            );
        }
        return (specifier, None);
    }
    match specifier.split_once('/') {
        Some((base, subpath)) => (base, Some(subpath).filter(|sub| !sub.is_empty())),
        None => (specifier, None),
    }
}

/// Indica se o especificador é um built-in Node conhecido.
pub fn is_node_builtin(specifier: &str) -> bool {
    specifier.starts_with("node:") || NODE_BUILTINS.contains(&specifier)
}

/// Indica se o nome é uma raiz conhecida da biblioteca padrão Python.
pub fn is_python_builtin(name: &str) -> bool {
    PYTHON_BUILTINS.contains(&name)
}

/// Normalização determinística usada apenas para nomes equivalentes por caso e separador.
fn normalize_distribution(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace('-', "_")
}

/// Incorpora um manifesto lido na 6.2 ao contexto de resolução.
fn add_manifest(root: &Path, observation: &ManifestObservation, context: &mut ResolutionContext) {
    if observation.status == ManifestStatus::Malformed {
        return;
    }
    for package in &observation.local_packages {
        context
            .local_packages
            .entry(package.name.clone())
            .or_insert_with(|| LocalPackageRef {
                name: package.name.clone(),
                manifest_path: package.manifest_path.clone(),
                source_path: package.source_path.clone(),
            });
    }
    let ecosystem = match observation.ecosystem {
        ManifestEcosystem::Npm => "npm",
        ManifestEcosystem::Python => "python",
        ManifestEcosystem::Go => "go",
    };
    for dependency in &observation.dependencies {
        if dependency.destination == DependencyDestination::LocalWorkspace {
            continue;
        }
        let scope = dependency.scope.as_str();
        let name = if ecosystem == "python" {
            normalize_distribution(&dependency.name)
        } else {
            dependency.name.clone()
        };
        if name.is_empty() || context.local_packages.contains_key(&name) {
            continue;
        }
        context
            .declared_external
            .entry(name.clone())
            .or_insert_with(|| DeclaredExternalRef {
                name: name.clone(),
                ecosystem,
                scope,
                origin: observation.path.clone(),
            });
        if ecosystem == "python" {
            context
                .python_equivalent
                .insert(name, dependency.name.clone());
        }
    }
    if root.join(&observation.path).is_file() && ecosystem == "python" {
        context.limitations.push(format!(
            "manifesto {}: nomes de distribuição sem equivalência determinística não são associados a imports",
            observation.path
        ));
    }
}

/// Configuração TypeScript lida de um arquivo dentro da raiz.
#[derive(Debug, Clone)]
struct TsConfig {
    /// Caminho relativo do arquivo.
    path: String,
    /// Diretório relativo que contém o arquivo.
    directory: String,
    /// Valor bruto de `extends`, quando presente.
    extends: Option<String>,
    /// `baseUrl` normalizado, relativo à raiz.
    base_url: Option<String>,
    /// Pares `paths` na ordem lida do JSON.
    paths: Vec<(String, Vec<String>)>,
}

/// Configuração mesclada da cadeia local de `extends`.
#[derive(Debug, Clone)]
struct MergedConfig {
    /// Diretório-base efetivo para os destinos de `paths`.
    base_url: String,
    /// Regras `paths` da cadeia, com a configuração filha prevalecendo.
    paths: Vec<(String, Vec<String>)>,
    /// Configuração mais próxima do arquivo analisado.
    origin: String,
}

/// Varre a raiz em busca de `tsconfig*.json`, sem seguir links simbólicos.
fn read_tsconfigs(root: &Path, limitations: &mut Vec<String>) -> Vec<TsConfig> {
    fn visit(
        base: &Path,
        directory: &Path,
        found: &mut Vec<TsConfig>,
        limitations: &mut Vec<String>,
    ) {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) => {
                limitations.push(format!(
                    "diretório {} inacessível na busca de tsconfig: {}",
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
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                limitations.push(format!(
                    "link simbólico ignorado na busca de tsconfig: {relative}"
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
            let name = entry.file_name().to_string_lossy().into_owned();
            if !file_type.is_file() || !name.starts_with("tsconfig") || !name.ends_with(".json") {
                continue;
            }
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(error) => {
                    limitations.push(format!(
                        "tsconfig {relative} não pôde ser lido: {}",
                        brief_error(&error)
                    ));
                    continue;
                }
            };
            let value: JsonValue = match serde_json::from_str(&text) {
                Ok(value) => value,
                Err(error) => {
                    limitations.push(format!(
                        "tsconfig {relative} inválido: {}",
                        brief_error(&error)
                    ));
                    continue;
                }
            };
            let directory = parent_relative(&relative);
            let options = value.get("compilerOptions");
            let base_url = options
                .and_then(|options| options.get("baseUrl"))
                .and_then(JsonValue::as_str)
                .and_then(|value| normalize_relative(&directory, value));
            let mut paths = Vec::new();
            if let Some(table) = options
                .and_then(|options| options.get("paths"))
                .and_then(JsonValue::as_object)
            {
                for (pattern, targets) in table {
                    let Some(values) = targets.as_array() else {
                        limitations.push(format!(
                            "tsconfig {relative}: paths.{pattern} não é uma lista"
                        ));
                        continue;
                    };
                    let targets = values
                        .iter()
                        .filter_map(JsonValue::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    if targets.len() != values.len() {
                        limitations.push(format!(
                            "tsconfig {relative}: paths.{pattern} contém valor não textual"
                        ));
                    }
                    paths.push((pattern.clone(), targets));
                }
            }
            let extends = value
                .get("extends")
                .and_then(JsonValue::as_str)
                .map(str::to_owned);
            found.push(TsConfig {
                path: relative,
                directory,
                extends,
                base_url,
                paths,
            });
        }
    }

    let mut found = Vec::new();
    visit(root, root, &mut found, limitations);
    found.sort_by(|left, right| left.path.cmp(&right.path));
    found
}

/// Constrói regras de alias a partir das configurações encontradas.
fn build_aliases(configs: &[TsConfig], limitations: &mut Vec<String>) -> Vec<AliasRule> {
    let mut rules = Vec::new();
    for config in configs {
        let Some(merged) = merge_config(config, configs, limitations) else {
            continue;
        };
        for (pattern, targets) in &merged.paths {
            let stars = pattern.matches('*').count();
            if stars > 1 {
                limitations.push(format!(
                    "tsconfig {}: padrão `{pattern}` com mais de um `*` não é suportado",
                    config.path
                ));
                continue;
            }
            let normalized = targets
                .iter()
                .filter_map(|target| normalize_relative(&merged.base_url, target))
                .collect::<Vec<_>>();
            if normalized.len() != targets.len() {
                limitations.push(format!(
                    "tsconfig {}: destino de `{pattern}` tenta sair da raiz",
                    config.path
                ));
            }
            rules.push(AliasRule {
                pattern: pattern.clone(),
                targets: normalized,
                config_path: merged.origin.clone(),
            });
        }
    }
    rules.sort_by(|left, right| {
        let left_specificity = (
            left.pattern.contains('*'),
            std::cmp::Reverse(left.pattern.len()),
        );
        let right_specificity = (
            right.pattern.contains('*'),
            std::cmp::Reverse(right.pattern.len()),
        );
        left_specificity
            .cmp(&right_specificity)
            .then_with(|| left.pattern.cmp(&right.pattern))
    });
    rules.dedup_by(|left, right| left.pattern == right.pattern && left.targets == right.targets);
    rules
}

/// Mescla a cadeia local de `extends`, da configuração mais externa para a mais próxima.
fn merge_config(
    config: &TsConfig,
    configs: &[TsConfig],
    limitations: &mut Vec<String>,
) -> Option<MergedConfig> {
    let mut chain = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current = config;
    loop {
        if !seen.insert(current.path.as_str()) {
            limitations.push(format!("tsconfig {}: ciclo em extends local", config.path));
            break;
        }
        chain.push(current);
        let Some(extends) = current.extends.as_deref() else {
            break;
        };
        if !extends.starts_with('.') {
            limitations.push(format!(
                "tsconfig {}: extends por pacote externo `{extends}` não foi carregado",
                current.path
            ));
            break;
        }
        let candidate =
            normalize_relative(&current.directory, extends).filter(|path| !path.is_empty());
        let Some(candidate) = candidate else {
            limitations.push(format!(
                "tsconfig {}: extends `{extends}` tenta sair da raiz",
                current.path
            ));
            break;
        };
        let with_extension = if candidate.ends_with(".json") {
            candidate.clone()
        } else {
            format!("{candidate}.json")
        };
        let Some(next) = configs
            .iter()
            .find(|item| item.path == candidate || item.path == with_extension)
        else {
            limitations.push(format!(
                "tsconfig {}: extends local `{extends}` sem arquivo correspondente",
                current.path
            ));
            break;
        };
        current = next;
    }
    chain.reverse();

    let mut base_url = String::new();
    let mut paths: Vec<(String, Vec<String>)> = Vec::new();
    for item in chain {
        if let Some(value) = &item.base_url {
            base_url = value.clone();
        }
        for (pattern, targets) in &item.paths {
            paths.retain(|(existing, _)| existing != pattern);
            paths.push((pattern.clone(), targets.clone()));
        }
    }
    Some(MergedConfig {
        base_url,
        paths,
        origin: config.path.clone(),
    })
}

/// Captura o trecho casado por um padrão com zero ou um `*`.
fn alias_capture<'a>(pattern: &str, specifier: &'a str) -> Option<&'a str> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if suffix.contains('*') || !specifier.starts_with(prefix) || !specifier.ends_with(suffix) {
            return None;
        }
        let start = prefix.len();
        let end = specifier.len().checked_sub(suffix.len())?;
        if end < start {
            return None;
        }
        return Some(&specifier[start..end]);
    }
    (pattern == specifier).then_some("")
}

/// Normaliza um caminho relativo à raiz, recusando qualquer saída dela.
fn normalize_relative(base: &str, child: &str) -> Option<String> {
    let child = child.trim().replace('\\', "/");
    let joined = if base.is_empty() {
        child
    } else {
        format!("{}/{}", base.trim_end_matches('/'), child)
    };
    let mut parts: Vec<String> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name.to_owned()),
        }
    }
    Some(parts.join("/"))
}

/// Diretório relativo de um arquivo, usando `/` em qualquer sistema operacional.
fn parent_relative(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_owned())
        .unwrap_or_default()
}

/// Caminho relativo estável, usando `/` em qualquer sistema operacional.
fn relative_path(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Primeira linha de um erro, sem trechos de conteúdo de configuração.
fn brief_error(error: &impl std::fmt::Display) -> String {
    error
        .to_string()
        .lines()
        .next()
        .unwrap_or("erro desconhecido")
        .trim()
        .chars()
        .take(160)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::discover_without_contract;
    use std::path::PathBuf;

    /// Contexto da fixture de resolução, montado pelo mesmo caminho da aplicação.
    fn context_for(name: &str) -> ResolutionContext {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name);
        let inventory = discover_without_contract(&root).expect("inventário deve passar");
        ResolutionContext::for_inventory(&inventory)
    }

    #[test]
    fn package_base_separates_scopes_and_subpaths() {
        assert_eq!(
            package_base("@scope/name/sub/path"),
            ("@scope/name", Some("sub/path"))
        );
        assert_eq!(package_base("@scope/name"), ("@scope/name", None));
        assert_eq!(package_base("lodash/fp"), ("lodash", Some("fp")));
        assert_eq!(package_base("node:fs"), ("node:fs", None));
    }

    #[test]
    fn builtins_are_recognized_without_any_runtime_lookup() {
        assert!(is_node_builtin("node:fs"));
        assert!(is_node_builtin("path"));
        assert!(!is_node_builtin("react"));
        assert!(is_python_builtin("os"));
        assert!(!is_python_builtin("httpx"));
    }

    #[test]
    fn local_extends_is_merged_and_external_extends_is_not_loaded() {
        let context = context_for("resolution_typescript");
        let aliases: Vec<&str> = context
            .aliases
            .iter()
            .map(|rule| rule.pattern.as_str())
            .collect();
        assert!(
            aliases.contains(&"@domain/*"),
            "alias do extends local deve entrar no contexto"
        );
        assert!(
            aliases.contains(&"@local/*"),
            "alias do arquivo filho deve permanecer"
        );
        assert!(context.limitations.iter().any(|limitation| {
            limitation.contains("extends por pacote externo")
                && limitation.contains("@tsconfig/node18")
        }));
    }

    #[test]
    fn aliases_resolve_to_inventory_files_or_report_ambiguity() {
        let context = context_for("resolution_typescript");
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects/resolution_typescript");
        let inventory = discover_without_contract(&root).expect("inventário deve passar");
        let files: BTreeSet<&str> = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect();

        assert_eq!(
            context.resolve_alias("@domain/entity", &files),
            AliasResolution::Resolved("src/domain/entity.ts".to_owned())
        );
        assert_eq!(
            context.resolve_alias("react", &files),
            AliasResolution::NoMatch
        );
        let ambiguous = context.resolve_alias("@local/duplo", &files);
        assert!(matches!(ambiguous, AliasResolution::Ambiguous(_)));
        let missing = context.resolve_alias("@local/ausente", &files);
        assert!(matches!(missing, AliasResolution::Unresolved(_)));
    }

    #[test]
    fn external_states_come_from_manifests_and_known_builtins() {
        let context = context_for("resolution_typescript");
        assert_eq!(
            context.classify_external("node:fs").state,
            ExternalPackageState::Builtin
        );
        assert_eq!(
            context.classify_external("react").state,
            ExternalPackageState::Declared
        );
        assert!(
            context
                .classify_external("react")
                .origin
                .contains("package.json")
        );
        assert_eq!(
            context.classify_external("desconhecido").state,
            ExternalPackageState::Unconfirmed
        );
        assert_eq!(
            context.classify_external("@demo/api").state,
            ExternalPackageState::WorkspaceLocal
        );
    }

    #[test]
    fn python_equivalence_is_limited_to_deterministic_names() {
        let context = context_for("resolution_python");
        assert_eq!(
            context.classify_python("httpx").state,
            ExternalPackageState::Declared
        );
        assert_eq!(
            context.classify_python("PIL").state,
            ExternalPackageState::Unconfirmed
        );
        assert_eq!(
            context.classify_python("sys").state,
            ExternalPackageState::Builtin
        );
    }
}
