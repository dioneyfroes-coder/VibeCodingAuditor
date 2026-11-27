import assert from 'node:assert/strict';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import test from 'node:test';

import BuilderWizard from '../out/BuilderWizard.js';

test('o assistente começa em estado de carregamento explícito', () => {
  const markup = renderToStaticMarkup(
    createElement(BuilderWizard, {
      root: 'C:\\projeto',
      onStatus: () => {},
      onClose: () => {},
    }),
  );

  assert.match(markup, /Architecture Builder/);
  assert.match(markup, /Carregando descoberta e rascunho/);
  assert.match(markup, /C:\\projeto/);
});
