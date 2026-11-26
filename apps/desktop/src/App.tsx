import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

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

type SkipReason =
  | 'default_ignored'
  | 'excluded'
  | 'not_included'
  | 'symlink'
  | 'inaccessible'
  | 'unsupported';

interface SkipReasonCount {
  reason: SkipReason;
  label: string;
  count: number;
  examples: string[];
  additional: number;
  reduces_coverage: boolean;
}

interface SkipReasonSummary {
  total: number;
  intentional: number;
  coverage_gaps: number;
  reasons: SkipReasonCount[];
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

const EDGE_LABELS: Record<DependencyKind, string> = {
  allowed: 'permitida',
  forbidden: 'proibida',
  unresolved: 'não resolvida',
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

/// Apresenta a cobertura recebida do núcleo, sem interpretar regras na interface.
function coverageLabel(coverage: AnalysisCoverage): string {
  const base = `Cobertura do código: ${coverage.code_files_analyzed} de ${coverage.code_files_included} arquivo(s) analisados`;
  const partial = coverage.partial ? ' (parcial)' : '';
  const unsupported = coverage.unsupported_code_extensions.length
    ? `; sem analisador: ${coverage.unsupported_code_extensions
        .map((item) => `.${item.extension} (${item.files})`)
        .join(', ')}`
    : '';
  return `${base}${partial}${unsupported}`;
}

/// Apresenta o cabeçalho do resumo de exclusões sem inventar regras de cobertura.
function skipSummaryHeadline(summary: SkipReasonSummary): string {
  const intentional = summary.intentional === 1 ? 'intencional' : 'intencionais';
  const gaps = summary.coverage_gaps === 1 ? 'reduz cobertura' : 'reduzem cobertura';
  return `Motivos dos não incluídos: ${summary.total} (${summary.intentional} ${intentional}, ${summary.coverage_gaps} ${gaps})`;
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
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const execute = async (operation: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await operation();
    } catch (failure) {
      setError(String(failure));
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
  };

  const selectProject = () =>
    execute(async () => {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        setProjectPath(selected);
      }
    });

  const runAudit = () =>
    execute(async () => {
      const result = await invoke<AuditReport>('audit_project', { path: projectPath });
      setReport(result);
      setAnalysis(null);
      setSeverityFilter('all');
      setRuleFilter('all');
    });

  const showGraph = () =>
    execute(async () => {
      setAnalysis(await invoke<Analysis>('analyze_project', { path: projectPath }));
    });

  const openContract = () =>
    execute(async () => {
      setContractText(await invoke<string>('read_contract', { path: projectPath }));
    });

  const recordRun = () =>
    execute(async () => {
      await invoke<number>('history_record', { path: projectPath, database });
      await refreshHistory();
    });

  // Filtros apenas apresentam o relatório recebido; nenhuma regra é avaliada aqui.
  const severities = report
    ? Array.from(new Set(report.findings.map((finding) => finding.severity))).sort()
    : [];
  const rules = report
    ? Array.from(new Set(report.findings.map((finding) => finding.rule_id))).sort()
    : [];
  const visibleFindings = report
    ? report.findings.filter(
        (finding) =>
          (severityFilter === 'all' || finding.severity === severityFilter) &&
          (ruleFilter === 'all' || finding.rule_id === ruleFilter),
      )
    : [];

  const graphNodes = analysis
    ? Array.from(new Set([...analysis.modules, ...analysis.edges.map((edge) => edge.to)])).sort()
    : [];
  const nodeIndexOf = (name: string) => graphNodes.indexOf(name);

  return (
    <main>
      <h1>Vibe Coding Auditor</h1>
      <p className="subtitle">Auditoria local de arquitetura. Nenhuma operação é automática.</p>

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
        <div className="actions">
          <button type="button" disabled={busy} onClick={selectProject}>
            Selecionar pasta
          </button>
          <button type="button" disabled={busy || !projectPath} onClick={runAudit}>
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
          <button type="button" disabled={busy} onClick={() => execute(refreshHistory)}>
            Atualizar histórico
          </button>
        </div>
        {error && <p className="error">Falha operacional: {error}</p>}
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
        <section className="panel">
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
                  <li key={reason.reason}>
                    {reason.label}: {reason.count} (
                    {reason.reduces_coverage ? 'reduz cobertura' : 'intencional'})
                    {reason.examples.length > 0 &&
                      `; exemplos: ${reason.examples.join(', ')}`}
                    {reason.additional > 0 && `; +${reason.additional} outros`}
                  </li>
                ))}
              </ul>
            </div>
          )}
          <p className="summary">
            {report.summary.findings} achados ({report.summary.errors} erros,{' '}
            {report.summary.warnings} avisos, {report.summary.infos} informações,{' '}
            {report.summary.needs_review} revisões)
          </p>

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
                    [{SEVERITY_LABELS[finding.severity]}] {finding.title}
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
        <section className="panel">
          <h2>Grafo de dependências</h2>
          <p className="detail">{coverageLabel(analysis.coverage)}</p>
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
                Arestas:{' '}
                {analysis.edges
                  .map((edge) => `${edge.from} → ${edge.to} (${EDGE_LABELS[edge.kind]})`)
                  .join('; ') || 'nenhuma'}
                . Nós tracejados não pertencem ao contrato.
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
        <section className="panel">
          <h2>Contrato do projeto</h2>
          <p>Visualização somente leitura; a interface não altera o contrato.</p>
          <pre>{contractText}</pre>
        </section>
      )}

      {runs.length > 0 && (
        <section className="panel">
          <h2>Histórico local</h2>
          <ol>
            {runs.map((run) => (
              <li key={run.id}>
                #{run.id} — {run.recorded_at} — {run.project_name} — {run.summary.findings} achados
              </li>
            ))}
          </ol>
        </section>
      )}
    </main>
  );
}
