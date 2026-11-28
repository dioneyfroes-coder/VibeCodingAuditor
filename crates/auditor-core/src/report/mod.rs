//! Geração de relatórios consistentes a partir dos achados da auditoria.
//!
//! O núcleo controla os três formatos para que CLI, desktop e futura extensão
//! mostrem os mesmos dados. A escrita em arquivo é responsabilidade do chamador.

use std::error::Error;
use std::fmt;

use serde::Serialize;

use crate::analysis::{Analysis, AnalysisCoverage, describe_unsupported_code_extensions};
use crate::audit::{ContractStatus, DiscoveryReport, ExecutionMode, execution_mode_label};
use crate::contract::Contract;
use crate::discovery::{Inventory, SkipReason, SkippedEntry};
use crate::findings::{Finding, Severity};
use crate::resolution::ExternalPackageRelation;

/// Formato de saída de um relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Texto legível no terminal.
    Text,
    /// JSON estruturado, estável para consumo por outras interfaces.
    Json,
    /// Markdown para revisão humana e versionamento.
    Markdown,
}

impl ReportFormat {
    /// Nome usado em argumentos de linha de comando e mensagens.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Markdown => "markdown",
        }
    }

    /// Extensão recomendada quando o relatório é gravado em arquivo.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Json => "json",
            Self::Markdown => "md",
        }
    }
}

impl fmt::Display for ReportFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Falha ao converter um relatório para o formato solicitado.
#[derive(Debug)]
pub enum ReportError {
    /// A serialização JSON falhou.
    Json(serde_json::Error),
}

impl fmt::Display for ReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(source) => write!(formatter, "falha ao gerar JSON: {source}"),
        }
    }
}

impl Error for ReportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(source) => Some(source),
        }
    }
}

/// Quantidade máxima de caminhos de exemplo mostrados por motivo.
const SKIP_EXAMPLE_LIMIT: usize = 3;

/// Resumo estruturado das entradas que ficaram de fora do inventário.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SkipReasonSummary {
    /// Total de entradas não incluídas.
    pub total: usize,
    /// Entradas deixadas de fora por escolha explícita de escopo.
    pub intentional: usize,
    /// Entradas que impediram a varredura de cobrir o projeto por completo.
    pub coverage_gaps: usize,
    /// Contagens por motivo, em ordem estável de apresentação.
    pub reasons: Vec<SkipReasonCount>,
}

impl SkipReasonSummary {
    /// Resume entradas já ordenadas pela descoberta, limitando os exemplos.
    pub fn from_entries(entries: &[SkippedEntry]) -> Self {
        let mut reasons = Vec::new();

        for reason in SkipReason::ALL {
            let paths: Vec<&str> = entries
                .iter()
                .filter(|entry| entry.reason == reason)
                .map(|entry| entry.path.as_str())
                .collect();
            if paths.is_empty() {
                continue;
            }

            reasons.push(SkipReasonCount {
                reason,
                label: reason.label().to_owned(),
                count: paths.len(),
                examples: paths
                    .iter()
                    .take(SKIP_EXAMPLE_LIMIT)
                    .map(|path| (*path).to_owned())
                    .collect(),
                additional: paths.len().saturating_sub(SKIP_EXAMPLE_LIMIT),
                reduces_coverage: reason.reduces_coverage(),
            });
        }

        let (intentional, coverage_gaps) = reasons.iter().fold((0, 0), |(chosen, gaps), entry| {
            if entry.reduces_coverage {
                (chosen, gaps + entry.count)
            } else {
                (chosen + entry.count, gaps)
            }
        });

        Self {
            total: entries.len(),
            intentional,
            coverage_gaps,
            reasons,
        }
    }
}

/// Contagem e exemplos limitados de um motivo de exclusão.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkipReasonCount {
    /// Identificador estável do motivo.
    pub reason: SkipReason,
    /// Rótulo legível produzido pelo núcleo.
    pub label: String,
    /// Quantidade de entradas com esse motivo.
    pub count: usize,
    /// Primeiros caminhos, em ordem determinística.
    pub examples: Vec<String>,
    /// Quantidade de caminhos omitidos dos exemplos.
    pub additional: usize,
    /// Indica se o motivo reduziu a cobertura da auditoria.
    pub reduces_coverage: bool,
}

/// Contagem determinística de um relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReportSummary {
    /// Arquivos incluídos na varredura.
    pub files_included: usize,
    /// Entradas não incluídas na varredura.
    pub files_skipped: usize,
    /// Módulos declarados no contrato.
    pub modules: usize,
    /// Pacotes Cargo encontrados no workspace.
    pub packages: usize,
    /// Arestas classificadas na análise.
    pub edges: usize,
    /// Total de achados.
    pub findings: usize,
    /// Achados de severidade `error`.
    pub errors: usize,
    /// Achados de severidade `warning`.
    pub warnings: usize,
    /// Achados de severidade `info`.
    pub infos: usize,
    /// Achados de severidade `needs_review`.
    pub needs_review: usize,
}

