import assert from 'node:assert/strict';
import test from 'node:test';

import {
  BUILDER_STEPS,
  addDraftLayer,
  addDraftModule,
  decideDependency,
  decideObservedDependency,
  detectedStatus,
  effectsLine,
  evidenceLine,
  nextStep,
  observedDecisionItems,
  previousStep,
  removeDraftModule,
  renameDraftLayer,
  renameDraftModule,
  setProfile,
  setResolution,
  toggleLayerModule,
  updateDraftLayer,
  updateDraftModule,
  validationMessage,
  warningText,
} from '../out/builderFlow.js';

function draft() {
  return {
    project_name: 'exemplo',
    include: ['src/**/*.rs'],
    exclude: [],
    style: 'modular_monolith',
    profile: 'modular_monolith',
    modules: [
      {
        name: 'domain',
        path: 'src/domain/**',
        responsibility: 'Domínio',
        allowed_dependencies: [],
        rejected_dependencies: [],
        evidence: ['Cargo.toml: domain'],
        needs_confirmation: true,
      },
      {
        name: 'application',
        path: 'src/application/**',
        responsibility: 'Aplicação',
        allowed_dependencies: [],
        rejected_dependencies: [],
        evidence: [],
        needs_confirmation: false,
      },
    ],
    observed_dependencies: [],
    discovered_ambiguities: [],
    resolved_ambiguities: [],
    layers: [],
    pending_decisions: [],
  };
}

function withReferences() {
  return {
    ...draft(),
    modules: [
      draft().modules[0],
      {
        ...draft().modules[1],
        allowed_dependencies: ['domain'],
      },
    ],
    layers: [
      {
        name: 'core',
        modules: ['domain', 'application'],
        allowed_dependencies: [],
      },
    ],
    resolved_ambiguities: [{ subject: 'origem', detail: 'conflito', choice: 'domain' }],
  };
}

test('as sete etapas mantêm ordem fixa e navegação limitada', () => {
  assert.deepEqual(
    BUILDER_STEPS.map((step) => step.id),
    ['descoberta', 'escopo', 'organizacao', 'modulos', 'camadas', 'dependencias', 'revisao'],
  );
  assert.equal(nextStep(0), 1);
  assert.equal(previousStep(0), 0);
  assert.equal(nextStep(BUILDER_STEPS.length - 1), BUILDER_STEPS.length - 1);
  assert.equal(previousStep(BUILDER_STEPS.length - 1), BUILDER_STEPS.length - 2);
});

test('editar módulo preserva os demais e não altera o rascunho original', () => {
  const original = draft();
  const edited = updateDraftModule(original, 'domain', { responsibility: 'Novo texto' });

  assert.equal(original.modules[0].responsibility, 'Domínio');
  assert.equal(edited.modules[0].responsibility, 'Novo texto');
  assert.equal(edited.modules[1].responsibility, original.modules[1].responsibility);
});

test('adicionar e remover módulo mantém as camadas coerentes', () => {
  const layered = setProfile(draft(), 'layered_monolith');
  const added = addDraftModule(layered);
  assert.equal(added.modules.length, 3);
  assert.ok(added.modules[2].name.startsWith('modulo-'));
  assert.ok(added.layers[0].modules.includes('modulo-1'));

  const inLayer = {
    ...added,
    layers: [{ name: 'core', modules: ['modulo-1'], allowed_dependencies: [] }],
  };
  const removed = removeDraftModule(inLayer, 'modulo-1');
  assert.equal(removed.modules.length, 2);
  assert.deepEqual(removed.layers[0].modules, []);
});

test('trocar perfil limpa camadas ou cria uma camada inicial', () => {
  const layered = setProfile(draft(), 'layered_monolith');
  assert.equal(layered.profile, 'layered_monolith');
  assert.equal(layered.layers.length, 1);
  assert.deepEqual(layered.layers[0].modules, ['domain', 'application']);

  const modular = setProfile(layered, 'modular_monolith');
  assert.equal(modular.profile, 'modular_monolith');
  assert.deepEqual(modular.layers, []);
});

test('camadas podem ser criadas, renomeadas e receber módulos', () => {
  const layered = setProfile(draft(), 'layered_monolith');
  const created = addDraftLayer(layered);
  assert.equal(created.layers.length, 2);
  assert.ok(created.layers[1].name.startsWith('camada-'));

  const renamed = updateDraftLayer(created, created.layers[1].name, { name: 'core' });
  assert.equal(renamed.layers[1].name, 'core');

  const toggled = toggleLayerModule(renamed, 'core', 'domain');
  assert.deepEqual(toggled.layers[1].modules, ['domain']);
  const untoggled = toggleLayerModule(toggled, 'core', 'domain');
  assert.deepEqual(untoggled.layers[1].modules, []);
});

test('renomear módulo atualiza referências em dependências, camadas e escolhas', () => {
  const renamed = renameDraftModule(withReferences(), 'domain', 'core-domain');

  assert.equal(renamed.modules[0].name, 'core-domain');
  assert.deepEqual(renamed.modules[1].allowed_dependencies, ['core-domain']);
  assert.deepEqual(renamed.layers[0].modules, ['core-domain', 'application']);
  assert.equal(renamed.resolved_ambiguities[0].choice, 'core-domain');
});

