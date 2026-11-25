//! Modelo de transporte e conversão pura do relatório do núcleo.
//!
//! Este arquivo não importa a API do VS Code para poder ser testado com o
//! executor nativo do Node. A extensão apenas transforma os descritores aqui
//! produzidos em `vscode.Diagnostic`.

/// Severidade conforme serializada pelo núcleo.
export type Severity = 'error' | 'warning' | 'info' | 'needs_review';

/// Classificação usada pelo VS Code, sem dependência da API do editor.
export type DiagnosticSeverityName = 'error' | 'warning' | 'information';

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
}

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
