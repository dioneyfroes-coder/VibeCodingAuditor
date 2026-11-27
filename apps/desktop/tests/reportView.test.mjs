import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

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
} from '../out/reportView.js';

/// Relatório real produzido pela CLI sobre a fixture de camadas.
const report = JSON.parse(
  await readFile(
    new URL('../../../docs/validation/etapa-3/cli-layer-boundaries.json', import.meta.url),
    'utf8',
  ),
);

test('resumo da interface usa linguagem corrente sem alterar contagens', () => {
  assert.equal(
    reportSummaryPhrase(report.summary),
    '3 problemas encontrados: 2 erros, 1 item a revisar',
  );
  assert.equal(
    reportSummaryPhrase({
      findings: 0,
      errors: 0,
      warnings: 0,
      infos: 0,
      needs_review: 0,
    }),
    'Nenhum problema encontrado.',
  );
});

test('prévia limita a apresentação sem recortar os dados recebidos', () => {
  const items = ['a', 'b', 'c'];
  assert.deepEqual(previewItems(items, false, 2), { visible: ['a', 'b'], hidden: 1 });
  assert.deepEqual(previewItems(items, true, 2), { visible: items, hidden: 0 });
  assert.deepEqual(previewItems(items, false, 5), { visible: items, hidden: 0 });
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
  assert.equal(coverageLabel(report.coverage), '5 de 5 arquivos verificados');
  assert.equal(
    coverageLabel({
      ...report.coverage,
      code_files_analyzed: 4,
      code_files_not_analyzed: 1,
      partial: true,
      unsupported_code_extensions: [{ extension: 'go', files: 1 }],
    }),
    '4 de 5 arquivos verificados (parcial); sem analisador: .go (1 arquivo)',
  );
});

test('histórico mostra modo e contrato sem transformar ausência em aprovação', () => {
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
    'Execução #1 — 2026-11-26T10:00:00Z — projeto — conformidade — contrato abc123 — 2 problemas',
  );
  assert.equal(
    historyRunLine(2, discovery),
    'Execução #2 — 2026-11-26T11:00:00Z — projeto — descoberta — sem contrato — 0 problemas',
  );
  assert.equal(
    historyRunLine(3, legacy),
    'Execução #3 — 2026-11-26T10:00:00Z — projeto — conformidade — contrato não identificado — 2 problemas',
  );
});

test('rótulos de regras conhecidas são legíveis e desconhecidas permanecem literais', () => {
  assert.equal(ruleLabel('dep.not_allowed'), 'Dependência não permitida');
  assert.equal(ruleLabel('code.duplicated'), 'Código repetido');
  assert.equal(ruleLabel('regra.nova'), 'regra.nova');
});

test('lista de ligações usa somente as classificações recebidas', () => {
  assert.equal(
    dependencyListLabel([
      { from: 'application', to: 'domain', kind: 'allowed' },
      { from: 'web', to: 'infrastructure', kind: 'forbidden' },
      { from: 'web', to: 'externo', kind: 'unresolved' },
    ]),
    'application → domain (permitida); web → infrastructure (bloqueada); web → externo (não identificada)',
  );
  assert.equal(dependencyListLabel([]), 'nenhuma ligação');
});
