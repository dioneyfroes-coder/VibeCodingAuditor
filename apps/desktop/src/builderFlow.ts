/// Helpers puros do assistente de arquitetura.
///
/// Este módulo apenas organiza o estado enviado pelo núcleo e as escolhas do
/// usuário. Nenhuma regra de auditoria, validação ou serialização é feita aqui.

export type BuilderProfile = 'modular_monolith' | 'layered_monolith';

export interface DraftModule {
  name: string;
  path: string;
  responsibility: string;
  allowed_dependencies: string[];
  rejected_dependencies: string[];
  evidence: string[];
  needs_confirmation: boolean;
}

export interface DraftLayer {
  name: string;
  modules: string[];
  allowed_dependencies: string[];
}

export interface AmbiguityResolution {
  subject: string;
  detail: string;
  choice: string | null;
}

/// Formato externamente marcado produzido por `PendingDecisionKind` no núcleo.
export type PendingDecisionKind =
  | { confirm_module: { name: string } }
  | { resolve_ambiguity: { subject: string; detail: string; options: string[] } }
  | {
      decide_dependency: { source: string; target: string; module: string };
    };

export interface PendingDecision {
  kind: PendingDecisionKind;
  detail: string;
}

/// Item pronto para decisão humana, sem recomputar globs nem regras.
export interface ObservedDecisionItem {
  source: string;
  target: string;
  detail: string;
  resolution: string;
  line: number | null;
  evidence: string | null;
  /// Módulo informado pelo núcleo quando a origem é única.
  moduleName: string | null;
  /// Assunto da ambiguidade de origem, quando o núcleo não encontrou módulo único.
  ambiguitySubject: string | null;
  /// Detalhe usado pelo núcleo para distinguir ambiguidades com o mesmo assunto.
  ambiguityDetail: string | null;
  /// Opções de módulo de origem informadas pelo núcleo.
  options: string[];
  /// Escolha já registrada no rascunho.
  selected: string | null;
}

export interface ObservedDependency {
  source: string;
  target: string;
  resolution: string;
  detail: string;
  line: number | null;
  evidence: string | null;
}

export interface Ambiguity {
  subject: string;
  detail: string;
  options: string[];
}

export interface ContractDraft {
  project_name: string;
  include: string[];
  exclude: string[];
  style: string;
  profile: BuilderProfile;
  modules: DraftModule[];
  observed_dependencies: ObservedDependency[];
  discovered_ambiguities: Ambiguity[];
  resolved_ambiguities: AmbiguityResolution[];
  layers: DraftLayer[];
  pending_decisions: PendingDecision[];
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  identity: string | null;
}

export interface DraftEffects {
  schema_version: number;
  modules: number;
  layers: number;
  allowed_dependencies: number;
  rule_ids: string[];
  notes: string[];
}

export type ContractDiffLineKind = 'context' | 'added' | 'removed';

export interface ContractDiffLine {
  kind: ContractDiffLineKind;
  text: string;
  old_line: number | null;
  new_line: number | null;
}

export interface ContractDiff {
  exists: boolean;
  current_text: string;
  changed: boolean;
  lines: ContractDiffLine[];
}

export type SaveAuthorization = 'create' | { replace: { expected_text: string } };

export interface AuditFinding {
  rule_id: string;
  severity: string;
  title: string;
  message: string;
  path: string;
  line_start: number | null;
}

export interface ComplianceReport {
  project: string;
  schema_version: number;
  summary: { findings: number; errors: number; warnings: number; needs_review: number };
  findings: AuditFinding[];
  limitations: string[];
}

export interface SaveOutcome {
  saved: { text: string; identity: string; schema_version: number };
  run: {
    mode: string;
    contract: { status: string; schema_version?: number };
    report: ComplianceReport;
  };
  replaced: boolean;
}

export interface ProjectRunStatus {
  mode: string;
  contract: { status: string; schema_version?: number };
}

export interface DiscoveryCandidates {
  candidate_modules: Array<{
    name: string;
    path: string;
    category: string;
    evidence: string[];
    needs_confirmation: boolean;
  }>;
  observed_dependencies: ObservedDependency[];
  ambiguities: Ambiguity[];
  languages: Array<{ extension: string; files: number; analyzer: string | null }>;
  limitations: string[];
}

