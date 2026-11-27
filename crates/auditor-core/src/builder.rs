//! Estado editável do Architecture Builder e conversão para contrato válido.
//!
//! O rascunho contém sugestões e decisões humanas. A prévia TOML é produzida
//! pelo módulo de contrato e precisa passar pelo mesmo parser usado na
//! auditoria. A gravação acontece somente com autorização explícita e reabre
//! o arquivo salvo antes de executar a conformidade.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::audit::{self, ComplianceRun, DiscoveryReport, ProjectRun};
use crate::contract::{
    Architecture, ArchitectureStyle, Contract, ContractError, Layer, Module, Project, Scan,
    contract_identity, load_contract, parse_contract, render_contract,
};
use crate::discovery::candidates::{Ambiguity, CandidateModule, ObservedDependency};
use crate::discovery::{ScanOptions, matches_pattern};

/// Texto neutro para responsabilidades que o usuário ainda não descreveu.
///
/// O motor não interpreta responsabilidade como regra; o texto existe apenas
/// para satisfazer o campo obrigatório do schema e ser editado depois.
pub const DEFAULT_RESPONSIBILITY: &str = "Responsabilidade a definir";

/// Perfil escolhido pelo usuário. Não é gravado no `auditor.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuilderProfile {
    /// Módulos sem camadas; `layers` precisa permanecer vazio.
    ModularMonolith,
    /// Módulos distribuídos em camadas declaradas no contrato v2.
    LayeredMonolith,
}

/// Módulo editável do rascunho.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftModule {
    /// Nome usado pelo motor em `allowed_dependencies` e nas arestas.
    pub name: String,
    /// Único padrão de caminho aceito pelo schema para este módulo.
    pub path: String,
    /// Descrição legível, sem interpretação por regras.
    pub responsibility: String,
    /// Destinos que o usuário aprovou explicitamente.
    pub allowed_dependencies: Vec<String>,
    /// Destinos que o usuário revisou e decidiu não permitir.
    ///
    /// Este campo é estado do Builder e nunca é gravado no contrato.
    pub rejected_dependencies: Vec<String>,
    /// Evidências da sugestão, para revisão humana.
    pub evidence: Vec<String>,
    /// `true` enquanto o usuário não confirmar o candidato.
    pub needs_confirmation: bool,
}

/// Camada editável do rascunho, representável no schema v2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftLayer {
    /// Nome usado em `allowed_dependencies` das camadas.
    pub name: String,
    /// Módulos pertencentes a esta camada.
    pub modules: Vec<String>,
    /// Camadas permitidas como destino.
    pub allowed_dependencies: Vec<String>,
}

/// Tipo de decisão que o usuário precisa tomar antes de gerar o contrato.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingDecisionKind {
    /// Confirmar um módulo sugerido pela descoberta.
    ConfirmModule {
        /// Nome do candidato.
        name: String,
    },
    /// Escolher uma opção ou excluir conscientemente um conflito.
    ResolveAmbiguity {
        /// Assunto que originou o conflito.
        subject: String,
        /// Detalhe que distingue conflitos com o mesmo assunto.
        detail: String,
        /// Opções conhecidas; vazias quando nenhum candidato cobre o caso.
        options: Vec<String>,
    },
    /// Permitir ou rejeitar uma relação observada no código.
    DecideDependency {
        /// Arquivo de origem da relação.
        source: String,
        /// Destino bruto encontrado no código.
        target: String,
        /// Módulo único que cobre a origem, calculado pelo núcleo.
        module: String,
    },
}

/// Decisão pendente com explicação para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingDecision {
    /// Ação necessária.
    pub kind: PendingDecisionKind,
    /// Explicação curta, não interpretada pelo motor.
    pub detail: String,
}

/// Escolha registrada para uma ambiguidade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmbiguityResolution {
    /// Assunto da ambiguidade resolvida.
    pub subject: String,
    /// Detalhe que distingue ambiguidades com o mesmo assunto.
    pub detail: String,
    /// Opção escolhida, ou `None` quando o usuário excluiu o caso.
    pub choice: Option<String>,
}

/// Rascunho completo, editável e convertível em contrato v2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDraft {
    /// Nome legível do projeto.
    pub project_name: String,
    /// Padrões de inclusão da varredura.
    pub include: Vec<String>,
    /// Padrões de exclusão da varredura.
    pub exclude: Vec<String>,
    /// Estilo arquitetural; apenas `modular_monolith` nesta versão.
    pub style: ArchitectureStyle,
    /// Perfil do assistente, que não vira campo do contrato.
    pub profile: BuilderProfile,
    /// Módulos propostos ou declarados manualmente.
    pub modules: Vec<DraftModule>,
    /// Relações observadas que exigem decisão explícita.
    pub observed_dependencies: Vec<ObservedDependency>,
    /// Ambiguidades da descoberta que exigem escolha humana.
    pub discovered_ambiguities: Vec<Ambiguity>,
    /// Camadas do perfil em camadas.
    pub layers: Vec<DraftLayer>,
    /// Escolhas registradas para ambiguidades.
    pub resolved_ambiguities: Vec<AmbiguityResolution>,
    /// Decisões ainda necessárias; impedem a geração final.
    pub pending_decisions: Vec<PendingDecision>,
}

/// Rascunho manual para projeto novo, sem módulos nem decisões pendentes.
pub fn empty_draft(project_name: &str) -> ContractDraft {
    let scope = ScanOptions::without_contract();
    ContractDraft {
        project_name: project_name.to_owned(),
        include: scope.include,
        exclude: scope.exclude,
        style: ArchitectureStyle::ModularMonolith,
        profile: BuilderProfile::ModularMonolith,
        modules: Vec::new(),
        observed_dependencies: Vec::new(),
        discovered_ambiguities: Vec::new(),
        layers: Vec::new(),
        resolved_ambiguities: Vec::new(),
        pending_decisions: Vec::new(),
    }
}

/// Importa candidatos da descoberta como sugestões, sem aprovar nada.
pub fn draft_from_discovery(report: &DiscoveryReport) -> ContractDraft {
    let mut draft = empty_draft(&suggested_project_name(&report.inventory.root));

    for candidate in &report.candidates.candidate_modules {
        draft.pending_decisions.push(PendingDecision {
            kind: PendingDecisionKind::ConfirmModule {
                name: candidate.name.clone(),
            },
            detail: format!(
                "Confirmar módulo {} no path {}",
                candidate.name, candidate.path
            ),
        });
        draft.modules.push(module_from_candidate(candidate));
    }

    for ambiguity in &report.candidates.ambiguities {
        draft.pending_decisions.push(PendingDecision {
            kind: PendingDecisionKind::ResolveAmbiguity {
                subject: ambiguity.subject.clone(),
                detail: ambiguity.detail.clone(),
                options: ambiguity.options.clone(),
            },
            detail: format!("{}: {}", ambiguity.subject, ambiguity.detail),
        });
    }

    for dependency in &report.candidates.observed_dependencies {
        let sources: Vec<&DraftModule> = draft
            .modules
            .iter()
            .filter(|module| matches_pattern(&module.path, &dependency.source))
            .collect();
        match sources.as_slice() {
            [module] => draft.pending_decisions.push(PendingDecision {
                kind: PendingDecisionKind::DecideDependency {
                    source: dependency.source.clone(),
                    target: dependency.target.clone(),
                    module: module.name.clone(),
                },
                detail: format!(
                    "{} -> {} — {}",
                    dependency.source, dependency.target, dependency.detail
                ),
            }),
            _ => draft.pending_decisions.push(PendingDecision {
                kind: PendingDecisionKind::ResolveAmbiguity {
                    subject: dependency_subject(dependency),
                    detail: dependency.detail.clone(),
                    options: sources.iter().map(|module| module.name.clone()).collect(),
                },
                detail: format!(
                    "Origem {} sem módulo único para a relação com {}",
                    dependency.source, dependency.target
                ),
            }),
        }
    }
    draft.observed_dependencies = report.candidates.observed_dependencies.clone();
    draft.discovered_ambiguities = report.candidates.ambiguities.clone();

    draft
}

