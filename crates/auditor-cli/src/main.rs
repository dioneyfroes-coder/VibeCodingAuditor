use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

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
    /// Aplica as regras e gera um relatório de auditoria.
    Audit {
        /// Diretório raiz do projeto a ser auditado.
        root: PathBuf,
        /// Formato do relatório.
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
        /// Arquivo onde gravar o relatório; sem este argumento, imprime no terminal.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Arquivo com saída de `git diff`; limita achados a arquivos e linhas alterados.
        #[arg(long, value_name = "ARQUIVO")]
        diff: Option<PathBuf>,
    },
    /// Registra, consulta e compara auditorias no banco local.
    History {
        /// Caminho do banco de histórico.
        #[arg(long, default_value = auditor_core::history::DEFAULT_DATABASE, global = true)]
        database: PathBuf,
        /// Operação de histórico desejada.
        #[command(subcommand)]
        action: HistoryCommands,
    },
}

/// Operações disponíveis no subcomando `history`.
#[derive(Subcommand, Debug)]
enum HistoryCommands {
    /// Executa a auditoria e grava o resultado no histórico.
    Record {
        /// Diretório raiz do projeto a ser auditado.
        root: PathBuf,
    },
    /// Lista execuções gravadas, da mais recente para a mais antiga.
    List {
        /// Filtra por nome de projeto do contrato.
        #[arg(long)]
        project: Option<String>,
        /// Número máximo de execuções mostradas.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Mostra uma execução gravada com achados e limitações.
    Show {
        /// Identificador da execução.
        id: i64,
    },
    /// Compara duas execuções e separa achados novos, resolvidos e persistentes.
    Compare {
        /// Identificador da execução usada como linha de base.
        before: i64,
        /// Identificador da execução comparada com a linha de base.
        after: i64,
    },
}

/// Formato de saída aceito pelo subcomando `audit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Texto legível no terminal.
    Text,
    /// JSON estruturado para consumo por outras ferramentas.
    Json,
    /// Markdown para revisão humana.
    Markdown,
}

impl From<OutputFormat> for auditor_core::report::ReportFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Text => Self::Text,
            OutputFormat::Json => Self::Json,
            OutputFormat::Markdown => Self::Markdown,
        }
    }
}

fn main() {
    let exit_code = match dispatch_command(Args::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Falha operacional: {error}");
            2
        }
    };
    std::process::exit(i32::from(exit_code));
}

