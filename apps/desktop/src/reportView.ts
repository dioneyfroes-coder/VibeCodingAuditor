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

/// Aresta devolvida pela análise.
export interface EdgeView {
  from: string;
  to: string;
  kind: DependencyKind;
}

const EDGE_LABELS: Record<DependencyKind, string> = {
  allowed: 'permitida',
  forbidden: 'proibida',
  unresolved: 'não resolvida',
};

/// Reproduz as contagens do relatório sem reinterpretar severidades.
export function reportSummaryPhrase(summary: CountSummary): string {
  return `${summary.findings} achados (${summary.errors} erros, ${summary.warnings} avisos, ${summary.infos} informações, ${summary.needs_review} revisões)`;
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

/// Mostra a cobertura exatamente como recebida, sem inferir qualidade.
export function coverageLabel(coverage: CoverageView): string {
  const base = `Cobertura do código: ${coverage.code_files_analyzed} de ${coverage.code_files_included} arquivo(s) analisados`;
  const partial = coverage.partial ? ' (parcial)' : '';
  const unsupported = coverage.unsupported_code_extensions.length
    ? `; sem analisador: ${coverage.unsupported_code_extensions
        .map((item) => `.${item.extension} (${item.files})`)
        .join(', ')}`
    : '';
  return `${base}${partial}${unsupported}`;
}

/// Descreve as arestas do grafo usando apenas a classificação do núcleo.
export function edgeListLabel(edges: EdgeView[]): string {
  return (
    edges.map((edge) => `${edge.from} → ${edge.to} (${EDGE_LABELS[edge.kind]})`).join('; ') ||
    'nenhuma'
  );
}