/// Relatório completo de uma execução de auditoria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditReport {
    /// Nome do projeto, conforme o contrato.
    pub project: String,
    /// Versão do schema do contrato.
    pub schema_version: u32,
    /// Contagens derivadas dos dados da execução.
    pub summary: ReportSummary,
    /// Motivos pelos quais entradas ficaram de fora da varredura.
    pub skip_summary: SkipReasonSummary,
    /// Achados ordenados por identificador.
    pub findings: Vec<Finding>,
    /// Limitações explícitas da análise, sem conclusões inventadas.
    pub limitations: Vec<String>,
    /// Cobertura efetiva da análise sobre o código incluído no inventário.
    pub coverage: AnalysisCoverage,
    /// Pacotes e workspaces observados, com estado, origem e evidência.
    pub external_packages: Vec<ExternalPackageRelation>,
    /// Modo explícito da execução, sempre `compliance` neste relatório.
    pub mode: ExecutionMode,
    /// Estado do contrato, sempre `present` neste relatório.
    pub contract: ContractStatus,
}

impl AuditReport {
    /// Monta o relatório a partir dos resultados já produzidos pelo núcleo.
    pub fn from_pipeline(
        contract: &Contract,
        inventory: &Inventory,
        analysis: &Analysis,
        findings: Vec<Finding>,
    ) -> Self {
        let (errors, warnings, infos, needs_review) = severity_counts(&findings);

        Self {
            project: contract.project.name.clone(),
            schema_version: contract.schema_version,
            summary: ReportSummary {
                files_included: inventory.files.len(),
                files_skipped: inventory.skipped.len(),
                modules: contract.architecture.modules.len(),
                packages: analysis
                    .workspace
                    .as_ref()
                    .map_or(0, |workspace| workspace.packages.len()),
                edges: analysis.edges.len(),
                findings: findings.len(),
                errors,
                warnings,
                infos,
                needs_review,
            },
            skip_summary: SkipReasonSummary::from_entries(&inventory.skipped),
            findings,
            limitations: analysis.limitations.clone(),
            coverage: analysis.coverage.clone(),
            external_packages: analysis.external_packages.clone(),
            mode: ExecutionMode::Compliance,
            contract: ContractStatus::Present {
                schema_version: contract.schema_version,
            },
        }
    }

    /// Cria um relatório com um subconjunto de achados e contagens coerentes.
    ///
    /// As contagens de escopo da execução são preservadas; a limitação informa
    /// que o relatório foi recortado.
    pub fn scoped_to(&self, findings: Vec<Finding>, limitation: String) -> Self {
        let (errors, warnings, infos, needs_review) = severity_counts(&findings);
        let mut limitations = self.limitations.clone();
        limitations.push(limitation);

        Self {
            project: self.project.clone(),
            schema_version: self.schema_version,
            summary: ReportSummary {
                files_included: self.summary.files_included,
                files_skipped: self.summary.files_skipped,
                modules: self.summary.modules,
                packages: self.summary.packages,
                edges: self.summary.edges,
                findings: findings.len(),
                errors,
                warnings,
                infos,
                needs_review,
            },
            skip_summary: self.skip_summary.clone(),
            findings,
            limitations,
            coverage: self.coverage.clone(),
            external_packages: self.external_packages.clone(),
            mode: self.mode,
            contract: self.contract.clone(),
        }
    }

    /// Indica se a execução produziu algum achado.
    pub fn has_findings(&self) -> bool {
        self.summary.findings > 0
    }

    /// Renderiza o relatório no formato solicitado.
    pub fn render(&self, format: ReportFormat) -> Result<String, ReportError> {
        match format {
            ReportFormat::Text => Ok(self.render_text()),
            ReportFormat::Json => self.render_json(),
            ReportFormat::Markdown => Ok(self.render_markdown()),
        }
    }

    /// Renderiza texto legível para o terminal.
    fn render_text(&self) -> String {
        let mut lines = vec![
            format!(
                "Auditoria: {} (schema v{})",
                self.project, self.schema_version
            ),
            execution_mode_label(self.mode, &self.contract),
            format!(
                "Arquivos: {} incluídos, {} não incluídos",
                self.summary.files_included, self.summary.files_skipped
            ),
            coverage_line(&self.coverage),
            format!(
                "Módulos: {} | Pacotes Cargo: {} | Arestas: {}",
                self.summary.modules, self.summary.packages, self.summary.edges
            ),
            format!(
                "Achados: {} ({} erros, {} avisos, {} informações, {} revisões)",
                self.summary.findings,
                self.summary.errors,
                self.summary.warnings,
                self.summary.infos,
                self.summary.needs_review
            ),
            String::new(),
        ];

        if self.skip_summary.total > 0 {
            let mut skip_lines = vec![skip_summary_headline(&self.skip_summary)];
            for reason in &self.skip_summary.reasons {
                skip_lines.push(format!("  - {}", skip_reason_description(reason)));
            }
            let insertion = lines.len() - 1;
            lines.splice(insertion..insertion, skip_lines);
        }

        if self.findings.is_empty() {
            lines.push("Nenhum achado.".to_owned());
        } else {
            lines.push("Achados:".to_owned());
            for finding in &self.findings {
                let location = finding_location(finding);
                lines.push(format!(
                    "  [{}] {} — {}",
                    finding.severity.label(),
                    finding.rule_id,
                    finding.title
                ));
                lines.push(format!("    id: {}", finding.id));
                lines.push(format!("    local: {location}"));
                lines.push(format!("    mensagem: {}", finding.message));
                if let Some(evidence) = &finding.evidence {
                    lines.push(format!("    evidência: {evidence}"));
                }
                lines.push(format!("    esperado: {}", finding.expected));
                lines.push(format!("    sugestão: {}", finding.suggestion));
            }
        }

        if !self.external_packages.is_empty() {
            lines.push(String::new());
            lines.push("Pacotes observados:".to_owned());
            for relation in &self.external_packages {
                let position = external_position(relation);
                lines.push(format!(
                    "  - {} [{}]{}",
                    relation.specifier,
                    relation.state.as_str(),
                    position
                ));
                lines.push(format!("      origem: {}", relation.origin));
                lines.push(format!("      evidência: {}", relation.evidence));
            }
        }

        if !self.limitations.is_empty() {
            lines.push(String::new());
            lines.push("Limitações:".to_owned());
            for limitation in &self.limitations {
                lines.push(format!("  - {limitation}"));
            }
        }

        lines.join("\n")
    }

