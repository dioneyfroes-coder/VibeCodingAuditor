//! Histórico local de auditorias em SQLite.
//!
//! O banco fica no sistema de arquivos do usuário e pode ser reaberto depois
//! do fechamento da aplicação. Somente metadados, resumos, achados e linhas de
//! evidência são gravados; o conteúdo integral dos arquivos auditados nunca é
//! copiado para o histórico.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::findings::{Finding, Severity};
use crate::report::{AuditReport, ReportSummary, finding_location};

/// Caminho padrão do banco, relativo ao diretório de trabalho da CLI.
pub const DEFAULT_DATABASE: &str = "data/auditor.db";

/// Versão do schema de histórico suportada por esta versão do núcleo.
pub const SCHEMA_VERSION: i64 = 1;

/// Colunas persistidas de uma execução, na ordem usada nas consultas.
const RUN_COLUMNS: &str = "id, recorded_at_unix, project_name, project_root, \
     schema_version, files_included, files_skipped, modules, packages, edges, \
     findings, errors, warnings, infos, needs_review";

/// Migração inicial: tabelas de execuções, achados, limitações e schema.
const MIGRATION_V1: &str = "
CREATE TABLE audit_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recorded_at_unix INTEGER NOT NULL,
    project_name TEXT NOT NULL,
    project_root TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    files_included INTEGER NOT NULL,
    files_skipped INTEGER NOT NULL,
    modules INTEGER NOT NULL,
    packages INTEGER NOT NULL,
    edges INTEGER NOT NULL,
    findings INTEGER NOT NULL,
    errors INTEGER NOT NULL,
    warnings INTEGER NOT NULL,
    infos INTEGER NOT NULL,
    needs_review INTEGER NOT NULL
);

CREATE TABLE audit_findings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES audit_runs(id) ON DELETE CASCADE,
    finding_id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    evidence TEXT,
    expected TEXT NOT NULL,
    suggestion TEXT NOT NULL,
    confidence TEXT,
    UNIQUE (run_id, finding_id)
);

CREATE TABLE audit_limitations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES audit_runs(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    UNIQUE (run_id, message)
);

CREATE INDEX audit_findings_by_run ON audit_findings(run_id);
CREATE INDEX audit_findings_by_rule ON audit_findings(rule_id);
CREATE INDEX audit_runs_by_project ON audit_runs(project_name);
";

/// Falha operacional ao abrir, migrar, gravar ou consultar o histórico.
#[derive(Debug)]
pub enum HistoryError {
    /// Não foi possível criar o diretório que conterá o banco.
    Directory {
        /// Diretório que não pôde ser criado.
        path: PathBuf,
        /// Erro de E/S original.
        source: std::io::Error,
    },
    /// Falha em uma operação SQLite.
    Database {
        /// Caminho do banco envolvido.
        path: PathBuf,
        /// Erro original do SQLite.
        source: rusqlite::Error,
    },
    /// Não foi possível canonicalizar a raiz informada.
    Root {
        /// Raiz informada pelo usuário.
        path: PathBuf,
        /// Erro de E/S original.
        source: std::io::Error,
    },
    /// O banco foi escrito por versão mais recente do auditor.
    UnsupportedSchema {
        /// Versão encontrada no banco.
        found: i64,
        /// Versão suportada por este binário.
        supported: i64,
    },
    /// A execução solicitada não existe.
    NotFound {
        /// Identificador solicitado.
        id: i64,
    },
    /// O relógio do sistema está anterior ao epoch Unix.
    Clock(std::time::SystemTimeError),
}

impl fmt::Display for HistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory { path, source } => {
                write!(
                    formatter,
                    "não foi possível criar o diretório {}: {source}",
                    path.display()
                )
            }
            Self::Database { path, source } => {
                write!(formatter, "erro no banco {}: {source}", path.display())
            }
            Self::Root { path, source } => {
                write!(formatter, "raiz inacessível {}: {source}", path.display())
            }
            Self::UnsupportedSchema { found, supported } => write!(
                formatter,
                "schema de histórico {found} não é suportado; este auditor aceita apenas {supported}"
            ),
            Self::NotFound { id } => write!(formatter, "execução {id} não encontrada"),
            Self::Clock(source) => write!(formatter, "relógio do sistema inválido: {source}"),
        }
    }
}

