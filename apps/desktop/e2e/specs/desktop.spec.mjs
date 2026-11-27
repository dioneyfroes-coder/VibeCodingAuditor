//! E2E do desktop em janela real: fluxos visíveis, sem duplicar regras do núcleo.

import assert from 'node:assert/strict';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  button,
  captureFailure,
  clickAndSettle,
  projectWithContract,
  projectWithoutContract,
  setDatabase,
  setProjectRoot,
  visiblePanelIds,
  waitForText,
} from '../helpers.mjs';

const database = path.join(mkdtempSync(path.join(tmpdir(), 'vca-desktop-e2e-')), 'auditor.db');

describe('Desktop Tauri — fluxos reais', () => {
  before(async () => {
    // No provedor externo o serviço tenta foco por um plugin ausente; uma troca
    // explícita para a própria janela marca a escolha e evita esse loop.
    await browser.switchToWindow(await browser.getWindowHandle());
  });

  afterEach(async function onTestEnd() {
    if (this.currentTest?.state === 'failed') {
      await captureFailure(this.currentTest.title);
    }
  });

  it('abre com tema escuro e sem resultados anteriores', async () => {
    assert.equal(await (await $('h1')).getText(), 'Vibe Coding Auditor');
    assert.deepEqual(await visiblePanelIds(), []);

    const theme = await browser.execute(() => ({
      scheme: getComputedStyle(document.documentElement).colorScheme,
      background: getComputedStyle(document.body).backgroundColor,
    }));
    assert.equal(theme.scheme, 'dark');
    const channels = theme.background.match(/\d+/g)?.map(Number) ?? [];
    assert.equal(channels.length, 3);
    assert.ok(channels.every((channel) => channel < 100), `fundo não escuro: ${theme.background}`);
  });

  it('detecta e descobre projeto sem contrato', async () => {
    await setProjectRoot(projectWithoutContract);
    await clickAndSettle('Detectar projeto');
    await waitForText('.status-linha', 'descoberta');
    assert.ok(await (await button('Definir arquitetura')).isExisting());

    await clickAndSettle('Auditar / descobrir');
    await waitForText('#painel-descoberta', 'auditor.toml');
    assert.deepEqual(await visiblePanelIds(), ['painel-descoberta']);
    assert.equal(await (await $('#painel-relatorio')).isExisting(), false);
  });

  it('limpa resultados ao trocar de projeto', async () => {
    await setProjectRoot(projectWithContract);
    await browser.waitUntil(
      async () => (await $('.resultado-vazio')).isExisting(),
      { timeout: 20_000, timeoutMsg: 'resultados anteriores não foram limpos' },
    );
    assert.deepEqual(await visiblePanelIds(), []);
    assert.equal(await (await $('nav.result-tabs')).isExisting(), false);
  });

  it('audita com contrato e mantém uma aba por vez', async () => {
    await clickAndSettle('Auditar / descobrir');
    await waitForText('#painel-relatorio', 'Resultado:');
    assert.ok((await $$('#painel-relatorio li')).length > 0, 'relatório deve listar achados');
    assert.deepEqual(await visiblePanelIds(), ['painel-relatorio']);

    await clickAndSettle('Ver mapa');
    await waitForText('#painel-grafo', 'Mapa de módulos');
    assert.deepEqual(await visiblePanelIds(), ['painel-grafo']);

    await clickAndSettle('Ver contrato');
    await waitForText('#painel-contrato', 'Contrato do projeto');
    assert.match(await (await $('#painel-contrato pre.contrato')).getText(), /schema_version\s*=\s*1/);
    assert.deepEqual(await visiblePanelIds(), ['painel-contrato']);
  });

  it('registra execução e consulta o histórico local', async () => {
    await setDatabase(database);
    await clickAndSettle('Registrar execução');
    await waitForText('#painel-historico', 'Histórico local');
    assert.ok((await $$('#painel-historico li')).length > 0, 'histórico deve listar a execução');

    await clickAndSettle('Atualizar histórico');
    await waitForText('#painel-historico', 'Histórico local');
    assert.deepEqual(await visiblePanelIds(), ['painel-historico']);
  });
});
