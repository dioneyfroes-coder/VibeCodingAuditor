//! Adaptador Tauri da aplicação desktop.
//!
//! Este crate não contém regras de auditoria: ele traduz chamadas da interface
//! para funções públicas de `auditor-core` e devolve os modelos serializados
//! produzidos pelo núcleo.

use std::fs;
use std::path::Path;

use auditor_core::analysis::{self, Analysis};
use auditor_core::audit;
use auditor_core::contract::load_contract;
use auditor_core::discovery::discover;
use auditor_core::history::{History, RunSummary};
use auditor_core::report::AuditReport;

/// Executa a auditoria completa na raiz informada.
#[tauri::command]
fn audit_project(path: String) -> Result<AuditReport, String> {
    audit::audit_project(Path::new(&path)).map_err(|error| error.to_string())
}

/// Produz o grafo de dependências com as classificações feitas pelo núcleo.
#[tauri::command]
fn analyze_project(path: String) -> Result<Analysis, String> {
    let root = Path::new(&path);
    let contract = load_contract(&root.join("auditor.toml")).map_err(|error| error.to_string())?;
    let inventory = discover(root, &contract).map_err(|error| error.to_string())?;
    analysis::analyze(root, &contract, &inventory).map_err(|error| error.to_string())
}

/// Devolve o texto do `auditor.toml` para visualização somente leitura.
#[tauri::command]
fn read_contract(path: String) -> Result<String, String> {
    let contract_path = Path::new(&path).join("auditor.toml");
    fs::read_to_string(contract_path).map_err(|error| error.to_string())
}

/// Lista execuções gravadas no banco local, da mais recente para a mais antiga.
#[tauri::command]
fn history_list(
    database: String,
    project: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<RunSummary>, String> {
    let store = History::open(Path::new(&database)).map_err(|error| error.to_string())?;
    store
        .list(project.as_deref(), limit.unwrap_or(50))
        .map_err(|error| error.to_string())
}

/// Executa a auditoria e registra uma nova execução no banco local.
#[tauri::command]
fn history_record(path: String, database: String) -> Result<i64, String> {
    let root = Path::new(&path);
    let report = audit::audit_project(root).map_err(|error| error.to_string())?;
    let mut store = History::open(Path::new(&database)).map_err(|error| error.to_string())?;
    store
        .record(root, &report)
        .map_err(|error| error.to_string())
}

/// Inicia a aplicação com apenas os comandos explícitos acima disponíveis.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            audit_project,
            analyze_project,
            read_contract,
            history_list,
            history_record
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar a aplicação desktop");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// Raiz de uma fixture de projeto, relativa ao crate do adaptador.
    fn project_fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/fixtures/projects")
            .join(name)
    }

    #[test]
    fn audit_command_delegates_to_the_core() {
        let root = project_fixture("clean");
        let from_command =
            audit_project(root.to_string_lossy().into_owned()).expect("command must succeed");
        let from_core = auditor_core::audit::audit_project(&root).expect("core must succeed");

        assert_eq!(from_command, from_core);
        assert!(!from_command.has_findings());
    }

    #[test]
    fn analyze_command_delegates_to_the_core() {
        let root = project_fixture("dependencies");
        let contract = load_contract(&root.join("auditor.toml")).expect("contract must load");
        let inventory = discover(&root, &contract).expect("discovery must succeed");
        let from_core = analysis::analyze(&root, &contract, &inventory).expect("core must succeed");
        let from_command =
            analyze_project(root.to_string_lossy().into_owned()).expect("command must succeed");

        assert_eq!(from_command, from_core);
        assert_eq!(from_command.edges.len(), 6);
    }

    #[test]
    fn contract_command_returns_the_project_contract() {
        let root = project_fixture("partial");
        let text =
            read_contract(root.to_string_lossy().into_owned()).expect("command must succeed");

        assert!(text.contains("schema_version = 1"));
        assert!(text.contains("projeto-parcial"));
    }
}
