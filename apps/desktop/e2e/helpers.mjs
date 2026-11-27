//! Helpers de observação da janela real; nenhuma regra é reimplementada aqui.

import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

/// Fixture com contrato, usada para o modo de conformidade.
export const projectWithContract = path.resolve(
  here,
  '..',
  '..',
  '..',
  'tests',
  'fixtures',
  'projects',
  'dependencies',
);

/// Fixture sem contrato, usada para o modo de descoberta.
export const projectWithoutContract = path.resolve(
  here,
  '..',
  '..',
  '..',
  'tests',
  'fixtures',
  'projects',
  'discovery_no_contract',
);

/// Diretório de artefatos de falhas, criado pela configuração.
export const artifacts = path.join(here, 'artifacts');

/// Espera por texto observável em um seletor, usando timeouts do WebDriver.
export async function waitForText(selector, expected, timeout = 20_000) {
  const element = await $(selector);
  await element.waitForExist({ timeout });
  await browser.waitUntil(
    async () => (await element.getText()).includes(expected),
    { timeout, timeoutMsg: `Texto "${expected}" não apareceu em ${selector}` },
  );
  return element;
}

/// Seleciona um botão pelo texto visível, sem depender de classes internas.
export function button(label) {
  return $(`//button[normalize-space(.)="${label}"]`);
}

/// Clica em um botão e espera a interface sair do estado de ocupado.
export async function clickAndSettle(label) {
  const target = await button(label);
  await target.waitForEnabled({ timeout: 20_000 });
  await target.click();
  await browser.waitUntil(
    async () => (await $$('//button[@disabled]')).length === 0,
    { timeout: 30_000, timeoutMsg: `Ação "${label}" não concluiu` },
  );
}

/// Troca a raiz pelo campo de texto, equivalente à seleção de pasta sem diálogo nativo.
export async function setProjectRoot(root) {
  const inputs = await $$('input');
  assert.ok(inputs.length >= 1, 'o campo de caminho da raiz deve existir');
  await inputs[0].setValue(root);
}

/// Aponta o histórico para um banco temporário fora do repositório.
export async function setDatabase(databasePath) {
  const settings = await $('details.painel-config summary');
  await settings.click();
  const inputs = await $$('details.painel-config input');
  assert.equal(inputs.length, 1, 'o campo do banco de histórico deve existir');
  await inputs[0].setValue(databasePath);
}

/// Conta os painéis de resultado simultaneamente montados no DOM.
export async function visiblePanelIds() {
  // O map de ElementArray do WDIO devolve um encadeável, não um array comum.
  const panels = Array.from(await $$('#resultado [id^="painel-"]'));
  return Promise.all(panels.map((panel) => panel.getAttribute('id')));
}

/// Registra screenshot em falha para o artefato de CI.
export async function captureFailure(testTitle) {
  const file = path.join(
    artifacts,
    `${testTitle.replace(/[^a-z0-9]+/gi, '-').toLowerCase()}.png`,
  );
  try {
    await browser.saveScreenshot(file);
    console.log(`[e2e] screenshot de falha: ${file}`);
  } catch (error) {
    console.log(`[e2e] falha ao capturar screenshot: ${error.message}`);
  }
}
