import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const releaseCli = path.join(repoRoot, 'target', 'release', 'vibe-coding-auditor-cli.exe');
const debugCli = path.join(repoRoot, 'target', 'debug', 'vibe-coding-auditor-cli.exe');
const cli = existsSync(releaseCli) ? releaseCli : debugCli;
const cmdShortcut = path.join(repoRoot, 'vibe-coding-auditor-cli.cmd');

function runCli(args) {
  if (!existsSync(cli)) {
    throw new Error('CLI não encontrada. Execute cargo build --release -p auditor-cli antes do E2E.');
  }
  const result = spawnSync(cli, args, { cwd: repoRoot, encoding: 'utf8', timeout: 60_000 });
  return {
    code: result.status ?? -1,
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

function fixture(name) {
  return path.join(repoRoot, 'tests', 'fixtures', 'projects', name);
}

function tempDir(label) {
  return mkdtempSync(path.join(tmpdir(), `vca-e2e-${label}-`));
}

test('CLI E2E — ajuda e versão usam o executável real', () => {
  const help = runCli(['--help']);
  assert.equal(help.code, 0, help.stderr);
  assert.match(help.stdout, /Auditor local de arquitetura/);
  assert.match(help.stdout, /Usage:/);

  const version = runCli(['--version']);
  assert.equal(version.code, 0, version.stderr);
  assert.match(version.stdout, /vibe-coding-auditor-cli 0\.1\.0/);
});

test('CLI E2E — execução sem argumentos mostra apresentação e ajuda', () => {
  const run = runCli([]);
  assert.equal(run.code, 0, run.stderr);
  assert.match(run.stdout, /Vibe Coding Auditor/);
  assert.match(run.stdout, /Usage:/);
  assert.equal(run.stderr, '');
});

test('CLI E2E — atalho .cmd repassa argumentos sem pausar', () => {
  const result = spawnSync('cmd', ['/c', cmdShortcut, '--version'], {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 60_000,
  });
  assert.equal(result.status, 0, result.stderr ?? '');
  assert.match(result.stdout ?? '', /vibe-coding-auditor-cli 0\.1\.0/);
  assert.doesNotMatch(result.stdout ?? '', /Pressione qualquer tecla/);
});

test('CLI E2E — init cria contrato sem sobrescrever existente', () => {
  const root = tempDir('init');
  try {
    const first = runCli(['init', root]);
    assert.equal(first.code, 0, first.stderr);
    const contractPath = path.join(root, 'auditor.toml');
    assert.ok(existsSync(contractPath));
    const original = readFileSync(contractPath, 'utf8');
    assert.match(original, /schema_version = 3/);
    assert.match(original, /\[\[architecture\.domains\]\]/);
    assert.match(original, /\[\[architecture\.rules\]\]/);
    assert.match(original, /\[\[architecture\.exceptions\]\]/);

    const second = runCli(['init', root]);
    assert.equal(second.code, 2);
    assert.match(second.stderr, /Falha operacional|já existe/i);
    assert.equal(readFileSync(contractPath, 'utf8'), original);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('CLI E2E — descoberta sem contrato gera JSON estável', () => {
  const run = runCli(['audit', fixture('discovery_no_contract'), '--format', 'json']);
  assert.equal(run.code, 0, run.stderr);
  const report = JSON.parse(run.stdout);
  assert.equal(report.mode, 'discovery');
  assert.equal(report.contract.status, 'missing');
  assert.ok(Array.isArray(report.candidates.candidate_modules));
  assert.ok(Array.isArray(report.candidates.observed_dependencies));
});

test('CLI E2E — auditoria com contrato preserva modo e achados', () => {
  const run = runCli(['audit', fixture('dependencies'), '--format', 'json']);
  assert.equal(run.code, 1);
  const report = JSON.parse(run.stdout);
  assert.equal(report.mode, 'compliance');
  assert.equal(report.contract.status, 'present');
  assert.ok(Array.isArray(report.findings));
  assert.ok(report.findings.length > 0);
});

test('CLI E2E — contrato v3 apresenta exceções sem achados ocultos', () => {
  const run = runCli(['audit', fixture('contract_v3'), '--format', 'json']);
  assert.equal(run.code, 0, run.stderr);
  const report = JSON.parse(run.stdout);
  assert.equal(report.mode, 'compliance');
  assert.equal(report.contract.schema_version, 3);
  assert.ok(Array.isArray(report.findings));
  assert.equal(report.findings.length, 0);
  assert.ok(Array.isArray(report.exceptions_applied));
  assert.equal(report.exceptions_applied.length, 2);
  assert.ok(report.exceptions_applied.some((item) => item.rule === 'dep.domain_not_allowed'));
  assert.ok(report.limitations.some((line) => line.startsWith('exceção aplicada:')));
});

test('CLI E2E — diff salvo é aplicado sem executar Git', () => {
  const patch = path.join(tempDir('diff'), 'changes.patch');
  writeFileSync(
    patch,
    'diff --git a/web/app.ts b/web/app.ts\n--- a/web/app.ts\n+++ b/web/app.ts\n@@ -1 +1 @@\n-old\n+new\n',
  );
  try {
    const run = runCli(['audit', fixture('discovery_candidates'), '--format', 'json', '--diff', patch]);
    assert.equal(run.code, 0, run.stderr);
    assert.match(run.stdout, /--diff não filtrou achados/);
    const report = JSON.parse(run.stdout);
    assert.equal(report.mode, 'discovery');
  } finally {
    rmSync(path.dirname(patch), { recursive: true, force: true });
  }
});

test('CLI E2E — histórico registra descoberta e consulta o banco', () => {
  const root = tempDir('history');
  const database = path.join(root, 'auditor.db');
  try {
    const record = runCli([
      'history',
      '--database',
      database,
      'record',
      fixture('discovery_no_contract'),
    ]);
    assert.equal(record.code, 0, record.stderr);
    assert.match(record.stdout, /Modo: descoberta/);

    const list = runCli(['history', '--database', database, 'list']);
    assert.equal(list.code, 0, list.stderr);
    assert.match(list.stdout, /descoberta/);
    assert.ok(existsSync(database));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
