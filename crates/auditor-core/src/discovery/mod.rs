//! Descoberta somente leitura de arquivos de um projeto.
//!
//! O inventorista nunca executa arquivos do projeto, nunca segue links
//! simbólicos e registra entradas ignoradas em vez de interromper a varredura.

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::contract::Contract;

/// Diretórios ignorados por padrão, em qualquer profundidade.
pub const DEFAULT_IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    ".venv",
    "venv",
];

/// Resultado de uma varredura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Raiz canonicalizada informada pelo usuário.
    pub root: PathBuf,
    /// Arquivos que passaram em `include` e não foram excluídos.
    pub files: Vec<DiscoveredFile>,
    /// Entradas que não entraram no inventário, com o motivo.
    pub skipped: Vec<SkippedEntry>,
}

/// Arquivo incluído no inventário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFile {
    /// Caminho relativo à raiz, usando `/` como separador.
    pub path: String,
    /// Extensão do arquivo, quando existir.
    pub extension: Option<String>,
    /// Tamanho em bytes.
    pub size: u64,
}

/// Entrada não incluída no inventário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedEntry {
    pub path: String,
    pub reason: SkipReason,
}

/// Motivo pelo qual uma entrada não entrou no inventário.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    DefaultIgnored,
    Excluded,
    NotIncluded,
    Symlink,
    Inaccessible,
    Unsupported,
}

/// Falha operacional que impede a varredura da raiz inteira.
#[derive(Debug)]
pub enum DiscoveryError {
    Root {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root { path, source } => {
                write!(formatter, "raiz inacessível {}: {source}", path.display())
            }
        }
    }
}

impl Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Root { source, .. } => Some(source),
        }
    }
}

/// Percorre a raiz e produz um inventário ordenado.
pub fn discover(root: &Path, contract: &Contract) -> Result<Inventory, DiscoveryError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| DiscoveryError::Root {
        path: root.to_path_buf(),
        source,
    })?;

    let mut inventory = Inventory {
        root: canonical_root.clone(),
        files: Vec::new(),
        skipped: Vec::new(),
    };

    walk(&canonical_root, &canonical_root, contract, &mut inventory);

    inventory
        .files
        .sort_by(|left, right| left.path.cmp(&right.path));
    inventory
        .skipped
        .sort_by(|left, right| left.path.cmp(&right.path));
    Ok(inventory)
}

fn walk(root: &Path, directory: &Path, contract: &Contract, inventory: &mut Inventory) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => {
            inventory.skipped.push(SkippedEntry {
                path: relative(root, directory),
                reason: SkipReason::Inaccessible,
            });
            return;
        }
    };

    let mut entries: Vec<_> = entries.collect();
    entries.sort_by_key(|entry| entry.as_ref().ok().map(|entry| entry.file_name()));

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                inventory.skipped.push(SkippedEntry {
                    path: relative(root, directory),
                    reason: SkipReason::Inaccessible,
                });
                continue;
            }
        };

        let entry_path = entry.path();
        let path = relative(root, &entry_path);

        let metadata = match fs::symlink_metadata(&entry_path) {
            Ok(metadata) => metadata,
            Err(_) => {
                inventory.skipped.push(SkippedEntry {
                    path,
                    reason: SkipReason::Inaccessible,
                });
                continue;
            }
        };

        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            inventory.skipped.push(SkippedEntry {
                path,
                reason: SkipReason::Symlink,
            });
            continue;
        }

        if file_type.is_dir() {
            let name = path.rsplit('/').next().unwrap_or("");
            if DEFAULT_IGNORED_DIRS.contains(&name) {
                inventory.skipped.push(SkippedEntry {
                    path,
                    reason: SkipReason::DefaultIgnored,
                });
                continue;
            }
            if is_excluded(&path, contract) {
                inventory.skipped.push(SkippedEntry {
                    path,
                    reason: SkipReason::Excluded,
                });
                continue;
            }
            walk(root, &entry_path, contract, inventory);
            continue;
        }

        if file_type.is_file() {
            if is_excluded(&path, contract) {
                inventory.skipped.push(SkippedEntry {
                    path,
                    reason: SkipReason::Excluded,
                });
                continue;
            }
            if !contract
                .scan
                .include
                .iter()
                .any(|pattern| matches_pattern(pattern, &path))
            {
                inventory.skipped.push(SkippedEntry {
                    path,
                    reason: SkipReason::NotIncluded,
                });
                continue;
            }

            inventory.files.push(DiscoveredFile {
                extension: entry_path
                    .extension()
                    .map(|extension| extension.to_string_lossy().into_owned()),
                size: metadata.len(),
                path,
            });
            continue;
        }

        inventory.skipped.push(SkippedEntry {
            path,
            reason: SkipReason::Unsupported,
        });
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join("/")
}

fn is_excluded(path: &str, contract: &Contract) -> bool {
    contract
        .scan
        .exclude
        .iter()
        .any(|pattern| matches_pattern(pattern, path))
}