export interface DiscoveryReport {
  candidates: DiscoveryCandidates;
  analysis: { limitations: string[] };
}

export interface BuilderStep {
  id: string;
  label: string;
  hint: string;
}

/// Ordem fixa das etapas, usada apenas para apresentação e navegação.
export const BUILDER_STEPS: BuilderStep[] = [
  { id: 'descoberta', label: 'Descoberta', hint: 'Observações e candidatos' },
  { id: 'escopo', label: 'Escopo', hint: 'Nome, inclusões e exclusões' },
  { id: 'organizacao', label: 'Organização', hint: 'Perfil da arquitetura' },
  { id: 'modulos', label: 'Módulos', hint: 'Nomes, paths e responsabilidades' },
  { id: 'camadas', label: 'Camadas', hint: 'Agrupamento quando aplicável' },
  { id: 'dependencias', label: 'Dependências', hint: 'Permitir ou rejeitar relações' },
  { id: 'revisao', label: 'Revisão', hint: 'Prévia e efeitos das regras' },
];

/// Aviso exibido em todas as etapas com sugestões vindas da descoberta.
export const BUILDER_WARNING =
  'As dependências observadas não são aprovadas automaticamente.';

/// Próxima etapa, sem ultrapassar o fim do assistente.
export function nextStep(current: number): number {
  return Math.min(current + 1, BUILDER_STEPS.length - 1);
}

/// Etapa anterior, sem ultrapassar o início do assistente.
export function previousStep(current: number): number {
  return Math.max(current - 1, 0);
}

/// Substitui um módulo mantendo os demais intocados.
export function updateDraftModule(
  current: ContractDraft,
  name: string,
  patch: Partial<DraftModule>,
): ContractDraft {
  return {
    ...current,
    modules: current.modules.map((module) =>
      module.name === name ? { ...module, ...patch } : module,
    ),
  };
}

/// Nome novo determinístico, sem sobrescrever módulos existentes.
function nextModuleName(modules: DraftModule[]): string {
  let index = 1;
  while (modules.some((module) => module.name === `modulo-${index}`)) {
    index += 1;
  }
  return `modulo-${index}`;
}

/// Adiciona um módulo manual, já confirmado porque não vem da descoberta.
export function addDraftModule(current: ContractDraft): ContractDraft {
  const name = nextModuleName(current.modules);
  const layers =
    current.profile === 'layered_monolith' && current.layers.length > 0
      ? current.layers.map((layer, index) =>
          index === 0 ? { ...layer, modules: [...layer.modules, name] } : layer,
        )
      : current.layers;
  return {
    ...current,
    modules: [
      ...current.modules,
      {
        name,
        path: `src/${name}/**`,
        responsibility: 'Responsabilidade a definir',
        allowed_dependencies: [],
        rejected_dependencies: [],
        evidence: [],
        needs_confirmation: false,
      },
    ],
    layers,
  };
}

/// Renomeia um módulo e todas as referências que o núcleo viria a fiscalizar.
export function renameDraftModule(
  current: ContractDraft,
  oldName: string,
  newName: string,
): ContractDraft {
  if (oldName === newName) {
    return current;
  }
  const renamed = (name: string) => (name === oldName ? newName : name);
  return {
    ...current,
    modules: current.modules.map((module) => ({
      ...module,
      name: renamed(module.name),
      allowed_dependencies: module.allowed_dependencies.map(renamed),
      rejected_dependencies: module.rejected_dependencies.map(renamed),
    })),
    layers: current.layers.map((layer) => ({
      ...layer,
      modules: layer.modules.map(renamed),
      allowed_dependencies: layer.allowed_dependencies.map(renamed),
    })),
    resolved_ambiguities: current.resolved_ambiguities.map((resolution) => ({
      ...resolution,
      choice: resolution.choice === null ? null : renamed(resolution.choice),
    })),
    pending_decisions: current.pending_decisions.map((decision) => {
      const kind = decision.kind;
      if ('confirm_module' in kind) {
        return {
          ...decision,
          kind: { confirm_module: { name: renamed(kind.confirm_module.name) } },
        };
      }
      if ('resolve_ambiguity' in kind) {
        return {
          ...decision,
          kind: {
            resolve_ambiguity: {
              ...kind.resolve_ambiguity,
              options: kind.resolve_ambiguity.options.map(renamed),
            },
          },
        };
      }
      return {
        ...decision,
        kind: {
          decide_dependency: {
            ...kind.decide_dependency,
            module: renamed(kind.decide_dependency.module),
          },
        },
      };
    }),
  };
}