    /// Serializa o relatório em JSON formatado.
    fn render_json(&self) -> Result<String, ReportError> {
        serde_json::to_string_pretty(self).map_err(ReportError::Json)
    }

    /// Renderiza o relatório em Markdown com uma seção por achado.
    fn render_markdown(&self) -> String {
        let mut lines = vec![
            format!("# Auditoria: {}", self.project),
            String::new(),
            format!("- {}", execution_mode_label(self.mode, &self.contract)),
            format!("- Schema: v{}", self.schema_version),
            format!("- Arquivos incluídos: {}", self.summary.files_included),
            format!("- Arquivos não incluídos: {}", self.summary.files_skipped),
            format!("- {}", coverage_line(&self.coverage)),
            format!("- Módulos: {}", self.summary.modules),
            format!("- Pacotes Cargo: {}", self.summary.packages),
            format!("- Arestas: {}", self.summary.edges),
            format!(
                "- Achados: {} ({} erros, {} avisos, {} informações, {} revisões)",
                self.summary.findings,
                self.summary.errors,
                self.summary.warnings,
                self.summary.infos,
                self.summary.needs_review
            ),
            String::new(),
            "## Achados".to_owned(),
        ];

        if self.skip_summary.total > 0 {
            let mut skip_lines = vec![format!("- {}", skip_summary_headline(&self.skip_summary))];
            for reason in &self.skip_summary.reasons {
                skip_lines.push(format!(
                    "  - {}",
                    markdown_text(&skip_reason_description(reason))
                ));
            }
            let insertion = lines.len() - 2;
            lines.splice(insertion..insertion, skip_lines);
        }

        if self.findings.is_empty() {
            lines.push(String::new());
            lines.push("Nenhum achado.".to_owned());
        }

        for finding in &self.findings {
            lines.push(String::new());
            lines.push(format!(
                "### [{}] {}",
                finding.severity.label(),
                markdown_text(&finding.title)
            ));
            lines.push(String::new());
            lines.push(format!("- **ID:** `{}`", finding.id));
            lines.push(format!("- **Regra:** `{}`", finding.rule_id));
            lines.push(format!("- **Local:** `{}`", finding_location(finding)));
            lines.push(format!(
                "- **Mensagem:** {}",
                markdown_text(&finding.message)
            ));
            if let Some(evidence) = &finding.evidence {
                lines.push(format!("- **Evidência:** `{}`", markdown_text(evidence)));
            }
            lines.push(format!(
                "- **Esperado:** {}",
                markdown_text(&finding.expected)
            ));
            lines.push(format!(
                "- **Sugestão:** {}",
                markdown_text(&finding.suggestion)
            ));
        }

        if !self.external_packages.is_empty() {
            lines.push(String::new());
            lines.push("## Pacotes observados".to_owned());
            for relation in &self.external_packages {
                lines.push(format!(
                    "- `{}` — {} {}",
                    markdown_text(&relation.specifier),
                    relation.state.as_str(),
                    external_position(relation)
                ));
                lines.push(format!("  - Origem: {}", markdown_text(&relation.origin)));
                lines.push(format!(
                    "  - Evidência: {}",
                    markdown_text(&relation.evidence)
                ));
            }
        }

        if !self.limitations.is_empty() {
            lines.push(String::new());
            lines.push("## Limitações".to_owned());
            for limitation in &self.limitations {
                lines.push(format!("- {}", markdown_text(limitation)));
            }
        }

        format!("{}\n", lines.join("\n"))
    }
}

impl DiscoveryReport {
    /// Renderiza a descoberta nos mesmos formatos da auditoria de conformidade.
    pub fn render(&self, format: ReportFormat) -> Result<String, ReportError> {
        match format {
            ReportFormat::Text => Ok(self.render_text()),
            ReportFormat::Json => self.render_json(),
            ReportFormat::Markdown => Ok(self.render_markdown()),
        }
    }

    /// Serializa a descoberta em JSON formatado, com modo e estado do contrato.
    fn render_json(&self) -> Result<String, ReportError> {
        serde_json::to_string_pretty(self).map_err(ReportError::Json)
    }