/// Corresponde ao subconjunto de glob usado pelo contrato: `**` como
/// componente inteiro e `*` dentro de um componente. Outros curingas são
/// tratados literalmente.
fn matches_pattern(pattern: &str, path: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    matches_parts(&pattern_parts, &path_parts)
}

fn matches_parts(pattern: &[&str], path: &[&str]) -> bool {
    match pattern {
        [] => path.is_empty(),
        [only] if *only == "**" => true,
        [first, rest @ ..] if *first == "**" => {
            (0..=path.len()).any(|skip| matches_parts(rest, &path[skip..]))
        }
        [first, rest @ ..] => match path.first() {
            Some(value) => matches_component(first, value) && matches_parts(rest, &path[1..]),
            None => false,
        },
    }
}

fn matches_component(pattern: &str, value: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let value: Vec<char> = value.chars().collect();

    fn compare(pattern: &[char], value: &[char]) -> bool {
        match pattern {
            [] => value.is_empty(),
            ['*', rest @ ..] => (0..=value.len()).any(|skip| compare(rest, &value[skip..])),
            [expected, rest @ ..] => value.first() == Some(expected) && compare(rest, &value[1..]),
        }
    }

    compare(&pattern, &value)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::contract::{load_contract, parse_contract};

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/simple")
    }

    fn contract() -> Contract {
        load_contract(&project_root().join("auditor.toml")).expect("fixture contract must load")
    }

    fn reason_for(inventory: &Inventory, path: &str) -> Option<SkipReason> {
        inventory
            .skipped
            .iter()
            .find(|entry| entry.path == path)
            .map(|entry| entry.reason)
    }

    #[test]
    fn inventory_respects_include_exclude_and_default_ignores() {
        let inventory = discover(&project_root(), &contract()).expect("discovery must succeed");

        let files: Vec<&str> = inventory
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        assert_eq!(
            files,
            [
                "crates/app/src/application/beta.rs",
                "crates/app/src/domain/alpha.rs",
                "crates/root.rs"
            ]
        );

        assert_eq!(reason_for(&inventory, "vendor"), Some(SkipReason::Excluded));
        assert_eq!(
            reason_for(&inventory, "target"),
            Some(SkipReason::DefaultIgnored)
        );
        assert_eq!(
            reason_for(&inventory, "node_modules"),
            Some(SkipReason::DefaultIgnored)
        );
        assert_eq!(
            reason_for(&inventory, "dist"),
            Some(SkipReason::DefaultIgnored)
        );
        assert_eq!(
            reason_for(&inventory, "build"),
            Some(SkipReason::DefaultIgnored)
        );
        assert_eq!(
            reason_for(&inventory, ".venv"),
            Some(SkipReason::DefaultIgnored)
        );
        assert_eq!(
            reason_for(&inventory, "venv"),
            Some(SkipReason::DefaultIgnored)
        );
        assert_eq!(
            reason_for(&inventory, "notes.txt"),
            Some(SkipReason::NotIncluded)
        );
        assert_eq!(
            reason_for(&inventory, "crates/app/src/unmapped/gamma.txt"),
            Some(SkipReason::NotIncluded)
        );
    }

    #[test]
    fn extension_and_size_are_recorded() {
        let inventory = discover(&project_root(), &contract()).expect("discovery must succeed");
        let file = inventory
            .files
            .iter()
            .find(|file| file.path == "crates/root.rs")
            .expect("crates/root.rs must be included");

        assert_eq!(file.extension.as_deref(), Some("rs"));
        assert_eq!(
            file.size,
            fs::metadata(project_root().join("crates/root.rs"))
                .expect("fixture file must be readable")
                .len()
        );
    }

    #[test]
    fn contract_exclude_can_remove_an_included_directory() {
        let contract = parse_contract(
            r#"
schema_version = 1

[project]
name = "x"

[scan]
include = ["crates/**/*.rs"]
exclude = ["crates/app/src/application/**"]

[architecture]
style = "modular_monolith"
modules = []
"#,
        )
        .expect("contract must be valid");

        let inventory = discover(&project_root(), &contract).expect("discovery must succeed");

        assert!(
            inventory
                .files
                .iter()
                .all(|file| file.path != "crates/app/src/application/beta.rs")
        );
        assert_eq!(
            reason_for(&inventory, "crates/app/src/application"),
            Some(SkipReason::Excluded)
        );
    }

    #[test]
    fn missing_root_is_an_operational_error() {
        let missing = project_root().join("nao-existe");
        assert!(matches!(
            discover(&missing, &contract()),
            Err(DiscoveryError::Root { .. })
        ));
    }

    #[test]
    fn wildcard_supports_zero_or_more_components() {
        assert!(matches_pattern("crates/**/*.rs", "crates/root.rs"));
        assert!(matches_pattern(
            "crates/**/*.rs",
            "crates/app/src/domain/alpha.rs"
        ));
        assert!(!matches_pattern("crates/**/*.rs", "other/root.rs"));
        assert!(matches_pattern("*.rs", "root.rs"));
        assert!(!matches_pattern("*.rs", "dir/root.rs"));
    }
}