/// Remove o módulo e limpa referências, decisões e escolhas que ficariam órfãs.
export function removeDraftModule(current: ContractDraft, name: string): ContractDraft {
  const kept = (candidate: string) => candidate !== name;
  return {
    ...current,
    modules: current.modules
      .filter((module) => module.name !== name)
      .map((module) => ({
        ...module,
        allowed_dependencies: module.allowed_dependencies.filter(kept),
        rejected_dependencies: module.rejected_dependencies.filter(kept),
      })),
    layers: current.layers.map((layer) => ({
      ...layer,
      modules: layer.modules.filter(kept),
      allowed_dependencies: layer.allowed_dependencies.filter(kept),
    })),
    resolved_ambiguities: current.resolved_ambiguities.map((resolution) => ({
      ...resolution,
      choice: resolution.choice === null || resolution.choice === name ? null : resolution.choice,
    })),
    pending_decisions: current.pending_decisions.flatMap((decision) => {
      const kind = decision.kind;
      if ('confirm_module' in kind) {
        return kind.confirm_module.name === name ? [] : [decision];
      }
      if ('resolve_ambiguity' in kind) {
        return [
          {
            ...decision,
            kind: {
              resolve_ambiguity: {
                ...kind.resolve_ambiguity,
                options: kind.resolve_ambiguity.options.filter(kept),
              },
            },
          },
        ];
      }
      return kind.decide_dependency.module === name ? [] : [decision];
    }),
  };
}

/// Renomeia uma camada e as referências de camadas que a apontam como destino.
export function renameDraftLayer(
  current: ContractDraft,
  oldName: string,
  newName: string,
): ContractDraft {
  if (oldName === newName) {
    return current;
  }
  const renamed = (name: string) => (name === oldName ? newName : name);
  return {
    ...current,
    layers: current.layers.map((layer) => ({
      ...layer,
      name: renamed(layer.name),
      allowed_dependencies: layer.allowed_dependencies.map(renamed),
    })),
  };
}

/// Ajusta as camadas ao perfil escolhido, sem avaliar regras arquiteturais.
export function setProfile(current: ContractDraft, profile: BuilderProfile): ContractDraft {
  const layers =
    profile === 'modular_monolith'
      ? []
      : current.layers.length > 0
        ? current.layers
        : [
            {
              name: 'principal',
              modules: current.modules.map((module) => module.name),
              allowed_dependencies: [],
            },
          ];
  return { ...current, profile, layers };
}

/// Nome novo determinístico para uma camada adicional.
function nextLayerName(layers: DraftLayer[]): string {
  let index = 1;
  while (layers.some((layer) => layer.name === `camada-${index}`)) {
    index += 1;
  }
  return `camada-${index}`;
}

/// Adiciona uma camada vazia; a associação de módulos continua explícita.
export function addDraftLayer(current: ContractDraft): ContractDraft {
  return {
    ...current,
    layers: [...current.layers, { name: nextLayerName(current.layers), modules: [], allowed_dependencies: [] }],
  };
}

/// Substitui uma camada mantendo as demais intocadas.
export function updateDraftLayer(
  current: ContractDraft,
  name: string,
  patch: Partial<DraftLayer>,
): ContractDraft {
  return {
    ...current,
    layers: current.layers.map((layer) => (layer.name === name ? { ...layer, ...patch } : layer)),
  };
}

/// Inclui ou remove um módulo de uma camada, sem decidir regras.
export function toggleLayerModule(
  current: ContractDraft,
  layerName: string,
  moduleName: string,
): ContractDraft {
  return {
    ...current,
    layers: current.layers.map((layer) => {
      if (layer.name !== layerName) {
        return layer;
      }
      return {
        ...layer,
        modules: layer.modules.includes(moduleName)
          ? layer.modules.filter((item) => item !== moduleName)
          : [...layer.modules, moduleName],
      };
    }),
  };
}

