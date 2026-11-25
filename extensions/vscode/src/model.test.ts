import assert from 'node:assert/strict';
import test from 'node:test';

import {
  AuditReport,
  DiagnosticSeverityName,
  Finding,
  Severity,
  buildDiagnostics,
  diagnosticMessage,
  diagnosticRange,
  severityToDiagnosticSeverity,
} from './model';

/// Achado mínimo com valores explícitos para os testes de conversão.
function finding(overrides: Partial<Finding> = {}): Finding {
  return {
    id: 'dep.not_allowed:domain:src/domain/lib.rs:2',
    rule_id: 'dep.not_allowed',
    severity: 'error' as Severity,
    title: 'Dependência não permitida',
    message: 'O módulo domain usa infrastructure.',
    path: 'src/domain/lib.rs',
    line_start: 2,
    line_end: 4,
    evidence: 'use infrastructure::Client;',
    expected: 'A dependência deve constar em allowed_dependencies.',
    suggestion: 'Ajuste o contrato ou a importação.',
    confidence: null,
    ...overrides,
  };
}

/// Relatório com um achado, usado para verificar a conversão completa.
function reportWith(severity: Severity): AuditReport {
  return {
    project: 'projeto-teste',
    schema_version: 1,
    summary: {
      files_included: 1,
      files_skipped: 0,
      modules: 1,
      packages: 0,
      edges: 1,
      findings: 1,
      errors: 1,
      warnings: 0,
      infos: 0,
      needs_review: 0,
    },
    findings: [finding({ severity })],
    limitations: [],
  };
}

test('severidades do núcleo são mapeadas sem inventar precisão', () => {
  const expected: Record<Severity, DiagnosticSeverityName> = {
    error: 'error',
    warning: 'warning',
    info: 'information',
    needs_review: 'warning',
  };
  for (const severity of Object.keys(expected) as Severity[]) {
    assert.equal(severityToDiagnosticSeverity(severity), expected[severity]);
  }
});

test('linhas 1-based viram intervalos 0-based', () => {
  assert.deepEqual(diagnosticRange(finding()), {
    startLine: 1,
    startCharacter: 0,
    endLine: 3,
    endCharacter: Number.MAX_SAFE_INTEGER,
  });
  assert.deepEqual(diagnosticRange(finding({ line_start: null, line_end: null })), {
    startLine: 0,
    startCharacter: 0,
    endLine: 0,
    endCharacter: Number.MAX_SAFE_INTEGER,
  });
});

test('mensagem traz id, evidência, expectativa e sugestão', () => {
  const message = diagnosticMessage(finding());
  assert.match(message, /^\[dep\.not_allowed:domain:src\/domain\/lib\.rs:2\]/);
  assert.match(message, /Evidência: use infrastructure::Client;/);
  assert.match(message, /Esperado: A dependência deve constar/);
  assert.match(message, /Sugestão: Ajuste o contrato/);
});

test('diagnósticos preservam arquivo, regra e severidade do relatório', () => {
  const report = reportWith('needs_review');
  const descriptors = buildDiagnostics(report);

  assert.equal(descriptors.length, 1);
  assert.equal(descriptors[0].path, 'src/domain/lib.rs');
  assert.equal(descriptors[0].code, 'dep.not_allowed');
  assert.equal(descriptors[0].severity, 'warning');
  assert.equal(descriptors[0].range.startLine, 1);
});