impl Error for HistoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Directory { source, .. } | Self::Root { source, .. } => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::Clock(source) => Some(source),
            Self::UnsupportedSchema { .. } | Self::NotFound { .. } => None,
        }
    }
}

/// Resumo de uma execução gravada no histórico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunSummary {
    /// Identificador local da execução.
    pub id: i64,
    /// Instante da gravação em segundos desde o epoch Unix.
    pub recorded_at_unix: i64,
    /// Instante da gravação em UTC, no formato RFC 3339.
    pub recorded_at: String,
    /// Nome do projeto conforme o contrato.
    pub project_name: String,
    /// Raiz canonicalizada do projeto auditado.
    pub project_root: String,
    /// Versão do schema do contrato usada na execução.
    pub schema_version: u32,
    /// Contagens da execução.
    pub summary: ReportSummary,
}

/// Execução gravada com seus achados e limitações.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunDetail {
    /// Cabeçalho da execução.
    pub run: RunSummary,
    /// Achados gravados, ordenados por identificador.
    pub findings: Vec<Finding>,
    /// Limitações registradas, ordenadas por texto.
    pub limitations: Vec<String>,
}

/// Diferença entre duas execuções, calculada pelo `id` dos achados.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionComparison {
    /// Execução usada como linha de base.
    pub before_id: i64,
    /// Execução comparada com a linha de base.
    pub after_id: i64,
    /// Achados presentes apenas na execução posterior.
    pub new_findings: Vec<Finding>,
    /// Achados presentes apenas na execução anterior.
    pub resolved_findings: Vec<Finding>,
    /// Achados presentes nas duas execuções, com os dados da posterior.
    pub persistent_findings: Vec<Finding>,
}

/// Conexão com o banco de histórico local.
pub struct History {
    path: PathBuf,
    connection: Connection,
}

impl History {
    /// Abre o banco, cria o diretório necessário e aplica migrações pendentes.
    pub fn open(path: &Path) -> Result<Self, HistoryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| HistoryError::Directory {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let connection = Connection::open(path).map_err(|source| HistoryError::Database {
            path: path.to_path_buf(),
            source,
        })?;
        connection
            .execute("PRAGMA foreign_keys = ON", [])
            .map_err(|source| HistoryError::Database {
                path: path.to_path_buf(),
                source,
            })?;

        let mut history = Self {
            path: path.to_path_buf(),
            connection,
        };
        history.migrate()?;
        Ok(history)
    }

