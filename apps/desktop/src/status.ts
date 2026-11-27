/// Status das ações explícitas da interface.
//!
/// Este módulo apenas descreve o que o núcleo devolveu. Nenhuma regra de
/// auditoria é avaliada aqui e nenhum dado é inventado quando não há resultado.

/// Natureza da mensagem, usada apenas para apresentação.
export type StatusKind = 'success' | 'info' | 'error';

/// Ação que produziu o status.
export type ActionName =
  | 'select'
  | 'audit'
  | 'graph'
  | 'contract'
  | 'record'
  | 'history'
  | 'builder';

/// Resultado legível de uma ação, mantido junto à interface.
export interface ActionStatus {
  action: ActionName;
  kind: StatusKind;
  message: string;
}

const ACTION_LABELS: Record<ActionName, string> = {
  select: 'Pasta',
  audit: 'Auditoria',
  graph: 'Grafo',
  contract: 'Contrato',
  record: 'Registro',
  history: 'Histórico',
  builder: 'Builder',
};

/// Contagem com flexão portuguesa simples, sem alterar os números recebidos.
function countLabel(count: number, singular: string, plural: string): string {
  return `${count} ${count === 1 ? singular : plural}`;
}

/// Linha única exibida na área de status.
export function statusLine(status: ActionStatus): string {
  return `${ACTION_LABELS[status.action]}: ${status.message}`;
}

/// Pasta escolhida pelo diálogo nativo.
export function selectionStatus(path: string): ActionStatus {
  return { action: 'select', kind: 'info', message: path };
}

/// Diálogo fechado sem escolha: ainda é uma resposta ao usuário.
export function cancelledSelectionStatus(): ActionStatus {
  return { action: 'select', kind: 'info', message: 'seleção cancelada' };
}

/// Auditoria concluída, com a frase já produzida a partir do relatório.
export function auditStatus(summaryPhrase: string): ActionStatus {
  return { action: 'audit', kind: 'success', message: summaryPhrase };
}

/// Grafo carregado, com as contagens devolvidas pela análise.
export function graphStatus(modules: number, edges: number): ActionStatus {
  return {
    action: 'graph',
    kind: 'success',
    message: `${countLabel(modules, 'módulo', 'módulos')}, ${countLabel(edges, 'aresta', 'arestas')}`,
  };
}

/// Contrato lido para visualização, sem interpretar o conteúdo.
export function contractStatus(text: string): ActionStatus {
  const label = countLabel(text.length, 'caractere', 'caracteres');
  return {
    action: 'contract',
    kind: 'success',
    message: `${label} lido${text.length === 1 ? '' : 's'}`,
  };
}

/// Execução gravada no banco local, com modo explícito quando informado.
export function recordStatus(id: number, mode?: string): ActionStatus {
  const suffix = mode ? ` em modo ${mode}` : '';
  return {
    action: 'record',
    kind: 'success',
    message: `execução #${id} gravada${suffix}`,
  };
}

/// Consulta ao histórico, inclusive quando não há registros.
export function historyStatus(count: number): ActionStatus {
  return count > 0
    ? {
        action: 'history',
        kind: 'success',
        message: `${countLabel(count, 'execução', 'execuções')} no banco`,
      }
    : { action: 'history', kind: 'info', message: 'nenhuma execução registrada' };
}

/// Estado do assistente: detecção, validação ou falha operacional.
export function builderStatus(message: string, kind: StatusKind = 'info'): ActionStatus {
  return { action: 'builder', kind, message };
}

/// Falha operacional, preservando a mensagem recebida do núcleo.
export function errorStatus(action: ActionName, failure: unknown): ActionStatus {
  return { action, kind: 'error', message: String(failure) };
}
