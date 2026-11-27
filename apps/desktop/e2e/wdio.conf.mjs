//! Configuração WebdriverIO do E2E desktop com o executável Tauri real.
//! Nenhuma regra de auditoria é avaliada aqui: o teste apenas observa a janela.

import { existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, '..', '..', '..');
const appBinaryPath = path.resolve(repoRoot, 'target', 'release', 'auditor-desktop.exe');
const artifacts = path.join(here, 'artifacts');
// Ambientes onde cargo install é bloqueado podem definir TAURI_DRIVER_PATH;
// o CI continua usando o tauri-driver instalado no PATH.
const tauriDriverPath = process.env.TAURI_DRIVER_PATH;

if (!existsSync(appBinaryPath)) {
  throw new Error(`Executável ausente: ${appBinaryPath}. Execute npm --prefix apps/desktop run tauri build antes do E2E.`);
}
mkdirSync(artifacts, { recursive: true });

export const config = {
  runner: 'local',
  specs: [path.join(here, 'specs', '**', '*.spec.mjs')],
  maxInstances: 1,
  capabilities: [
    {
      browserName: 'tauri',
      'tauri:options': { application: appBinaryPath },
    },
  ],
  services: [
    [
      '@wdio/tauri-service',
      {
        appBinaryPath,
        driverProvider: 'external',
        // Permite usar um driver instalado fora do PATH; padrão no CI continua sendo o PATH.
        ...(tauriDriverPath ? { tauriDriverPath } : {}),
        autoInstallTauriDriver: !tauriDriverPath,
        autoDownloadEdgeDriver: true,
        tauriDriverPort: 4444,
        logLevel: 'warn',
        startTimeout: 60_000,
        commandTimeout: 30_000,
        // A captura de logs injeta scripts antes de cada comando e sobrecarrega o driver externo.
        captureBackendLogs: false,
        captureFrontendLogs: false,
      },
    ],
  ],
  logLevel: 'warn',
  bail: 1,
  baseUrl: 'http://localhost:4444',
  waitforTimeout: 10_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 1,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 120_000,
  },
};
