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
}

fn main() {
    let args = Args::parse();

    if let Some(Commands::Init { path }) = args.command {
        let directory = path.unwrap_or_else(|| PathBuf::from("."));
        if let Err(error) = init_contract(&directory) {
            eprintln!("Falha ao criar modelo de contrato: {error}");
            std::process::exit(1);
        }
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

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;

    use super::Args;
    use super::Commands;

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
}
