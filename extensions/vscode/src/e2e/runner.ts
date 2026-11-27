//! Runner executado dentro do Extension Host em host real do VS Code.
//!
//! Não avalia regras: apenas aciona comandos e observa efeitos públicos da
//! extensão (diagnósticos, API mínima de observação e configuração).

import assert from 'node:assert/strict';
import { copyFileSync, existsSync } from 'node:fs';
import path from 'node:path';

import * as vscode from 'vscode';

import type { VcaExtensionApi } from '../extension';

const EXTENSION_ID = 'dioneyfroes-coder.vibe-coding-auditor';

/// Log da extensão incluído em falhas para diagnóstico no host real.
let extensionOutput = () => '<log indisponível>';

/// Espera por uma condição observável, sem sleep fixo grande.
async function waitFor(label: string, predicate: () => boolean, timeoutMs = 20_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Tempo esgotado esperando: ${label}`);
}

/// Soma todos os diagnósticos publicados no host.
function diagnosticCount(): number {
  return vscode.languages.getDiagnostics().reduce((total, entry) => total + entry[1].length, 0);
}

/// Executa a auditoria sem aguardar o popup informativo do VS Code.
function startAudit(): void {
  void vscode.commands.executeCommand('vibeCodingAuditor.audit');
}

/// Aceita o primeiro achado no QuickPick e confirma a navegação do editor.
async function openFirstFinding(uris: vscode.Uri[]): Promise<void> {
  void vscode.commands.executeCommand('vibeCodingAuditor.openFinding');
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    await vscode.commands.executeCommand('workbench.action.acceptSelectedQuickOpenItem');
    const active = vscode.window.activeTextEditor?.document.uri.fsPath;
    if (active && uris.some((uri) => uri.fsPath === active)) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error('Tempo esgotado abrindo o primeiro achado no editor');
}

/// Ponto de entrada exigido por @vscode/test-electron.
export async function run(): Promise<void> {
  try {
    await runChecks();
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`${detail}\n\n[canal Vibe Coding Auditor]\n${extensionOutput()}`);
  }
}

/// Asserções do fluxo real; separadas para anexar o log em caso de falha.
async function runChecks(): Promise<void> {
  const extension = vscode.extensions.getExtension<VcaExtensionApi>(EXTENSION_ID);
  assert.ok(extension, `extensão ${EXTENSION_ID} não foi carregada`);
  const api = await extension.activate();
  extensionOutput = () => api.getOutputLog();
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, 'o runner E2E precisa de uma pasta aberta');

  const root = workspaceFolder.uri.fsPath;
  assert.equal(existsSync(path.join(root, 'auditor.toml')), false);

  startAudit();
  await waitFor(
    'observações de descoberta no canal',
    () => /descoberta em/.test(api.getOutputLog()),
  );
  await waitFor(
    'modo descoberta',
    () => api.getLastRunMode() === 'discovery',
  );
  assert.equal(diagnosticCount(), 0);
  assert.match(api.getOutputLog(), /sem auditor\.toml|sem contrato/i);

  const contractSource = path.resolve(
    extension.extensionPath,
    '..',
    '..',
    'tests',
    'fixtures',
    'projects',
    'dependencies',
    'auditor.toml',
  );
  assert.ok(existsSync(contractSource), `fixture ausente: ${contractSource}`);
  copyFileSync(contractSource, path.join(root, 'auditor.toml'));

  startAudit();
  await waitFor('modo conformidade', () => api.getLastRunMode() === 'compliance');
  await waitFor('diagnósticos de conformidade', () => diagnosticCount() > 0);
  const diagnostics = vscode.languages.getDiagnostics();
  const findingUris = diagnostics
    .filter(([, items]) => items.some((item) => String(item.code ?? '').startsWith('dep.')))
    .map(([uri]) => uri);
  assert.ok(findingUris.length > 0, 'pelo menos um diagnóstico deve manter o código da regra');
  await openFirstFinding(findingUris);

  await vscode.commands.executeCommand('vibeCodingAuditor.clearDiagnostics');
  assert.equal(diagnosticCount(), 0);

  const configuration = vscode.workspace.getConfiguration('vibeCodingAuditor');
  const originalCliPath = configuration.get<string>('cliPath');
  await configuration.update(
    'cliPath',
    path.join(root, 'cli-inexistente.exe'),
    vscode.ConfigurationTarget.Workspace,
  );
  try {
    startAudit();
    await waitFor('falha com CLI ausente', () => /Configure vibeCodingAuditor\.cliPath/.test(api.getOutputLog()));
  } finally {
    await configuration.update(
      'cliPath',
      originalCliPath,
      vscode.ConfigurationTarget.Workspace,
    );
  }
}
