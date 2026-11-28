import assert from 'node:assert/strict';
import test from 'node:test';

import {
  BUILDER_STEPS,
  addDraftDomain,
  addDraftException,
  addDraftLayer,
  addDraftModule,
  addDraftRule,
  complianceLines,
  decideDependency,
  decideObservedDependency,
  detectedStatus,
  diffLinePrefix,
  diffSummary,
  effectsLine,
  evidenceLine,
  exceptionLines,
  nextStep,
  observationLines,
  observedDecisionItems,
  previousStep,
  removeDraftDomain,
  removeDraftException,
  removeDraftModule,
  removeDraftRule,
  renameDraftLayer,
  runModeLabel,
  renameDraftModule,
  saveAuthorization,
  saveConfirmationText,
  savedOutcomeLine,
  setProfile,
  setResolution,
  toggleLayerModule,
  updateDraftDomain,
  updateDraftException,
  updateDraftLayer,
  updateDraftModule,
  updateDraftRule,
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
    domains: [],
    exceptions: [],
    rules: [],
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

test('as dez etapas mantêm ordem fixa e navegação limitada', () => {
  assert.deepEqual(
    BUILDER_STEPS.map((step) => step.id),
    [
      'descoberta',
      'escopo',
      'organizacao',
      'modulos',
      'camadas',
      'dominios',
      'regras',
      'excecoes',
      'dependencias',
      'revisao',
    ],
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

test('perfil clean hexagonal usa o preset vindo do núcleo sem recalcular direções', () => {
  const preset = [
    { name: 'domain', modules: [], allowed_dependencies: [] },
    { name: 'application', modules: [], allowed_dependencies: ['domain'] },
  ];
  const clean = setProfile(draft(), 'clean_hexagonal', preset);

  assert.equal(clean.profile, 'clean_hexagonal');
  assert.deepEqual(clean.layers, preset);
  assert.equal(setProfile(clean, 'modular_monolith').layers.length, 0);

  const layered = setProfile(draft(), 'layered_monolith', []);
  assert.deepEqual(layered.layers[0].modules, ['domain', 'application']);
});

test('domínios, regras e exceções são editáveis sem tocar no rascunho original', () => {
  const original = draft();
  const withDomain = addDraftDomain(original);
  const renamedDomain = updateDraftDomain(withDomain, 'dominio-1', { name: 'core' });
  const withRule = addDraftRule(renamedDomain);
  const reviewedRule = updateDraftRule(withRule, 'regra-1', {
    scope: 'domain',
    from: 'core',
    justification: 'Revisado',
  });
  const withException = addDraftException(reviewedRule);
  const justified = updateDraftException(withException, 0, {
    rule: 'regra-1',
    from: 'core',
    justification: 'Revisado',
  });

  assert.deepEqual(original.domains, []);
  assert.deepEqual(original.rules, []);
  assert.deepEqual(original.exceptions, []);
  assert.equal(justified.domains[0].name, 'core');
  assert.equal(justified.rules[0].scope, 'domain');
  assert.equal(justified.exceptions[0].justification, 'Revisado');
  assert.equal(removeDraftDomain(justified, 'core').rules.length, 0);
  assert.equal(removeDraftRule(justified, 'regra-1').exceptions.length, 0);
  assert.equal(removeDraftException(justified, 0).exceptions.length, 0);
});

test('renomear e remover módulo atualiza domínios e regras', () => {
  const base = {
    ...draft(),
    domains: [{ name: 'core', modules: ['domain'], allowed_dependencies: [] }],
    rules: [
      {
        id: 'guarda',
        scope: 'module',
        from: 'domain',
        to: 'application',
        severity: 'needs_review',
        justification: 'Revisar',
      },
    ],
  };
  const renamed = renameDraftModule(base, 'domain', 'nucleo');
  assert.deepEqual(renamed.domains[0].modules, ['nucleo']);
  assert.equal(renamed.rules[0].from, 'nucleo');

  const removed = removeDraftModule(base, 'domain');
  assert.deepEqual(removed.domains[0].modules, []);
  assert.deepEqual(removed.rules, []);
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
  assert.equal(
    runModeLabel({ mode: 'discovery', contract: { status: 'missing' } }),
    'descoberta (sem contrato)',
  );
  assert.equal(
    runModeLabel({ mode: 'compliance', contract: { status: 'present', schema_version: 2 } }),
    'conformidade (contrato v2)',
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

function diff() {
  return {
    exists: true,
    current_text: 'schema_version = 1\n',
    changed: true,
    lines: [
      { kind: 'removed', text: 'schema_version = 1', old_line: 1, new_line: null },
      { kind: 'added', text: 'schema_version = 2', old_line: null, new_line: 1 },
      { kind: 'context', text: '', old_line: 2, new_line: 2 },
    ],
  };
}

function saveOutcome() {
  return {
    saved: { text: 'schema_version = 2\n', identity: 'abc123', schema_version: 2 },
    run: {
      mode: 'compliance',
      contract: { status: 'present', schema_version: 2 },
      report: {
        project: 'exemplo',
        schema_version: 2,
        summary: {
          files_included: 2,
          files_skipped: 0,
          modules: 1,
          packages: 0,
          edges: 1,
          findings: 2,
          errors: 1,
          warnings: 1,
          infos: 0,
          needs_review: 0,
        },
        skip_summary: [],
        exceptions_applied: [
          {
            rule: 'guarda',
            rule_id: 'dep.custom_not_allowed',
            from: 'domain',
            to: 'application',
            path: 'src/domain/lib.rs',
            justification: 'Integração temporária revisada',
            expires_on: '2026-12-31',
            finding_id: 'dep.custom_not_allowed:guarda:1',
            finding_path: 'src/domain/lib.rs',
            finding_line: 7,
          },
        ],
        findings: [
          {
            rule_id: 'dep.not_allowed',
            severity: 'error',
            title: 'Dependência não permitida',
            message: 'application -> domain',
            path: 'src/lib.rs',
            line_start: 3,
          },
          {
            rule_id: 'file.unmapped',
            severity: 'warning',
            title: 'Arquivo sem módulo',
            message: 'src/solto.rs',
            path: 'src/solto.rs',
            line_start: null,
          },
        ],
        limitations: [],
        coverage: {
          code_files_included: 2,
          code_files_analyzed: 2,
          code_files_not_analyzed: 0,
          unsupported_code_extensions: [],
          partial: false,
        },
      },
    },
    replaced: true,
  };
}

test('autorização de gravação preserva o texto conferido pelo núcleo', () => {
  assert.equal(saveAuthorization({ ...diff(), exists: false, current_text: '' }), 'create');
  assert.deepEqual(saveAuthorization(diff()), {
    replace: { expected_text: 'schema_version = 1\n' },
  });
});

test('diff e confirmação são descritos sem reavaliar o contrato', () => {
  assert.equal(diffLinePrefix('context'), ' ');
  assert.equal(diffLinePrefix('added'), '+');
  assert.equal(diffLinePrefix('removed'), '-');
  assert.match(diffSummary(diff()), /1 linha adicionada/);
  assert.match(diffSummary(diff()), /1 linha removida/);
  assert.match(diffSummary({ ...diff(), changed: false, lines: [] }), /inalterado/);
  assert.match(saveConfirmationText({ ...diff(), exists: false }), /Criar/);
  assert.match(saveConfirmationText(diff()), /Substituir/);
});

test('exceções aplicadas aparecem como informação explícita', () => {
  const lines = exceptionLines(saveOutcome());
  assert.equal(lines.length, 1);
  assert.match(lines[0], /exceção aplicada/);
  assert.match(lines[0], /guarda/);
  assert.match(lines[0], /Integração temporária revisada/);
});

test('painel lado a lado recorta apenas os dados do núcleo', () => {
  const report = {
    candidates: {
      candidate_modules: [],
      observed_dependencies: [
        { source: 'a.rs', target: 'domain', resolution: 'observed', detail: 'use', line: 1, evidence: 'x' },
        { source: 'b.rs', target: 'serde', resolution: 'external', detail: 'use', line: 2, evidence: 'y' },
      ],
      ambiguities: [],
      languages: [],
      limitations: ['analisador parcial'],
    },
    analysis: { limitations: [] },
  };
  const observations = observationLines(report, 1);
  const findings = complianceLines(saveOutcome(), 1);

  assert.equal(observations.length, 2, 'limite aplica-se às relações, não aos avisos');
  assert.match(observations[0], /a\.rs → domain/);
  assert.match(observations[1], /analisador parcial/);
  assert.equal(findings.length, 1);
  assert.match(findings[0], /dep\.not_allowed/);
});

test('resultado gravado informa identidade, modo e contrato aprovado', () => {
  const line = savedOutcomeLine(saveOutcome());
  assert.match(line, /abc123/);
  assert.match(line, /conformidade/);
  assert.match(line, /v2/);
});
