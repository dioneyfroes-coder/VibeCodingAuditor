import assert from 'node:assert/strict';
import { join, resolve } from 'node:path';
import test from 'node:test';

import { cliCandidates, cliFailureMessage, DEFAULT_CLI_NAME } from '../out/cliPath.js';

const root = join('unidade', 'projeto-antigo');
const extensionPath = join('unidade', 'repositorio', 'extensions', 'vscode');

test('o padrão procura na pasta aberta, no repositório da extensão e no PATH', () => {
  const executable = 'vibe-coding-auditor-cli.exe';
  const candidates = cliCandidates(undefined, root, extensionPath, 'win32');

  assert.deepEqual(candidates, [
    join(root, 'target', 'release', executable),
    join(root, 'target', 'debug', executable),
    join(resolve(extensionPath, '..', '..'), 'target', 'release', executable),
    join(resolve(extensionPath, '..', '..'), 'target', 'debug', executable),
    join(resolve(extensionPath, '..', '..', '..'), 'target', 'release', executable),
    join(resolve(extensionPath, '..', '..', '..'), 'target', 'debug', executable),
    DEFAULT_CLI_NAME,
  ]);
});

test('em sistemas não Windows o nome do executável não tem extensão', () => {
  const candidates = cliCandidates('', root, extensionPath, 'linux');

  assert.equal(candidates[0], join(root, 'target', 'release', DEFAULT_CLI_NAME));
  assert.equal(candidates.at(-1), DEFAULT_CLI_NAME);
});

test('um caminho configurado é o único candidato', () => {
  const configured = join('unidade', 'bin', 'cli.exe');
  assert.deepEqual(cliCandidates(configured, root, extensionPath, 'win32'), [configured]);
  assert.deepEqual(cliCandidates('  ', root, extensionPath, 'win32'),
    cliCandidates(undefined, root, extensionPath, 'win32'));
});

test('falha ENOENT recebe orientação de configuração; outras falhas permanecem literais', () => {
  assert.equal(
    cliFailureMessage(new Error('spawn vibe-coding-auditor-cli ENOENT')),
    'spawn vibe-coding-auditor-cli ENOENT. Configure vibeCodingAuditor.cliPath com o caminho completo da CLI.',
  );
  assert.equal(cliFailureMessage('código de saída 2'), 'código de saída 2');
});
