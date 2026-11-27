import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

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
} from '../out/status.js';
import { reportSummaryPhrase } from '../out/reportView.js';

/// Relatório real produzido pela CLI sobre a fixture de camadas.
const report = JSON.parse(
  await readFile(
    new URL('../../../docs/validation/etapa-3/cli-layer-boundaries.json', import.meta.url),
    'utf8',
  ),
);

test('a natureza da mensagem aparece em português', () => {
  assert.equal(statusKindLabel('success'), 'concluído');
  assert.equal(statusKindLabel('info'), 'informação');
  assert.equal(statusKindLabel('error'), 'falha');
});

test('cada ação tem rótulo próprio na linha de status', () => {
  assert.equal(
    statusLine(auditStatus(reportSummaryPhrase(report.summary))),
    'Auditoria: 3 problemas encontrados: 2 erros, 1 item a revisar',
  );
  assert.equal(statusLine(graphStatus(4, 5)), 'Mapa: 4 módulos, 5 ligações');
  assert.equal(statusLine(contractStatus('abc')), 'Contrato: 3 caracteres lidos');
  assert.equal(statusLine(recordStatus(7)), 'Registro: execução #7 gravada');
  assert.equal(
    statusLine(recordStatus(8, 'descoberta (sem contrato)')),
    'Registro: execução #8 gravada em modo descoberta (sem contrato)',
  );
});

test('seleção de pasta informa o caminho e também o cancelamento', () => {
  assert.deepEqual(selectionStatus('C:\\projeto'), {
    action: 'select',
    kind: 'info',
    message: 'C:\\projeto',
  });
  assert.equal(statusLine(cancelledSelectionStatus()), 'Pasta: seleção cancelada');
});

test('mapa e contrato usam singular e plural conforme a contagem', () => {
  assert.equal(statusLine(graphStatus(1, 0)), 'Mapa: 1 módulo, 0 ligações');
  assert.equal(statusLine(contractStatus('x')), 'Contrato: 1 caractere lido');
});

test('o assistente tem rótulo próprio na linha de status', () => {
  assert.equal(
    statusLine(builderStatus('Modo: descoberta (sem contrato)')),
    'Assistente: Modo: descoberta (sem contrato)',
  );
  assert.equal(builderStatus('falha', 'error').kind, 'error');
});

test('histórico vazio é uma informação explícita, não silêncio', () => {
  assert.deepEqual(historyStatus(0), {
    action: 'history',
    kind: 'info',
    message: 'nenhuma execução registrada',
  });
  assert.deepEqual(historyStatus(1), {
    action: 'history',
    kind: 'success',
    message: '1 execução no banco',
  });
  assert.equal(statusLine(historyStatus(2)), 'Histórico: 2 execuções no banco');
});

test('falha operacional mantém a mensagem original e identifica a ação', () => {
  const status = errorStatus('audit', 'contrato: não foi possível ler auditor.toml');
  assert.equal(status.kind, 'error');
  assert.equal(status.action, 'audit');
  assert.equal(
    statusLine(status),
    'Auditoria: contrato: não foi possível ler auditor.toml',
  );
});
