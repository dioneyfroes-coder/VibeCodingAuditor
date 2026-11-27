import assert from 'node:assert/strict';
import test from 'node:test';

import { isProjectChange, projectViewReset } from '../out/viewState.js';

test('trocar de projeto limpa todos os resultados e mantém a configuração fora do helper', () => {
  const reset = projectViewReset();

  assert.equal(reset.projectPath, '');
  assert.equal(reset.report, null);
  assert.equal(reset.discoveryReport, null);
  assert.equal(reset.analysis, null);
  assert.equal(reset.contractText, null);
  assert.deepEqual(reset.runs, []);
  assert.equal(reset.severityFilter, 'all');
  assert.equal(reset.ruleFilter, 'all');
  assert.deepEqual(reset.statuses, []);
  assert.equal(reset.historyLoaded, false);
  assert.equal(reset.detected, null);
  assert.equal(reset.wizardOpen, false);
  assert.equal(reset.activePanel, null);
  assert.equal(reset.showAllFindings, false);
  assert.equal(reset.showAllObservations, false);
});

test('cada chamada devolve listas novas, sem compartilhar resultados anteriores', () => {
  const first = projectViewReset();
  const second = projectViewReset();

  assert.notStrictEqual(first.runs, second.runs);
  assert.notStrictEqual(first.statuses, second.statuses);
});

test('escolher a mesma pasta preserva resultados; pasta diferente dispara a limpeza', () => {
  assert.equal(isProjectChange('C:\\projeto', 'C:\\projeto'), false);
  assert.equal(isProjectChange('C:\\projeto', 'C:\\outro'), true);
  assert.equal(isProjectChange('', 'C:\\projeto'), true);
});
