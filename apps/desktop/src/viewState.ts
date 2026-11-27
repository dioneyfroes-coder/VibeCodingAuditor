/// Estado da tela que depende do projeto aberto.
///
/// Este módulo apenas descreve o retorno ao estado inicial. Nenhuma regra de
/// auditoria é avaliada aqui e nenhum dado do projeto é reinterpretado.

/// Valores usados ao abrir um projeto diferente e na primeira abertura.
export interface ProjectViewReset {
  projectPath: string;
  report: null;
  discoveryReport: null;
  analysis: null;
  contractText: null;
  runs: never[];
  severityFilter: 'all';
  ruleFilter: 'all';
  statuses: never[];
  historyLoaded: boolean;
  detected: null;
  wizardOpen: boolean;
  activePanel: null;
  showAllFindings: boolean;
  showAllObservations: boolean;
}

/// Devolve o estado limpo sem guardar referências a resultados anteriores.
export function projectViewReset(): ProjectViewReset {
  return {
    projectPath: '',
    report: null,
    discoveryReport: null,
    analysis: null,
    contractText: null,
    runs: [],
    severityFilter: 'all',
    ruleFilter: 'all',
    statuses: [],
    historyLoaded: false,
    detected: null,
    wizardOpen: false,
    activePanel: null,
    showAllFindings: false,
    showAllObservations: false,
  };
}

/// Só troca de projeto limpa a tela; escolher a mesma pasta preserva resultados.
export function isProjectChange(currentPath: string, nextPath: string): boolean {
  return currentPath !== nextPath;
}