    /// Caminho do banco associado a esta conexão.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Executa a auditoria e grava o resultado com a hora atual.
    pub fn record(&mut self, root: &Path, report: &AuditReport) -> Result<i64, HistoryError> {
        let recorded_at_unix = i64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(HistoryError::Clock)?
                .as_secs(),
        )
        .unwrap_or(i64::MAX);
        self.record_at(root, report, recorded_at_unix)
    }

    /// Lista execuções da mais recente para a mais antiga.
    pub fn list(
        &self,
        project_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RunSummary>, HistoryError> {
        let condition = if project_name.is_some() {
            " WHERE project_name = ?1"
        } else {
            ""
        };
        let sql =
            format!("SELECT {RUN_COLUMNS} FROM audit_runs{condition} ORDER BY id DESC LIMIT ?2");
        let limit_value = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut statement = self
            .connection
            .prepare(&sql)
            .map_err(|source| database_error(&self.path, source))?;
        let rows = statement
            .query_map(params![project_name, limit_value], |row| {
                Self::read_run(row)
            })
            .map_err(|source| database_error(&self.path, source))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|source| database_error(&self.path, source))
    }

    /// Carrega uma execução completa pelo identificador.
    pub fn get(&self, id: i64) -> Result<RunDetail, HistoryError> {
        let run = {
            let mut statement = self
                .connection
                .prepare(&format!(
                    "SELECT {RUN_COLUMNS} FROM audit_runs WHERE id = ?1"
                ))
                .map_err(|source| database_error(&self.path, source))?;
            statement
                .query_row([id], Self::read_run)
                .optional()
                .map_err(|source| database_error(&self.path, source))?
                .ok_or(HistoryError::NotFound { id })?
        };

        Ok(RunDetail {
            run,
            findings: self.findings_for(id)?,
            limitations: self.limitations_for(id)?,
        })
    }

    /// Compara duas execuções e classifica os achados como novos, resolvidos ou persistentes.
    pub fn compare(
        &self,
        before_id: i64,
        after_id: i64,
    ) -> Result<ExecutionComparison, HistoryError> {
        let before = self.get(before_id)?;
        let after = self.get(after_id)?;
        let before_ids: BTreeSet<&str> = before
            .findings
            .iter()
            .map(|finding| finding.id.as_str())
            .collect();
        let after_ids: BTreeSet<&str> = after
            .findings
            .iter()
            .map(|finding| finding.id.as_str())
            .collect();

        Ok(ExecutionComparison {
            before_id,
            after_id,
            new_findings: after
                .findings
                .iter()
                .filter(|finding| !before_ids.contains(finding.id.as_str()))
                .cloned()
                .collect(),
            resolved_findings: before
                .findings
                .iter()
                .filter(|finding| !after_ids.contains(finding.id.as_str()))
                .cloned()
                .collect(),
            persistent_findings: after
                .findings
                .iter()
                .filter(|finding| before_ids.contains(finding.id.as_str()))
                .cloned()
                .collect(),
        })
    }

    /// Grava uma execução com instante explícito, usado também nos testes.
    fn record_at(
        &mut self,
        root: &Path,
        report: &AuditReport,
        recorded_at_unix: i64,
    ) -> Result<i64, HistoryError> {
        let canonical_root = fs::canonicalize(root).map_err(|source| HistoryError::Root {
            path: root.to_path_buf(),
            source,
        })?;
        let database_path = self.path.clone();

        let transaction = self
            .connection
            .transaction()
            .map_err(|source| database_error(&database_path, source))?;
        transaction
            .execute(
                "INSERT INTO audit_runs (
                    recorded_at_unix, project_name, project_root, schema_version,
                    files_included, files_skipped, modules, packages, edges,
                    findings, errors, warnings, infos, needs_review
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    recorded_at_unix,
                    report.project,
                    display_root(&canonical_root),
                    i64::from(report.schema_version),
                    i64::try_from(report.summary.files_included).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.files_skipped).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.modules).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.packages).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.edges).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.findings).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.errors).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.warnings).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.infos).unwrap_or(i64::MAX),
                    i64::try_from(report.summary.needs_review).unwrap_or(i64::MAX),
                ],
            )
            .map_err(|source| database_error(&database_path, source))?;
        let run_id = transaction.last_insert_rowid();

        for finding in &report.findings {
            transaction
                .execute(
                    "INSERT INTO audit_findings (
                        run_id, finding_id, rule_id, severity, title, message,
                        path, line_start, line_end, evidence, expected,
                        suggestion, confidence
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        run_id,
                        finding.id,
                        finding.rule_id,
                        finding.severity.as_str(),
                        finding.title,
                        finding.message,
                        finding.path,
                        finding.line_start.map(|value| value as i64),
                        finding.line_end.map(|value| value as i64),
                        finding.evidence,
                        finding.expected,
                        finding.suggestion,
                        finding.confidence,
                    ],
                )
                .map_err(|source| database_error(&database_path, source))?;
        }

        for limitation in &report.limitations {
            transaction
                .execute(
                    "INSERT INTO audit_limitations (run_id, message) VALUES (?1, ?2)",
                    params![run_id, limitation],
                )
                .map_err(|source| database_error(&database_path, source))?;
        }

        transaction
            .commit()
            .map_err(|source| database_error(&database_path, source))?;
        Ok(run_id)
    }

    /// Aplica a migração inicial quando o banco ainda não tem schema.
    fn migrate(&mut self) -> Result<(), HistoryError> {
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at_unix INTEGER NOT NULL
                 )",
                [],
            )
            .map_err(|source| database_error(&self.path, source))?;

        let found = self
            .connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get::<_, Option<i64>>(0)
            })
            .optional()
            .map_err(|source| database_error(&self.path, source))?
            .flatten();

        match found {
            None => {
                let database_path = self.path.clone();
                let transaction = self
                    .connection
                    .transaction()
                    .map_err(|source| database_error(&database_path, source))?;
                transaction
                    .execute_batch(MIGRATION_V1)
                    .map_err(|source| database_error(&database_path, source))?;
                transaction
                    .execute(
                        "INSERT INTO schema_migrations (version, applied_at_unix) VALUES (?1, ?2)",
                        params![SCHEMA_VERSION, 0],
                    )
                    .map_err(|source| database_error(&database_path, source))?;
                transaction
                    .commit()
                    .map_err(|source| database_error(&database_path, source))
            }
            Some(version) if version == SCHEMA_VERSION => Ok(()),
            Some(version) => Err(HistoryError::UnsupportedSchema {
                found: version,
                supported: SCHEMA_VERSION,
            }),
        }
    }

    /// Carrega os achados de uma execução, ordenados por identificador.
    fn findings_for(&self, run_id: i64) -> Result<Vec<Finding>, HistoryError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT finding_id, rule_id, severity, title, message, path,
                        line_start, line_end, evidence, expected, suggestion, confidence
                 FROM audit_findings WHERE run_id = ?1 ORDER BY finding_id",
            )
            .map_err(|source| database_error(&self.path, source))?;
        let rows = statement
            .query_map([run_id], Self::read_finding)
            .map_err(|source| database_error(&self.path, source))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|source| database_error(&self.path, source))
    }

    /// Carrega as limitações de uma execução, ordenadas por texto.
    fn limitations_for(&self, run_id: i64) -> Result<Vec<String>, HistoryError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT message FROM audit_limitations
                 WHERE run_id = ?1 ORDER BY message",
            )
            .map_err(|source| database_error(&self.path, source))?;
        let rows = statement
            .query_map([run_id], |row| row.get(0))
            .map_err(|source| database_error(&self.path, source))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|source| database_error(&self.path, source))
    }

    /// Constrói um resumo a partir de uma linha da tabela `audit_runs`.
    fn read_run(row: &Row<'_>) -> rusqlite::Result<RunSummary> {
        let recorded_at_unix = row.get(1)?;
        Ok(RunSummary {
            id: row.get(0)?,
            recorded_at_unix,
            recorded_at: format_utc(recorded_at_unix),
            project_name: row.get(2)?,
            project_root: row.get(3)?,
            schema_version: read_u32(row, 4)?,
            summary: ReportSummary {
                files_included: read_usize(row, 5)?,
                files_skipped: read_usize(row, 6)?,
                modules: read_usize(row, 7)?,
                packages: read_usize(row, 8)?,
                edges: read_usize(row, 9)?,
                findings: read_usize(row, 10)?,
                errors: read_usize(row, 11)?,
                warnings: read_usize(row, 12)?,
                infos: read_usize(row, 13)?,
                needs_review: read_usize(row, 14)?,
            },
        })
    }

    /// Constrói um achado a partir de uma linha da tabela `audit_findings`.
    fn read_finding(row: &Row<'_>) -> rusqlite::Result<Finding> {
        let severity_text: String = row.get(2)?;
        let severity = parse_severity(&severity_text).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                Type::Text,
                Box::new(std::io::Error::other(format!(
                    "severidade desconhecida: {severity_text}"
                ))),
            )
        })?;

        Ok(Finding {
            id: row.get(0)?,
            rule_id: row.get(1)?,
            severity,
            title: row.get(3)?,
            message: row.get(4)?,
            path: row.get(5)?,
            line_start: read_optional_usize(row, 6)?,
            line_end: read_optional_usize(row, 7)?,
            evidence: row.get(8)?,
            expected: row.get(9)?,
            suggestion: row.get(10)?,
            confidence: row.get(11)?,
        })
    }
}