    /// Texto legível que deixa claro que não houve conformidade avaliada.
    fn render_text(&self) -> String {
        let mut lines = vec![
            execution_mode_label(self.mode, &self.contract),
            format!("Raiz: {}", self.inventory.root.display()),
            format!(
                "Arquivos: {} incluídos, {} não incluídos",
                self.inventory.files.len(),
                self.inventory.skipped.len()
            ),
            coverage_line(&self.analysis.coverage),
            format!(
                "Módulos candidatos: {} (todos exigem confirmação)",
                self.candidates.candidate_modules.len()
            ),
            format!(
                "Relações observadas: {}",
                self.candidates.observed_dependencies.len()
            ),
            format!(
                "Manifests reconhecidos: {}",
                self.candidates.manifests.len()
            ),
            format!("Ambiguidades: {}", self.candidates.ambiguities.len()),
            "Achados: nenhum (descoberta não emite achados de conformidade).".to_owned(),
            "Linguagens:".to_owned(),
        ];

        for language in &self.candidates.languages {
            let analyzer = language
                .analyzer
                .clone()
                .unwrap_or_else(|| "sem analisador".to_owned());
            lines.push(format!(
                "  - {}: {} arquivo(s), analisador: {analyzer}",
                language.extension, language.files
            ));
        }

        if self.skip_summary.total > 0 {
            lines.push(skip_summary_headline(&self.skip_summary));
            for reason in &self.skip_summary.reasons {
                lines.push(format!("  - {}", skip_reason_description(reason)));
            }
        }

        lines.push(String::new());
        lines.push("Módulos candidatos sugeridos:".to_owned());
        for candidate in &self.candidates.candidate_modules {
            lines.push(format!(
                "  - {} [{}] {}",
                candidate.name,
                candidate.category.as_str(),
                candidate.path
            ));
            for evidence in &candidate.evidence {
                lines.push(format!("      evidência: {evidence}"));
            }
        }

        if !self.candidates.manifests.is_empty() {
            lines.push(String::new());
            lines.push("Manifests observados:".to_owned());
            for manifest in &self.candidates.manifests {
                lines.push(format!(
                    "  - {} [{}] [{}]",
                    manifest.path,
                    manifest.ecosystem.as_str(),
                    manifest.status.as_str()
                ));
                for package in &manifest.local_packages {
                    let version = package.version.as_deref().unwrap_or("sem versão");
                    lines.push(format!(
                        "      pacote local: {} ({version}) em {}",
                        package.name, package.source_path
                    ));
                    for evidence in &package.evidence {
                        lines.push(format!("          evidência: {evidence}"));
                    }
                }
                for dependency in &manifest.dependencies {
                    let group = dependency.group.as_deref().unwrap_or("-");
                    let position = match dependency.line {
                        Some(line) => format!(" (linha {line})"),
                        None => String::new(),
                    };
                    lines.push(format!(
                        "      dependência: {} {} [{} / {} / grupo {}]{}",
                        dependency.name,
                        dependency.version,
                        dependency.scope.as_str(),
                        dependency.destination.as_str(),
                        group,
                        position
                    ));
                    lines.push(format!("          evidência: {}", dependency.evidence));
                }
                for limitation in &manifest.limitations {
                    lines.push(format!("      limitação: {limitation}"));
                }
            }
        }

        if !self.candidates.observed_dependencies.is_empty() {
            lines.push(String::new());
            lines.push("Relações observadas:".to_owned());
            for dependency in &self.candidates.observed_dependencies {
                let position = match dependency.line {
                    Some(line) => format!(" (linha {line})"),
                    None => String::new(),
                };
                lines.push(format!(
                    "  - {} -> {} [{}]{}",
                    dependency.source,
                    dependency.target,
                    dependency.resolution.as_str(),
                    position
                ));
                if let Some(evidence) = &dependency.evidence {
                    lines.push(format!("      evidência: {evidence}"));
                }
            }
        }

        if !self.analysis.external_packages.is_empty() {
            lines.push(String::new());
            lines.push("Pacotes observados:".to_owned());
            for relation in &self.analysis.external_packages {
                lines.push(format!(
                    "  - {} [{}] {}",
                    relation.specifier,
                    relation.state.as_str(),
                    external_position(relation)
                ));
                lines.push(format!("      origem: {}", relation.origin));
                lines.push(format!("      evidência: {}", relation.evidence));
            }
        }

        if !self.candidates.ambiguities.is_empty() {
            lines.push(String::new());
            lines.push("Ambiguidades:".to_owned());
            for ambiguity in &self.candidates.ambiguities {
                lines.push(format!("  - {} — {}", ambiguity.subject, ambiguity.detail));
                lines.push(format!("      opções: {}", ambiguity.options.join(", ")));
            }
        }

        if !self.candidates.limitations.is_empty() {
            lines.push(String::new());
            lines.push("Limitações:".to_owned());
            for limitation in &self.candidates.limitations {
                lines.push(format!("  - {limitation}"));
            }
        }

        lines.join("\n")
    }

