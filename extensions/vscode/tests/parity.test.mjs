import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import { buildDiagnostics, parseCliRun } from '../out/model.js';

/// Mesmo JSON real da CLI usado pelo teste de apresentação do desktop.
const report = parseCliRun(
  JSON.parse(
    await readFile(
      new URL('../../../docs/validation/etapa-3/cli-layer-boundaries.json', import.meta.url),
      'utf8',
    ),
  ),
);

test('diagnósticos da extensão preservam achados, IDs e severidades da CLI', () => {
  const descriptors = buildDiagnostics(report);
  const counts = { error: 0, warning: 0, information: 0 };

  for (const descriptor of descriptors) {
    counts[descriptor.severity] += 1;
    assert.match(descriptor.message, /^\[.+\]/);
  }

  assert.equal(descriptors.length, report.summary.findings);
  assert.deepEqual(
    descriptors.map((descriptor) => descriptor.code),
    report.findings.map((finding) => finding.rule_id),
  );
  assert.deepEqual(
    descriptors.map((descriptor) => descriptor.path),
    report.findings.map((finding) => finding.path),
  );
  assert.deepEqual(
    descriptors.map((descriptor) => descriptor.range.startLine),
    report.findings.map((finding) => finding.line_start - 1),
  );
  assert.equal(counts.error, report.summary.errors);
  assert.equal(counts.warning, report.summary.warnings + report.summary.needs_review);
  assert.equal(counts.information, report.summary.infos);
});

test('cada mensagem de diagnóstico contém o ID do achado correspondente', () => {
  const descriptors = buildDiagnostics(report);

  for (const [index, descriptor] of descriptors.entries()) {
    assert.ok(descriptor.message.startsWith(`[${report.findings[index].id}]`));
  }
});