/// Cria um erro de banco que identifica o arquivo afetado.
fn database_error(path: &Path, source: rusqlite::Error) -> HistoryError {
    HistoryError::Database {
        path: path.to_path_buf(),
        source,
    }
}

/// Renderiza a listagem de execuções em texto estável.
pub fn render_run_list(runs: &[RunSummary]) -> String {
    if runs.is_empty() {
        return "Nenhuma execução gravada.".to_owned();
    }

    let mut lines = vec!["Histórico de auditorias:".to_owned()];
    for run in runs {
        lines.push(format!(
            "  #{} — {} — {} — {}",
            run.id, run.recorded_at, run.project_name, run.project_root
        ));
        lines.push(format!(
            "      schema v{} | {}",
            run.schema_version,
            summary_phrase(&run.summary)
        ));
    }
    lines.join("\n")
}

/// Renderiza uma execução completa em texto estável.
pub fn render_run_detail(detail: &RunDetail) -> String {
    let run = &detail.run;
    let mut lines = vec![
        format!("Execução #{}", run.id),
        format!("Data: {}", run.recorded_at),
        format!("Projeto: {}", run.project_name),
        format!("Raiz: {}", run.project_root),
        format!("Schema do contrato: v{}", run.schema_version),
        format!(
            "Arquivos: {} incluídos, {} não incluídos",
            run.summary.files_included, run.summary.files_skipped
        ),
        format!(
            "Módulos: {} | Pacotes Cargo: {} | Arestas: {}",
            run.summary.modules, run.summary.packages, run.summary.edges
        ),
        format!("Achados: {}", summary_phrase(&run.summary)),
    ];

    if detail.findings.is_empty() {
        lines.push("Nenhum achado.".to_owned());
    } else {
        lines.push("Achados:".to_owned());
        push_detailed_findings(&mut lines, &detail.findings);
    }

    if !detail.limitations.is_empty() {
        lines.push("Limitações:".to_owned());
        for limitation in &detail.limitations {
            lines.push(format!("  - {limitation}"));
        }
    }

    lines.join("\n")
}

