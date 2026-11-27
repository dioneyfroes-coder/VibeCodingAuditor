//! Modelo de transporte e conversão pura do relatório do núcleo.
//!
//! Este arquivo não importa a API do VS Code para poder ser testado com o
//! executor nativo do Node. A extensão apenas transforma os descritores aqui
//! produzidos em `vscode.Diagnostic`.

/// Severidade conforme serializada pelo núcleo.
export type Severity = 'error' | 'warning' | 'info' | 'needs_review';

/// Classificação usada pelo VS Code, sem dependência da API do editor.
export type DiagnosticSeverityName = 'error' | 'warning' | 'information';

/// Modo explícito serializado pela CLI.
export type ExecutionModeName = 'discovery' | 'compliance';

/// Estado do contrato serializado pela CLI.
export type ContractStatus =
  | { status: 'missing' }
  | { status: 'present'; schema_version: number };

/// Achado devolvido por `auditor-cli audit --format json`.
export interface Finding {
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

/// Contagens devolvidas pelo núcleo.
export interface ReportSummary {
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

/// Relatório completo devolvido pela CLI.
export interface AuditReport {
  project: string;
  schema_version: number;
  summary: ReportSummary;
  findings: Finding[];
  limitations: string[];
  /// Campos aditivos da CLI atual; ausentes em relatórios legados.
  mode?: ExecutionModeName;
  contract?: ContractStatus;
}

/// Relação observada na descoberta, classificada somente pelo núcleo.
export interface ObservedDependency {
  source: string;
  target: string;
  resolution: string;
  detail: string;
  line: number | null;
  evidence: string | null;
}

/// Material de descoberta devolvido pela CLI, sem conclusões de conformidade.
export interface DiscoveryCandidates {
  candidate_modules: unknown[];
  observed_dependencies: ObservedDependency[];
  ambiguities: unknown[];
  languages: unknown[];
  limitations: string[];
}

/// Execução de conformidade, com diagnósticos preservados.
export interface ComplianceRun extends AuditReport {
  mode: 'compliance';
  contract: { status: 'present'; schema_version: number };
}

/// Execução de descoberta, exibida como informação.
export interface DiscoveryRun {
  mode: 'discovery';
  contract: { status: 'missing' };
  candidates: DiscoveryCandidates;
}

/// Forma discriminada da resposta da CLI.
export type CliRun = ComplianceRun | DiscoveryRun;

/// Intervalo zero-indexado pronto para `vscode.Range`.
export interface DiagnosticRange {
  startLine: number;
  startCharacter: number;
  endLine: number;
  endCharacter: number;
}

/// Descritor de diagnóstico independente da API do VS Code.
export interface DiagnosticDescriptor {
  path: string;
  range: DiagnosticRange;
  severity: DiagnosticSeverityName;
  code: string;
  message: string;
}

/// Converte a severidade do núcleo para a severidade do editor.
///
/// `needs_review` vira aviso porque exige decisão humana, mas não é uma
/// violação comprovada; `info` permanece informativo.
export function severityToDiagnosticSeverity(severity: Severity): DiagnosticSeverityName {
  switch (severity) {
    case 'error':
      return 'error';
    case 'warning':
    case 'needs_review':
      return 'warning';
    case 'info':
      return 'information';
    default:
      return 'warning';
  }
}

/// Monta uma mensagem rastreável, com identificador, evidência e sugestão.
export function diagnosticMessage(finding: Finding): string {
  const parts: string[] = [];
  if (finding.evidence) {
    parts.push(`Evidência: ${finding.evidence}`);
  }
  parts.push(`Esperado: ${finding.expected}`);
  parts.push(`Sugestão: ${finding.suggestion}`);
  return `[${finding.id}] ${finding.title}: ${finding.message}\n${parts.join(' | ')}`;
}

/// Converte linhas 1-based do relatório para o intervalo 0-based do editor.
export function diagnosticRange(finding: Finding): DiagnosticRange {
  const startLine = Math.max(0, (finding.line_start ?? 1) - 1);
  const endLine = Math.max(startLine, (finding.line_end ?? finding.line_start ?? 1) - 1);
  return {
    startLine,
    startCharacter: 0,
    endLine,
    endCharacter: Number.MAX_SAFE_INTEGER,
  };
}

/// Converte todos os achados do relatório em descritores de diagnóstico.
export function buildDiagnostics(report: AuditReport): DiagnosticDescriptor[] {
  return report.findings.map((finding) => ({
    path: finding.path,
    range: diagnosticRange(finding),
    severity: severityToDiagnosticSeverity(finding.severity),
    code: finding.rule_id,
    message: diagnosticMessage(finding),
  }));
}

/// Objeto JSON simples, sem interpretar regras nem conteúdo de arquivos.
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/// Interpreta a resposta da CLI pelo modo explícito, sem gerar regras locais.
export function parseCliRun(value: unknown): CliRun {
  if (!isRecord(value)) {
    throw new Error('Resposta da CLI inválida: esperado um objeto.');
  }
  const mode = value.mode;
  if (mode === 'discovery') {
    const contract = value.contract;
    const candidates = value.candidates;
    if (
      !isRecord(contract) ||
      contract.status !== 'missing' ||
      !isRecord(candidates) ||
      !Array.isArray(candidates.observed_dependencies) ||
      !Array.isArray(candidates.limitations)
    ) {
      throw new Error('Resposta de descoberta sem candidatos válidos.');
    }
    return value as unknown as DiscoveryRun;
  }
  if (mode !== 'compliance' && mode !== undefined) {
    throw new Error(`Modo desconhecido na resposta da CLI: ${String(mode)}`);
  }
  if (!Array.isArray(value.findings) || !isRecord(value.summary)) {
    throw new Error('Relatório de conformidade inválido.');
  }
  const contract =
    isRecord(value.contract) && value.contract.status === 'present'
      ? { status: 'present' as const, schema_version: Number(value.contract.schema_version) }
      : { status: 'present' as const, schema_version: Number(value.schema_version) };
  return { ...value, mode: 'compliance', contract } as unknown as ComplianceRun;
}

/// Rótulo neutro da classificação feita pelo núcleo na descoberta.
function resolutionLabel(resolution: string): string {
  switch (resolution) {
    case 'resolved':
      return 'resolvida';
    case 'unresolved':
      return 'não resolvida';
    case 'external':
      return 'externa';
    case 'ambiguous':
      return 'ambígua';
    default:
      return resolution;
  }
}

/// Mensagens informativas da descoberta; nenhuma é um diagnóstico de violação.
export function discoveryMessages(run: DiscoveryRun, limit = 8): string[] {
  const messages = [
    'Pasta sem auditor.toml: modo de descoberta; nenhuma violação foi classificada.',
  ];
  for (const dependency of run.candidates.observed_dependencies.slice(0, limit)) {
    messages.push(
      `${dependency.source} → ${dependency.target} [${resolutionLabel(dependency.resolution)}]`,
    );
  }
  messages.push(...run.candidates.limitations);
  messages.push(
    'Abra o projeto no aplicativo desktop e use o Architecture Builder para criar auditor.toml.',
  );
  return messages;
}

/// Indica se a execução pode alimentar o painel de diagnósticos do editor.
export function hasViolationDiagnostics(run: CliRun): run is ComplianceRun {
  return run.mode === 'compliance';
}

/// Resumo curto para a barra de status e mensagens do editor.
export function runSummaryMessage(run: CliRun): string {
  if (run.mode === 'discovery') {
    const count = run.candidates.observed_dependencies.length;
    return `${count} ${count === 1 ? 'relação observada' : 'relações observadas'} · sem contrato`;
  }
  const count = run.summary.findings;
  return `${count} ${count === 1 ? 'achado' : 'achados'}`;
}

/// Rótulo do modo, idêntico ao significado usado por CLI e desktop.
export function modeLabel(run: CliRun): string {
  return run.mode === 'discovery'
    ? 'descoberta (sem contrato)'
    : `conformidade (contrato v${run.contract.schema_version})`;
}
