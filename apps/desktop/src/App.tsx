import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useState } from 'react';

import BuilderWizard from './BuilderWizard';
import { detectedStatus, type ProjectRunStatus } from './builderFlow';

import {
  availableRules,
  availableSeverities,
  coverageLabel,
  edgeListLabel,
  filterFindings,
  reportSummaryPhrase,
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
  statusLine,
  type ActionName,
  type ActionStatus,
} from './status';
import {
  skipSummaryHeadline,
  skipSummaryLine,
  type SkipReasonSummary,
} from './skipSummary';

type Severity = 'error' | 'warning' | 'info' | 'needs_review';
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
}

const SEVERITY_LABELS: Record<Severity, string> = {
  error: 'erro',
  warning: 'aviso',
  info: 'informação',
  needs_review: 'revisão necessária',
};

const EDGE_COLORS: Record<DependencyKind, string> = {
  allowed: '#2f855a',
  forbidden: '#c53030',
  unresolved: '#b7791f',
};

const GRAPH_WIDTH = 440;
const GRAPH_HEIGHT = 320;

/// Posição determinística de um nó no grafo, derivada apenas da ordem dos nomes.
function nodePosition(index: number, total: number): { x: number; y: number } {
  const angle = (2 * Math.PI * index) / Math.max(total, 1) - Math.PI / 2;
  return {
    x: GRAPH_WIDTH / 2 + 150 * Math.cos(angle),
    y: GRAPH_HEIGHT / 2 + 105 * Math.sin(angle),
  };
}