    /// Markdown com seções separadas para cada tipo de evidência coletada.
    fn render_markdown(&self) -> String {
        let mut lines = vec![
            format!("# Descoberta: {}", self.inventory.root.display()),
            String::new(),
            format!("- {}", execution_mode_label(self.mode, &self.contract)),
            format!("- Arquivos incluídos: {}", self.inventory.files.len()),
            format!("- Arquivos não incluídos: {}", self.inventory.skipped.len()),
            format!("- {}", coverage_line(&self.analysis.coverage)),
            format!(
                "- Módulos candidatos: {} (todos exigem confirmação)",
                self.candidates.candidate_modules.len()
            ),
            format!(
                "- Relações observadas: {}",
                self.candidates.observed_dependencies.len()
            ),
            format!(
                "- Manifests reconhecidos: {}",
                self.candidates.manifests.len()
            ),
            format!("- Ambiguidades: {}", self.candidates.ambiguities.len()),
            "- Achados: nenhum (descoberta não emite achados de conformidade)".to_owned(),
            "- Linguagens:".to_owned(),
        ];

        for language in &self.candidates.languages {
            let analyzer = language
                .analyzer
                .clone()
                .unwrap_or_else(|| "sem analisador".to_owned());
            lines.push(format!(
                "  - {}: {} arquivo(s), analisador: {analyzer}",
                language.extension, language.files
            ));
        }

        if self.skip_summary.total > 0 {
            lines.push(format!("- {}", skip_summary_headline(&self.skip_summary)));
            for reason in &self.skip_summary.reasons {
                lines.push(format!(
                    "  - {}",
                    markdown_text(&skip_reason_description(reason))
                ));
            }
        }

        lines.push(String::new());
        lines.push("## Módulos candidatos".to_owned());
        for candidate in &self.candidates.candidate_modules {
            lines.push(String::new());
            lines.push(format!(
                "### {} [{}]",
                markdown_text(&candidate.name),
                candidate.category.as_str()
            ));
            lines.push(format!("- **Path:** `{}`", markdown_text(&candidate.path)));
            lines.push("- **Confirmação:** necessária".to_owned());
            for evidence in &candidate.evidence {
                lines.push(format!("- **Evidência:** {}", markdown_text(evidence)));
            }
        }

        if !self.candidates.manifests.is_empty() {
            lines.push(String::new());
            lines.push("## Manifests observados".to_owned());
            for manifest in &self.candidates.manifests {
                lines.push(String::new());
                lines.push(format!(
                    "### `{}` [{}] [{}]",
                    markdown_text(&manifest.path),
                    manifest.ecosystem.as_str(),
                    manifest.status.as_str()
                ));
                for package in &manifest.local_packages {
                    let version = package.version.as_deref().unwrap_or("sem versão");
                    lines.push(format!(
                        "- **Pacote local:** {} ({version}) em `{}`",
                        markdown_text(&package.name),
                        markdown_text(&package.source_path)
                    ));
                    for evidence in &package.evidence {
                        lines.push(format!("- **Evidência:** {}", markdown_text(evidence)));
                    }
                }
                for dependency in &manifest.dependencies {
                    let group = dependency.group.as_deref().unwrap_or("-");
                    lines.push(format!(
                        "- **Dependência:** `{}` = `{}` [{} / {} / grupo {}]",
                        markdown_text(&dependency.name),
                        markdown_text(&dependency.version),
                        dependency.scope.as_str(),
                        dependency.destination.as_str(),
                        markdown_text(group)
                    ));
                    if let Some(line) = dependency.line {
                        lines.push(format!("  - **Linha:** {line}"));
                    }
                    lines.push(format!(
                        "  - **Evidência:** `{}`",
                        markdown_text(&dependency.evidence)
                    ));
                }
                for limitation in &manifest.limitations {
                    lines.push(format!("- **Limitação:** {}", markdown_text(limitation)));
                }
            }
        }

        lines.push(String::new());
        lines.push("## Relações observadas".to_owned());
        for dependency in &self.candidates.observed_dependencies {
            lines.push(String::new());
            lines.push(format!(
                "- **Origem:** `{}`",
                markdown_text(&dependency.source)
            ));
            lines.push(format!(
                "- **Destino:** `{}`",
                markdown_text(&dependency.target)
            ));
            lines.push(format!(
                "- **Classificação:** {}",
                dependency.resolution.as_str()
            ));
            if let Some(line) = dependency.line {
                lines.push(format!("- **Linha:** {line}"));
            }
            if let Some(evidence) = &dependency.evidence {
                lines.push(format!("- **Evidência:** `{}`", markdown_text(evidence)));
            }
        }

        if !self.analysis.external_packages.is_empty() {
            lines.push(String::new());
            lines.push("## Pacotes observados".to_owned());
            for relation in &self.analysis.external_packages {
                lines.push(String::new());
                lines.push(format!(
                    "- **Pacote:** `{}`",
                    markdown_text(&relation.specifier)
                ));
                lines.push(format!("- **Estado:** {}", relation.state.as_str()));
                lines.push(format!("- **Local:** `{}`", external_position(relation)));
                lines.push(format!("- **Origem:** {}", markdown_text(&relation.origin)));
                lines.push(format!(
                    "- **Evidência:** `{}`",
                    markdown_text(&relation.evidence)
                ));
            }
        }

        if !self.candidates.ambiguities.is_empty() {
            lines.push(String::new());
            lines.push("## Ambiguidades".to_owned());
            for ambiguity in &self.candidates.ambiguities {
                lines.push(String::new());
                lines.push(format!("### {}", markdown_text(&ambiguity.subject)));
                lines.push(format!(
                    "- **Motivo:** {}",
                    markdown_text(&ambiguity.detail)
                ));
                lines.push(format!(
                    "- **Opções:** {}",
                    markdown_text(&ambiguity.options.join(", "))
                ));
            }
        }

        if !self.candidates.limitations.is_empty() {
            lines.push(String::new());
            lines.push(format!(
                "- Limitações: {}",
                self.candidates.limitations.len()
            ));
            lines.push(String::new());
            lines.push("## Limitações".to_owned());
            for limitation in &self.candidates.limitations {
                lines.push(format!("- {}", markdown_text(limitation)));
            }
        }

        format!("{}\n", lines.join("\n"))
    }
}