/// Registra a decisão humana sobre uma relação observada.
export function decideDependency(
  current: ContractDraft,
  moduleName: string,
  target: string,
  allow: boolean,
): ContractDraft {
  return updateDraftModule(current, moduleName, {
    allowed_dependencies: allow
      ? [...new Set([...currentModule(current, moduleName).allowed_dependencies, target])]
      : currentModule(current, moduleName).allowed_dependencies.filter((item) => item !== target),
    rejected_dependencies: allow
      ? currentModule(current, moduleName).rejected_dependencies.filter((item) => item !== target)
      : [...new Set([...currentModule(current, moduleName).rejected_dependencies, target])],
  });
}

/// Módulo atual, usado apenas para calcular a próxima lista de escolhas.
function currentModule(current: ContractDraft, name: string): DraftModule {
  return (
    current.modules.find((module) => module.name === name) ?? {
      name,
      path: '',
      responsibility: '',
      allowed_dependencies: [],
      rejected_dependencies: [],
      evidence: [],
      needs_confirmation: true,
    }
  );
}

/// Cria ou substitui a escolha de uma ambiguidade com o mesmo par assunto/detalhe.
export function setResolution(
  current: ContractDraft,
  subject: string,
  detail: string,
  choice: string | null,
): ContractDraft {
  const entry: AmbiguityResolution = { subject, detail, choice };
  const exists = current.resolved_ambiguities.some(
    (resolution) => resolution.subject === subject && resolution.detail === detail,
  );
  return {
    ...current,
    resolved_ambiguities: exists
      ? current.resolved_ambiguities.map((resolution) =>
          resolution.subject === subject && resolution.detail === detail ? entry : resolution,
        )
      : [...current.resolved_ambiguities, entry],
  };
}

/// Ambiguidade de origem, no formato serializado pelo núcleo.
type ResolutionKind = {
  subject: string;
  detail: string;
  options: string[];
};

function decisionResolution(kind: PendingDecisionKind): ResolutionKind | null {
  return 'resolve_ambiguity' in kind ? kind.resolve_ambiguity : null;
}

function decidedDependency(
  kind: PendingDecisionKind,
): { source: string; target: string; module: string } | null {
  return 'decide_dependency' in kind ? kind.decide_dependency : null;
}

/// Traduz as decisões do núcleo em itens editáveis, sem recomputar correspondências.
export function observedDecisionItems(current: ContractDraft): ObservedDecisionItem[] {
  const direct = new Map<string, string>();
  const sourceAmbiguities = new Map<string, ResolutionKind>();
  for (const decision of current.pending_decisions) {
    const dependency = decidedDependency(decision.kind);
    if (dependency) {
      direct.set(`${dependency.source}->${dependency.target}`, dependency.module);
    }
    const resolution = decisionResolution(decision.kind);
    if (resolution?.subject.startsWith('origem da relação:')) {
      sourceAmbiguities.set(resolution.subject, resolution);
    }
  }

  return current.observed_dependencies.map((dependency) => {
    const moduleName = direct.get(`${dependency.source}->${dependency.target}`) ?? null;
    // O assunto é o identificador estável usado pelo núcleo para esta relação.
    const ambiguity =
      moduleName === null
        ? sourceAmbiguities.get(
            `origem da relação: ${dependency.source} -> ${dependency.target}`,
          ) ?? null
        : null;
    const selected =
      current.resolved_ambiguities.find(
        (resolution) =>
          resolution.subject === ambiguity?.subject && resolution.detail === ambiguity?.detail,
      )?.choice ?? null;

    return {
      source: dependency.source,
      target: dependency.target,
      detail: dependency.detail,
      resolution: dependency.resolution,
      line: dependency.line,
      evidence: dependency.evidence,
      moduleName,
      ambiguitySubject: ambiguity?.subject ?? null,
      ambiguityDetail: ambiguity?.detail ?? null,
      options: ambiguity?.options ?? [],
      selected,
    };
  });
}