export default function App() {
  const [projectPath, setProjectPath] = useState('');
  const [database, setDatabase] = useState('data/auditor.db');
  const [report, setReport] = useState<AuditReport | null>(null);
  const [analysis, setAnalysis] = useState<Analysis | null>(null);
  const [contractText, setContractText] = useState<string | null>(null);
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [severityFilter, setSeverityFilter] = useState('all');
  const [ruleFilter, setRuleFilter] = useState('all');
  const [statuses, setStatuses] = useState<ActionStatus[]>([]);
  const [historyLoaded, setHistoryLoaded] = useState(false);
  const [busy, setBusy] = useState(false);
  const [detected, setDetected] = useState<ProjectRunStatus | null>(null);
  const [wizardOpen, setWizardOpen] = useState(false);

  const pushStatus = useCallback(
    (status: ActionStatus) => setStatuses((current) => [status, ...current].slice(0, 5)),
    [],
  );

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
        setProjectPath(selected);
        setDetected(null);
        setWizardOpen(false);
        pushStatus(selectionStatus(selected));
      } else {
        pushStatus(cancelledSelectionStatus());
      }
    });

  const runAudit = () =>
    execute('audit', async () => {
      const result = await invoke<AuditReport>('audit_project', { path: projectPath });
      setReport(result);
      setAnalysis(null);
      setSeverityFilter('all');
      setRuleFilter('all');
      pushStatus(auditStatus(reportSummaryPhrase(result.summary)));
      reveal('painel-relatorio');
    });

  const showGraph = () =>
    execute('graph', async () => {
      const result = await invoke<Analysis>('analyze_project', { path: projectPath });
      setAnalysis(result);
      pushStatus(graphStatus(result.modules.length, result.edges.length));
      reveal('painel-grafo');
    });

  const openContract = () =>
    execute('contract', async () => {
      const text = await invoke<string>('read_contract', { path: projectPath });
      setContractText(text);
      pushStatus(contractStatus(text));
      reveal('painel-contrato');
    });

  const recordRun = () =>
    execute('record', async () => {
      const id = await invoke<number>('history_record', { path: projectPath, database });
      pushStatus(recordStatus(id));
      await refreshHistory();
      reveal('painel-historico');
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

  return (
    <main>
      <header className="cabecalho">
        <h1>Vibe Coding Auditor</h1>
        <p className="subtitle">Auditoria local de arquitetura. Nenhuma operação é automática.</p>
      </header>

      <section className="panel" aria-label="Status das ações">
        <h2>Status</h2>
        {statuses.length === 0 ? (
          <p className="detail">Nenhuma ação executada nesta sessão.</p>
        ) : (
          <ul className="status-list">
            {statuses.map((status, index) => (
              <li key={`${status.action}-${index}`} className={`status-${status.kind}`}>
                <span className="status-kind">{status.kind === 'error' ? 'falha' : status.kind}</span>
                {statusLine(status)}
              </li>
            ))}
          </ul>
        )}
        {busy && <p className="detail">Operação em andamento…</p>}
      </section>

      <section className="panel">
        <h2>Projeto</h2>
        <label>
          Caminho da raiz
          <input
            value={projectPath}
            onChange={(event) => setProjectPath(event.target.value)}
            placeholder="C:\caminho\do\projeto"
          />
        </label>
        <p className="detail hint">
          Selecione a raiz do projeto. “Detectar projeto” distingue descoberta e conformidade; o
          assistente mostra sugestões, mas não grava nenhum arquivo nesta etapa.
        </p>
        {projectPath && (
          <p className="detail">
            Contrato verificado: <code>{`${projectPath}\\auditor.toml`}</code>
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
          <button type="button" className="primario" disabled={busy || !projectPath} onClick={runAudit}>
            Auditar
          </button>
          <button type="button" disabled={busy || !projectPath} onClick={showGraph}>
            Ver grafo
          </button>
          <button type="button" disabled={busy || !projectPath} onClick={openContract}>
            Ver contrato
          </button>
          <button type="button" disabled={busy || !projectPath} onClick={recordRun}>
            Registrar execução
          </button>
          <button type="button" disabled={busy} onClick={() => execute('history', refreshHistory)}>
            Atualizar histórico
          </button>
        </div>
      </section>

      <section className="panel">
        <h2>Configurações locais</h2>
        <label>
          Banco de histórico
          <input
            value={database}
            onChange={(event) => setDatabase(event.target.value)}
            placeholder="data/auditor.db"
          />
        </label>
        <p className="detail">
          O caminho do banco é relativo ao diretório de execução do aplicativo. O contrato é
          visualizado, nunca editado pela interface; não há execução automática em segundo plano.
        </p>
      </section>

      {report && (
        <section className="panel" id="painel-relatorio">
          <h2>Relatório: {report.project}</h2>
          <p>
            Schema v{report.schema_version} · {report.summary.files_included} arquivos incluídos ·{' '}
            {report.summary.modules} módulos · {report.summary.packages} pacotes ·{' '}
            {report.summary.edges} arestas
          </p>
          <p className="detail">{coverageLabel(report.coverage)}</p>
          {report.skip_summary.total > 0 && (
            <div>
              <p className="detail">{skipSummaryHeadline(report.skip_summary)}</p>
              <ul className="detail">
                {report.skip_summary.reasons.map((reason) => (
                  <li key={reason.reason}>{skipSummaryLine(reason)}</li>
                ))}
              </ul>
            </div>
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
              Regra
              <select value={ruleFilter} onChange={(event) => setRuleFilter(event.target.value)}>
                <option value="all">todas</option>
                {rules.map((rule) => (
                  <option key={rule} value={rule}>
                    {rule}
                  </option>
                ))}
              </select>
            </label>
          </div>

          {visibleFindings.length === 0 ? (
            <p>Nenhum achado para os filtros atuais.</p>
          ) : (
            <ul className="findings">
              {visibleFindings.map((finding) => (
                <li key={finding.id}>
                  <strong>
                    <span className={`badge severidade-${finding.severity}`}>
                      {SEVERITY_LABELS[finding.severity]}
                    </span>{' '}
                    {finding.title}
                  </strong>
                  <div className="detail">id: {finding.id}</div>
                  <div className="detail">
                    local: {finding.path}
                    {finding.line_start === null ? '' : `:${finding.line_start}`}
                  </div>
                  <div className="detail">{finding.message}</div>
                  {finding.evidence && <div className="detail">evidência: {finding.evidence}</div>}
                  <div className="detail">esperado: {finding.expected}</div>
                  <div className="detail">sugestão: {finding.suggestion}</div>
                </li>
              ))}
            </ul>
          )}

          {report.limitations.length > 0 && (
            <div>
              <h3>Limitações</h3>
              <ul>
                {report.limitations.map((limitation) => (
                  <li key={limitation}>{limitation}</li>
                ))}
              </ul>
            </div>
          )}
        </section>
      )}

      {analysis && (
        <section className="panel" id="painel-grafo">
          <h2>Grafo de dependências</h2>
          <p className="detail">{coverageLabel(analysis.coverage)}</p>
          <ul className="legenda">
            <li><span className="linha-legenda" style={{ background: EDGE_COLORS.allowed }} /> permitida</li>
            <li><span className="linha-legenda" style={{ background: EDGE_COLORS.forbidden }} /> proibida</li>
            <li><span className="linha-legenda" style={{ background: EDGE_COLORS.unresolved }} /> não resolvida</li>
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
                    <path d="M0,0 L8,4 L0,8 Z" fill="#586273" />
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
                        fill={declared ? '#edf2f7' : '#fff5f5'}
                        stroke={declared ? '#2c5282' : '#c53030'}
                        strokeDasharray={declared ? undefined : '4 3'}
                      />
                      <text
                        x={position.x}
                        y={position.y + 4}
                        textAnchor="middle"
                        fontSize="11"
                        fill="#1c2430"
                      >
                        {node}
                      </text>
                    </g>
                  );
                })}
              </svg>
              <p className="detail">
                Arestas: {edgeListLabel(analysis.edges)}. Nós tracejados não pertencem ao contrato.
              </p>
              {analysis.limitations.length > 0 && (
                <ul>
                  {analysis.limitations.map((limitation) => (
                    <li key={limitation}>{limitation}</li>
                  ))}
                </ul>
              )}
            </>
          )}
        </section>
      )}

      {contractText !== null && (
        <section className="panel" id="painel-contrato">
          <h2>Contrato do projeto</h2>
          <p>Visualização somente leitura; a interface não altera o contrato.</p>
          <pre>{contractText}</pre>
        </section>
      )}

      {wizardOpen && projectPath && (
        <BuilderWizard
          root={projectPath}
          onStatus={pushStatus}
          onClose={() => setWizardOpen(false)}
        />
      )}

      {historyLoaded && (
        <section className="panel" id="painel-historico">
          <h2>Histórico local</h2>
          {runs.length === 0 ? (
            <p className="detail">Nenhuma execução registrada neste banco.</p>
          ) : (
            <ol className="historico">
              {runs.map((run) => (
                <li key={run.id}>
                  <strong>#{run.id}</strong> — {run.recorded_at} — {run.project_name} —{' '}
                  {run.summary.findings} achados
                </li>
              ))}
            </ol>
          )}
        </section>
      )}
    </main>
  );
}