/// Resume a cobertura da análise sem inventar uma nota geral de qualidade.
fn coverage_line(coverage: &AnalysisCoverage) -> String {
    let mut line = format!(
        "Cobertura do código: {} de {} arquivo(s) analisados",
        coverage.code_files_analyzed, coverage.code_files_included
    );
    if coverage.partial {
        line.push_str(" (parcial)");
    }
    if !coverage.unsupported_code_extensions.is_empty() {
        line.push_str(&format!(
            "; sem analisador: {}",
            describe_unsupported_code_extensions(&coverage.unsupported_code_extensions)
        ));
    }
    line
}

/// Local curto e estável de uma relação com pacote observada.
fn external_position(relation: &ExternalPackageRelation) -> String {
    match relation.line {
        Some(line) => format!("({}:{})", relation.path, line),
        None => format!("({})", relation.path),
    }
}

/// Cabeçalho comum das saídas legíveis para o resumo de entradas ignoradas.
fn skip_summary_headline(summary: &SkipReasonSummary) -> String {
    format!(
        "Motivos dos não incluídos: {} ({} {}, {} {})",
        summary.total,
        summary.intentional,
        if summary.intentional == 1 {
            "intencional"
        } else {
            "intencionais"
        },
        summary.coverage_gaps,
        if summary.coverage_gaps == 1 {
            "reduz cobertura"
        } else {
            "reduzem cobertura"
        }
    )
}

/// Descreve um motivo sem listar todas as entradas correspondentes.
fn skip_reason_description(reason: &SkipReasonCount) -> String {
    let mut line = format!(
        "{}: {} ({})",
        reason.label,
        reason.count,
        if reason.reduces_coverage {
            "reduz cobertura"
        } else {
            "intencional"
        }
    );
    if !reason.examples.is_empty() {
        line.push_str(&format!("; exemplos: {}", reason.examples.join(", ")));
    }
    if reason.additional > 0 {
        line.push_str(&format!("; +{} outros", reason.additional));
    }
    line
}

/// Conta achados por severidade, usada em relatórios completos e recortados.
fn severity_counts(findings: &[Finding]) -> (usize, usize, usize, usize) {
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();
    let infos = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Info)
        .count();
    let needs_review = findings
        .iter()
        .filter(|finding| finding.severity == Severity::NeedsReview)
        .count();
    (errors, warnings, infos, needs_review)
}

/// Formata o caminho e as linhas de um achado para saída legível.
pub fn finding_location(finding: &Finding) -> String {
    match (finding.line_start, finding.line_end) {
        (Some(start), Some(end)) if start == end => format!("{}:{}", finding.path, start),
        (Some(start), Some(end)) => format!("{}:{}-{}", finding.path, start, end),
        (Some(start), None) => format!("{}:{}", finding.path, start),
        (None, Some(end)) => format!("{}:~{}", finding.path, end),
        (None, None) => finding.path.clone(),
    }
}