/// Renderiza a diferença entre duas execuções em texto estável.
pub fn render_comparison(comparison: &ExecutionComparison) -> String {
    let mut lines = vec![format!(
        "Comparação: execução #{} -> #{}",
        comparison.before_id, comparison.after_id
    )];

    lines.push(format!(
        "Novos achados ({}):",
        comparison.new_findings.len()
    ));
    push_compact_findings(&mut lines, &comparison.new_findings);
    lines.push(format!(
        "Achados resolvidos ({}):",
        comparison.resolved_findings.len()
    ));
    push_compact_findings(&mut lines, &comparison.resolved_findings);
    lines.push(format!(
        "Achados persistentes ({}):",
        comparison.persistent_findings.len()
    ));
    push_compact_findings(&mut lines, &comparison.persistent_findings);

    lines.join("\n")
}

/// Resume as contagens de um relatório em uma frase determinística.
fn summary_phrase(summary: &ReportSummary) -> String {
    format!(
        "{} ({} erros, {} avisos, {} informações, {} revisões)",
        summary.findings, summary.errors, summary.warnings, summary.infos, summary.needs_review
    )
}

/// Acrescenta o cabeçalho comum de um achado às linhas de saída.
fn push_finding_header(lines: &mut Vec<String>, finding: &Finding) {
    lines.push(format!(
        "  [{}] {} — {}",
        finding.severity.label(),
        finding.rule_id,
        finding.title
    ));
    lines.push(format!("    id: {}", finding.id));
    lines.push(format!("    local: {}", finding_location(finding)));
}

/// Acrescenta achados com mensagem, evidência e sugestão.
fn push_detailed_findings(lines: &mut Vec<String>, findings: &[Finding]) {
    for finding in findings {
        push_finding_header(lines, finding);
        lines.push(format!("    mensagem: {}", finding.message));
        if let Some(evidence) = &finding.evidence {
            lines.push(format!("    evidência: {evidence}"));
        }
        lines.push(format!("    esperado: {}", finding.expected));
        lines.push(format!("    sugestão: {}", finding.suggestion));
    }
}

/// Acrescenta achados de forma compacta, sem repetir detalhes já gravados.
fn push_compact_findings(lines: &mut Vec<String>, findings: &[Finding]) {
    if findings.is_empty() {
        lines.push("  (nenhum)".to_owned());
        return;
    }

    for finding in findings {
        push_finding_header(lines, finding);
    }
}

/// Remove o prefixo de caminho estendido do Windows para exibição.
fn display_root(path: &Path) -> String {
    let text = path.to_string_lossy();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
}

/// Converte segundos desde o epoch Unix para timestamp UTC legível.
fn format_utc(unix_seconds: i64) -> String {
    let days = unix_seconds.div_euclid(86_400);
    let seconds_of_day = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Converte dias desde 01/01/1970 em ano, mês e dia do calendário gregoriano.
fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let shifted = days_since_epoch + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year, month, day)
}

