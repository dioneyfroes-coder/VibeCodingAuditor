//! Adaptador Tauri da aplicação desktop.
//!
//! Este crate não contém regras de auditoria: ele traduz chamadas da interface
//! para funções públicas de `auditor-core` e devolve os modelos serializados
//! produzidos pelo núcleo.

use std::fs;
use std::io;
use std::path::Path;

use auditor_core::analysis::{self, Analysis};
use auditor_core::audit::{self, DiscoveryReport, ProjectRun};
use auditor_core::builder::{
    self, ContractDiff, ContractDraft, DraftEffects, SaveAuthorization, SaveOutcome,
    ValidationResult,
};
use auditor_core::contract::{self, load_contract};
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

/// Detecta a raiz com ou sem contrato e escolhe o modo no núcleo.
#[tauri::command]
fn load_project(path: String) -> Result<ProjectRun, String> {
    audit::project_run(Path::new(&path)).map_err(|error| error.to_string())
}

/// Produz a descoberta somente leitura para a tela inicial do assistente.
#[tauri::command]
fn discover_project(path: String) -> Result<DiscoveryReport, String> {
    audit::discover_project(Path::new(&path)).map_err(|error| error.to_string())
}

/// Inicia o rascunho a partir da descoberta ou de um contrato existente.
#[tauri::command]
fn start_draft(path: String) -> Result<ContractDraft, String> {
    let root = Path::new(&path);
    let contract_path = root.join("auditor.toml");
    if contract_path.exists() {
        let contract = load_contract(&contract_path).map_err(|error| error.to_string())?;
        Ok(builder::draft_from_contract(&contract))
    } else {
        let report = audit::discover_project(root).map_err(|error| error.to_string())?;
        Ok(builder::draft_from_discovery(&report))
    }
}

/// Valida escolhas sem tratar decisões pendentes como falha operacional.
#[tauri::command]
fn validate_contract_draft(draft: ContractDraft) -> Result<ValidationResult, String> {
    builder::validate_draft_result(&draft).map_err(|error| error.to_string())
}

/// Devolve a prévia TOML do rascunho, sem gravar nenhum arquivo.
#[tauri::command]
fn preview_contract(draft: ContractDraft) -> Result<String, String> {
    let contract = builder::validate_draft(&draft).map_err(|error| error.to_string())?;
    contract::render_contract(&contract).map_err(|error| error.to_string())
}

/// Descreve os efeitos do rascunho com os identificadores de regra do núcleo.
#[tauri::command]
fn draft_effects(draft: ContractDraft) -> DraftEffects {
    builder::draft_effects(&draft)
}

/// Mostra o diff entre o contrato atual e a prévia, sem gravar arquivos.
#[tauri::command]
fn contract_diff(path: String, draft: ContractDraft) -> Result<ContractDiff, String> {
    let root = Path::new(&path);
    let contract_path = root.join("auditor.toml");
    let current = match fs::read_to_string(&contract_path) {
        Ok(text) => text,
        Err(source) if source.kind() == io::ErrorKind::NotFound => String::new(),
        Err(source) => return Err(source.to_string()),
    };
    let contract = builder::validate_draft(&draft).map_err(|error| error.to_string())?;
    let preview = contract::render_contract(&contract).map_err(|error| error.to_string())?;
    let mut diff = builder::contract_diff(&current, &preview);
    diff.exists = contract_path
        .try_exists()
        .map_err(|error| error.to_string())?;
    Ok(diff)
}