/// Escapa caracteres que alterariam a formatação Markdown.
fn markdown_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::analysis::{AnalysisCoverage, UnsupportedCodeExtension};
    use crate::discovery::{SkipReason, SkippedEntry};
    use crate::findings::Severity;

    /// Achado mínimo usado para testar a renderização.
    fn sample_finding() -> Finding {
        Finding {
            id: "dep.not_allowed:domain:application:src/lib.rs:3".to_owned(),
            rule_id: "dep.not_allowed".to_owned(),
            severity: Severity::Error,
            title: "Dependência não permitida: domain -> application".to_owned(),
            message: "O uso não está no contrato.".to_owned(),
            path: "src/lib.rs".to_owned(),
            line_start: Some(3),
            line_end: Some(3),
            evidence: Some("use application::Service;".to_owned()),
            expected: "nenhuma dependência permitida".to_owned(),
            suggestion: "Revise o uso ou o contrato.".to_owned(),
            confidence: None,
        }
    }

    /// Relatório em memória com um achado e uma limitação.
    fn sample_report() -> AuditReport {
        AuditReport {
            project: "projeto-teste".to_owned(),
            schema_version: 1,
            summary: ReportSummary {
                files_included: 4,
                files_skipped: 3,
                modules: 3,
                packages: 2,
                edges: 5,
                findings: 1,
                errors: 1,
                warnings: 0,
                infos: 0,
                needs_review: 0,
            },
            skip_summary: SkipReasonSummary::from_entries(&[
                SkippedEntry {
                    path: "build/artifact.rs".to_owned(),
                    reason: SkipReason::DefaultIgnored,
                },
                SkippedEntry {
                    path: "src/link.rs".to_owned(),
                    reason: SkipReason::Symlink,
                },
                SkippedEntry {
                    path: "src/locked.rs".to_owned(),
                    reason: SkipReason::Inaccessible,
                },
            ]),
            findings: vec![sample_finding()],
            limitations: vec!["arquivo.rs não pôde ser analisado".to_owned()],
            coverage: AnalysisCoverage {
                code_files_included: 4,
                code_files_analyzed: 2,
                code_files_not_analyzed: 2,
                unsupported_code_extensions: vec![
                    UnsupportedCodeExtension {
                        extension: "go".to_owned(),
                        files: 1,
                    },
                    UnsupportedCodeExtension {
                        extension: "rb".to_owned(),
                        files: 1,
                    },
                ],
                partial: true,
            },
            external_packages: Vec::new(),
            mode: ExecutionMode::Compliance,
            contract: ContractStatus::Present { schema_version: 1 },
        }
    }

    #[test]
    fn formats_have_stable_names_and_extensions() {
        assert_eq!(ReportFormat::Text.as_str(), "text");
        assert_eq!(ReportFormat::Json.as_str(), "json");
        assert_eq!(ReportFormat::Markdown.as_str(), "markdown");
        assert_eq!(ReportFormat::Text.extension(), "txt");
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Markdown.extension(), "md");
        assert_eq!(ReportFormat::Markdown.to_string(), "markdown");
    }

    #[test]
    fn text_report_contains_actionable_fields() {
        let output = sample_report()
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");

        assert!(output.contains("Auditoria: projeto-teste (schema v1)"));
        assert!(output.contains(
            "Cobertura do código: 2 de 4 arquivo(s) analisados (parcial); sem analisador: .go (1), .rb (1)"
        ));
        assert!(output.contains("[erro] dep.not_allowed"));
        assert!(output.contains("id: dep.not_allowed:domain:application:src/lib.rs:3"));
        assert!(output.contains("local: src/lib.rs:3"));
        assert!(output.contains("evidência: use application::Service;"));
        assert!(output.contains("Limitações:"));
    }

    #[test]
    fn json_report_exposes_the_full_finding_model() {
        let output = sample_report()
            .render(ReportFormat::Json)
            .expect("JSON rendering must succeed");
        let value: serde_json::Value = serde_json::from_str(&output).expect("JSON must be valid");

        assert_eq!(value["project"], "projeto-teste");
        assert_eq!(value["summary"]["findings"], 1);
        assert_eq!(value["coverage"]["code_files_included"], 4);
        assert_eq!(value["coverage"]["code_files_analyzed"], 2);
        assert_eq!(value["coverage"]["code_files_not_analyzed"], 2);
        assert_eq!(value["coverage"]["partial"], true);
        assert_eq!(
            value["coverage"]["unsupported_code_extensions"][0]["extension"],
            "go"
        );
        assert_eq!(
            value["coverage"]["unsupported_code_extensions"][1]["files"],
            1
        );
        assert_eq!(value["findings"][0]["severity"], "error");
        assert_eq!(value["findings"][0]["line_start"], 3);
        assert_eq!(
            value["findings"][0]["evidence"],
            "use application::Service;"
        );
        assert!(value["findings"][0]["confidence"].is_null());
    }

    #[test]
    fn markdown_report_escapes_pipes_and_newlines() {
        let mut report = sample_report();
        report.findings[0].title = "canal | linha\nnova".to_owned();
        let output = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");

        assert!(output.contains("# Auditoria: projeto-teste"));
        assert!(output.contains(
            "- Cobertura do código: 2 de 4 arquivo(s) analisados (parcial); sem analisador: .go (1), .rb (1)"
        ));
        assert!(output.contains("canal \\| linha<br>nova"));
        assert!(output.contains("**Evidência:** `use application::Service;`"));
        assert!(output.contains("## Limitações"));
    }

    #[test]
    fn skip_summary_separates_intentional_exclusions_from_coverage_gaps() {
        let report = sample_report();

        assert_eq!(report.skip_summary.total, 3);
        assert_eq!(report.skip_summary.intentional, 1);
        assert_eq!(report.skip_summary.coverage_gaps, 2);
        assert_eq!(
            report.skip_summary.reasons[0].reason,
            SkipReason::DefaultIgnored
        );
        assert_eq!(report.skip_summary.reasons[1].reason, SkipReason::Symlink);
        assert_eq!(
            report.skip_summary.reasons[2].reason,
            SkipReason::Inaccessible
        );

        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        assert!(text.contains("Motivos dos não incluídos: 3 (1 intencional, 2 reduzem cobertura)"));
        assert!(text.contains("padrão ignorado: 1 (intencional)"));
        assert!(text.contains("link simbólico: 1 (reduz cobertura)"));
        assert!(text.contains("inacessível: 1 (reduz cobertura)"));

        let markdown = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");
        assert!(
            markdown
                .contains("- Motivos dos não incluídos: 3 (1 intencional, 2 reduzem cobertura)")
        );
        assert!(markdown.contains("link simbólico: 1 (reduz cobertura)"));
    }

    #[test]
    fn json_report_exposes_bounded_skip_examples() {
        let entry = SkipReasonSummary::from_entries(
            &(0..20)
                .map(|index| SkippedEntry {
                    path: format!("build/artefato-{index:02}.rs"),
                    reason: SkipReason::DefaultIgnored,
                })
                .collect::<Vec<_>>(),
        );
        let mut report = sample_report();
        report.skip_summary = entry;
        let output = report
            .render(ReportFormat::Json)
            .expect("JSON rendering must succeed");
        let value: serde_json::Value = serde_json::from_str(&output).expect("JSON must be valid");

        assert_eq!(value["skip_summary"]["total"], 20);
        assert_eq!(value["skip_summary"]["intentional"], 20);
        assert_eq!(value["skip_summary"]["coverage_gaps"], 0);
        assert_eq!(
            value["skip_summary"]["reasons"][0]["label"],
            "padrão ignorado"
        );
        assert_eq!(
            value["skip_summary"]["reasons"][0]["examples"]
                .as_array()
                .map(Vec::len),
            Some(3)
        );
        assert_eq!(value["skip_summary"]["reasons"][0]["additional"], 17);
        assert_eq!(
            value["skip_summary"]["reasons"][0]["reduces_coverage"],
            false
        );
    }

    #[test]
    fn empty_report_is_explicit() {
        let mut report = sample_report();
        report.findings.clear();
        report.summary.findings = 0;
        report.summary.errors = 0;

        assert!(!report.has_findings());
        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        assert!(text.contains("Nenhum achado."));
    }

    #[test]
    fn complete_coverage_does_not_claim_a_partial_audit() {
        let mut report = sample_report();
        report.coverage = AnalysisCoverage {
            code_files_included: 4,
            code_files_analyzed: 4,
            code_files_not_analyzed: 0,
            unsupported_code_extensions: Vec::new(),
            partial: false,
        };

        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        let markdown = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");

        assert!(text.contains("Cobertura do código: 4 de 4 arquivo(s) analisados"));
        assert!(markdown.contains("Cobertura do código: 4 de 4 arquivo(s) analisados"));
        assert!(!text.contains("parcial"));
        assert!(!markdown.contains("parcial"));
    }

    #[test]
    fn finding_location_handles_missing_and_ranged_lines() {
        let mut finding = sample_finding();
        finding.line_end = Some(5);
        assert_eq!(finding_location(&finding), "src/lib.rs:3-5");

        finding.line_start = None;
        finding.line_end = None;
        assert_eq!(finding_location(&finding), "src/lib.rs");
    }

    /// Raiz de uma fixture de projeto, relativa ao crate.
    fn project_fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name)
    }

    #[test]
    fn compliance_json_keeps_existing_fields_and_adds_mode() {
        let report = crate::audit::audit_project(&project_fixture("clean"))
            .expect("compliance audit must succeed");
        let output = report
            .render(ReportFormat::Json)
            .expect("JSON rendering must succeed");
        let value: serde_json::Value = serde_json::from_str(&output).expect("JSON must be valid");

        assert_eq!(value["mode"], "compliance");
        assert_eq!(value["contract"]["status"], "present");
        assert_eq!(value["contract"]["schema_version"], 1);
        assert!(value["project"].is_string());
        assert!(value["schema_version"].is_number());
        assert!(value["summary"].is_object());
        assert!(value["skip_summary"].is_object());
        assert!(value["findings"].is_array());
        assert!(value["limitations"].is_array());
        assert!(value["coverage"].is_object());
    }

    #[test]
    fn compliance_text_and_markdown_state_the_mode() {
        let report = crate::audit::audit_project(&project_fixture("clean"))
            .expect("compliance audit must succeed");
        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        let markdown = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");

        assert!(text.contains("Modo: conformidade (contrato v1)"));
        assert!(markdown.contains("Modo: conformidade (contrato v1)"));
        assert!(!text.contains("descoberta"));
        assert!(!markdown.contains("descoberta"));
    }

    #[test]
    fn discovery_report_states_mode_and_evidence_in_all_formats() {
        let report = crate::audit::discover_project(&project_fixture("discovery_candidates"))
            .expect("discovery must succeed");
        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        let markdown = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");
        let output = report
            .render(ReportFormat::Json)
            .expect("JSON rendering must succeed");
        let value: serde_json::Value = serde_json::from_str(&output).expect("JSON must be valid");

        assert!(text.contains("Modo: descoberta (sem contrato)"));
        assert!(markdown.contains("Modo: descoberta (sem contrato)"));
        assert_eq!(value["mode"], "discovery");
        assert_eq!(value["contract"]["status"], "missing");
        assert_eq!(value["findings"], serde_json::json!([]));
        for label in [
            "Módulos candidatos:",
            "Relações observadas:",
            "Manifests reconhecidos:",
            "Ambiguidades:",
            "Linguagens:",
            "Limitações:",
        ] {
            assert!(text.contains(label), "texto deve conter {label}");
            assert!(markdown.contains(label), "Markdown deve conter {label}");
        }
        assert!(text.contains("direct"));
        assert!(text.contains("typescript_javascript"));
    }

    #[test]
    fn discovery_diff_note_is_explicit_in_all_formats() {
        let report = crate::audit::discover_project(&project_fixture("discovery_no_contract"))
            .expect("discovery must succeed")
            .with_diff_note();
        let text = report
            .render(ReportFormat::Text)
            .expect("text rendering cannot fail");
        let markdown = report
            .render(ReportFormat::Markdown)
            .expect("Markdown rendering cannot fail");
        let output = report
            .render(ReportFormat::Json)
            .expect("JSON rendering must succeed");
        let value: serde_json::Value = serde_json::from_str(&output).expect("JSON must be valid");
        let note = crate::audit::DISCOVERY_DIFF_NOTE;

        assert!(text.contains(note));
        assert!(markdown.contains(note));
        assert!(
            value["candidates"]["limitations"]
                .as_array()
                .expect("limitations must be an array")
                .iter()
                .any(|item| item.as_str() == Some(note))
        );
    }
}