/// Converte o rascunho em contrato v2 validado pelo parser oficial.
pub fn validate_draft(draft: &ContractDraft) -> Result<Contract, ContractError> {
    validate_decisions(draft)?;
    validate_profile(draft)?;

    let contract = Contract {
        schema_version: 2,
        project: Project {
            name: draft.project_name.clone(),
        },
        scan: Scan {
            include: draft.include.clone(),
            exclude: draft.exclude.clone(),
        },
        architecture: Architecture {
            style: draft.style,
            modules: draft
                .modules
                .iter()
                .map(|module| Module {
                    name: module.name.clone(),
                    path: module.path.clone(),
                    responsibility: module.responsibility.clone(),
                    allowed_dependencies: module.allowed_dependencies.clone(),
                })
                .collect(),
            layers: draft
                .layers
                .iter()
                .map(|layer| Layer {
                    name: layer.name.clone(),
                    modules: layer.modules.clone(),
                    allowed_dependencies: layer.allowed_dependencies.clone(),
                })
                .collect(),
        },
    };

    let text = render_contract(&contract)?;
    parse_contract(&text)
}

/// Resultado de validação sem transformar erros de escolha em falha operacional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationResult {
    /// `true` somente quando o contrato equivalente passou no parser oficial.
    pub valid: bool,
    /// Mensagens de erro de validação, vazias quando `valid` é `true`.
    pub errors: Vec<String>,
    /// Identidade do contrato válido, ausente quando há erros.
    pub identity: Option<String>,
}

/// Valida o rascunho mantendo erros de escolha como estado da interface.
pub fn validate_draft_result(draft: &ContractDraft) -> Result<ValidationResult, ContractError> {
    match validate_draft(draft) {
        Ok(contract) => Ok(ValidationResult {
            valid: true,
            errors: Vec::new(),
            identity: Some(crate::contract::contract_identity(&contract)),
        }),
        Err(ContractError::Validation(message)) => Ok(ValidationResult {
            valid: false,
            errors: vec![message],
            identity: None,
        }),
        Err(error) => Err(error),
    }
}

/// Efeitos descritivos do rascunho, produzidos no núcleo para a revisão visual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DraftEffects {
    /// Versão do schema que será gravada.
    pub schema_version: u32,
    /// Quantidade de módulos que o motor fiscalizará.
    pub modules: usize,
    /// Quantidade de camadas ativas.
    pub layers: usize,
    /// Quantidade de dependências explicitamente permitidas.
    pub allowed_dependencies: usize,
    /// Identificadores das regras que passam a valer para o contrato.
    pub rule_ids: Vec<String>,
    /// Explicações curtas sobre o que o motor interpreta.
    pub notes: Vec<String>,
}

/// Descreve os efeitos do rascunho sem executar nenhuma regra ou auditoria.
pub fn draft_effects(draft: &ContractDraft) -> DraftEffects {
    use crate::rules::{
        RULE_CODE_DUPLICATED, RULE_DEPENDENCY_CYCLE, RULE_DEPENDENCY_NOT_ALLOWED,
        RULE_DEPENDENCY_UNRESOLVED, RULE_FILE_AMBIGUOUS_MODULE, RULE_FILE_UNMAPPED,
        RULE_LAYER_NOT_ALLOWED, RULE_MODULE_CONCENTRATION, RULE_MODULE_UNREFERENCED,
        RULE_MODULE_WITHOUT_FILES,
    };

    let mut rule_ids = vec![
        RULE_DEPENDENCY_NOT_ALLOWED.to_owned(),
        RULE_DEPENDENCY_UNRESOLVED.to_owned(),
        RULE_FILE_UNMAPPED.to_owned(),
        RULE_FILE_AMBIGUOUS_MODULE.to_owned(),
        RULE_DEPENDENCY_CYCLE.to_owned(),
        RULE_MODULE_WITHOUT_FILES.to_owned(),
        RULE_MODULE_CONCENTRATION.to_owned(),
        RULE_CODE_DUPLICATED.to_owned(),
        RULE_MODULE_UNREFERENCED.to_owned(),
    ];
    if !draft.layers.is_empty() {
        rule_ids.push(RULE_LAYER_NOT_ALLOWED.to_owned());
    }
    rule_ids.sort();

    DraftEffects {
        schema_version: 2,
        modules: draft.modules.len(),
        layers: draft.layers.len(),
        allowed_dependencies: draft
            .modules
            .iter()
            .map(|module| module.allowed_dependencies.len())
            .sum(),
        rule_ids,
        notes: vec![
            "responsabilidade é texto descritivo, não é interpretada pelo motor".to_owned(),
            "dependências observadas não são aprovadas automaticamente".to_owned(),
        ],
    }
}

/// Converte um contrato existente em rascunho de revisão, sem perdas.
///
/// Módulos e camadas já confirmados viram estado editável; nenhuma decisão
/// pendente ou relação observada é inventada para um contrato aprovado.
pub fn draft_from_contract(contract: &Contract) -> ContractDraft {
    ContractDraft {
        project_name: contract.project.name.clone(),
        include: contract.scan.include.clone(),
        exclude: contract.scan.exclude.clone(),
        style: contract.architecture.style,
        profile: if contract.architecture.layers.is_empty() {
            BuilderProfile::ModularMonolith
        } else {
            BuilderProfile::LayeredMonolith
        },
        modules: contract
            .architecture
            .modules
            .iter()
            .map(|module| DraftModule {
                name: module.name.clone(),
                path: module.path.clone(),
                responsibility: module.responsibility.clone(),
                allowed_dependencies: module.allowed_dependencies.clone(),
                rejected_dependencies: Vec::new(),
                evidence: Vec::new(),
                needs_confirmation: false,
            })
            .collect(),
        observed_dependencies: Vec::new(),
        discovered_ambiguities: Vec::new(),
        layers: contract
            .architecture
            .layers
            .iter()
            .map(|layer| DraftLayer {
                name: layer.name.clone(),
                modules: layer.modules.clone(),
                allowed_dependencies: layer.allowed_dependencies.clone(),
            })
            .collect(),
        resolved_ambiguities: Vec::new(),
        pending_decisions: Vec::new(),
    }
}

