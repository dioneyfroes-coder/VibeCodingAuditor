use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

/// Auditor local de arquitetura de projetos.
#[derive(Parser, Debug)]
#[command(name = "vibe-coding-auditor", version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Cria um modelo inicial de `auditor.toml` no diretório informado.
    Init {
        /// Diretório onde o modelo será criado (padrão: diretório atual).
        path: Option<PathBuf>,
    },
    /// Inventaria arquivos de um projeto a partir do `auditor.toml` da raiz.
    Scan {
        /// Diretório raiz do projeto a ser auditado.
        root: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Init { path }) => {
            let directory = path.unwrap_or_else(|| PathBuf::from("."));
            if let Err(error) = init_contract(&directory) {
                eprintln!("Falha ao criar modelo de contrato: {error}");
                std::process::exit(1);
            }
        }
        Some(Commands::Scan { root }) => {
            if let Err(error) = scan_project(&root) {
                eprintln!("Falha na descoberta: {error}");
                std::process::exit(1);
            }
        }
        None => {}
    }
}

/// Escreve o modelo sem sobrescrever um contrato existente.
fn init_contract(directory: &Path) -> std::io::Result<()> {
    let target = directory.join("auditor.toml");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)?;
    file.write_all(auditor_core::contract::EXAMPLE_CONTRACT.as_bytes())?;

    println!("Modelo criado em {}", target.display());
    println!("Revise e confirme o contrato antes de tratá-lo como aprovado.");
    Ok(())
}

/// Carrega o contrato da raiz e apresenta o inventário resultante.
fn scan_project(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let contract = auditor_core::contract::load_contract(&root.join("auditor.toml"))?;
    let inventory = auditor_core::discovery::discover(root, &contract)?;

    println!("Raiz: {}", inventory.root.display());
    println!("Arquivos incluídos: {}", inventory.files.len());
    for file in &inventory.files {
        println!(
            "  {} ({} bytes, ext: {:?})",
            file.path, file.size, file.extension
        );
    }
    println!("Entradas não incluídas: {}", inventory.skipped.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;

    use super::Args;
    use super::Commands;
    use super::Path;

    #[test]
    fn help_flag_is_recognized() {
        let error = Args::try_parse_from(["vibe-coding-auditor", "--help"])
            .expect_err("--help must be handled by clap");
        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn version_flag_is_recognized() {
        let error = Args::try_parse_from(["vibe-coding-auditor", "--version"])
            .expect_err("--version must be handled by clap");
        assert_eq!(error.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn init_subcommand_is_recognized() {
        let args = Args::try_parse_from(["vibe-coding-auditor", "init", "projeto"])
            .expect("init must parse");
        assert!(matches!(args.command, Some(Commands::Init { .. })));
    }

    #[test]
    fn scan_subcommand_requires_explicit_root() {
        let args = Args::try_parse_from(["vibe-coding-auditor", "scan", "projeto"])
            .expect("scan must parse");
        assert!(
            matches!(args.command, Some(Commands::Scan { root }) if root.as_path() == Path::new("projeto"))
        );

        let error = Args::try_parse_from(["vibe-coding-auditor", "scan"])
            .expect_err("scan must require a root");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }
}
