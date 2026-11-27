import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useState } from 'react';

import BuilderWizard from './BuilderWizard';
import {
  detectedStatus,
  observationLines,
  runModeLabel,
  type DiscoveryReport,
  type ProjectRunStatus,
} from './builderFlow';

import {
  availableRules,
  availableSeverities,
  coverageLabel,
  dependencyListLabel,
  filterFindings,
  historyRunLine,
  previewItems,
  reportSummaryPhrase,
  ruleLabel,
} from './reportView';
import {
  auditStatus,
  builderStatus,
  cancelledSelectionStatus,
  contractStatus,
  errorStatus,
  graphStatus,
  historyStatus,
  recordStatus,
  selectionStatus,
  statusKindLabel,
  statusLine,
  type ActionName,
  type ActionStatus,
} from './status';
import {
  skipSummaryHeadline,
  skipSummaryLine,
  type SkipReasonSummary,
} from './skipSummary';
import { isProjectChange, projectViewReset } from './viewState';

type Severity = 'error' | 'warning' | 'info' | 'needs_review';
type ResultPanel = 'discovery' | 'report' | 'graph' | 'contract' | 'history';
type DependencyKind = 'allowed' | 'forbidden' | 'unresolved';

interface Finding {
  id: string;
  rule_id: string;
  severity: Severity;
  title: string;
  message: string;
  path: string;
  line_start: number | null;
  line_end: number | null;
  evidence: string | null;
  expected: string;
  suggestion: string;
  confidence: string | null;
}

interface UnsupportedCodeExtension {
  extension: string;
  files: number;
}

interface AnalysisCoverage {
  code_files_included: number;
  code_files_analyzed: number;
  code_files_not_analyzed: number;
  unsupported_code_extensions: UnsupportedCodeExtension[];
  partial: boolean;
}

interface ReportSummary {
  files_included: number;
  files_skipped: number;
  modules: number;
  packages: number;
  edges: number;
  findings: number;
  errors: number;
  warnings: number;
  infos: number;
  needs_review: number;
}

interface AuditReport {
  project: string;
  schema_version: number;
  summary: ReportSummary;
  skip_summary: SkipReasonSummary;
  findings: Finding[];
  limitations: string[];
  coverage: AnalysisCoverage;
}

interface DependencyEdge {
  from: string;
  to: string;
  kind: DependencyKind;
  path: string;
  line: number | null;
  evidence: string | null;
  detail: string;
}

interface Analysis {
  modules: string[];
  edges: DependencyEdge[];
  limitations: string[];
  coverage: AnalysisCoverage;
}

interface RunSummary {
  id: number;
  recorded_at: string;
  project_name: string;
  project_root: string;
  summary: ReportSummary;
  execution_mode: 'discovery' | 'compliance';
  contract_identity: string | null;
}

const SEVERITY_LABELS: Record<Severity, string> = {
  error: 'erro',
  warning: 'aviso',
  info: 'informação',
  needs_review: 'a revisar',
};

const EDGE_COLORS: Record<DependencyKind, string> = {
  allowed: '#57d38c',
  forbidden: '#ff7b7b',
  unresolved: '#f2c35c',
};

const GRAPH_WIDTH = 440;
const GRAPH_HEIGHT = 320;
const FINDING_PREVIEW_LIMIT = 5;
const OBSERVATION_PREVIEW_LIMIT = 6;

/// Estado inicial compartilhado pela primeira abertura e pela troca de projeto.
const INITIAL_VIEW = projectViewReset();

/// Posição determinística de um nó no grafo, derivada apenas da ordem dos nomes.
function nodePosition(index: number, total: number): { x: number; y: number } {
  const angle = (2 * Math.PI * index) / Math.max(total, 1) - Math.PI / 2;
  return {
    x: GRAPH_WIDTH / 2 + 150 * Math.cos(angle),
    y: GRAPH_HEIGHT / 2 + 105 * Math.sin(angle),
  };
}

