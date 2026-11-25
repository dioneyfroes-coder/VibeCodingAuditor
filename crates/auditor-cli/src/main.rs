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
    /// Analisa dependências Rust e mostra o grafo de módulos do contrato.
    Analyze {
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
        Some(Commands::Analyze { root }) => {
            if let Err(error) = analyze_project(&root) {
                eprintln!("Falha na análise: {error}");
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

/// Carrega o contrato, o inventário e apresenta o grafo de dependências.
fn analyze_project(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let contract = auditor_core::contract::load_contract(&root.join("auditor.toml"))?;
    let inventory = auditor_core::discovery::discover(root, &contract)?;
    let analysis = auditor_core::analysis::analyze(root, &contract, &inventory)?;

    println!("Módulos: {}", analysis.modules.join(", "));
    if let Some(workspace) = &analysis.workspace {
        let packages: Vec<&str> = workspace
            .packages
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        println!("Pacotes Cargo: {}", packages.join(", "));
    }

    println!("Arestas: {}", analysis.edges.len());
    for edge in &analysis.edges {
        match edge.line {
            Some(line) => println!(
                "  {} -> {} [{}] {}:{}",
                edge.from,
                edge.to,
                dependency_kind_label(edge.kind),
                edge.path,
                line
            ),
            None => println!(
                "  {} -> {} [{}] {}",
                edge.from,
                edge.to,
                dependency_kind_label(edge.kind),
                edge.path
            ),
        }
        if let Some(evidence) = &edge.evidence {
            println!("      evidência: {evidence}");
        }
    }

    println!("Revisão necessária: {}", analysis.reviews.len());
    for review in &analysis.reviews {
        match review.line {
            Some(line) => println!("  {}:{} — {}", review.path, line, review.detail),
            None => println!("  {} — {}", review.path, review.detail),
        }
    }

    println!("Declarações mod: {}", analysis.module_declarations.len());
    println!("Limitações: {}", analysis.limitations.len());
    for limitation in &analysis.limitations {
        println!("  - {limitation}");
    }

    Ok(())
}

/// Texto estável usado na saída do terminal para cada tipo de aresta.
fn dependency_kind_label(kind: auditor_core::analysis::DependencyKind) -> &'static str {
    match kind {
        auditor_core::analysis::DependencyKind::Allowed => "permitida",
        auditor_core::analysis::DependencyKind::Forbidden => "proibida",
        auditor_core::analysis::DependencyKind::Unresolved => "não resolvida",
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;

    use super::Args;
    use super::Commands;
    use super::Path;
    use super::dependency_kind_label;
    use auditor_core::analysis::DependencyKind;

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

    #[test]
    fn analyze_subcommand_requires_explicit_root() {
        let args = Args::try_parse_from(["vibe-coding-auditor", "analyze", "projeto"])
            .expect("analyze must parse");
        assert!(
            matches!(args.command, Some(Commands::Analyze { root }) if root.as_path() == Path::new("projeto"))
        );

        let error = Args::try_parse_from(["vibe-coding-auditor", "analyze"])
            .expect_err("analyze must require a root");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn dependency_kind_labels_are_stable() {
        assert_eq!(dependency_kind_label(DependencyKind::Allowed), "permitida");
        assert_eq!(dependency_kind_label(DependencyKind::Forbidden), "proibida");
        assert_eq!(
            dependency_kind_label(DependencyKind::Unresolved),
            "não resolvida"
        );
    }
}