/// Lê um número de linha opcional e não negativo do banco.
fn read_optional_usize(row: &Row<'_>, index: usize) -> rusqlite::Result<Option<usize>> {
    let Some(value) = row.get::<_, Option<i64>>(index)? else {
        return Ok(None);
    };
    usize::try_from(value)
        .map(Some)
        .map_err(|_| conversion_error(index, "número de linha negativo no histórico"))
}

/// Lê um contador não negativo do banco.
fn read_usize(row: &Row<'_>, index: usize) -> rusqlite::Result<usize> {
    let value: i64 = row.get(index)?;
    usize::try_from(value).map_err(|_| conversion_error(index, "contagem negativa no histórico"))
}

/// Lê uma versão de schema não negativa do banco.
fn read_u32(row: &Row<'_>, index: usize) -> rusqlite::Result<u32> {
    let value: i64 = row.get(index)?;
    u32::try_from(value)
        .map_err(|_| conversion_error(index, "schema_version inválido no histórico"))
}

/// Cria um erro de conversão que identifica a coluna afetada.
fn conversion_error(index: usize, message: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        Type::Integer,
        Box::new(std::io::Error::other(message)),
    )
}

/// Converte o valor persistido de severidade de volta para o modelo.
fn parse_severity(value: &str) -> Option<Severity> {
    match value {
        "error" => Some(Severity::Error),
        "warning" => Some(Severity::Warning),
        "info" => Some(Severity::Info),
        "needs_review" => Some(Severity::NeedsReview),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::audit_project;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DATABASE: AtomicU64 = AtomicU64::new(0);

    /// Banco temporário removido automaticamente ao final do teste.
    struct TestDatabase {
        path: PathBuf,
    }

    impl TestDatabase {
        /// Cria um caminho único no diretório temporário do sistema.
        fn new() -> Self {
            let sequence = NEXT_DATABASE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "vibe-coding-auditor-history-{}-{sequence}.db",
                std::process::id()
            ));
            Self { path }
        }

        /// Abre o banco com o histórico já migrado.
        fn open(&self) -> History {
            History::open(&self.path).expect("temporary database must open")
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    /// Raiz da fixture de dependências, relativa ao crate.
    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/projects/dependencies")
    }

    /// Relatório real da fixture, usado para gravar histórico nos testes.
    fn fixture_report() -> AuditReport {
        audit_project(&project_root()).expect("fixture audit must succeed")
    }

    #[test]
    fn windows_extended_prefix_is_hidden_for_display() {
        assert_eq!(
            display_root(Path::new(r"\\?\C:\projetos\app")),
            r"C:\projetos\app"
        );
        assert_eq!(display_root(Path::new("/tmp/app")), "/tmp/app");
    }

    #[test]
    fn utc_formatting_is_stable() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(1_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(format_utc(-1), "1969-12-31T23:59:59Z");
    }

    #[test]
    fn severity_values_round_trip() {
        for severity in [
            Severity::Error,
            Severity::Warning,
            Severity::Info,
            Severity::NeedsReview,
        ] {
            assert_eq!(parse_severity(severity.as_str()), Some(severity));
        }
        assert_eq!(parse_severity("desconhecida"), None);
    }

    #[test]
    fn recorded_history_survives_reopening() {
        let database = TestDatabase::new();
        let report = fixture_report();
        let expected_findings = report.findings.clone();
        let expected_limitations = report.limitations.clone();

        {
            let mut history = database.open();
            history
                .record(&project_root(), &report)
                .expect("record must succeed");
        }

        let history = database.open();
        let list = history.list(None, 10).expect("list must succeed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].project_name, "projeto-dependencias");
        assert_eq!(list[0].summary.findings, 5);

        let detail = history.get(list[0].id).expect("get must succeed");
        assert_eq!(detail.findings, expected_findings);
        assert_eq!(detail.limitations, expected_limitations);
        assert!(detail.run.project_root.ends_with("dependencies"));
    }

    #[test]
    fn list_filters_project_and_limits_results() {
        let database = TestDatabase::new();
        let report = fixture_report();
        let mut history = database.open();
        history
            .record(&project_root(), &report)
            .expect("first record must succeed");
        history
            .record(&project_root(), &report)
            .expect("second record must succeed");

        assert_eq!(history.list(None, 10).expect("list must succeed").len(), 2);
        assert_eq!(
            history
                .list(Some("projeto-dependencias"), 10)
                .expect("filtered list must succeed")
                .len(),
            2
        );
        assert_eq!(
            history
                .list(Some("outro-projeto"), 10)
                .expect("filtered list must succeed")
                .len(),
            0
        );
        assert_eq!(
            history
                .list(None, 1)
                .expect("limited list must succeed")
                .len(),
            1
        );
    }

    #[test]
    fn comparison_classifies_findings_by_stable_id() {
        let database = TestDatabase::new();
        let report = fixture_report();
        let mut history = database.open();
        let before_id = history
            .record(&project_root(), &report)
            .expect("baseline record must succeed");

        let mut changed = report.clone();
        let resolved = changed.findings.remove(0);
        let mut new_finding = resolved.clone();
        new_finding.id = "dep.not_allowed:novo:destino:src/lib.rs:9".to_owned();
        changed.findings.push(new_finding.clone());
        let after_id = history
            .record_at(&project_root(), &changed, 1_000_000_000)
            .expect("changed record must succeed");

        let comparison = history
            .compare(before_id, after_id)
            .expect("comparison must succeed");
        assert_eq!(comparison.before_id, before_id);
        assert_eq!(comparison.after_id, after_id);
        assert_eq!(comparison.new_findings, [new_finding]);
        assert_eq!(comparison.resolved_findings, [resolved]);
        assert_eq!(comparison.persistent_findings.len(), 4);
    }

    #[test]
    fn missing_execution_is_reported() {
        let database = TestDatabase::new();
        let history = database.open();
        let error = history.get(999).expect_err("missing execution must fail");

        assert!(matches!(error, HistoryError::NotFound { id: 999 }));
        assert!(error.to_string().contains("999"));
    }

    #[test]
    fn newer_schema_is_rejected_without_destructive_changes() {
        let database = TestDatabase::new();
        drop(database.open());
        let connection = Connection::open(&database.path).expect("database must exist");
        connection
            .execute(
                "UPDATE schema_migrations SET version = 2 WHERE version = 1",
                [],
            )
            .expect("schema version must be changed");
        drop(connection);

        let error = History::open(&database.path)
            .err()
            .expect("newer schema must be rejected");
        assert!(matches!(
            error,
            HistoryError::UnsupportedSchema {
                found: 2,
                supported: 1
            }
        ));
    }

    #[test]
    fn rendered_run_list_is_explicit_and_stable() {
        assert_eq!(render_run_list(&[]), "Nenhuma execução gravada.");

        let database = TestDatabase::new();
        let mut history = database.open();
        history
            .record(&project_root(), &fixture_report())
            .expect("record must succeed");
        let runs = history.list(None, 10).expect("list must succeed");
        let output = render_run_list(&runs);

        assert!(output.contains("Histórico de auditorias:"));
        assert!(output.contains("projeto-dependencias"));
        assert!(output.contains("5 (2 erros, 1 avisos, 0 informações, 2 revisões)"));
        assert_eq!(output, render_run_list(&runs));
    }

    #[test]
    fn rendered_run_detail_keeps_evidence_and_limitations() {
        let database = TestDatabase::new();
        let mut history = database.open();
        let id = history
            .record(&project_root(), &fixture_report())
            .expect("record must succeed");
        let detail = history.get(id).expect("get must succeed");
        let output = render_run_detail(&detail);

        assert!(output.contains(&format!("Execução #{id}")));
        assert!(output.contains("[erro] dep.not_allowed"));
        assert!(output.contains("evidência:"));
        assert!(output.contains("Limitações:"));
    }

    #[test]
    fn rendered_comparison_groups_each_category() {
        let database = TestDatabase::new();
        let mut history = database.open();
        let before_id = history
            .record(&project_root(), &fixture_report())
            .expect("baseline record must succeed");
        let after_id = history
            .record(&project_root(), &fixture_report())
            .expect("second record must succeed");
        let comparison = history
            .compare(before_id, after_id)
            .expect("comparison must succeed");
        let output = render_comparison(&comparison);

        assert!(output.contains(&format!("Comparação: execução #{before_id} -> #{after_id}")));
        assert!(output.contains("Novos achados (0):\n  (nenhum)"));
        assert!(output.contains("Achados resolvidos (0):\n  (nenhum)"));
        assert!(output.contains("Achados persistentes (5):"));
    }
}