/// Despacha o comando selecionado e devolve o código de saída.
fn dispatch_command(args: Args) -> Result<u8, Box<dyn std::error::Error>> {
    match args.command {
        Some(Commands::Init { path }) => {
            let directory = path.unwrap_or_else(|| PathBuf::from("."));
            init_contract(&directory)?;
            Ok(0)
        }
        Some(Commands::Scan { root }) => {
            scan_project(&root)?;
            Ok(0)
        }
        Some(Commands::Analyze { root }) => {
            analyze_project(&root)?;
            Ok(0)
        }
        Some(Commands::Audit {
            root,
            format,
            output,
            diff,
        }) => audit_command(&root, format, output.as_deref(), diff.as_deref()),
        Some(Commands::History { database, action }) => history_command(&database, action),
        None => Ok(0),
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
        println!(
            "Dependências externas declaradas: {}",
            workspace.declared_dependencies.join(", ")
        );
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
    println!(
        "Cobertura do código: {} de {} arquivo(s) analisados",
        analysis.coverage.code_files_analyzed, analysis.coverage.code_files_included
    );
    if !analysis.coverage.unsupported_code_extensions.is_empty() {
        println!(
            "  sem analisador: {}",
            auditor_core::analysis::describe_unsupported_code_extensions(
                &analysis.coverage.unsupported_code_extensions
            )
        );
    }
    println!("Limitações: {}", analysis.limitations.len());
    for limitation in &analysis.limitations {
        println!("  - {limitation}");
    }

    Ok(())
}

/// Executa a auditoria, renderiza o relatório e o envia ao destino escolhido.
fn audit_command(
    root: &Path,
    format: OutputFormat,
    output: Option<&Path>,
    diff: Option<&Path>,
) -> Result<u8, Box<dyn std::error::Error>> {
    let report = auditor_core::audit::audit_project(root)?;
    let report = match diff {
        Some(path) => {
            let patch = fs::read_to_string(path)?;
            let changes = auditor_core::diff::parse_unified_diff(&patch)?;
            auditor_core::diff::scope_report(&report, &changes)
        }
        None => report,
    };
    let content = report.render(format.into())?;

    match output {
        Some(path) => {
            fs::write(path, content)?;
            println!("Relatório gravado em {}", path.display());
        }
        None => println!("{content}"),
    }

    Ok(audit_exit_code(&report))
}

/// Código `0` para execução sem achados e `1` quando há achados.
fn audit_exit_code(report: &auditor_core::report::AuditReport) -> u8 {
    if report.has_findings() { 1 } else { 0 }
}

/// Executa uma operação de histórico e devolve o código de saída.
fn history_command(
    database: &Path,
    action: HistoryCommands,
) -> Result<u8, Box<dyn std::error::Error>> {
    use auditor_core::history::{self, History};

    match action {
        HistoryCommands::Record { root } => {
            let report = auditor_core::audit::audit_project(&root)?;
            let mut store = History::open(database)?;
            let id = store.record(&root, &report)?;
            println!("Execução #{id} gravada em {}", store.path().display());
            let detail = store.get(id)?;
            println!("{}", history::render_run_detail(&detail));
            Ok(audit_exit_code(&report))
        }
        HistoryCommands::List { project, limit } => {
            let store = History::open(database)?;
            let runs = store.list(project.as_deref(), limit)?;
            println!("{}", history::render_run_list(&runs));
            Ok(0)
        }
        HistoryCommands::Show { id } => {
            let store = History::open(database)?;
            let detail = store.get(id)?;
            println!("{}", history::render_run_detail(&detail));
            Ok(0)
        }
        HistoryCommands::Compare { before, after } => {
            let store = History::open(database)?;
            let comparison = store.compare(before, after)?;
            println!("{}", history::render_comparison(&comparison));
            Ok(0)
        }
    }
}

/// Texto estável usado na saída do terminal para cada tipo de aresta.
fn dependency_kind_label(kind: auditor_core::analysis::DependencyKind) -> &'static str {
    match kind {
        auditor_core::analysis::DependencyKind::Allowed => "permitida",
        auditor_core::analysis::DependencyKind::Forbidden => "proibida",
        auditor_core::analysis::DependencyKind::Unresolved => "não resolvida",
        auditor_core::analysis::DependencyKind::Observed => "observada",
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;

    use super::Args;
    use super::Commands;
    use super::HistoryCommands;
    use super::OutputFormat;
    use super::Path;
    use super::audit_exit_code;
    use super::dependency_kind_label;
    use auditor_core::analysis::{AnalysisCoverage, DependencyKind};
    use auditor_core::report::ReportFormat;
    use auditor_core::report::{AuditReport, ReportSummary, SkipReasonSummary};

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
    fn audit_subcommand_parses_format_and_output() {
        let args = Args::try_parse_from([
            "vibe-coding-auditor",
            "audit",
            "projeto",
            "--format",
            "json",
            "--output",
            "relatorio.json",
            "--diff",
            "alteracoes.patch",
        ])
        .expect("audit must parse");
        match args.command {
            Some(Commands::Audit {
                root,
                format,
                output,
                diff,
            }) => {
                assert_eq!(root.as_path(), Path::new("projeto"));
                assert_eq!(format, OutputFormat::Json);
                assert_eq!(output.as_deref(), Some(Path::new("relatorio.json")));
                assert_eq!(diff.as_deref(), Some(Path::new("alteracoes.patch")));
            }
            _ => panic!("audit command must be selected"),
        }

        let error = Args::try_parse_from(["vibe-coding-auditor", "audit"])
            .expect_err("audit must require a root");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);

        let error =
            Args::try_parse_from(["vibe-coding-auditor", "audit", "projeto", "--format", "xml"])
                .expect_err("unknown format must be rejected");
        assert_eq!(error.kind(), ErrorKind::InvalidValue);
    }

    #[test]
    fn history_subcommands_parse_database_and_arguments() {
        let args = Args::try_parse_from([
            "vibe-coding-auditor",
            "history",
            "--database",
            "historico.db",
            "record",
            "projeto",
        ])
        .expect("history record must parse");
        match args.command {
            Some(Commands::History { database, action }) => {
                assert_eq!(database.as_path(), Path::new("historico.db"));
                assert!(
                    matches!(action, HistoryCommands::Record { root } if root.as_path() == Path::new("projeto"))
                );
            }
            _ => panic!("history command must be selected"),
        }

        let args = Args::try_parse_from([
            "vibe-coding-auditor",
            "history",
            "compare",
            "1",
            "2",
            "--database",
            "outro.db",
        ])
        .expect("history compare must accept a global database option");
        match args.command {
            Some(Commands::History { database, action }) => {
                assert_eq!(database.as_path(), Path::new("outro.db"));
                assert!(
                    matches!(action, HistoryCommands::Compare { before, after } if before == 1 && after == 2)
                );
            }
            _ => panic!("history command must be selected"),
        }

        let error = Args::try_parse_from(["vibe-coding-auditor", "history"])
            .expect_err("history must require an operation");
        assert_eq!(
            error.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }

    #[test]
    fn output_formats_map_to_the_core_model() {
        assert_eq!(ReportFormat::from(OutputFormat::Text), ReportFormat::Text);
        assert_eq!(ReportFormat::from(OutputFormat::Json), ReportFormat::Json);
        assert_eq!(
            ReportFormat::from(OutputFormat::Markdown),
            ReportFormat::Markdown
        );
    }

    /// Relatório em memória com a quantidade informada de achados.
    fn report_with_findings(findings: usize) -> AuditReport {
        AuditReport {
            project: "projeto".to_owned(),
            schema_version: 1,
            summary: ReportSummary {
                files_included: 0,
                files_skipped: 0,
                modules: 0,
                packages: 0,
                edges: 0,
                findings,
                errors: findings,
                warnings: 0,
                infos: 0,
                needs_review: 0,
            },
            skip_summary: SkipReasonSummary::default(),
            findings: Vec::new(),
            limitations: Vec::new(),
            coverage: AnalysisCoverage::default(),
        }
    }

    #[test]
    fn audit_exit_codes_reflect_finding_presence() {
        assert_eq!(audit_exit_code(&report_with_findings(0)), 0);
        assert_eq!(audit_exit_code(&report_with_findings(1)), 1);
    }

    #[test]
    fn dependency_kind_labels_are_stable() {
        assert_eq!(dependency_kind_label(DependencyKind::Allowed), "permitida");
        assert_eq!(dependency_kind_label(DependencyKind::Forbidden), "proibida");
        assert_eq!(
            dependency_kind_label(DependencyKind::Unresolved),
            "não resolvida"
        );
        assert_eq!(dependency_kind_label(DependencyKind::Observed), "observada");
    }
}
