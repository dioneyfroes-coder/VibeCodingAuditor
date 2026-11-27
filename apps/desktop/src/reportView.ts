/// Apresentação determinística de dados já classificados pelo núcleo.
/// Nenhuma regra de auditoria é avaliada neste módulo.

/// Severidade conforme serializada pelo núcleo.
export type Severity = 'error' | 'warning' | 'info' | 'needs_review';

/// Classificação de aresta produzida pela análise.
export type DependencyKind = 'allowed' | 'forbidden' | 'unresolved';

/// Contagens usadas no resumo do relatório.
export interface CountSummary {
  findings: number;
  errors: number;
  warnings: number;
  infos: number;
  needs_review: number;
}

/// Campos necessários para filtrar achados na interface.
export interface FilterableFinding {
  severity: Severity;
  rule_id: string;
}

/// Cobertura devolvida pela análise.
export interface CoverageView {
  code_files_included: number;
  code_files_analyzed: number;
  unsupported_code_extensions: { extension: string; files: number }[];
  partial: boolean;
}

/// Resumo de uma execução gravada, no formato devolvido pelo núcleo.
export interface HistoryRunView {
  recorded_at: string;
  project_name: string;
  execution_mode: 'discovery' | 'compliance';
  contract_identity: string | null;
  summary: CountSummary;
}

/// Aresta devolvida pela análise.
export interface EdgeView {
  from: string;
  to: string;
  kind: DependencyKind;
}

const RELATION_LABELS: Record<DependencyKind, string> = {
  allowed: 'permitida',
  forbidden: 'bloqueada',
  unresolved: 'não identificada',
};

/// Nomes amigáveis para regras conhecidas; identificadores desconhecidos seguem literais.
const RULE_LABELS: Record<string, string> = {
  'dep.not_allowed': 'Dependência não permitida',
  'dep.layer_not_allowed': 'Dependência entre camadas não permitida',
  'dep.unresolved': 'Dependência não identificada',
  'file.unmapped': 'Arquivo sem módulo',
  'file.ambiguous_module': 'Arquivo em mais de um módulo',
  'dep.cycle': 'Ciclo de dependências',
  'module.no_files': 'Módulo sem arquivos',
  'module.concentration': 'Módulo concentrado',
  'code.duplicated': 'Código repetido',
  'module.unreferenced': 'Módulo não usado',
};

/// Traduz apenas o rótulo; nenhuma regra é avaliada ou renomeada no relatório.
export function ruleLabel(ruleId: string): string {
  return RULE_LABELS[ruleId] ?? ruleId;
}

/// Reproduz as contagens do relatório em linguagem corrente, sem reclassificar severidades.
export function reportSummaryPhrase(summary: CountSummary): string {
  if (summary.findings === 0) {
    return 'Nenhum problema encontrado.';
  }
  const groups: string[] = [];
  if (summary.errors > 0) {
    groups.push(`${summary.errors} ${summary.errors === 1 ? 'erro' : 'erros'}`);
  }
  if (summary.warnings > 0) {
    groups.push(`${summary.warnings} ${summary.warnings === 1 ? 'aviso' : 'avisos'}`);
  }
  if (summary.infos > 0) {
    groups.push(`${summary.infos} ${summary.infos === 1 ? 'informação' : 'informações'}`);
  }
  if (summary.needs_review > 0) {
    groups.push(
      `${summary.needs_review} ${summary.needs_review === 1 ? 'item a revisar' : 'itens a revisar'}`,
    );
  }
  const headline = `${summary.findings} ${summary.findings === 1 ? 'problema encontrado' : 'problemas encontrados'}`;
  return groups.length > 0 ? `${headline}: ${groups.join(', ')}` : headline;
}

/// Recorta listas longas para a tela, mantendo os dados originais intactos.
export function previewItems<T>(items: T[], showAll: boolean, limit: number): {
  visible: T[];
  hidden: number;
} {
  const visible = showAll ? items : items.slice(0, limit);
  return { visible, hidden: items.length - visible.length };
}

/// Recorta a lista recebida; `all` mantém todos os itens de cada dimensão.
export function filterFindings<T extends FilterableFinding>(
  findings: T[],
  severityFilter: string,
  ruleFilter: string,
): T[] {
  return findings.filter(
    (finding) =>
      (severityFilter === 'all' || finding.severity === severityFilter) &&
      (ruleFilter === 'all' || finding.rule_id === ruleFilter),
  );
}

/// Lista severidades presentes no relatório, em ordem estável.
export function availableSeverities(findings: FilterableFinding[]): Severity[] {
  return Array.from(new Set(findings.map((finding) => finding.severity))).sort();
}

/// Lista regras presentes no relatório, em ordem estável.
export function availableRules(findings: FilterableFinding[]): string[] {
  return Array.from(new Set(findings.map((finding) => finding.rule_id))).sort();
}

/// Mostra a cobertura exatamente como recebida, em texto mais simples.
export function coverageLabel(coverage: CoverageView): string {
  const included = coverage.code_files_included;
  const files = included === 1 ? 'arquivo' : 'arquivos';
  const checked = included === 1 ? 'verificado' : 'verificados';
  const base = `${coverage.code_files_analyzed} de ${included} ${files} ${checked}`;
  const partial = coverage.partial ? ' (parcial)' : '';
  const unsupported = coverage.unsupported_code_extensions.length
    ? `; sem analisador: ${coverage.unsupported_code_extensions
        .map((item) => `.${item.extension} (${item.files} ${item.files === 1 ? 'arquivo' : 'arquivos'})`)
        .join(', ')}`
    : '';
  return `${base}${partial}${unsupported}`;
}

/// Linha do histórico com modo e contrato, sem inferir contrato ausente.
export function historyRunLine(id: number, run: HistoryRunView): string {
  const mode = run.execution_mode === 'discovery' ? 'descoberta' : 'conformidade';
  const contract = run.execution_mode === 'discovery'
    ? 'sem contrato'
    : run.contract_identity === null
      ? 'contrato não identificado'
      : `contrato ${run.contract_identity}`;
  const findings = `${run.summary.findings} ${run.summary.findings === 1 ? 'problema' : 'problemas'}`;
  return `Execução #${id} — ${run.recorded_at} — ${run.project_name} — ${mode} — ${contract} — ${findings}`;
}

/// Descreve as ligações do mapa usando apenas a classificação do núcleo.
export function dependencyListLabel(edges: EdgeView[]): string {
  return (
    edges
      .map((edge) => `${edge.from} → ${edge.to} (${RELATION_LABELS[edge.kind]})`)
      .join('; ') || 'nenhuma ligação'
  );
}