/// Autorização explícita para a gravação final.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveAuthorization {
    /// Cria `auditor.toml`; falha se o arquivo já existir.
    Create,
    /// Substitui o arquivo somente se o texto conferido ainda for o atual.
    Replace {
        /// Texto integral lido do contrato apresentado ao usuário.
        expected_text: String,
    },
}

/// Tipo de uma linha no diff determinístico do contrato.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractDiffLineKind {
    /// Linha presente nos dois textos.
    Context,
    /// Linha existente somente no texto proposto.
    Added,
    /// Linha existente somente no texto atual.
    Removed,
}

/// Linha do diff, com as posições dos dois textos quando aplicável.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractDiffLine {
    /// Tipo da mudança.
    pub kind: ContractDiffLineKind,
    /// Conteúdo da linha, sem o terminador.
    pub text: String,
    /// Número da linha no contrato atual, começando em `1`.
    pub old_line: Option<usize>,
    /// Número da linha no contrato proposto, começando em `1`.
    pub new_line: Option<usize>,
}

/// Comparação determinística entre o contrato atual e a prévia aprovada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractDiff {
    /// Indica se há arquivo na raiz; preenchido pelo chamador quando a origem é o disco.
    pub exists: bool,
    /// Texto atual conferido antes da substituição.
    pub current_text: String,
    /// `true` quando pelo menos uma linha mudou.
    pub changed: bool,
    /// Linhas em ordem, incluindo as contextualmente iguais.
    pub lines: Vec<ContractDiffLine>,
}

/// Compara linhas por igualdade exata; nenhuma regra arquitetural é aplicada.
pub fn contract_diff(current: &str, proposed: &str) -> ContractDiff {
    let old: Vec<&str> = current.lines().collect();
    let new: Vec<&str> = proposed.lines().collect();
    let (old_len, new_len) = (old.len(), new.len());
    let mut longest = vec![vec![0usize; new_len + 1]; old_len + 1];
    for index in (0..old_len).rev() {
        for other in (0..new_len).rev() {
            longest[index][other] = if old[index] == new[other] {
                longest[index + 1][other + 1] + 1
            } else {
                longest[index + 1][other].max(longest[index][other + 1])
            };
        }
    }

    let mut lines = Vec::new();
    let (mut index, mut other) = (0usize, 0usize);
    while index < old_len && other < new_len {
        if old[index] == new[other] {
            lines.push(ContractDiffLine {
                kind: ContractDiffLineKind::Context,
                text: old[index].to_owned(),
                old_line: Some(index + 1),
                new_line: Some(other + 1),
            });
            index += 1;
            other += 1;
        } else if longest[index + 1][other] >= longest[index][other + 1] {
            lines.push(ContractDiffLine {
                kind: ContractDiffLineKind::Removed,
                text: old[index].to_owned(),
                old_line: Some(index + 1),
                new_line: None,
            });
            index += 1;
        } else {
            lines.push(ContractDiffLine {
                kind: ContractDiffLineKind::Added,
                text: new[other].to_owned(),
                old_line: None,
                new_line: Some(other + 1),
            });
            other += 1;
        }
    }
    while index < old_len {
        lines.push(ContractDiffLine {
            kind: ContractDiffLineKind::Removed,
            text: old[index].to_owned(),
            old_line: Some(index + 1),
            new_line: None,
        });
        index += 1;
    }
    while other < new_len {
        lines.push(ContractDiffLine {
            kind: ContractDiffLineKind::Added,
            text: new[other].to_owned(),
            old_line: None,
            new_line: Some(other + 1),
        });
        other += 1;
    }

    ContractDiff {
        exists: !current.is_empty(),
        current_text: current.to_owned(),
        changed: lines
            .iter()
            .any(|line| line.kind != ContractDiffLineKind::Context),
        lines,
    }
}

/// Dados do contrato efetivamente gravado e reaberto pelo parser oficial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SavedContract {
    /// Texto lido do arquivo após a renomeação.
    pub text: String,
    /// Identidade calculada a partir do contrato reaberto.
    pub identity: String,
    /// Versão de schema do arquivo gravado.
    pub schema_version: u32,
}

/// Resultado da gravação aprovada, incluindo a auditoria de conformidade posterior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveOutcome {
    /// Contrato lido do disco depois da gravação.
    pub saved: SavedContract,
    /// Auditoria executada com o contrato recém-gravado.
    pub run: ComplianceRun,
    /// `true` quando existia contrato antes da operação.
    pub replaced: bool,
}

/// Falhas da gravação final, explícitas para a interface.
#[derive(Debug)]
pub enum SaveError {
    /// O rascunho não passou nas regras do núcleo.
    Validation(ContractError),
    /// Não foi possível ler o contrato atual para conferir a autorização.
    Read {
        /// Arquivo lido.
        path: PathBuf,
        /// Erro original.
        source: io::Error,
    },
    /// A autorização não corresponde ao estado atual do arquivo.
    Conflict {
        /// Arquivo que causou a recusa.
        path: PathBuf,
        /// Motivo legível da recusa.
        reason: String,
    },
    /// Falha de E/S em uma das etapas da gravação.
    Io {
        /// Arquivo envolvido na etapa.
        path: PathBuf,
        /// Erro original.
        source: io::Error,
        /// Etapa que falhou.
        phase: &'static str,
    },
    /// O arquivo gravado não passou no parser oficial.
    Reparse {
        /// Arquivo reaberto.
        path: PathBuf,
        /// Erro de parsing.
        source: ContractError,
    },
    /// A auditoria posterior falhou; o contrato permanece gravado.
    Audit(audit::AuditError),
    /// Inconsistência interna detectada depois da gravação.
    Unexpected(&'static str),
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(source) => write!(formatter, "rascunho inválido: {source}"),
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "não foi possível ler {}: {source}",
                    path.display()
                )
            }
            Self::Conflict { path, reason } => {
                write!(
                    formatter,
                    "gravação recusada em {}: {reason}",
                    path.display()
                )
            }
            Self::Io {
                path,
                source,
                phase,
            } => write!(formatter, "falha ao {phase} {}: {source}", path.display()),
            Self::Reparse { path, source } => {
                write!(
                    formatter,
                    "contrato gravado em {} é inválido: {source}",
                    path.display()
                )
            }
            Self::Audit(source) => write!(formatter, "auditoria após a gravação falhou: {source}"),
            Self::Unexpected(message) => write!(formatter, "inconsistência interna: {message}"),
        }
    }
}

impl Error for SaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(source) | Self::Reparse { source, .. } => Some(source),
            Self::Read { source, .. } | Self::Io { source, .. } => Some(source),
            Self::Audit(source) => Some(source),
            Self::Conflict { .. } | Self::Unexpected(_) => None,
        }
    }
}