/// Grava o contrato aprovado e executa a conformidade, tudo no núcleo.
#[tauri::command]
fn save_contract(
    path: String,
    draft: ContractDraft,
    authorization: SaveAuthorization,
) -> Result<SaveOutcome, String> {
    builder::save_draft(Path::new(&path), &draft, &authorization).map_err(|error| error.to_string())
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
            load_project,
            discover_project,
            start_draft,
            validate_contract_draft,
            preview_contract,
            draft_effects,
            contract_diff,
            save_contract,
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
    fn audit_command_exposes_v2_layer_finding_without_adapter_rules() {
        let root = project_fixture("layer_boundaries");
        let from_command =
            audit_project(root.to_string_lossy().into_owned()).expect("command must succeed");
        let from_core = auditor_core::audit::audit_project(&root).expect("core must succeed");

        assert_eq!(from_command, from_core);
        assert!(from_command.findings.iter().any(|finding| {
            finding.rule_id == "dep.layer_not_allowed"
                && finding.id
                    == "dep.layer_not_allowed:application:infrastructure:src/application/rules.rs"
        }));
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

    #[test]
    fn load_project_command_detects_the_execution_mode() {
        let missing = project_fixture("discovery_no_contract");
        let from_command =
            load_project(missing.to_string_lossy().into_owned()).expect("command must succeed");
        let from_core = audit::project_run(&missing).expect("core must succeed");
        assert_eq!(from_command, from_core);
        assert_eq!(
            from_command.mode(),
            auditor_core::audit::ExecutionMode::Discovery
        );

        let present = project_fixture("clean");
        let from_command =
            load_project(present.to_string_lossy().into_owned()).expect("command must succeed");
        assert_eq!(
            from_command.mode(),
            auditor_core::audit::ExecutionMode::Compliance
        );
    }

    /// Cópia temporária de uma fixture, removida ao fim do teste.
    struct TempProject(PathBuf);

    impl TempProject {
        fn new(fixture: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            use std::time::{SystemTime, UNIX_EPOCH};

            static NEXT: AtomicU64 = AtomicU64::new(0);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("relógio do sistema")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "auditor-tauri-e46-{}-{nanos}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            copy_directory(&project_fixture(fixture), &root);
            Self(root)
        }

        fn text(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn copy_directory(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).expect("destino da cópia");
        for entry in std::fs::read_dir(from).expect("origem da cópia") {
            let entry = entry.expect("entrada legível");
            let target = to.join(entry.file_name());
            if entry.file_type().expect("tipo da entrada").is_dir() {
                copy_directory(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).expect("arquivo copiado");
            }
        }
    }

    #[test]
    fn contract_diff_command_delegates_to_the_core() {
        let root = project_fixture("clean");
        let draft = start_draft(root.to_string_lossy().into_owned()).expect("draft must start");
        let current = read_contract(root.to_string_lossy().into_owned()).expect("contract text");
        let from_command =
            contract_diff(root.to_string_lossy().into_owned(), draft.clone()).expect("command");

        let contract = builder::validate_draft(&draft).expect("draft must be valid");
        let preview = contract::render_contract(&contract).expect("preview must render");
        let mut from_core = builder::contract_diff(&current, &preview);
        from_core.exists = true;

        assert_eq!(from_command, from_core);
        assert!(from_command.exists);
    }

    #[test]
    fn save_contract_command_refuses_overwrite_without_confirmation() {
        let project = TempProject::new("clean");
        let path = project.0.join("auditor.toml");
        let original = std::fs::read_to_string(&path).expect("contract must exist");
        let draft = start_draft(project.text()).expect("draft must start");

        let error = save_contract(project.text(), draft, SaveAuthorization::Create)
            .expect_err("create must not overwrite");
        assert!(error.contains("substituição exige confirmação"), "{error}");
        assert_eq!(
            std::fs::read_to_string(&path).expect("contract preserved"),
            original
        );
    }

    #[test]
    fn save_contract_command_delegates_to_the_core() {
        let command_project = TempProject::new("discovery_no_contract");
        let core_project = TempProject::new("discovery_no_contract");
        let draft = builder::empty_draft("delegado");

        let from_command = save_contract(
            command_project.text(),
            draft.clone(),
            SaveAuthorization::Create,
        )
        .expect("command must save");
        let from_core = builder::save_draft(&core_project.0, &draft, &SaveAuthorization::Create)
            .expect("core must save");

        assert_eq!(from_command.saved, from_core.saved);
        assert_eq!(from_command.replaced, from_core.replaced);
        assert_eq!(from_command.run.mode, from_core.run.mode);
        assert_eq!(from_command.run.contract, from_core.run.contract);
        assert_eq!(
            from_command.run.report.summary,
            from_core.run.report.summary
        );
    }

    #[test]
    fn discover_command_delegates_to_the_core() {
        let root = project_fixture("discovery_candidates");
        let from_command =
            discover_project(root.to_string_lossy().into_owned()).expect("command must succeed");
        let from_core = audit::discover_project(&root).expect("core must succeed");

        assert_eq!(from_command, from_core);
        assert_eq!(from_command.candidates.candidate_modules.len(), 3);
    }

    #[test]
    fn start_draft_command_uses_suggestions_without_a_contract() {
        let root = project_fixture("discovery_candidates");
        let draft = start_draft(root.to_string_lossy().into_owned()).expect("command must succeed");

        assert_eq!(draft.modules.len(), 3);
        assert!(draft.modules.iter().all(|module| module.needs_confirmation));
        assert!(!draft.pending_decisions.is_empty());
    }

    #[test]
    fn start_draft_command_uses_the_existing_contract() {
        let root = project_fixture("clean");
        let draft = start_draft(root.to_string_lossy().into_owned()).expect("command must succeed");

        assert!(!draft.modules.is_empty());
        assert!(
            draft
                .modules
                .iter()
                .all(|module| !module.needs_confirmation)
        );
        assert!(draft.pending_decisions.is_empty());
    }

    #[test]
    fn validation_command_keeps_pending_choices_as_interface_state() {
        let mut draft = start_draft(project_fixture("clean").to_string_lossy().into_owned())
            .expect("draft must start");
        let accepted = validate_contract_draft(draft.clone()).expect("command must succeed");
        assert!(accepted.valid);
        assert!(accepted.identity.is_some());

        draft.modules[0].needs_confirmation = true;
        let rejected = validate_contract_draft(draft).expect("command must succeed");
        assert!(!rejected.valid);
        assert!(rejected.identity.is_none());
        assert!(rejected.errors[0].contains("módulo ainda não confirmado"));
    }

    #[test]
    fn preview_command_uses_the_official_serializer() {
        let draft = start_draft(project_fixture("clean").to_string_lossy().into_owned())
            .expect("draft must start");
        let text = preview_contract(draft.clone()).expect("command must succeed");
        let contract = auditor_core::contract::parse_contract(&text).expect("TOML must be valid");

        assert_eq!(contract.schema_version, 2);
        assert_eq!(
            text,
            auditor_core::contract::render_contract(&contract).expect("core must render")
        );
    }

    #[test]
    fn effects_command_delegates_to_the_core() {
        let draft = start_draft(project_fixture("clean").to_string_lossy().into_owned())
            .expect("draft must start");
        let from_command = draft_effects(draft.clone());
        let from_core = auditor_core::builder::draft_effects(&draft);

        assert_eq!(from_command, from_core);
        assert!(
            from_command
                .rule_ids
                .contains(&"dep.not_allowed".to_owned())
        );
        assert!(
            !from_command
                .rule_ids
                .contains(&"dep.layer_not_allowed".to_owned())
        );
    }
}
