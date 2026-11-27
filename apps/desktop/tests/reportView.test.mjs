import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import {
  availableRules,
  availableSeverities,
  coverageLabel,
  edgeListLabel,
  filterFindings,
  historyRunLine,
  reportSummaryPhrase,
} from '../out/reportView.js';

/// Relatório real produzido pela CLI sobre a fixture de camadas.
const report = JSON.parse(
  await readFile(
    new URL('../../../docs/validation/etapa-3/cli-layer-boundaries.json', import.meta.url),
    'utf8',
  ),
);

test('resumo da interface repete exatamente as contagens do núcleo', () => {
  assert.equal(
    reportSummaryPhrase(report.summary),
    '3 achados (2 erros, 0 avisos, 0 informações, 1 revisões)',
  );
});

test('filtros da interface apenas recortam os achados recebidos', () => {
  assert.deepEqual(
    filterFindings(report.findings, 'error', 'all').map((finding) => finding.id),
    [
      'dep.layer_not_allowed:application:infrastructure:src/application/rules.rs',
      'dep.not_allowed:web:infrastructure:src/web/lib.rs',
    ],
  );
  assert.deepEqual(
    filterFindings(report.findings, 'all', 'dep.layer_not_allowed').map((finding) => finding.id),
    ['dep.layer_not_allowed:application:infrastructure:src/application/rules.rs'],
  );
  assert.deepEqual(
    filterFindings(report.findings, 'needs_review', 'dep.layer_not_allowed'),
    [],
  );
  assert.equal(filterFindings(report.findings, 'all', 'all').length, report.findings.length);
});

test('opções dos filtros vêm somente dos achados do relatório', () => {
  assert.deepEqual(availableSeverities(report.findings), ['error', 'needs_review']);
  assert.deepEqual(availableRules(report.findings), [
    'dep.layer_not_allowed',
    'dep.not_allowed',
    'dep.unresolved',
  ]);
});

test('cobertura exibida usa os dados da análise, sem inferir conclusão', () => {
  assert.equal(coverageLabel(report.coverage), 'Cobertura do código: 5 de 5 arquivo(s) analisados');
  assert.equal(
    coverageLabel({
      ...report.coverage,
      code_files_included: 5,
      code_files_analyzed: 4,
      code_files_not_analyzed: 1,
      partial: true,
      unsupported_code_extensions: [{ extension: 'go', files: 1 }],
    }),
    'Cobertura do código: 4 de 5 arquivo(s) analisados (parcial); sem analisador: .go (1)',
  );
});

test('histórico mostra modo e identidade sem transformar ausência em contrato', () => {
  const compliance = {
    recorded_at: '2026-11-26T10:00:00Z',
    project_name: 'projeto',
    execution_mode: 'compliance',
    contract_identity: 'abc123',
    summary: { findings: 2 },
  };
  const discovery = {
    recorded_at: '2026-11-26T11:00:00Z',
    project_name: 'projeto',
    execution_mode: 'discovery',
    contract_identity: null,
    summary: { findings: 0 },
  };
  const legacy = { ...compliance, contract_identity: null };

  assert.equal(
    historyRunLine(1, compliance),
    '#1 — 2026-11-26T10:00:00Z — projeto — conformidade — identidade abc123 — 2 achados',
  );
  assert.equal(
    historyRunLine(2, discovery),
    '#2 — 2026-11-26T11:00:00Z — projeto — descoberta (sem contrato) — sem contrato — 0 achados',
  );
  assert.equal(
    historyRunLine(3, legacy),
    '#3 — 2026-11-26T10:00:00Z — projeto — conformidade — identidade desconhecida — 2 achados',
  );
});

test('lista de arestas usa somente as classificações recebidas', () => {
  assert.equal(
    edgeListLabel([
      { from: 'application', to: 'domain', kind: 'allowed' },
      { from: 'web', to: 'infrastructure', kind: 'forbidden' },
      { from: 'web', to: 'externo', kind: 'unresolved' },
    ]),
    'application → domain (permitida); web → infrastructure (proibida); web → externo (não resolvida)',
  );
  assert.equal(edgeListLabel([]), 'nenhuma');
});
