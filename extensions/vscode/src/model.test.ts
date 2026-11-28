import assert from 'node:assert/strict';
import test from 'node:test';

import {
  AuditReport,
  ComplianceRun,
  DiagnosticSeverityName,
  DiscoveryRun,
  Finding,
  Severity,
  buildDiagnostics,
  diagnosticMessage,
  diagnosticRange,
  discoveryMessages,
  exceptionMessages,
  hasViolationDiagnostics,
  modeLabel,
  parseCliRun,
  runSummaryMessage,
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

/// Execução de conformidade com o modo explícito produzido pela CLI.
function complianceRun(): ComplianceRun {
  return {
    ...reportWith('error'),
    mode: 'compliance',
    contract: { status: 'present', schema_version: 1 },
  };
}

/// Execução de descoberta com dados brutos produzidos pelo núcleo.
function discoveryRun(): DiscoveryRun {
  return {
    mode: 'discovery',
    contract: { status: 'missing' },
    candidates: {
      candidate_modules: [],
      observed_dependencies: [
        {
          source: 'src/lib.rs',
          target: 'domain',
          resolution: 'resolved',
          detail: 'módulo local',
          line: 2,
          evidence: 'use domain::x;',
        },
        {
          source: 'src/main.rs',
          target: 'serde',
          resolution: 'external',
          detail: 'crate externo',
          line: null,
          evidence: null,
        },
      ],
      ambiguities: [],
      languages: [],
      limitations: ['sem analisador para .rb'],
    },
  };
}

test('JSON da CLI distingue conformidade de descoberta sem reavaliar regras', () => {
  const compliance = parseCliRun(complianceRun());
  assert.equal(compliance.mode, 'compliance');
  assert.equal(hasViolationDiagnostics(compliance), true);
  assert.equal(buildDiagnostics(compliance).length, 1);

  const discovery = parseCliRun(discoveryRun());
  assert.equal(discovery.mode, 'discovery');
  assert.equal(hasViolationDiagnostics(discovery), false);
});

test('JSON legado sem modo continua sendo tratado como conformidade', () => {
  const legacy = parseCliRun(reportWith('warning'));
  assert.equal(legacy.mode, 'compliance');
  assert.deepEqual(legacy.contract, { status: 'present', schema_version: 1 });
});

test('modo desconhecido e descoberta malformada são recusados explicitamente', () => {
  assert.throws(() => parseCliRun({ mode: 'outro' }), /Modo desconhecido/);
  assert.throws(
    () => parseCliRun({ mode: 'discovery', contract: { status: 'missing' } }),
    /candidatos/,
  );
});

test('descoberta mostra observações e orientação, nunca diagnósticos de violação', () => {
  const messages = discoveryMessages(discoveryRun());
  assert.ok(messages.some((message) => message.includes('sem auditor.toml')));
  assert.ok(messages.some((message) => message.includes('src/lib.rs → domain')));
  assert.ok(messages.some((message) => message.includes('externa')));
  assert.ok(messages.some((message) => message.includes('sem analisador para .rb')));
  assert.ok(messages.some((message) => message.includes('Architecture Builder')));
  assert.ok(!messages.some((message) => message.includes('dep.not_allowed')));
});

test('resumo e rótulo do modo vêm dos dados da CLI', () => {
  assert.match(runSummaryMessage(complianceRun()), /1 achado/);
  assert.match(runSummaryMessage(discoveryRun()), /2 relações observadas/);
  assert.equal(modeLabel(complianceRun()), 'conformidade (contrato v1)');
  assert.equal(modeLabel(discoveryRun()), 'descoberta (sem contrato)');
});

test('exceções aplicadas aparecem no resumo e nas mensagens sem ocultar achados', () => {
  const report: AuditReport = {
    ...reportWith('error'),
    exceptions_applied: [
      {
        rule: 'guarda',
        rule_id: 'dep.custom_not_allowed',
        from: 'domain',
        to: 'application',
        path: 'src/domain/lib.rs',
        justification: 'Integração temporária revisada',
        expires_on: '2026-12-31',
        finding_id: 'dep.custom_not_allowed:guarda:1',
        finding_path: 'src/domain/lib.rs',
        finding_line: 7,
      },
    ],
  };
  const run = parseCliRun({
    ...report,
    mode: 'compliance',
    contract: { status: 'present', schema_version: 3 },
  });
  assert.equal(hasViolationDiagnostics(run), true);
  if (!hasViolationDiagnostics(run)) {
    return;
  }

  assert.match(runSummaryMessage(run), /1 achado/);
  assert.match(runSummaryMessage(run), /1 exceção aplicada/);
  assert.equal(buildDiagnostics(run).length, 1);
  const lines = exceptionMessages(run);
  assert.equal(lines.length, 1);
  assert.match(lines[0], /exceção aplicada: guarda domain → application/);
  assert.match(lines[0], /Integração temporária revisada/);
  assert.match(lines[0], /dep\.custom_not_allowed: src\/domain\/lib\.rs/);
  assert.deepEqual(exceptionMessages(reportWith('error')), []);
});

test('novas regras de domínio e personalizada são convertidas sem lógica específica', () => {
  for (const ruleId of ['dep.domain_not_allowed', 'dep.custom_not_allowed']) {
    const report = {
      ...reportWith('needs_review'),
      findings: [
        finding({
          id: `${ruleId}:domain:application:src/domain/lib.rs`,
          rule_id: ruleId,
          title: 'Dependência não permitida pela regra declarada',
          severity: 'needs_review' as Severity,
        }),
      ],
    };
    const [descriptor] = buildDiagnostics(report);
    assert.equal(descriptor.code, ruleId);
    assert.equal(descriptor.severity, 'warning');
    assert.match(descriptor.message, new RegExp(`^\\[${ruleId.replaceAll('.', '\\.')}:`));
  }
});

test('nova regra de camada é convertida sem lógica específica na extensão', () => {
  const layerFinding = finding({
    id: 'dep.layer_not_allowed:web:infrastructure:src/web/lib.rs',
    rule_id: 'dep.layer_not_allowed',
    title: 'Dependência entre camadas não permitida: presentation -> infrastructure',
    message: 'A camada presentation usa infrastructure.',
    path: 'src/web/lib.rs',
    line_start: 3,
    line_end: 3,
    evidence: 'use infrastructure::Database;',
    expected: 'application',
    suggestion: 'Revise o uso de infrastructure.',
  });
  const report = { ...reportWith('error'), findings: [layerFinding] };
  const [descriptor] = buildDiagnostics(report);

  assert.equal(descriptor.code, 'dep.layer_not_allowed');
  assert.equal(descriptor.severity, 'error');
  assert.equal(descriptor.range.startLine, 2);
  assert.match(descriptor.message, /^\[dep\.layer_not_allowed:web:infrastructure:/);
  assert.match(descriptor.message, /Evidência: use infrastructure::Database;/);
});