/// Contador do arquivo temporário, único dentro do processo.
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Revalida, grava atomicamente e executa a conformidade com o contrato salvo.
pub fn save_draft(
    root: &Path,
    draft: &ContractDraft,
    authorization: &SaveAuthorization,
) -> Result<SaveOutcome, SaveError> {
    let temporary = root.join(format!(
        "auditor.toml.{}-{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    save_draft_at(root, draft, authorization, &temporary)
}

/// Mesma operação de `save_draft`, com o caminho temporário explícito para testes.
fn save_draft_at(
    root: &Path,
    draft: &ContractDraft,
    authorization: &SaveAuthorization,
    temporary: &Path,
) -> Result<SaveOutcome, SaveError> {
    let path = root.join("auditor.toml");
    let contract = validate_draft(draft).map_err(SaveError::Validation)?;
    let text = render_contract(&contract).map_err(SaveError::Validation)?;
    parse_contract(&text).map_err(|source| SaveError::Reparse {
        path: path.clone(),
        source,
    })?;

    let expected_text = match authorization {
        SaveAuthorization::Create => None,
        SaveAuthorization::Replace { expected_text } => Some(expected_text.as_str()),
    };
    let current = match fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(source) if source.kind() == io::ErrorKind::NotFound => None,
        Err(source) => return Err(SaveError::Read { path, source }),
    };
    let replaced = match (&current, expected_text) {
        (Some(_), None) => {
            return Err(SaveError::Conflict {
                path,
                reason: "auditor.toml já existe; substituição exige confirmação".to_owned(),
            });
        }
        (None, Some(_)) => {
            return Err(SaveError::Conflict {
                path,
                reason: "auditor.toml não existe; não há o que substituir".to_owned(),
            });
        }
        (Some(existing), Some(expected)) if existing != expected => {
            return Err(SaveError::Conflict {
                path,
                reason: "o contrato mudou desde a confirmação".to_owned(),
            });
        }
        (Some(_), Some(_)) => true,
        (None, None) => false,
    };

    // Reduz a janela entre a conferência da autorização e a renomeação.
    if replaced {
        let latest = fs::read_to_string(&path).map_err(|source| SaveError::Read {
            path: path.clone(),
            source,
        })?;
        if Some(&latest) != current.as_ref() {
            return Err(SaveError::Conflict {
                path,
                reason: "o contrato mudou desde a conferência".to_owned(),
            });
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(|source| SaveError::Io {
            path: temporary.to_path_buf(),
            source,
            phase: "criar arquivo temporário",
        })?;
    file.write_all(text.as_bytes())
        .map_err(|source| SaveError::Io {
            path: temporary.to_path_buf(),
            source,
            phase: "escrever arquivo temporário",
        })?;
    file.sync_all().map_err(|source| SaveError::Io {
        path: temporary.to_path_buf(),
        source,
        phase: "sincronizar arquivo temporário",
    })?;
    drop(file);

    if let Err(source) = fs::rename(temporary, &path) {
        let _ = fs::remove_file(temporary);
        return Err(SaveError::Io {
            path,
            source,
            phase: "renomear arquivo temporário",
        });
    }

    let reopened = load_contract(&path).map_err(|source| SaveError::Reparse {
        path: path.clone(),
        source,
    })?;
    if reopened != contract {
        return Err(SaveError::Unexpected(
            "contrato reaberto difere do rascunho aprovado",
        ));
    }
    let written = fs::read_to_string(&path).map_err(|source| SaveError::Io {
        path: path.clone(),
        source,
        phase: "rel contrato gravado",
    })?;
    if written != text {
        return Err(SaveError::Unexpected(
            "texto reaberto difere da prévia aprovada",
        ));
    }
    let run = match audit::project_run(root).map_err(SaveError::Audit)? {
        ProjectRun::Compliance(run) => run,
        ProjectRun::Discovery(_) => {
            return Err(SaveError::Unexpected(
                "a raiz voltou ao modo de descoberta após a gravação",
            ));
        }
    };

    Ok(SaveOutcome {
        saved: SavedContract {
            text: written,
            identity: contract_identity(&reopened),
            schema_version: reopened.schema_version,
        },
        run,
        replaced,
    })
}

/// Nome sugerido a partir do diretório raiz; sem inferência de manifesto.
fn suggested_project_name(root: &Path) -> String {
    root.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "projeto".to_owned())
}

/// Converte um candidato em módulo editável, sem dependências aprovadas.
fn module_from_candidate(candidate: &CandidateModule) -> DraftModule {
    DraftModule {
        name: candidate.name.clone(),
        path: candidate.path.clone(),
        responsibility: DEFAULT_RESPONSIBILITY.to_owned(),
        allowed_dependencies: Vec::new(),
        rejected_dependencies: Vec::new(),
        evidence: candidate.evidence.clone(),
        needs_confirmation: candidate.needs_confirmation,
    }
}

/// Exige que todas as decisões obrigatórias tenham sido tomadas no rascunho.
///
/// A verificação usa os fatos importados da descoberta, não a lista de
/// pendências: remover uma pendência sem registrar a escolha não pode liberar
/// a gravação.
fn validate_decisions(draft: &ContractDraft) -> Result<(), ContractError> {
    for module in &draft.modules {
        if module.needs_confirmation {
            return Err(invalid(format!(
                "módulo ainda não confirmado: {}",
                module.name
            )));
        }
    }

    for ambiguity in &draft.discovered_ambiguities {
        let resolution = find_resolution(draft, &ambiguity.subject, &ambiguity.detail)
            .ok_or_else(|| invalid(format!("ambiguidade não resolvida: {}", ambiguity.subject)))?;
        validate_choice(resolution, &ambiguity.options)?;
    }

    validate_observed_dependencies(draft)
}

/// Exige uma decisão explícita para cada relação observada na descoberta.
fn validate_observed_dependencies(draft: &ContractDraft) -> Result<(), ContractError> {
    for dependency in &draft.observed_dependencies {
        let sources: Vec<&DraftModule> = draft
            .modules
            .iter()
            .filter(|module| matches_pattern(&module.path, &dependency.source))
            .collect();
        if sources.len() == 1 {
            let module = sources[0];
            if !module.allowed_dependencies.contains(&dependency.target)
                && !module.rejected_dependencies.contains(&dependency.target)
            {
                return Err(invalid(format!(
                    "dependência observada sem decisão: {} -> {}",
                    dependency.source, dependency.target
                )));
            }
            continue;
        }

        let subject = dependency_subject(dependency);
        let resolution = find_resolution(draft, &subject, &dependency.detail).ok_or_else(|| {
            invalid(format!(
                "relação observada sem decisão: {} -> {}",
                dependency.source, dependency.target
            ))
        })?;
        let options: Vec<String> = sources.iter().map(|module| module.name.clone()).collect();
        validate_choice(resolution, &options)?;
    }
    Ok(())
}

/// Encontra a escolha registrada para um par assunto/detalhe.
fn find_resolution<'a>(
    draft: &'a ContractDraft,
    subject: &str,
    detail: &str,
) -> Option<&'a AmbiguityResolution> {
    draft
        .resolved_ambiguities
        .iter()
        .find(|candidate| candidate.subject == subject && candidate.detail == detail)
}