test('renomear camada atualiza referências entre camadas', () => {
  const current = {
    ...draft(),
    layers: [
      { name: 'principal', modules: [], allowed_dependencies: [] },
      { name: 'core', modules: [], allowed_dependencies: ['principal'] },
    ],
  };
  const renamed = renameDraftLayer(current, 'principal', 'base');

  assert.equal(renamed.layers[0].name, 'base');
  assert.deepEqual(renamed.layers[1].allowed_dependencies, ['base']);
});

test('remover módulo limpa referências e escolhas que apontavam para ele', () => {
  const removed = removeDraftModule(withReferences(), 'domain');

  assert.equal(removed.modules.length, 1);
  assert.deepEqual(removed.modules[0].allowed_dependencies, []);
  assert.deepEqual(removed.layers[0].modules, ['application']);
  assert.equal(removed.resolved_ambiguities[0].choice, null);
});

test('itens observados distinguem origem única de origem ambígua', () => {
  const current = draft();
  current.observed_dependencies = [
    {
      source: 'crates/application/src/lib.rs',
      target: 'serde',
      resolution: 'external',
      detail: 'pacote externo',
      line: 2,
      evidence: 'use serde;',
    },
    {
      source: 'shared/mod.rs',
      target: 'domain',
      resolution: 'ambiguous',
      detail: 'dois candidatos',
      line: null,
      evidence: null,
    },
  ];
  current.pending_decisions = [
    { kind: { confirm_module: { name: 'domain' } }, detail: 'confirmar' },
    {
      kind: {
        decide_dependency: {
          source: 'crates/application/src/lib.rs',
          target: 'serde',
          module: 'application',
        },
      },
      detail: 'decidir',
    },
    {
      kind: {
        resolve_ambiguity: {
          subject: 'origem da relação: shared/mod.rs -> domain',
          detail: 'dois candidatos',
          options: ['domain', 'application'],
        },
      },
      detail: 'resolver',
    },
  ];

  const items = observedDecisionItems(current);

  assert.equal(items.length, 2);
  assert.equal(items[0].moduleName, 'application');
  assert.deepEqual(items[0].options, []);
  assert.equal(items[0].ambiguitySubject, null);
  assert.equal(items[0].evidence, 'use serde;');
  assert.equal(items[1].ambiguitySubject, 'origem da relação: shared/mod.rs -> domain');
  assert.deepEqual(items[1].options, ['domain', 'application']);
  assert.equal(items[1].selected, null);
});

test('decidir relação ambígua registra origem e escolha juntas', () => {
  const current = draft();
  current.observed_dependencies = [
    {
      source: 'shared/mod.rs',
      target: 'domain',
      resolution: 'ambiguous',
      detail: 'dois candidatos',
      line: null,
      evidence: null,
    },
  ];
  current.pending_decisions = [
    {
      kind: {
        resolve_ambiguity: {
          subject: 'origem da relação: shared/mod.rs -> domain',
          detail: 'dois candidatos',
          options: ['domain', 'application'],
        },
      },
      detail: 'resolver',
    },
  ];
  const [item] = observedDecisionItems(current);

  const decided = decideObservedDependency(current, item, 'application', false);

  assert.deepEqual(decided.modules[1].rejected_dependencies, ['domain']);
  assert.deepEqual(decided.modules[1].allowed_dependencies, []);
  assert.equal(decided.resolved_ambiguities.length, 1);
  assert.equal(decided.resolved_ambiguities[0].choice, 'application');
});

test('decidir dependência permite ou rejeita sem manter as duas escolhas', () => {
  const allowed = decideDependency(draft(), 'domain', 'serde', true);
  assert.deepEqual(allowed.modules[0].allowed_dependencies, ['serde']);
  assert.deepEqual(allowed.modules[0].rejected_dependencies, []);

  const rejected = decideDependency(allowed, 'domain', 'serde', false);
  assert.deepEqual(rejected.modules[0].allowed_dependencies, []);
  assert.deepEqual(rejected.modules[0].rejected_dependencies, ['serde']);
});

test('resolver ambiguidade substitui escolha anterior com o mesmo par', () => {
  const first = setResolution(draft(), 'shared', 'conflito', 'shared_a');
  const second = setResolution(first, 'shared', 'conflito', 'shared_b');

  assert.equal(second.resolved_ambiguities.length, 1);
  assert.equal(second.resolved_ambiguities[0].choice, 'shared_b');
});

test('estados de detecção e validação são explícitos', () => {
  assert.match(detectedStatus({ mode: 'discovery', contract: { status: 'missing' } }), /descoberta/);
  assert.match(
    detectedStatus({ mode: 'compliance', contract: { status: 'present', schema_version: 2 } }),
    /v2/,
  );
  assert.match(validationMessage({ valid: true, errors: [], identity: 'abc' }), /válido/);
  assert.match(
    validationMessage({ valid: false, errors: ['módulo ainda não confirmado'], identity: null }),
    /módulo ainda não confirmado/,
  );
});

test('efeitos, evidências e aviso vêm dos dados recebidos', () => {
  const effects = {
    schema_version: 2,
    modules: 3,
    layers: 1,
    allowed_dependencies: 2,
    rule_ids: ['dep.not_allowed', 'dep.layer_not_allowed'],
    notes: ['responsabilidade é texto descritivo'],
  };
  assert.match(effectsLine(effects), /3 módulos/);
  assert.match(effectsLine(effects), /dep\.layer_not_allowed/);
  assert.equal(evidenceLine('x'.repeat(300)).length, 123);
  assert.match(warningText(), /não são aprovadas automaticamente/);
});
