import assert from 'node:assert/strict';
import test from 'node:test';

import { skipSummaryHeadline, skipSummaryLine } from '../out/skipSummary.js';

const summary = {
  total: 9,
  intentional: 6,
  coverage_gaps: 3,
  reasons: [
    {
      reason: 'default_ignored',
      label: 'padrão ignorado',
      count: 4,
      examples: ['.next', '.turbo', '.venv'],
      additional: 1,
      reduces_coverage: false,
    },
    {
      reason: 'excluded',
      label: 'excluído',
      count: 1,
      examples: ['vendor'],
      additional: 0,
      reduces_coverage: false,
    },
    {
      reason: 'not_included',
      label: 'fora do include',
      count: 1,
      examples: ['notes.txt'],
      additional: 0,
      reduces_coverage: false,
    },
    {
      reason: 'symlink',
      label: 'link simbólico',
      count: 1,
      examples: ['atalho.rs'],
      additional: 0,
      reduces_coverage: true,
    },
    {
      reason: 'inaccessible',
      label: 'inacessível',
      count: 1,
      examples: ['bloqueado.rs'],
      additional: 0,
      reduces_coverage: true,
    },
    {
      reason: 'unsupported',
      label: 'tipo não tratado',
      count: 1,
      examples: ['fifo'],
      additional: 0,
      reduces_coverage: true,
    },
  ],
};

test('cabeçalho distingue exclusão intencional de lacuna de cobertura', () => {
  assert.equal(
    skipSummaryHeadline(summary),
    'Motivos dos não incluídos: 9 (6 intencionais, 3 reduzem cobertura)',
  );
  assert.equal(
    skipSummaryHeadline({ ...summary, total: 2, intentional: 1, coverage_gaps: 1 }),
    'Motivos dos não incluídos: 2 (1 intencional, 1 reduz cobertura)',
  );
});

test('linha mostra rótulo, categoria, exemplos limitados e restante', () => {
  assert.equal(
    skipSummaryLine(summary.reasons[0]),
    'padrão ignorado: 4 (intencional); exemplos: .next, .turbo, .venv; +1 outros',
  );
  assert.equal(
    skipSummaryLine(summary.reasons[3]),
    'link simbólico: 1 (reduz cobertura); exemplos: atalho.rs',
  );
  assert.equal(
    skipSummaryLine({ ...summary.reasons[5], examples: [], additional: 0 }),
    'tipo não tratado: 1 (reduz cobertura)',
  );
  assert.ok(!skipSummaryLine(summary.reasons[0]).includes('build/artefato.txt'));
});
