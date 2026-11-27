import { cpSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { runTests } from '@vscode/test-electron';

const here = path.dirname(fileURLToPath(import.meta.url));
const extensionPath = path.resolve(here, '..');
const repoRoot = path.resolve(extensionPath, '..', '..');
const testsPath = path.join(extensionPath, 'out', 'e2e', 'runner.js');
const releaseCli = path.join(repoRoot, 'target', 'release', 'vibe-coding-auditor-cli.exe');
const debugCli = path.join(repoRoot, 'target', 'debug', 'vibe-coding-auditor-cli.exe');
const cliPath = existsSync(releaseCli) ? releaseCli : debugCli;
const version = process.env.VCA_VSCODE_VERSION ?? '1.146.0';

if (!existsSync(testsPath)) {
  throw new Error('Compile a extensão antes do E2E: npm run compile');
}
if (!existsSync(cliPath)) {
  throw new Error('Compile a CLI antes do E2E: cargo build --release -p auditor-cli');
}

const workspace = mkdtempSync(path.join(tmpdir(), 'vca-vscode-e2e-'));
cpSync(path.join(repoRoot, 'tests', 'fixtures', 'projects', 'dependencies'), workspace, {
  recursive: true,
});
rmSync(path.join(workspace, 'auditor.toml'), { force: true });
mkdirSync(path.join(workspace, '.vscode'), { recursive: true });
writeFileSync(
  path.join(workspace, '.vscode', 'settings.json'),
  `${JSON.stringify({ 'vibeCodingAuditor.cliPath': cliPath }, null, 2)}\n`,
);

try {
  await runTests({
    version,
    extensionDevelopmentPath: extensionPath,
    extensionTestsPath: testsPath,
    launchArgs: [
      workspace,
      '--disable-extensions',
      '--disable-gpu',
      '--disable-updates',
      '--skip-welcome',
      '--skip-release-notes',
    ],
  });
} finally {
  rmSync(workspace, { recursive: true, force: true });
}