/// Garante que a escolha, quando informada, pertence às opções conhecidas.
fn validate_choice(
    resolution: &AmbiguityResolution,
    options: &[String],
) -> Result<(), ContractError> {
    if let Some(choice) = &resolution.choice
        && !options.is_empty()
        && !options.contains(choice)
    {
        return Err(invalid(format!(
            "escolha {choice} não está entre as opções de {}",
            resolution.subject
        )));
    }
    Ok(())
}

/// Identificador estável da relação sem módulo de origem único.
fn dependency_subject(dependency: &ObservedDependency) -> String {
    format!(
        "origem da relação: {} -> {}",
        dependency.source, dependency.target
    )
}

/// Aplica as regras do perfil que não cabem no validador do contrato.
fn validate_profile(draft: &ContractDraft) -> Result<(), ContractError> {
    match draft.profile {
        BuilderProfile::ModularMonolith => {
            if !draft.layers.is_empty() {
                return Err(invalid(
                    "perfil sem camadas não admite camadas declaradas".to_owned(),
                ));
            }
        }
        BuilderProfile::LayeredMonolith => {
            if draft.layers.is_empty() {
                return Err(invalid(
                    "perfil em camadas exige ao menos uma camada".to_owned(),
                ));
            }
            let mut assignments = BTreeMap::new();
            for layer in &draft.layers {
                for module in &layer.modules {
                    *assignments.entry(module.as_str()).or_insert(0usize) += 1;
                }
            }
            for module in &draft.modules {
                let count = assignments.get(module.name.as_str()).copied().unwrap_or(0);
                if count != 1 {
                    return Err(invalid(format!(
                        "módulo {} deve pertencer a exatamente uma camada (encontrado {count})",
                        module.name
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Erro de validação do rascunho, no mesmo tipo usado pelo contrato.
fn invalid(message: String) -> ContractError {
    ContractError::Validation(message)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;
    use crate::audit::discover_project;
    use crate::contract::{ArchitectureStyle, contract_identity, parse_contract};

    /// Raiz de uma fixture de projeto, relativa ao crate.
    fn project_fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/projects")
            .join(name)
    }

    /// Descoberta real de uma fixture, sem contrato.
    fn discovered(name: &str) -> crate::audit::DiscoveryReport {
        discover_project(&project_fixture(name)).expect("discovery must succeed")
    }

    /// Módulo manual com os campos obrigatórios preenchidos.
    fn module(name: &str, path: &str) -> DraftModule {
        DraftModule {
            name: name.to_owned(),
            path: path.to_owned(),
            responsibility: format!("Responsabilidade de {name}"),
            ..DraftModule::default()
        }
    }

    /// Confirma módulos, resolve ambiguidades por exclusão e decide cada relação.
    fn resolve_all_decisions(draft: &mut ContractDraft) {
        let decisions = draft.pending_decisions.clone();
        for decision in decisions {
            match decision.kind {
                PendingDecisionKind::ConfirmModule { name } => {
                    let module = draft
                        .modules
                        .iter_mut()
                        .find(|candidate| candidate.name == name)
                        .expect("decisão deve referir módulo existente");
                    module.needs_confirmation = false;
                }
                PendingDecisionKind::ResolveAmbiguity {
                    subject,
                    detail,
                    options: _,
                } => {
                    draft.resolved_ambiguities.push(AmbiguityResolution {
                        subject,
                        detail,
                        choice: None,
                    });
                }
                PendingDecisionKind::DecideDependency { module, target, .. } => {
                    let index = draft
                        .modules
                        .iter()
                        .position(|candidate| candidate.name == module)
                        .expect("decisão deve referir módulo existente");
                    if target == "domain" {
                        draft.modules[index].allowed_dependencies.push(target);
                    } else {
                        draft.modules[index].rejected_dependencies.push(target);
                    }
                }
            }
        }
    }

    #[test]
    fn empty_draft_produces_valid_v2_without_modules() {
        let draft = empty_draft("projeto-novo");
        assert_eq!(draft.profile, BuilderProfile::ModularMonolith);
        assert!(draft.modules.is_empty());
        assert!(draft.layers.is_empty());
        assert!(draft.pending_decisions.is_empty());

        let contract = validate_draft(&draft).expect("draft vazio deve ser válido");
        assert_eq!(contract.schema_version, 2);
        assert_eq!(
            contract.architecture.style,
            ArchitectureStyle::ModularMonolith
        );
        assert!(contract.architecture.modules.is_empty());
        assert!(contract.architecture.layers.is_empty());

        let text = render_contract(&contract).expect("contrato deve serializar");
        assert!(text.contains("schema_version = 2"));
        assert!(text.contains("style = \"modular_monolith\""));
    }

    #[test]
    fn planned_path_without_files_is_valid() {
        let mut draft = empty_draft("planejado");
        draft.modules.push(module("futuro", "src/futuro/**"));

        let contract = validate_draft(&draft).expect("path planejado não exige arquivos");
        assert_eq!(contract.architecture.modules[0].path, "src/futuro/**");
    }

    #[test]
    fn discovery_becomes_editable_suggestions_without_approval() {
        let draft = draft_from_discovery(&discovered("discovery_candidates"));

        assert_eq!(draft.modules.len(), 3);
        assert_eq!(draft.modules[0].name, "application");
        assert_eq!(draft.modules[0].path, "crates/application/src/**");
        assert!(draft.modules.iter().all(|module| module.needs_confirmation));
        assert!(
            draft
                .modules
                .iter()
                .all(|module| !module.evidence.is_empty())
        );
        assert!(
            draft
                .modules
                .iter()
                .all(|module| module.allowed_dependencies.is_empty())
        );
        assert!(draft.layers.is_empty());
        assert_eq!(draft.pending_decisions.len(), 7);
        assert!(
            draft.pending_decisions.iter().any(|decision| matches!(
                decision.kind,
                PendingDecisionKind::DecideDependency { .. }
            ))
        );
    }

    #[test]
    fn dependency_from_unmapped_source_is_a_pending_decision() {
        let draft = draft_from_discovery(&discovered("discovery_no_contract"));
        let unmapped = draft.pending_decisions.iter().any(|decision| {
            matches!(
                &decision.kind,
                PendingDecisionKind::ResolveAmbiguity { subject, options, .. }
                    if subject.contains("script.py") && options.is_empty()
            )
        });
        assert!(
            unmapped,
            "arquivo na raiz não pode virar dependência aprovada"
        );
    }

    #[test]
    fn pending_decisions_block_validation() {
        let draft = draft_from_discovery(&discovered("discovery_candidates"));
        let error = validate_draft(&draft).expect_err("sugestões não confirmadas bloqueiam");
        assert!(matches!(error, ContractError::Validation(_)));
        assert!(error.to_string().contains("módulo"));
    }

    #[test]
    fn ambiguities_block_validation_even_after_modules_are_confirmed() {
        let mut draft = draft_from_discovery(&discovered("discovery_candidates_ambiguous"));
        for module in &mut draft.modules {
            module.needs_confirmation = false;
        }
        let error = validate_draft(&draft).expect_err("ambiguidade não resolvida bloqueia");
        assert!(
            error.to_string().contains("ambiguidade"),
            "erro inesperado: {error}"
        );
    }

    #[test]
    fn clearing_pending_decisions_does_not_bypass_required_choices() {
        let mut draft = draft_from_discovery(&discovered("discovery_candidates"));
        draft.pending_decisions.clear();
        for module in &mut draft.modules {
            module.needs_confirmation = false;
        }

        let error =
            validate_draft(&draft).expect_err("fatos observados continuam exigindo decisão");
        assert!(
            error
                .to_string()
                .contains("dependência observada sem decisão"),
            "erro inesperado: {error}"
        );
    }

    #[test]
    fn resolved_draft_round_trips_through_the_official_parser() {
        let mut draft = draft_from_discovery(&discovered("discovery_candidates"));
        resolve_all_decisions(&mut draft);

        let contract = validate_draft(&draft).expect("draft com decisões tomadas deve ser válido");
        assert_eq!(contract.schema_version, 2);
        let application = contract
            .architecture
            .modules
            .iter()
            .find(|module| module.name == "application")
            .expect("candidato application deve existir");
        assert_eq!(application.allowed_dependencies, vec!["domain"]);

        let text = render_contract(&contract).expect("contrato deve serializar");
        assert_eq!(
            parse_contract(&text).expect("TOML deve ser válido"),
            contract
        );
        assert_eq!(
            render_contract(&contract).expect("segunda renderização"),
            text,
            "a prévia deve ser determinística"
        );
        for draft_only in [
            "needs_confirmation",
            "rejected_dependencies",
            "pending_decisions",
            "evidence",
        ] {
            assert!(
                !text.contains(draft_only),
                "{draft_only} não é campo do schema"
            );
        }
    }

    #[test]
    fn identity_is_stable_and_changes_with_the_contract() {
        let draft = empty_draft("projeto");
        let first = validate_draft(&draft).expect("draft deve ser válido");
        let mut changed = draft.clone();
        changed.project_name = "outro-projeto".to_owned();
        let second = validate_draft(&changed).expect("draft alterado deve ser válido");

        let identity = contract_identity(&first);
        assert_eq!(identity.len(), 16);
        assert_eq!(identity, contract_identity(&first));
        assert_ne!(identity, contract_identity(&second));
    }

    #[test]
    fn layered_profile_requires_one_layer_per_module() {
        let mut draft = empty_draft("camadas");
        draft.profile = BuilderProfile::LayeredMonolith;
        draft.modules.push(module("domain", "crates/domain/src/**"));

        let error = validate_draft(&draft).expect_err("perfil em camadas exige layers");
        assert!(error.to_string().contains("camadas"));

        draft.layers.push(DraftLayer {
            name: "core".to_owned(),
            modules: vec!["domain".to_owned()],
            allowed_dependencies: Vec::new(),
        });
        let contract = validate_draft(&draft).expect("camada única deve ser válida");
        assert_eq!(contract.architecture.layers.len(), 1);
        assert_eq!(contract.architecture.layers[0].modules, vec!["domain"]);

        draft.layers.push(DraftLayer {
            name: "extra".to_owned(),
            modules: vec!["domain".to_owned()],
            allowed_dependencies: Vec::new(),
        });
        let error = validate_draft(&draft).expect_err("módulo em duas camadas deve falhar");
        assert!(error.to_string().contains("camada"));
    }

    #[test]
    fn modular_profile_rejects_layers() {
        let mut draft = empty_draft("sem-camadas");
        draft.modules.push(module("domain", "crates/domain/src/**"));
        draft.layers.push(DraftLayer {
            name: "core".to_owned(),
            modules: vec!["domain".to_owned()],
            allowed_dependencies: Vec::new(),
        });

        let error = validate_draft(&draft).expect_err("perfil sem camadas não admite layers");
        assert!(error.to_string().contains("camadas"));
    }

    #[test]
    fn builder_delegates_structural_validation_to_the_contract_validator() {
        let mut draft = empty_draft("sobrepostos");
        draft.modules.push(module("geral", "src/**"));
        draft.modules.push(module("domain", "src/domain/**"));

        let error = validate_draft(&draft).expect_err("paths sobrepostos devem falhar");
        assert!(
            error.to_string().contains("sobrepostos"),
            "erro inesperado: {error}"
        );
    }

    #[test]
    fn pending_decisions_expose_the_module_needed_by_the_desktop() {
        let draft = draft_from_discovery(&discovered("discovery_candidates"));
        let value = serde_json::to_value(&draft).expect("draft must serialize");
        let decisions = value["pending_decisions"]
            .as_array()
            .expect("decisions must be an array");

        assert!(
            decisions
                .iter()
                .any(|item| item["kind"]["confirm_module"].is_object())
        );
        let decided = decisions
            .iter()
            .find(|item| item["kind"]["decide_dependency"].is_object())
            .expect("observed dependency must become a decision");
        assert_eq!(
            decided["kind"]["decide_dependency"]["module"],
            "application"
        );
    }

    #[test]
    fn draft_survives_a_json_round_trip_for_the_desktop_boundary() {
        let mut draft = draft_from_discovery(&discovered("discovery_candidates"));
        draft.profile = BuilderProfile::LayeredMonolith;
        draft.layers.push(DraftLayer {
            name: "core".to_owned(),
            modules: draft
                .modules
                .iter()
                .map(|module| module.name.clone())
                .collect(),
            allowed_dependencies: Vec::new(),
        });

        let text = serde_json::to_string(&draft).expect("draft must serialize");
        let restored: ContractDraft = serde_json::from_str(&text).expect("draft must deserialize");
        assert_eq!(restored, draft);
        assert_eq!(
            serde_json::to_value(&restored).expect("profile must serialize")["profile"],
            "layered_monolith"
        );
    }

    #[test]
    fn existing_contract_becomes_a_confirmed_editable_draft() {
        let contract =
            crate::contract::load_contract(&project_fixture("clean").join("auditor.toml"))
                .expect("fixture contract must load");
        let draft = draft_from_contract(&contract);

        assert_eq!(draft.project_name, contract.project.name);
        assert_eq!(draft.include, contract.scan.include);
        assert_eq!(draft.exclude, contract.scan.exclude);
        assert_eq!(draft.modules.len(), contract.architecture.modules.len());
        assert!(
            draft
                .modules
                .iter()
                .all(|module| !module.needs_confirmation)
        );
        assert!(draft.pending_decisions.is_empty());
        assert!(draft.observed_dependencies.is_empty());
        let reviewed = validate_draft(&draft).expect("draft must be valid");
        assert_eq!(reviewed.schema_version, 2, "novo contrato é sempre v2");
        assert_eq!(reviewed.project, contract.project);
        assert_eq!(reviewed.scan, contract.scan);
        assert_eq!(reviewed.architecture, contract.architecture);
    }

    #[test]
    fn v2_layers_are_preserved_in_a_review_draft() {
        let contract = crate::contract::parse_contract(
            r#"
schema_version = 2

[project]
name = "camadas"

[scan]
include = ["src/**/*.rs"]
exclude = []

[architecture]
style = "modular_monolith"

[[architecture.modules]]
name = "domain"
path = "src/domain/**"
responsibility = "Domínio"
allowed_dependencies = []

[[architecture.layers]]
name = "domain"
modules = ["domain"]
allowed_dependencies = []
"#,
        )
        .expect("v2 contract must load");
        let draft = draft_from_contract(&contract);

        assert_eq!(draft.profile, BuilderProfile::LayeredMonolith);
        assert_eq!(draft.layers.len(), 1);
        assert_eq!(draft.layers[0].modules, ["domain"]);
        assert_eq!(
            validate_draft(&draft).expect("draft must be valid"),
            contract
        );
    }

    #[test]
    fn validation_result_reports_choices_without_failing_the_command() {
        let mut draft = empty_draft("pendente");
        draft.modules.push(module("domain", "src/domain/**"));
        draft.modules[0].needs_confirmation = true;

        let result = validate_draft_result(&draft).expect("validation must be a state");
        assert!(!result.valid);
        assert!(result.identity.is_none());
        assert!(result.errors[0].contains("módulo ainda não confirmado"));

        draft.modules[0].needs_confirmation = false;
        let accepted = validate_draft_result(&draft).expect("validation must be a state");
        assert!(accepted.valid);
        assert!(accepted.errors.is_empty());
        assert_eq!(
            accepted.identity,
            Some(crate::contract::contract_identity(
                &validate_draft(&draft).expect("draft must be valid")
            ))
        );
    }

    #[test]
    fn effects_describe_rules_without_evaluating_them() {
        let mut draft = empty_draft("regras");
        draft.modules.push(module("domain", "src/domain/**"));
        draft.modules[0]
            .allowed_dependencies
            .push("outro".to_owned());
        draft.profile = BuilderProfile::LayeredMonolith;
        draft.layers.push(DraftLayer {
            name: "core".to_owned(),
            modules: vec!["domain".to_owned()],
            allowed_dependencies: Vec::new(),
        });

        let effects = draft_effects(&draft);
        assert_eq!(effects.schema_version, 2);
        assert_eq!(effects.modules, 1);
        assert_eq!(effects.layers, 1);
        assert_eq!(effects.allowed_dependencies, 1);
        assert!(
            effects
                .rule_ids
                .contains(&crate::rules::RULE_DEPENDENCY_NOT_ALLOWED.to_owned())
        );
        assert!(
            effects
                .rule_ids
                .contains(&crate::rules::RULE_LAYER_NOT_ALLOWED.to_owned())
        );
        assert!(
            effects
                .notes
                .iter()
                .any(|note| note.contains("responsabilidade"))
        );
    }

    /// Diretório temporário que remove a própria cópia ao fim do teste.
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
                "auditor-etapa46-{}-{nanos}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            copy_directory(&project_fixture(fixture), &root);
            Self(root)
        }

        fn path(&self) -> &Path {
            &self.0
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

    /// Conteúdo completo de uma árvore, usado para provar que cancelar não grava.
    fn tree_snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
        fn visit(base: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
            for entry in std::fs::read_dir(dir).expect("diretório legível") {
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
                    files.insert(relative, std::fs::read(&path).expect("arquivo legível"));
                }
            }
        }

        let mut files = BTreeMap::new();
        visit(root, root, &mut files);
        files
    }

    /// Texto atual de um contrato, antes de qualquer operação de revisão.
    fn before_str(path: &Path) -> String {
        std::fs::read_to_string(path).expect("contrato legível")
    }

    #[test]
    fn save_authorization_serializes_for_the_desktop_boundary() {
        let create = serde_json::to_string(&SaveAuthorization::Create).expect("create serializa");
        assert_eq!(create, "\"create\"");

        let replace = serde_json::to_string(&SaveAuthorization::Replace {
            expected_text: "atual".to_owned(),
        })
        .expect("replace serializa");
        assert_eq!(replace, "{\"replace\":{\"expected_text\":\"atual\"}}");

        let restored: SaveAuthorization =
            serde_json::from_str(&replace).expect("authorization desserializa");
        assert_eq!(
            restored,
            SaveAuthorization::Replace {
                expected_text: "atual".to_owned()
            }
        );
    }

    #[test]
    fn contract_diff_reports_only_the_changed_lines() {
        let current = "schema_version = 2\nname = \"antes\"\nfim = true\n";
        let proposed = "schema_version = 2\nname = \"depois\"\nfim = true\n";
        let diff = contract_diff(current, proposed);

        assert!(diff.changed);
        assert_eq!(
            diff.lines
                .iter()
                .filter(|line| line.kind == ContractDiffLineKind::Context)
                .count(),
            2
        );
        assert!(diff.lines.iter().any(|line| {
            line.kind == ContractDiffLineKind::Removed && line.text == "name = \"antes\""
        }));
        assert!(diff.lines.iter().any(|line| {
            line.kind == ContractDiffLineKind::Added && line.text == "name = \"depois\""
        }));

        let equal = contract_diff(current, current);
        assert!(!equal.changed);
        assert!(
            equal
                .lines
                .iter()
                .all(|line| line.kind == ContractDiffLineKind::Context)
        );
    }

    #[test]
    fn invalid_draft_is_not_written() {
        let project = TempProject::new("discovery_no_contract");
        let mut draft = empty_draft("pendente");
        draft.modules.push(module("futuro", "src/futuro/**"));
        draft.modules[0].needs_confirmation = true;

        let error = save_draft(project.path(), &draft, &SaveAuthorization::Create)
            .expect_err("decisão pendente impede a gravação");
        assert!(
            matches!(error, SaveError::Validation(ContractError::Validation(_))),
            "erro inesperado: {error}"
        );
        assert!(!project.path().join("auditor.toml").exists());
    }

    #[test]
    fn empty_project_fixture_accepts_a_planned_contract() {
        let project = TempProject::new("discovery_empty_project");
        let report = discover_project(project.path()).expect("descoberta do projeto vazio");
        assert!(report.candidates.candidate_modules.is_empty());
        let draft = empty_draft("projeto-novo");
        let expected = validate_draft(&draft).expect("draft planejado válido");

        let outcome = save_draft(project.path(), &draft, &SaveAuthorization::Create)
            .expect("projeto vazio pode receber contrato planejado");

        assert_eq!(outcome.run.mode, crate::audit::ExecutionMode::Compliance);
        assert_eq!(outcome.saved.schema_version, 2);
        assert_eq!(
            parse_contract(&outcome.saved.text).expect("reparso do contrato gravado"),
            expected
        );
    }

    #[test]
    fn save_draft_creates_contract_and_reopens_it_in_compliance_mode() {
        let project = TempProject::new("discovery_no_contract");
        let draft = empty_draft("projeto-salvo");
        let expected = validate_draft(&draft).expect("draft válido");

        let outcome = save_draft(project.path(), &draft, &SaveAuthorization::Create)
            .expect("gravação deve suceder");

        assert!(!outcome.replaced);
        assert_eq!(outcome.run.mode, crate::audit::ExecutionMode::Compliance);
        assert_eq!(outcome.saved.schema_version, 2);
        let written =
            std::fs::read_to_string(project.path().join("auditor.toml")).expect("contrato gravado");
        assert_eq!(written, outcome.saved.text);
        assert_eq!(parse_contract(&written).expect("reparso"), expected);
        assert_eq!(
            outcome.saved.identity,
            crate::contract::contract_identity(&expected)
        );
        assert!(
            std::fs::read_dir(project.path())
                .expect("raiz legível")
                .all(|entry| !entry
                    .expect("entrada legível")
                    .file_name()
                    .to_string_lossy()
                    .contains(".tmp")),
            "arquivo temporário não pode restar"
        );
        let value = serde_json::to_value(&outcome).expect("outcome serializa");
        assert_eq!(value["run"]["mode"], "compliance");
        assert_eq!(value["replaced"], false);
    }

    #[test]
    fn replacing_a_contract_requires_the_exact_current_text() {
        let project = TempProject::new("clean");
        let path = project.path().join("auditor.toml");
        let current = std::fs::read_to_string(&path).expect("contrato da fixture");
        let original = parse_contract(&current).expect("fixture válida");
        let mut draft = draft_from_contract(&original);
        draft.project_name = "projeto-revisado".to_owned();

        let create = save_draft(project.path(), &draft, &SaveAuthorization::Create)
            .expect_err("criação não pode sobrescrever");
        assert!(matches!(create, SaveError::Conflict { .. }));
        let stale = save_draft(
            project.path(),
            &draft,
            &SaveAuthorization::Replace {
                expected_text: "texto antigo".to_owned(),
            },
        )
        .expect_err("aprovação obsoleta não pode sobrescrever");
        assert!(matches!(stale, SaveError::Conflict { .. }));
        assert_eq!(
            std::fs::read_to_string(&path).expect("contrato preservado"),
            current
        );

        let outcome = save_draft(
            project.path(),
            &draft,
            &SaveAuthorization::Replace {
                expected_text: current.clone(),
            },
        )
        .expect("substituição autorizada");
        assert!(outcome.replaced);
        assert_eq!(outcome.run.mode, crate::audit::ExecutionMode::Compliance);
        let written = std::fs::read_to_string(&path).expect("contrato gravado");
        assert!(written.contains("projeto-revisado"));
        assert!(!written.contains(&original.project.name));
        assert_eq!(outcome.saved.text, written);
    }

    #[test]
    fn failed_temporary_creation_preserves_the_previous_contract() {
        let project = TempProject::new("clean");
        let path = project.path().join("auditor.toml");
        let current = std::fs::read_to_string(&path).expect("contrato da fixture");
        let draft = draft_from_contract(&parse_contract(&current).expect("fixture válida"));
        let blocked = project.path().join("bloqueio.tmp");
        std::fs::create_dir(&blocked).expect("diretório que bloqueia o temporário");

        let error = save_draft_at(
            project.path(),
            &draft,
            &SaveAuthorization::Replace {
                expected_text: current.clone(),
            },
            &blocked,
        )
        .expect_err("criação do temporário deve falhar");
        assert!(matches!(error, SaveError::Io { .. }));
        assert_eq!(
            std::fs::read_to_string(&path).expect("contrato preservado"),
            current
        );
        assert!(blocked.is_dir());
    }

    #[test]
    fn v1_contract_fixture_is_read_without_rewrite() {
        let project = TempProject::new("builder_contract_v1");
        let path = project.path().join("auditor.toml");
        let before = std::fs::read(&path).expect("contrato v1 da fixture");

        let original = parse_contract(&before_str(&path)).expect("v1 válido");
        let draft = draft_from_contract(&original);
        let validation = validate_draft_result(&draft).expect("rascunho revisado é válido");
        assert!(validation.valid, "rascunho revisado deve ser válido");

        let preview = render_contract(&validate_draft(&draft).expect("draft válido"))
            .expect("prévia serializa");
        let _ = contract_diff(&before_str(&path), &preview);

        assert_eq!(std::fs::read(&path).expect("contrato preservado"), before);
        assert!(path.exists(), "a fixture possui contrato v1");
    }

    #[test]
    fn v2_contract_fixture_keeps_layers_in_the_review_draft() {
        let project = TempProject::new("builder_contract_v2");
        let current = before_str(&project.path().join("auditor.toml"));
        let original = parse_contract(&current).expect("v2 válido");
        let draft = draft_from_contract(&original);

        assert!(!draft.layers.is_empty(), "camadas devem entrar no rascunho");
        let contract = validate_draft(&draft).expect("draft v2 válido");
        assert_eq!(contract.schema_version, 2);
        assert_eq!(contract.architecture.layers.len(), draft.layers.len());
        assert_eq!(
            std::fs::read(project.path().join("auditor.toml")).expect("contrato preservado"),
            current.into_bytes()
        );
    }

    #[test]
    fn write_safety_fixture_refuses_overwrite_and_failed_temporary_creation() {
        let project = TempProject::new("builder_write_safety");
        let path = project.path().join("auditor.toml");
        let current = before_str(&path);
        let draft = draft_from_contract(&parse_contract(&current).expect("fixture válida"));

        let create = save_draft(project.path(), &draft, &SaveAuthorization::Create)
            .expect_err("criação não sobrescreve");
        assert!(matches!(create, SaveError::Conflict { .. }));
        let stale = save_draft(
            project.path(),
            &draft,
            &SaveAuthorization::Replace {
                expected_text: "texto antigo".to_owned(),
            },
        )
        .expect_err("aprovação obsoleta não sobrescreve");
        assert!(matches!(stale, SaveError::Conflict { .. }));

        let blocked = project.path().join("auditor.toml.0-0.tmp");
        std::fs::create_dir(&blocked).expect("diretório bloqueia o temporário");
        let failed = save_draft_at(
            project.path(),
            &draft,
            &SaveAuthorization::Replace {
                expected_text: current.clone(),
            },
            &blocked,
        )
        .expect_err("criação do temporário deve falhar");
        assert!(matches!(failed, SaveError::Io { .. }));

        std::fs::remove_dir(&blocked).expect("remove o bloqueio criado pelo teste");
        assert_eq!(
            std::fs::read_to_string(&path).expect("contrato preservado"),
            current
        );
        assert!(
            std::fs::read_dir(project.path())
                .expect("raiz legível")
                .all(|entry| !entry
                    .expect("entrada legível")
                    .file_name()
                    .to_string_lossy()
                    .contains(".tmp")),
            "nenhum temporário pode restar"
        );
    }

    #[test]
    fn cancelling_before_save_creates_no_contract() {
        let project = TempProject::new("builder_existing_project");
        let before = tree_snapshot(project.path());

        let report = discover_project(project.path()).expect("descoberta deve suceder");
        let draft = draft_from_discovery(&report);
        if let Ok(validation) = validate_draft_result(&draft)
            && validation.valid
        {
            let preview = render_contract(&validate_draft(&draft).expect("draft válido"))
                .expect("prévia serializa");
            let _ = contract_diff("", &preview);
        }

        assert!(!project.path().join("auditor.toml").exists());
        assert_eq!(tree_snapshot(project.path()), before);
    }
}
