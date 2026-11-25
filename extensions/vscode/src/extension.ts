//! Extensão VS Code: converte o relatório da CLI em diagnósticos do editor.
//!
//! Nenhuma regra é avaliada aqui. A extensão invoca o executável
//! `vibe-coding-auditor`, aceita os códigos 0 e 1 da CLI e transforma o
//! `AuditReport` recebido em diagnósticos navegáveis.

import * as path from 'node:path';
import { existsSync } from 'node:fs';
import { spawn } from 'node:child_process';

import * as vscode from 'vscode';

import { AuditReport, DiagnosticSeverityName, Finding, buildDiagnostics } from './model';

/// Estado da última auditoria, guardado no escopo do workspace.
interface StoredAudit {
  root: string;
  report: AuditReport;
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
let lastAudit: StoredAudit | undefined;

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

/// Atualiza o item da barra de status com o estado da última auditoria.
function updateStatus(): void {
  if (!lastAudit) {
    const root = workspaceRoot();
    const configured = root ? existsSync(path.join(root, 'auditor.toml')) : false;
    statusBar.text = configured ? '$(shield) VCA: sem auditoria' : '$(warning) VCA: sem auditor.toml';
    statusBar.tooltip = configured
      ? 'Execute “Vibe Coding Auditor: Auditar projeto”.'
      : 'A pasta aberta não contém auditor.toml.';
  } else {
    const count = lastAudit.report.summary.findings;
    statusBar.text =
      count === 0 ? '$(check) VCA: 0 achados' : `$(warning) VCA: ${count} achado${count === 1 ? '' : 's'}`;
    statusBar.tooltip = [
      `Última auditoria: ${lastAudit.recordedAt}`,
      `${lastAudit.report.project} · schema v${lastAudit.report.schema_version}`,
      `${count} achado(s); ${lastAudit.report.limitations.length} limitação(ões)`,
    ].join('\n');
  }
  statusBar.show();
}

/// Aplica um relatório do núcleo aos diagnósticos e ao estado local.
function applyAudit(root: string, report: AuditReport, recordedAt: string): void {
  lastAudit = { root, report, recordedAt };
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

/// Auditoria manual da pasta apontada ou da raiz do workspace.
async function runAudit(target?: vscode.Uri): Promise<void> {
  const root = target?.fsPath ?? workspaceRoot();
  if (!root) {
    vscode.window.showWarningMessage('Abra uma pasta antes de auditar.');
    return;
  }
  if (!existsSync(path.join(root, 'auditor.toml'))) {
    vscode.window.showWarningMessage(`Pasta sem auditor.toml: ${root}`);
    return;
  }

  const cli = await resolveCli(root);
  try {
    const result = await runCli(cli, ['audit', root, '--format', 'json'], root);
    if (result.code > 1) {
      throw new Error(result.stderr.trim() || `código de saída ${result.code}`);
    }

    const report = JSON.parse(result.stdout) as AuditReport;
    const recordedAt = new Date().toISOString();
    applyAudit(root, report, recordedAt);
    await extensionContext.workspaceState.update('lastAudit', { root, report, recordedAt });
    vscode.window.showInformationMessage(
      `Auditoria concluída: ${report.summary.findings} achado(s) no painel Problems.`,
    );
  } catch (error) {
    vscode.window.showErrorMessage(`Falha na auditoria: ${errorMessage(error)}`);
  }
}

/// Abre no editor o arquivo e a linha do achado selecionado.
async function openFinding(): Promise<void> {
  if (!lastAudit || lastAudit.report.findings.length === 0) {
    vscode.window.showInformationMessage('Não há achados da última auditoria.');
    return;
  }

  const items: FindingPick[] = lastAudit.report.findings.map((finding) => ({
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

  const uri = vscode.Uri.file(path.resolve(lastAudit.root, picked.finding.path));
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

/// Remove apenas os diagnósticos; o estado da última auditoria é preservado.
function clearDiagnostics(): void {
  diagnosticCollection.clear();
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

  const previous = context.workspaceState.get<StoredAudit>('lastAudit');
  if (previous?.report && previous.root) {
    applyAudit(previous.root, previous.report, previous.recordedAt);
  }
  void refreshContext();
}

/// Limpa diagnósticos quando a extensão é desativada.
export function deactivate(): void {
  diagnosticCollection.clear();
}