/// Registra permitir/rejeitar e, quando necessário, a origem da relação.
export function decideObservedDependency(
  current: ContractDraft,
  item: ObservedDecisionItem,
  moduleName: string,
  allow: boolean,
): ContractDraft {
  const decided = decideDependency(current, moduleName, item.target, allow);
  return item.ambiguitySubject === null
    ? decided
    : setResolution(decided, item.ambiguitySubject, item.ambiguityDetail ?? '', moduleName);
}

/// Modo devolvido pelo núcleo, em texto legível.
export function detectedStatus(run: ProjectRunStatus): string {
  if (run.contract.status === 'present') {
    return `Modo: conformidade (contrato v${run.contract.schema_version ?? '?'})`;
  }
  return 'Modo: descoberta (sem contrato)';
}

/// Resultado da validação feita pelo núcleo, sem interpretá-lo.
export function validationMessage(result: ValidationResult): string {
  return result.valid
    ? `Rascunho válido${result.identity ? ` (identidade ${result.identity})` : ''}`
    : `Rascunho inválido: ${result.errors.join('; ')}`;
}

/// Contagens e regras devolvidas pelo núcleo para a tela de revisão.
export function effectsLine(effects: DraftEffects): string {
  return `${effects.modules} módulos · ${effects.layers} camadas · ${effects.allowed_dependencies} dependências permitidas · regras: ${effects.rule_ids.join(', ')}`;
}

/// Evidência longa é resumida sem alterar o texto recebido quando é curta.
export function evidenceLine(evidence: string): string {
  return evidence.length > 120 ? `${evidence.slice(0, 120)}...` : evidence;
}

/// Aviso curto exibido junto às sugestões.
export function warningText(): string {
  return BUILDER_WARNING;
}

/// Símbolo textual de cada linha do diff; nenhum texto é alterado.
export function diffLinePrefix(kind: ContractDiffLineKind): string {
  if (kind === 'added') {
    return '+';
  }
  if (kind === 'removed') {
    return '-';
  }
  return ' ';
}

/// Resumo das contagens devolvidas pelo núcleo.
export function diffSummary(diff: ContractDiff): string {
  if (!diff.changed) {
    return 'Contrato inalterado';
  }
  const added = diff.lines.filter((line) => line.kind === 'added').length;
  const removed = diff.lines.filter((line) => line.kind === 'removed').length;
  return `${added} ${added === 1 ? 'linha adicionada' : 'linhas adicionadas'} · ${removed} ${removed === 1 ? 'linha removida' : 'linhas removidas'}`;
}

/// Autorização enviada ao núcleo, com o texto conferido no painel de diff.
export function saveAuthorization(diff: ContractDiff): SaveAuthorization {
  return diff.exists
    ? { replace: { expected_text: diff.current_text } }
    : 'create';
}

/// Texto da confirmação final, distinto para criação e substituição.
export function saveConfirmationText(diff: ContractDiff): string {
  return diff.exists
    ? 'Substituir auditor.toml com o texto revisado'
    : 'Criar auditor.toml com o texto revisado';
}

/// Observações da descoberta, exibidas sem classificação de conformidade.
export function observationLines(report: DiscoveryReport | null, limit = 8): string[] {
  if (report === null) {
    return [];
  }
  const dependencies = report.candidates.observed_dependencies
    .slice(0, limit)
    .map((dependency) =>
      `${dependency.source} → ${dependency.target} [${dependency.resolution}]`,
    );
  return [...dependencies, ...report.candidates.limitations];
}

/// Achados da auditoria de conformidade executada após a gravação.
export function complianceLines(outcome: SaveOutcome, limit = 8): string[] {
  return outcome.run.report.findings
    .slice(0, limit)
    .map((finding) => `${finding.rule_id}: ${finding.message} (${finding.path})`);
}

/// Linha de identidade do contrato gravado e do modo da auditoria posterior.
export function savedOutcomeLine(outcome: SaveOutcome): string {
  const version = outcome.run.contract.schema_version ?? outcome.saved.schema_version;
  const mode = outcome.run.mode === 'compliance' ? 'conformidade' : outcome.run.mode;
  return `Contrato aprovado v${version} · identidade ${outcome.saved.identity} · modo ${mode}`;
}
