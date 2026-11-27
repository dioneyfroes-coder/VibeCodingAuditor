//! Testes de ponta a ponta da CLI, incluindo modo descoberta e códigos de saída.
//!
//! Os comandos são executados como processos reais para verificar o que o
//! usuário vê, sem duplicar regras do núcleo no teste.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Caminho absoluto de uma fixture, relativo ao crate da CLI.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/projects")
        .join(name)
}

/// Caminho temporário único, sem dependência externa.
fn temp_path(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock must be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("vca-cli-{label}-{}-{nanos}", std::process::id()))
}

/// Executa o binário e devolve código, saída padrão e saída de erro.
fn run(args: &[&Path]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vibe-coding-auditor"))
        .args(args)
        .output()
        .expect("CLI binary must run");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Argumento de caminho como `&Path`, usado por `run`.
fn arg(value: &str) -> &Path {
    Path::new(value)
}

#[test]
fn scan_analyze_and_audit_work_without_contract() {
    let root = fixture("discovery_no_contract");

    let (code, stdout, stderr) = run(&[arg("scan"), root.as_path()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Modo: descoberta (sem contrato)"));
    assert!(stdout.contains("Arquivos incluídos:"));

    let (code, stdout, stderr) = run(&[arg("analyze"), root.as_path()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Modo: descoberta (sem contrato)"));
    assert!(stdout.contains("Arestas:"));
    assert!(stdout.contains("observada"));
    assert!(!stdout.contains("[permitida]"));
    assert!(!stdout.contains("[proibida]"));

    let (code, stdout, stderr) = run(&[arg("audit"), root.as_path(), arg("--format"), arg("json")]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("\"mode\": \"discovery\""));
    assert!(stdout.contains("\"status\": \"missing\""));
    assert!(stdout.contains("\"candidate_modules\""));

    let (code, stdout, stderr) = run(&[
        arg("audit"),
        root.as_path(),
        arg("--format"),
        arg("markdown"),
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Modo: descoberta (sem contrato)"));
    assert!(stdout.contains("Módulos candidatos"));
    assert!(!stdout.contains("dep.not_allowed"));
}

#[test]
fn compliance_exit_codes_and_json_fields_are_preserved() {
    let clean = fixture("clean");
    let (code, stdout, stderr) =
        run(&[arg("audit"), clean.as_path(), arg("--format"), arg("json")]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("\"mode\": \"compliance\""));
    assert!(stdout.contains("\"status\": \"present\""));
    assert!(stdout.contains("\"schema_version\": 1"));
    assert!(stdout.contains("\"findings\""));

    let with_findings = fixture("dependencies");
    let (code, stdout, stderr) = run(&[
        arg("audit"),
        with_findings.as_path(),
        arg("--format"),
        arg("markdown"),
    ]);
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stdout.contains("Modo: conformidade (contrato v1)"));
}

#[test]
fn invalid_contract_and_operational_failures_keep_exit_code_two() {
    let invalid = fixture("discovery_invalid_contract");
    let (code, _, stderr) = run(&[arg("audit"), invalid.as_path()]);
    assert_eq!(code, 2);
    assert!(stderr.contains("Falha operacional"));
    assert!(stderr.contains("contrato"));

    let missing_root = temp_path("raiz-inexistente");
    let (code, _, stderr) = run(&[arg("scan"), missing_root.as_path()]);
    assert_eq!(code, 2);
    assert!(stderr.contains("Falha operacional"));
}

#[test]
fn discovery_diff_reports_that_no_findings_were_filtered() {
    let root = fixture("discovery_candidates");
    let patch = temp_path("discovery.patch");
    std::fs::write(&patch, "diff --git a/web/app.ts b/web/app.ts\n--- a/web/app.ts\n+++ b/web/app.ts\n@@ -1 +1 @@\n-old\n+new\n").expect("patch must be writable");

    let (code, stdout, stderr) = run(&[
        arg("audit"),
        root.as_path(),
        arg("--format"),
        arg("json"),
        arg("--diff"),
        patch.as_path(),
    ]);
    let _ = std::fs::remove_file(&patch);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("--diff não filtrou achados"));
    assert!(stdout.contains("\"mode\": \"discovery\""));
}

#[test]
fn history_record_without_contract_records_discovery_mode() {
    let root = fixture("discovery_no_contract");
    let database = temp_path("historico.db");
    let database_text = database.to_string_lossy().into_owned();

    let (code, stdout, stderr) = run(&[
        arg("history"),
        arg("--database"),
        Path::new(&database_text),
        arg("record"),
        root.as_path(),
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("Modo: descoberta (sem contrato)"));
    assert!(stdout.contains("Contrato: sem contrato"));

    let (code, stdout, stderr) = run(&[
        arg("history"),
        arg("--database"),
        Path::new(&database_text),
        arg("list"),
    ]);
    let _ = std::fs::remove_file(&database);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("modo descoberta"));
    assert!(!stdout.contains("dep.not_allowed"));
}

#[test]
fn history_compare_rejects_runs_from_different_modes() {
    let discovery_root = fixture("discovery_no_contract");
    let compliance_root = fixture("dependencies");
    let database = temp_path("comparacao.db");
    let database_text = database.to_string_lossy().into_owned();

    let (code, _, stderr) = run(&[
        arg("history"),
        arg("--database"),
        Path::new(&database_text),
        arg("record"),
        discovery_root.as_path(),
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");

    let (code, _, stderr) = run(&[
        arg("history"),
        arg("--database"),
        Path::new(&database_text),
        arg("record"),
        compliance_root.as_path(),
    ]);
    assert_eq!(code, 1, "stderr: {stderr}");

    let (code, _, stderr) = run(&[
        arg("history"),
        arg("--database"),
        Path::new(&database_text),
        arg("compare"),
        arg("1"),
        arg("2"),
    ]);
    let _ = std::fs::remove_file(&database);

    assert_eq!(code, 2);
    assert!(stderr.contains("não são comparáveis"), "stderr: {stderr}");
    assert!(stderr.contains("modos diferentes"), "stderr: {stderr}");
}