export default function App() {
  const [projectPath, setProjectPath] = useState(INITIAL_VIEW.projectPath);
  const [database, setDatabase] = useState('data/auditor.db');
  const [report, setReport] = useState<AuditReport | null>(INITIAL_VIEW.report);
  const [discoveryReport, setDiscoveryReport] = useState<DiscoveryReport | null>(
    INITIAL_VIEW.discoveryReport,
  );
  const [analysis, setAnalysis] = useState<Analysis | null>(INITIAL_VIEW.analysis);
  const [contractText, setContractText] = useState<string | null>(INITIAL_VIEW.contractText);
  const [runs, setRuns] = useState<RunSummary[]>(INITIAL_VIEW.runs);
  const [severityFilter, setSeverityFilter] = useState<string>(INITIAL_VIEW.severityFilter);
  const [ruleFilter, setRuleFilter] = useState<string>(INITIAL_VIEW.ruleFilter);
  const [statuses, setStatuses] = useState<ActionStatus[]>(INITIAL_VIEW.statuses);
  const [historyLoaded, setHistoryLoaded] = useState(INITIAL_VIEW.historyLoaded);
  const [busy, setBusy] = useState(false);
  const [detected, setDetected] = useState<ProjectRunStatus | null>(INITIAL_VIEW.detected);
  const [wizardOpen, setWizardOpen] = useState(INITIAL_VIEW.wizardOpen);
  const [activePanel, setActivePanel] = useState<ResultPanel | null>(INITIAL_VIEW.activePanel);
  const [showAllFindings, setShowAllFindings] = useState(INITIAL_VIEW.showAllFindings);
  const [showAllObservations, setShowAllObservations] = useState(
    INITIAL_VIEW.showAllObservations,
  );

  const pushStatus = useCallback(
    (status: ActionStatus) => setStatuses((current) => [status, ...current].slice(0, 5)),
    [],
  );

  /// Remove resultados e mensagens de outro projeto sem apagar a configuração local.
  const clearResults = useCallback(() => {
    setReport(INITIAL_VIEW.report);
    setDiscoveryReport(INITIAL_VIEW.discoveryReport);
    setAnalysis(INITIAL_VIEW.analysis);
    setContractText(INITIAL_VIEW.contractText);
    setRuns(INITIAL_VIEW.runs);
    setSeverityFilter(INITIAL_VIEW.severityFilter);
    setRuleFilter(INITIAL_VIEW.ruleFilter);
    setStatuses(INITIAL_VIEW.statuses);
    setHistoryLoaded(INITIAL_VIEW.historyLoaded);
    setDetected(INITIAL_VIEW.detected);
    setWizardOpen(INITIAL_VIEW.wizardOpen);
    setActivePanel(INITIAL_VIEW.activePanel);
    setShowAllFindings(INITIAL_VIEW.showAllFindings);
    setShowAllObservations(INITIAL_VIEW.showAllObservations);
  }, []);

  /// Toda mudança de raiz começa do zero, inclusive quando o caminho é digitado.
  const changeProject = (nextPath: string) => {
    if (!isProjectChange(projectPath, nextPath)) {
      return;
    }
    clearResults();
    setProjectPath(nextPath);
  };

  /// Leva o usuário ao painel gerado pela ação, depois da atualização da tela.
  const reveal = (id: string) =>
    requestAnimationFrame(() => {
      document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    });

  const execute = async (action: ActionName, operation: () => Promise<void>) => {
    setBusy(true);
    try {
      await operation();
    } catch (failure) {
      pushStatus(errorStatus(action, failure));
    } finally {
      setBusy(false);
    }
  };

  const refreshHistory = async () => {
    const history = await invoke<RunSummary[]>('history_list', {
      database,
      project: null,
      limit: 50,
    });
    setRuns(history);
    setHistoryLoaded(true);
    pushStatus(historyStatus(history.length));
  };

  const detectProject = () =>
    execute('builder', async () => {
      const mode = await invoke<ProjectRunStatus>('load_project', { path: projectPath });
      setDetected(mode);
      pushStatus(builderStatus(detectedStatus(mode), 'success'));
    });

  const selectProject = () =>
    execute('select', async () => {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        changeProject(selected);
        pushStatus(selectionStatus(selected));
      } else {
        pushStatus(cancelledSelectionStatus());
      }
    });

  /// Após a gravação aprovada, a interface passa a mostrar conformidade.
  const refreshAfterSave = async () => {
    const mode = await invoke<ProjectRunStatus>('load_project', { path: projectPath });
    setDetected(mode);
    const result = await invoke<AuditReport>('audit_project', { path: projectPath });
    setReport(result);
    setDiscoveryReport(null);
    setAnalysis(null);
    setSeverityFilter(INITIAL_VIEW.severityFilter);
    setRuleFilter(INITIAL_VIEW.ruleFilter);
    setShowAllFindings(INITIAL_VIEW.showAllFindings);
    setActivePanel('report');
  };

  /// Executa o modo escolhido pelo núcleo; nenhum modo é inferido na interface.
  const runAudit = () =>
    execute('audit', async () => {
      const mode = await invoke<ProjectRunStatus>('load_project', { path: projectPath });
      setDetected(mode);
      if (mode.contract.status === 'missing') {
        const discovery = await invoke<DiscoveryReport>('discover_project', { path: projectPath });
        setDiscoveryReport(discovery);
        setReport(null);
        setAnalysis(null);
        setShowAllObservations(INITIAL_VIEW.showAllObservations);
        setActivePanel('discovery');
        pushStatus(builderStatus(runModeLabel(mode), 'success'));
        reveal('resultado');
        return;
      }
      const result = await invoke<AuditReport>('audit_project', { path: projectPath });
      setReport(result);
      setDiscoveryReport(null);
      setAnalysis(null);
      setSeverityFilter(INITIAL_VIEW.severityFilter);
      setRuleFilter(INITIAL_VIEW.ruleFilter);
      setShowAllFindings(INITIAL_VIEW.showAllFindings);
      setActivePanel('report');
      pushStatus(auditStatus(reportSummaryPhrase(result.summary)));
      reveal('resultado');
    });

  const showGraph = () =>
    execute('graph', async () => {
      const result = await invoke<Analysis>('analyze_project', { path: projectPath });
      setAnalysis(result);
      setActivePanel('graph');
      pushStatus(graphStatus(result.modules.length, result.edges.length));
      reveal('resultado');
    });

  const openContract = () =>
    execute('contract', async () => {
      const text = await invoke<string>('read_contract', { path: projectPath });
      setContractText(text);
      setActivePanel('contract');
      pushStatus(contractStatus(text));
      reveal('resultado');
    });

  const recordRun = () =>
    execute('record', async () => {
      const mode = await invoke<ProjectRunStatus>('load_project', { path: projectPath });
      setDetected(mode);
      const id = await invoke<number>('history_record', { path: projectPath, database });
      pushStatus(recordStatus(id, runModeLabel(mode)));
      await refreshHistory();
      setActivePanel('history');
      reveal('resultado');
    });

  // Filtros apenas apresentam o relatório recebido; nenhuma regra é avaliada aqui.
  const severities = report ? availableSeverities(report.findings) : [];
  const rules = report ? availableRules(report.findings) : [];
  const visibleFindings = report
    ? filterFindings(report.findings, severityFilter, ruleFilter)
    : [];

  const graphNodes = analysis
    ? Array.from(new Set([...analysis.modules, ...analysis.edges.map((edge) => edge.to)])).sort()
    : [];
  const nodeIndexOf = (name: string) => graphNodes.indexOf(name);

  const resultTabs: Array<{ id: ResultPanel; label: string; available: boolean }> = [
    { id: 'discovery', label: 'Descoberta', available: discoveryReport !== null },
    { id: 'report', label: 'Relatório', available: report !== null },
    { id: 'graph', label: 'Mapa', available: analysis !== null },
    { id: 'contract', label: 'Contrato', available: contractText !== null },
    { id: 'history', label: 'Histórico', available: historyLoaded },
  ];
  const activeResult = resultTabs.find((tab) => tab.id === activePanel && tab.available) ?? null;
  const findingPreview = previewItems(visibleFindings, showAllFindings, FINDING_PREVIEW_LIMIT);
  const observationPreview = previewItems(
    observationLines(discoveryReport, 50),
    showAllObservations,
    OBSERVATION_PREVIEW_LIMIT,
  );

  return (
    <main className="app-shell">
      <header className="cabecalho">
        <div>
          <h1>Vibe Coding Auditor</h1>
          <p className="subtitle">Auditoria local de arquitetura. Nada é automático.</p>
        </div>
        <p className="detail contexto">
          {projectPath ? `Projeto: ${projectPath}` : 'Selecione uma pasta para começar.'}
        </p>
      </header>

      <div className="workspace">
        <aside className="coluna-lateral">
          <section className="panel" aria-label="Status das ações">
            <h2>Status</h2>
            {statuses.length === 0 ? (
              <p className="detail">Nenhuma ação executada neste projeto.</p>
            ) : (
              <>
                <div className={`status-linha status-${statuses[0].kind}`}>
                  <span className="status-kind">{statusKindLabel(statuses[0].kind)}</span>
                  {statusLine(statuses[0])}
                </div>
                {statuses.length > 1 && (
                  <details className="detalhe-bloco">
                    <summary>Mensagens anteriores ({statuses.length - 1})</summary>
                    <ul className="status-list">
                      {statuses.slice(1).map((status, index) => (
                        <li key={`${status.action}-${index}`} className={`status-${status.kind}`}>
                          <span className="status-kind">{statusKindLabel(status.kind)}</span>
                          {statusLine(status)}
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
              </>
            )}
            {busy && <p className="detail">Operação em andamento…</p>}
          </section>

          <section className="panel">
            <h2>Projeto</h2>
            <label>
              Caminho da raiz
              <input
                value={projectPath}
                onChange={(event) => changeProject(event.target.value)}
                placeholder="C:\caminho\do\projeto"
              />
            </label>
            <p className="detail hint">
              Trocar a pasta limpa os resultados anteriores. Nada é alterado sem sua confirmação.
            </p>
            {projectPath && (
              <p className="detail">
                Contrato: <code>{`${projectPath}\\auditor.toml`}</code>
              </p>
            )}
            <div className="actions">
              <button type="button" className="primario" disabled={busy} onClick={selectProject}>
                Selecionar pasta
              </button>
              <button type="button" disabled={busy || !projectPath} onClick={detectProject}>
                Detectar projeto
              </button>
              {detected && (
                <button
                  type="button"
                  className="primario"
                  disabled={busy}
                  onClick={() => setWizardOpen(true)}
                >
                  {detected.contract.status === 'missing'
                    ? 'Definir arquitetura'
                    : 'Revisar arquitetura'}
                </button>
              )}
              <button
                type="button"
                className="primario"
                disabled={busy || !projectPath}
                onClick={runAudit}
              >
                Auditar / descobrir
              </button>
              <button type="button" disabled={busy || !projectPath} onClick={showGraph}>
                Ver mapa
              </button>
              <button type="button" disabled={busy || !projectPath} onClick={openContract}>
                Ver contrato
              </button>
              <button type="button" disabled={busy || !projectPath} onClick={recordRun}>
                Registrar execução
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={() =>
                  execute('history', async () => {
                    await refreshHistory();
                    setActivePanel('history');
                    reveal('resultado');
                  })
                }
              >
                Atualizar histórico
              </button>
            </div>
          </section>

          <details className="panel detalhe-bloco painel-config">
            <summary>Configurações locais</summary>
            <label>
              Banco de histórico
              <input
                value={database}
                onChange={(event) => setDatabase(event.target.value)}
                placeholder="data/auditor.db"
              />
            </label>
            <p className="detail">
              Caminho relativo ao diretório de execução do aplicativo. O contrato é somente leitura.
            </p>
          </details>
        </aside>

        <section className="coluna-conteudo" id="resultado">

          <section className="panel painel-resultado">
            <div className="resultado-toolbar">
              <div>
                <h2>Resultado</h2>
                <p className="detail">
                  {activeResult
                    ? 'Troque de aba para ver outro resultado sem empilhar painéis.'
                    : 'Execute uma ação para ver o resultado aqui.'}
                </p>
              </div>
              {resultTabs.some((tab) => tab.available) && (
                <nav className="result-tabs" aria-label="Resultados disponíveis">
                  {resultTabs
                    .filter((tab) => tab.available)
                    .map((tab) => (
                      <button
                        key={tab.id}
                        type="button"
                        className={`aba${activeResult?.id === tab.id ? ' ativa' : ''}`}
                        onClick={() => setActivePanel(tab.id)}
                      >
                        {tab.label}
                      </button>
                    ))}
                </nav>
              )}
            </div>

            <div className="resultado-corpo">
              {!activeResult && (
                <div className="resultado-vazio">
                  <p className="detail">
                    Os resultados aparecem aqui. A tela fica em uma aba por vez para evitar rolagem longa.
                  </p>
                </div>
              )}

              {activeResult?.id === 'discovery' && discoveryReport && (
                <div id="painel-descoberta">
          <h2>O que foi encontrado</h2>
          <p className="detail">
            Sem <code>auditor.toml</code>, o programa apenas observa a estrutura. Nenhuma
            dependência foi aprovada e nada foi comparado com regras.
          </p>
          {observationPreview.visible.length === 0 ? (
            <p>Nada a observar nesta pasta.</p>
          ) : (
            <ul className="observacoes">
              {observationPreview.visible.map((line, index) => (
                <li key={`${index}-${line}`}>{line}</li>
              ))}
            </ul>
          )}
          {observationPreview.hidden > 0 && (
            <button type="button" className="secundario" onClick={() => setShowAllObservations(true)}>
              Ver mais {observationPreview.hidden} observações
            </button>
          )}
          {showAllObservations && observationPreview.visible.length > OBSERVATION_PREVIEW_LIMIT && (
            <button
              type="button"
              className="secundario"
              onClick={() => setShowAllObservations(false)}
            >
              Mostrar menos
            </button>
          )}
                  {detected && (
                    <button type="button" className="primario" onClick={() => setWizardOpen(true)}>
                      Definir arquitetura com o assistente
                    </button>
                  )}
                </div>
              )}

              {activeResult?.id === 'report' && report && (
                <div id="painel-relatorio">
          <h2>Resultado: {report.project}</h2>
          <p className="detail">
            {report.summary.files_included} arquivos incluídos · {report.summary.modules} módulos ·{' '}
            {report.summary.packages} pacotes · {report.summary.edges} ligações
          </p>
          <p className="detail">{coverageLabel(report.coverage)}</p>
          {report.skip_summary.total > 0 && (
            <details className="detalhe-bloco">
              <summary>{skipSummaryHeadline(report.skip_summary)}</summary>
              <ul className="detail">
                {report.skip_summary.reasons.map((reason) => (
                  <li key={reason.reason}>{skipSummaryLine(reason)}</li>
                ))}
              </ul>
            </details>
          )}
          <p className="summary">{reportSummaryPhrase(report.summary)}</p>

          <div className="filters">
            <label>
              Severidade
              <select value={severityFilter} onChange={(event) => setSeverityFilter(event.target.value)}>
                <option value="all">todas</option>
                {severities.map((severity) => (
                  <option key={severity} value={severity}>
                    {SEVERITY_LABELS[severity]}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Tipo de problema
              <select value={ruleFilter} onChange={(event) => setRuleFilter(event.target.value)}>
                <option value="all">todos</option>
                {rules.map((rule) => (
                  <option key={rule} value={rule}>
                    {ruleLabel(rule)}
                  </option>
                ))}
              </select>
            </label>
          </div>

          {findingPreview.visible.length === 0 ? (
            <p>Nada a mostrar com esses filtros.</p>
          ) : (
            <>
              <ul className="findings">
                {findingPreview.visible.map((finding) => (
                  <li key={finding.id}>
                    <div className="achado-cabecalho">
                      <span className={`badge severidade-${finding.severity}`}>
                        {SEVERITY_LABELS[finding.severity]}
                      </span>
                      <strong>{finding.title}</strong>
                    </div>
                    <p className="detail">{finding.message}</p>
                    <p className="detail achado-local">
                      Em {finding.path}
                      {finding.line_start === null ? '' : `, linha ${finding.line_start}`}
                    </p>
                    <details className="detalhe-bloco">
                      <summary>Como resolver</summary>
                      <p className="detail">
                        <strong>Tipo:</strong> {ruleLabel(finding.rule_id)}
                      </p>
                      {finding.evidence && (
                        <p className="detail">
                          <strong>O que foi visto:</strong> {finding.evidence}
                        </p>
                      )}
                      <p className="detail">
                        <strong>O que se esperava:</strong> {finding.expected}
                      </p>
                      <p className="detail">
                        <strong>Sugestão:</strong> {finding.suggestion}
                      </p>
                      <p className="detail referencia">Referência: {finding.id}</p>
                    </details>
                  </li>
                ))}
              </ul>
              {findingPreview.hidden > 0 && (
                <button
                  type="button"
                  className="secundario"
                  onClick={() => setShowAllFindings(true)}
                >
                  Ver mais {findingPreview.hidden} problemas
                </button>
              )}
              {showAllFindings && visibleFindings.length > FINDING_PREVIEW_LIMIT && (
                <button
                  type="button"
                  className="secundario"
                  onClick={() => setShowAllFindings(false)}
                >
                  Mostrar menos
                </button>
              )}
            </>
          )}

                  {report.limitations.length > 0 && (
                    <details className="detalhe-bloco">
                      <summary>O que este resultado não cobre ({report.limitations.length})</summary>
                      <ul className="detail">
                        {report.limitations.map((limitation) => (
                          <li key={limitation}>{limitation}</li>
                        ))}
                      </ul>
                    </details>
                  )}
                </div>
              )}

              {activeResult?.id === 'graph' && analysis && (
                <div id="painel-grafo">
          <h2>Mapa de módulos</h2>
          <p className="detail">{coverageLabel(analysis.coverage)}</p>
          <ul className="legenda">
            <li><span className="linha-legenda" style={{ background: EDGE_COLORS.allowed }} /> permitida</li>
            <li><span className="linha-legenda" style={{ background: EDGE_COLORS.forbidden }} /> bloqueada</li>
            <li><span className="linha-legenda" style={{ background: EDGE_COLORS.unresolved }} /> não identificada</li>
          </ul>
          {graphNodes.length === 0 ? (
            <p>Nenhum módulo declarado no contrato.</p>
          ) : (
            <>
              <svg
                className="graph"
                viewBox={`0 0 ${GRAPH_WIDTH} ${GRAPH_HEIGHT}`}
                role="img"
                aria-label="Grafo de dependências entre módulos"
              >
                <defs>
                  <marker
                    id="seta"
                    markerWidth="8"
                    markerHeight="8"
                    refX="7"
                    refY="4"
                    orient="auto"
                  >
                    <path d="M0,0 L8,4 L0,8 Z" fill="#9fb0c9" />
                  </marker>
                </defs>
                {analysis.edges.map((edge) => {
                  const source = nodePosition(nodeIndexOf(edge.from), graphNodes.length);
                  const target = nodePosition(nodeIndexOf(edge.to), graphNodes.length);
                  return (
                    <line
                      key={`${edge.from}-${edge.to}-${edge.path}-${edge.line ?? 0}`}
                      x1={source.x}
                      y1={source.y}
                      x2={target.x}
                      y2={target.y}
                      stroke={EDGE_COLORS[edge.kind]}
                      strokeWidth="2"
                      markerEnd="url(#seta)"
                    />
                  );
                })}
                {graphNodes.map((node, index) => {
                  const position = nodePosition(index, graphNodes.length);
                  const declared = analysis.modules.includes(node);
                  return (
                    <g key={node}>
                      <circle
                        cx={position.x}
                        cy={position.y}
                        r="20"
                        fill={declared ? '#1d3557' : '#3a2028'}
                        stroke={declared ? '#7cc4ff' : '#ff7b7b'}
                        strokeDasharray={declared ? undefined : '4 3'}
                      />
                      <text
                        x={position.x}
                        y={position.y + 4}
                        textAnchor="middle"
                        fontSize="11"
                        fill="#e6edf7"
                      >
                        {node}
                      </text>
                    </g>
                  );
                })}
              </svg>
              <details className="detalhe-bloco">
                <summary>Ver as {analysis.edges.length} ligações</summary>
                <p className="detail">{dependencyListLabel(analysis.edges)}</p>
              </details>
              <p className="detail">Pontos tracejados ficam fora do contrato.</p>
                  {analysis.limitations.length > 0 && (
                    <details className="detalhe-bloco">
                      <summary>O que este mapa não cobre ({analysis.limitations.length})</summary>
                      <ul className="detail">
                        {analysis.limitations.map((limitation) => (
                          <li key={limitation}>{limitation}</li>
                        ))}
                      </ul>
                    </details>
                  )}
                </>
              )}
            </div>
          )}

              {activeResult?.id === 'contract' && contractText !== null && (
                <div id="painel-contrato">
                  <h2>Contrato do projeto</h2>
                  <p className="detail">Somente leitura; a interface não altera este arquivo.</p>
                  <pre className="contrato">{contractText}</pre>
                </div>
              )}

              {activeResult?.id === 'history' && historyLoaded && (
                <div id="painel-historico">
                  <h2>Histórico local</h2>
                  {runs.length === 0 ? (
                    <p className="detail">Nenhuma execução registrada neste banco.</p>
                  ) : (
                    <ol className="historico">
                      {runs.map((run) => (
                        <li key={run.id}>{historyRunLine(run.id, run)}</li>
                      ))}
                    </ol>
                  )}
                </div>
              )}
            </div>
          </section>
        </section>
      </div>

      {wizardOpen && projectPath && (
        <BuilderWizard
          root={projectPath}
          onStatus={pushStatus}
          onClose={() => setWizardOpen(false)}
          onSaved={refreshAfterSave}
        />
      )}
    </main>
  );
}
