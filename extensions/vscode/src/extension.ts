//! Extensão VS Code: converte a resposta da CLI em diagnósticos ou observações.
//!
//! Nenhuma regra é avaliada aqui. A extensão invoca o executável
//! `vibe-coding-auditor`, aceita os códigos 0 e 1 da CLI e usa o `mode`
//! explícito do JSON para decidir entre diagnósticos de conformidade e
//! mensagens informativas de descoberta.

import * as path from 'node:path';
import { existsSync } from 'node:fs';
import { spawn } from 'node:child_process';

import * as vscode from 'vscode';

import {
  CliRun,
  ComplianceRun,
  DiagnosticSeverityName,
  DiscoveryRun,
  Finding,
  buildDiagnostics,
  discoveryMessages,
  hasViolationDiagnostics,
  modeLabel,
  parseCliRun,
  runSummaryMessage,
} from './model';

/// Estado da última execução, guardado no escopo do workspace.
interface StoredRun {
  root: string;
  run: CliRun;
  recordedAt: string;
}

/// Resultado bruto da execução da CLI.
interface CliResult {
  stdout: string;
  stderr: string;
  code: number;
}

/// Item do seletor de achados com o finding original.
interface FindingPick extends vscode.QuickPickItem {
  finding: Finding;
}

let extensionContext!: vscode.ExtensionContext;
let diagnosticCollection!: vscode.DiagnosticCollection;
let statusBar!: vscode.StatusBarItem;
let lastRun: StoredRun | undefined;

/// Raiz da primeira pasta aberta no workspace.
function workspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

/// Converte o nome de severidade puro para o enum da API do VS Code.
function diagnosticSeverity(name: DiagnosticSeverityName): vscode.DiagnosticSeverity {
  switch (name) {
    case 'error':
      return vscode.DiagnosticSeverity.Error;
    case 'warning':
      return vscode.DiagnosticSeverity.Warning;
    case 'information':
      return vscode.DiagnosticSeverity.Information;
    default:
      return vscode.DiagnosticSeverity.Warning;
  }
}

/// Atualiza o item da barra de status com o modo e o resumo da última execução.
function updateStatus(): void {
  if (!lastRun) {
    const root = workspaceRoot();
    const configured = root ? existsSync(path.join(root, 'auditor.toml')) : false;
    statusBar.text = configured ? '$(shield) VCA: sem auditoria' : '$(warning) VCA: sem auditor.toml';
    statusBar.tooltip = configured
      ? 'Execute “Vibe Coding Auditor: Auditar projeto”.'
      : 'A pasta aberta não contém auditor.toml. Use a paleta para executar a descoberta.';
  } else {
    const run = lastRun.run;
    const label = modeLabel(run);
    statusBar.text =
      run.mode === 'compliance' && run.summary.findings > 0
        ? `$(warning) VCA: ${label}, ${runSummaryMessage(run)}`
        : `$(check) VCA: ${label}, ${runSummaryMessage(run)}`;
    const details = [`Última execução: ${lastRun.recordedAt}`, `Modo: ${label}`];
    if (run.mode === 'compliance') {
      details.push(
        `${run.project} · schema v${run.schema_version}`,
        `${run.limitations.length} limitação(ões)`,
      );
    }
    statusBar.tooltip = details.join('\n');
  }
  statusBar.show();
}

/// Aplica uma execução de conformidade aos diagnósticos do editor.
function applyCompliance(
  root: string,
  report: ComplianceRun,
  recordedAt: string,
): void {
  lastRun = { root, run: report, recordedAt };
  const byFile = new Map<string, vscode.Diagnostic[]>();

  for (const descriptor of buildDiagnostics(report)) {
    const uri = vscode.Uri.file(path.resolve(root, descriptor.path));
    const diagnostic = new vscode.Diagnostic(
      new vscode.Range(
        descriptor.range.startLine,
        descriptor.range.startCharacter,
        descriptor.range.endLine,
        descriptor.range.endCharacter,
      ),
      descriptor.message,
      diagnosticSeverity(descriptor.severity),
    );
    diagnostic.source = 'Vibe Coding Auditor';
    diagnostic.code = descriptor.code;

    const key = uri.toString();
    const list = byFile.get(key) ?? [];
    list.push(diagnostic);
    byFile.set(key, list);
  }

  diagnosticCollection.clear();
  for (const [key, list] of byFile) {
    diagnosticCollection.set(vscode.Uri.parse(key), list);
  }
  updateStatus();
}

/// Descoberta limpa diagnósticos antigos e mostra apenas observações do núcleo.
function applyDiscovery(root: string, run: DiscoveryRun, recordedAt: string): void {
  lastRun = { root, run, recordedAt };
  diagnosticCollection.clear();
  updateStatus();
}

/// Distribui a execução conforme o modo explícito devolvido pela CLI.
function applyRun(root: string, run: CliRun, recordedAt: string): void {
  if (hasViolationDiagnostics(run)) {
    applyCompliance(root, run, recordedAt);
  } else {
    applyDiscovery(root, run, recordedAt);
  }
}

/// Reflete a presença de `auditor.toml` no contexto de comandos do VS Code.
async function refreshContext(): Promise<void> {
  const root = workspaceRoot();
  const configured = root ? existsSync(path.join(root, 'auditor.toml')) : false;
  await vscode.commands.executeCommand('setContext', 'vibeCodingAuditor.projectConfigured', configured);
  updateStatus();
}

