//! Fixtures de validação da Etapa 4.8.
//!
//! Os testes executam a CLI como processo real, repetem cada descoberta e
//! comparam o conteúdo dos arquivos antes e depois. Nenhuma regra é
//! reimplementada aqui: só são verificados modo, determinismo e imutabilidade.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Caminho absoluto de uma fixture, relativo ao crate da CLI.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/projects")
        .join(name)
}

/// Executa o binário e devolve código, saída padrão e saída de erro.
fn run(args: &[&Path]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vibe-coding-auditor-cli"))
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

/// Conteúdo completo de uma árvore, indexado por caminho relativo estável.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(base: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).expect("diretório da fixture deve ser legível") {
            let entry = entry.expect("entrada legível");
            let path = entry.path();
            let relative = path
                .strip_prefix(base)
                .expect("entrada dentro da raiz")
                .to_string_lossy()
                .replace('\\', "/");
            if entry.file_type().expect("tipo legível").is_dir() {
                visit(base, &path, files);
            } else {
                let bytes = std::fs::read(&path).expect("arquivo legível");
                files.insert(relative, bytes);
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

/// Executa `audit --format json` em uma fixture sem contrato.
fn discovery_json(root: &Path) -> (i32, String, String) {
    run(&[arg("audit"), root, arg("--format"), arg("json")])
}

/// Nomes das fixtures sem contrato usadas na validação da Etapa 4.
const DISCOVERY_FIXTURES: &[&str] = &[
    "discovery_rust_no_contract",
    "discovery_ts_no_contract",
    "discovery_python_no_contract",
    "discovery_workspace",
    "discovery_ambiguous",
    "discovery_empty_project",
    "builder_existing_project",
];

#[test]
fn discovery_fixtures_are_deterministic_and_read_only() {
    for name in DISCOVERY_FIXTURES {
        let root = fixture(name);
        assert!(
            !root.join("auditor.toml").exists(),
            "{name} não deve ter contrato"
        );
        let before = snapshot(&root);

        let (first_code, first_stdout, first_stderr) = discovery_json(&root);
        assert_eq!(first_code, 0, "{name}: {first_stderr}");
        assert!(first_stdout.contains("\"mode\": \"discovery\""), "{name}");
        assert!(first_stdout.contains("\"status\": \"missing\""), "{name}");
        assert!(first_stdout.contains("\"findings\": []"), "{name}");
        assert!(!first_stdout.contains("dep.not_allowed"), "{name}");
        assert!(!first_stdout.contains("dep.layer_not_allowed"), "{name}");

        let (second_code, second_stdout, second_stderr) = discovery_json(&root);
        assert_eq!(second_code, 0, "{name}: {second_stderr}");
        assert_eq!(
            first_stdout, second_stdout,
            "{name} deve ser determinística"
        );
        assert_eq!(before, snapshot(&root), "{name} não pode ser modificada");
    }
}

#[test]
fn language_specific_fixtures_expose_the_expected_extensions() {
    let cases = [
        ("discovery_rust_no_contract", "\"extension\": \"rs\""),
        ("discovery_ts_no_contract", "\"extension\": \"ts\""),
        ("discovery_python_no_contract", "\"extension\": \"py\""),
    ];

    for (name, extension) in cases {
        let (code, stdout, stderr) = discovery_json(&fixture(name));
        assert_eq!(code, 0, "{name}: {stderr}");
        assert!(
            stdout.contains(extension),
            "{name} deve mostrar {extension}"
        );
    }
}

#[test]
fn workspace_and_ambiguous_fixtures_expose_evidence_without_verdicts() {
    let workspace = discovery_json(&fixture("discovery_workspace"));
    assert_eq!(workspace.0, 0, "{}", workspace.2);
    assert!(workspace.1.contains("\"packages\""), "pacotes do workspace");
    assert!(workspace.1.contains("\"observed_dependencies\""));

    let ambiguous = discovery_json(&fixture("discovery_ambiguous"));
    assert_eq!(ambiguous.0, 0, "{}", ambiguous.2);
    assert!(ambiguous.1.contains("\"ambiguities\""));
    assert!(ambiguous.1.contains("\"needs_confirmation\": true"));
}

#[test]
fn empty_project_discovery_has_no_false_findings() {
    let (code, stdout, stderr) = discovery_json(&fixture("discovery_empty_project"));
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("\"findings\": []"));
    assert!(stdout.contains("\"candidate_modules\": []"));
    assert!(!stdout.contains("Falha operacional"));
}

#[test]
fn v1_and_v2_contract_fixtures_are_not_rewritten_by_audit() {
    let cases = [
        ("builder_contract_v1", "\"schema_version\": 1", 0),
        ("builder_contract_v2", "\"schema_version\": 2", 1),
    ];

    for (name, version, expected_code) in cases {
        let root = fixture(name);
        let contract_path = root.join("auditor.toml");
        let before = std::fs::read(&contract_path).expect("contrato da fixture");

        let (code, stdout, stderr) =
            run(&[arg("audit"), root.as_path(), arg("--format"), arg("json")]);
        assert_eq!(code, expected_code, "{name}: {stderr}");
        assert!(stdout.contains("\"mode\": \"compliance\""), "{name}");
        assert!(stdout.contains(version), "{name}");
        assert_eq!(
            std::fs::read(&contract_path).expect("contrato preservado"),
            before,
            "{name} não pode ser reescrito"
        );
    }
}
