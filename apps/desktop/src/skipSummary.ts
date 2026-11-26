/// Identificador estável do motivo, produzido pelo núcleo.
export type SkipReason =
  | 'default_ignored'
  | 'excluded'
  | 'not_included'
  | 'symlink'
  | 'inaccessible'
  | 'unsupported';

/// Contagem e exemplos limitados de um motivo de exclusão.
export interface SkipReasonCount {
  reason: SkipReason;
  label: string;
  count: number;
  examples: string[];
  additional: number;
  reduces_coverage: boolean;
}

/// Resumo estruturado das entradas que ficaram de fora do inventário.
export interface SkipReasonSummary {
  total: number;
  intentional: number;
  coverage_gaps: number;
  reasons: SkipReasonCount[];
}

/// Apresenta o cabeçalho do resumo sem inventar regras de cobertura na interface.
export function skipSummaryHeadline(summary: SkipReasonSummary): string {
  const intentional = summary.intentional === 1 ? 'intencional' : 'intencionais';
  const gaps = summary.coverage_gaps === 1 ? 'reduz cobertura' : 'reduzem cobertura';
  return `Motivos dos não incluídos: ${summary.total} (${summary.intentional} ${intentional}, ${summary.coverage_gaps} ${gaps})`;
}

/// Apresenta um motivo com exemplos limitados e a contagem dos demais.
export function skipSummaryLine(reason: SkipReasonCount): string {
  const category = reason.reduces_coverage ? 'reduz cobertura' : 'intencional';
  const examples =
    reason.examples.length > 0 ? `; exemplos: ${reason.examples.join(', ')}` : '';
  const additional = reason.additional > 0 ? `; +${reason.additional} outros` : '';
  return `${reason.label}: ${reason.count} (${category})${examples}${additional}`;
}
