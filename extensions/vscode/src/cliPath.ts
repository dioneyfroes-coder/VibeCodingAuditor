/// Resolução declarativa do executável da CLI da extensão.
///
/// Nenhuma regra de auditoria é avaliada aqui: a extensão apenas escolhe qual
/// binário chamar e converte falhas de execução em mensagens operacionais.

import { join, resolve } from 'node:path';

export const DEFAULT_CLI_NAME = 'vibe-coding-auditor-cli';

/// Caminhos tentados na ordem, do mais específico para o `PATH`.
export function cliCandidates(
  configured: string | undefined,
  root: string,
  extensionPath: string,
  platform: NodeJS.Platform,
): string[] {
  const chosen = configured?.trim() || DEFAULT_CLI_NAME;
  if (chosen !== DEFAULT_CLI_NAME) {
    return [chosen];
  }

  const executable = platform === 'win32' ? `${DEFAULT_CLI_NAME}.exe` : DEFAULT_CLI_NAME;
  const fromBase = (base: string) => [
    join(base, 'target', 'release', executable),
    join(base, 'target', 'debug', executable),
  ];

  return [
    ...fromBase(root),
    ...fromBase(resolve(extensionPath, '..', '..')),
    ...fromBase(resolve(extensionPath, '..', '..', '..')),
    DEFAULT_CLI_NAME,
  ];
}

/// Explicação curta quando o sistema não encontra o executável configurado.
export function cliFailureMessage(error: unknown): string {
  const detail = error instanceof Error ? error.message : String(error);
  return /ENOENT/i.test(detail)
    ? `${detail}. Configure vibeCodingAuditor.cliPath com o caminho completo da CLI.`
    : detail;
}
