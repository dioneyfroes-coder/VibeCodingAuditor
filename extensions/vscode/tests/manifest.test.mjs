import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const manifest = JSON.parse(
  await readFile(new URL('../package.json', import.meta.url), 'utf8'),
);

test('o menu de pasta não exige auditor.toml', () => {
  const entries = manifest.contributes.menus['explorer/context'];
  const audit = entries.find((entry) => entry.command === 'vibeCodingAuditor.audit');

  assert.ok(audit, 'o comando de auditoria deve aparecer no menu de contexto');
  assert.equal(audit.when, 'explorerResourceIsFolder');
  assert.ok(!JSON.stringify(manifest).includes('projectConfigured'));
});

test('o comando de auditoria continua disponível na paleta', () => {
  assert.ok(
    manifest.contributes.commands.some(
      (command) => command.command === 'vibeCodingAuditor.audit',
    ),
  );
});