/// Resolve o executável da CLI a partir da configuração ou do workspace.
async function resolveCli(root: string): Promise<string> {
  const configured =
    vscode.workspace.getConfiguration('vibeCodingAuditor').get<string>('cliPath') ??
    'vibe-coding-auditor';
  if (configured !== 'vibe-coding-auditor' || path.isAbsolute(configured)) {
    return configured;
  }

  const executable = process.platform === 'win32' ? 'vibe-coding-auditor.exe' : 'vibe-coding-auditor';
  for (const candidate of [
    path.join(root, 'target', 'release', executable),
    path.join(root, 'target', 'debug', executable),
  ]) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return configured;
}

/// Executa a CLI sem tratar o código 1 como falha: ele significa “há achados”.
function runCli(command: string, args: string[], cwd: string): Promise<CliResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd });
    let stdout = '';
    let stderr = '';

    child.stdout?.setEncoding('utf8');
    child.stderr?.setEncoding('utf8');
    child.stdout?.on('data', (chunk: string) => {
      stdout += chunk;
    });
    child.stderr?.on('data', (chunk: string) => {
      stderr += chunk;
    });
    child.on('error', reject);
    child.on('close', (code) => resolve({ stdout, stderr, code: code ?? -1 }));
  });
}

/// Mensagem operacional uniforme para falhas inesperadas.
function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/// Auditoria manual da pasta apontada ou da raiz do workspace, em qualquer modo.
async function runAudit(target?: vscode.Uri): Promise<void> {
  const root = target?.fsPath ?? workspaceRoot();
  if (!root) {
    vscode.window.showWarningMessage('Abra uma pasta antes de auditar.');
    return;
  }

  const cli = await resolveCli(root);
  try {
    const result = await runCli(cli, ['audit', root, '--format', 'json'], root);
    if (result.code > 1) {
      throw new Error(result.stderr.trim() || `código de saída ${result.code}`);
    }

    const run = parseCliRun(JSON.parse(result.stdout));
    const recordedAt = new Date().toISOString();
    applyRun(root, run, recordedAt);
    await extensionContext.workspaceState.update('lastRun', { root, run, recordedAt });
    if (run.mode === 'discovery') {
      const messages = discoveryMessages(run);
      for (const message of messages) {
        void vscode.window.showInformationMessage(`VCA · ${message}`);
      }
      void vscode.window.showInformationMessage(
        `VCA · conclusão: nenhuma regra de conformidade foi avaliada. ${messages[messages.length - 1]}`,
      );
    } else {
      vscode.window.showInformationMessage(
        `Auditoria concluída: ${run.summary.findings} achado(s) no painel Problems.`,
      );
    }
  } catch (error) {
    vscode.window.showErrorMessage(`Falha na auditoria: ${errorMessage(error)}`);
  }
}

/// Abre no editor o arquivo e a linha do achado selecionado.
async function openFinding(): Promise<void> {
  if (
    !lastRun ||
    lastRun.run.mode !== 'compliance' ||
    lastRun.run.findings.length === 0
  ) {
    vscode.window.showInformationMessage('Não há achados da última auditoria.');
    return;
  }

  const items: FindingPick[] = lastRun.run.findings.map((finding) => ({
    label: `[${finding.severity}] ${finding.title}`,
    description: `${finding.path}:${finding.line_start ?? 1}`,
    detail: finding.id,
    finding,
  }));
  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: 'Achados da última auditoria',
  });
  if (!picked) {
    return;
  }

  const uri = vscode.Uri.file(path.resolve(lastRun.root, picked.finding.path));
  try {
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);
    const line = Math.max(0, (picked.finding.line_start ?? 1) - 1);
    const position = new vscode.Position(line, 0);
    editor.selection = new vscode.Selection(position, position);
    editor.revealRange(new vscode.Range(position, position), vscode.TextEditorRevealType.InCenter);
  } catch (error) {
    vscode.window.showErrorMessage(
      `Não foi possível abrir ${picked.finding.path}: ${errorMessage(error)}`,
    );
  }
}

/// Remove apenas os diagnósticos; o estado da última execução é preservado.
function clearDiagnostics(): void {
  diagnosticCollection.clear();
}

/// Restaura o estado gravado pela versão atual ou antes da separação de modos.
function restoreRun(value: unknown): StoredRun | undefined {
  if (typeof value !== 'object' || value === null) {
    return undefined;
  }
  const state = value as Record<string, unknown>;
  if (typeof state.root !== 'string' || typeof state.recordedAt !== 'string') {
    return undefined;
  }
  try {
    return { root: state.root, recordedAt: state.recordedAt, run: parseCliRun(state.run) };
  } catch {
    // O formato antigo guardava o relatório diretamente em `report`.
    try {
      const run = parseCliRun(state.report);
      return { root: state.root, recordedAt: state.recordedAt, run };
    } catch {
      return undefined;
    }
  }
}

/// Registra comandos, restaura o último estado e detecta o contrato na abertura.
export function activate(context: vscode.ExtensionContext): void {
  extensionContext = context;
  diagnosticCollection = vscode.languages.createDiagnosticCollection('vibe-coding-auditor');
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBar.command = 'vibeCodingAuditor.openFinding';
  context.subscriptions.push(diagnosticCollection, statusBar);

  context.subscriptions.push(
    vscode.commands.registerCommand('vibeCodingAuditor.audit', (uri?: vscode.Uri) => runAudit(uri)),
    vscode.commands.registerCommand('vibeCodingAuditor.openFinding', () => openFinding()),
    vscode.commands.registerCommand('vibeCodingAuditor.clearDiagnostics', clearDiagnostics),
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      void refreshContext();
    }),
  );

  const previous = restoreRun(context.workspaceState.get('lastRun')) ??
    restoreRun(context.workspaceState.get('lastAudit'));
  if (previous) {
    applyRun(previous.root, previous.run, previous.recordedAt);
  }
  void refreshContext();
}

/// Limpa diagnósticos quando a extensão é desativada.
export function deactivate(): void {
  diagnosticCollection.clear();
}
