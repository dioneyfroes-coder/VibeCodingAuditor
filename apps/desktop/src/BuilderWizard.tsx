import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState, type ReactNode } from 'react';

import {
  BUILDER_STEPS,
  addDraftLayer,
  addDraftModule,
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
  type ContractDraft,
  type DiscoveryReport,
  type DraftEffects,
  type ProjectRunStatus,
  type ValidationResult,
} from './builderFlow.js';
import { builderStatus, type ActionStatus } from './status.js';

interface BuilderWizardProps {
  root: string;
  onStatus: (status: ActionStatus) => void;
  onClose: () => void;
}

/// Linhas de texto viram padrões; linhas vazias não são enviadas ao núcleo.
function patternLines(value: string): string[] {
  return value
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function patternText(patterns: string[]): string {
  return patterns.join('\n');
}

/// Assistente somente leitura até a confirmação final da Etapa 4.6.
export default function BuilderWizard({ root, onStatus, onClose }: BuilderWizardProps) {
  const [draft, setDraft] = useState<ContractDraft | null>(null);
  const [report, setReport] = useState<DiscoveryReport | null>(null);
  const [step, setStep] = useState(0);
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const [effects, setEffects] = useState<DraftEffects | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setBusy(true);
    (async () => {
      try {
        const mode = await invoke<ProjectRunStatus>('load_project', { path: root });
        const [initialDraft, discovery] = await Promise.all([
          invoke<ContractDraft>('start_draft', { path: root }),
          mode.mode === 'discovery'
            ? invoke<DiscoveryReport>('discover_project', { path: root })
            : Promise.resolve(null),
        ]);
        if (!active) {
          return;
        }
        setDraft(initialDraft);
        setReport(discovery);
        onStatus(builderStatus(detectedStatus(mode), 'success'));
      } catch (failure) {
        if (active) {
          setError(String(failure));
          onStatus(builderStatus(String(failure), 'error'));
        }
      } finally {
        if (active) {
          setBusy(false);
        }
      }
    })();
    return () => {
      active = false;
    };
  }, [root, onStatus]);

  /// Toda edição invalida a prévia anterior, que passa a estar desatualizada.
  const mutate = (change: (current: ContractDraft) => ContractDraft) => {
    setDraft((current) => (current ? change(current) : current));
    setValidation(null);
    setPreview(null);
    setEffects(null);
  };

  const review = async () => {
    if (!draft) {
      return;
    }
    setBusy(true);
    try {
      const result = await invoke<ValidationResult>('validate_contract_draft', { draft });
      setValidation(result);
      setPreview(
        result.valid ? await invoke<string>('preview_contract', { draft }) : null,
      );
      setEffects(await invoke<DraftEffects>('draft_effects', { draft }));
      onStatus(
        builderStatus(validationMessage(result), result.valid ? 'success' : 'error'),
      );
    } catch (failure) {
      onStatus(builderStatus(String(failure), 'error'));
    } finally {
      setBusy(false);
    }
  };

  if (busy && draft === null) {
    return (
      <section className="panel" id="painel-builder">
        <h2>Architecture Builder</h2>
        <p className="detail">Raiz: {root}</p>
        <p className="detail">Carregando descoberta e rascunho…</p>
      </section>
    );
  }

  if (error !== null && draft === null) {
    return (
      <section className="panel" id="painel-builder">
        <h2>Architecture Builder</h2>
        <p className="status-error">{error}</p>
        <button type="button" onClick={onClose}>
          Fechar
        </button>
      </section>
    );
  }

  if (draft === null) {
    return null;
  }

  const current = BUILDER_STEPS[step];
  const keyFor = (source: string, target: string) => `${source}->${target}`;

  const discoveryStep = () => (
    <>
      <p className="hint">{warningText()}</p>
      <h3>Módulos candidatos</h3>
      {report?.candidates.candidate_modules.length ? (
        <ul className="builder-list">
          {report.candidates.candidate_modules.map((candidate) => (
            <li key={`${candidate.name}-${candidate.path}`}>
              <strong>{candidate.name}</strong> [{candidate.category}] <code>{candidate.path}</code>
              <ul className="detail">
                {candidate.evidence.map((evidence) => (
                  <li key={evidence}>{evidenceLine(evidence)}</li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      ) : (
        <p className="detail">Nenhum módulo candidato; é possível configurar manualmente.</p>
      )}
      <h3>Relações observadas</h3>
      {report?.candidates.observed_dependencies.length ? (
        <ul className="builder-list">
          {report.candidates.observed_dependencies.map((dependency) => (
            <li key={keyFor(dependency.source, dependency.target)}>
              <code>{dependency.source}</code> → <code>{dependency.target}</code> [{dependency.resolution}]
              <div className="detail">{dependency.detail}</div>
              {dependency.evidence && (
                <div className="detail">evidência: {evidenceLine(dependency.evidence)}</div>
              )}
            </li>
          ))}
        </ul>
      ) : (
        <p className="detail">Nenhuma relação observada nos analisadores disponíveis.</p>
      )}
      <h3>Cobertura</h3>
      {report?.candidates.languages.length ? (
        <ul className="detail">
          {report.candidates.languages.map((language) => (
            <li key={language.extension}>
              {language.extension}: {language.files} arquivo(s) ·{' '}
              {language.analyzer ?? 'sem analisador'}
            </li>
          ))}
        </ul>
      ) : (
        <p className="detail">Nenhum arquivo com analisador conhecido.</p>
      )}
      <h3>Ambiguidades e limitações</h3>
      <ul className="detail">
        {(report?.candidates.ambiguities ?? []).map((ambiguity) => (
          <li key={`${ambiguity.subject}-${ambiguity.detail}`}>
            {ambiguity.subject}: {ambiguity.detail}
          </li>
        ))}
        {(report?.candidates.limitations ?? []).map((limitation) => (
          <li key={limitation}>{limitation}</li>
        ))}
      </ul>
      {draft.pending_decisions.length > 0 && (
        <p className="detail">
          {draft.pending_decisions.length} decisão(ões) pendente(s) registrada(s) no rascunho.
        </p>
      )}
    </>
  );

  const scopeStep = () => (
    <>
      <label>
        Nome do projeto
        <input
          value={draft.project_name}
          onChange={(event) => mutate((currentDraft) => ({
            ...currentDraft,
            project_name: event.target.value,
          }))}
        />
      </label>
      <label>
        Inclusões (um padrão por linha)
        <textarea
          rows={4}
          value={patternText(draft.include)}
          onChange={(event) => mutate((currentDraft) => ({
            ...currentDraft,
            include: patternLines(event.target.value),
          }))}
        />
      </label>
      <label>
        Exclusões (um padrão por linha)
        <textarea
          rows={4}
          value={patternText(draft.exclude)}
          onChange={(event) => mutate((currentDraft) => ({
            ...currentDraft,
            exclude: patternLines(event.target.value),
          }))}
        />
      </label>
      <p className="detail">O contrato gerado será v2, mesmo ao revisar um contrato v1.</p>
    </>
  );

  const organizationStep = () => (
    <>
      <label className="choice">
        <input
          type="radio"
          name="perfil"
          checked={draft.profile === 'modular_monolith'}
          onChange={() => mutate((currentDraft) => setProfile(currentDraft, 'modular_monolith'))}
        />
        Monólito modular sem camadas
      </label>
      <label className="choice">
        <input
          type="radio"
          name="perfil"
          checked={draft.profile === 'layered_monolith'}
          onChange={() => mutate((currentDraft) => setProfile(currentDraft, 'layered_monolith'))}
        />
        Monólito modular em camadas
      </label>
      <p className="detail">
        Estilo: <code>{draft.style}</code>. Camadas só são gravadas no perfil em camadas.
      </p>
    </>
  );

  const modulesStep = () => (
    <>
      {draft.modules.length === 0 ? (
        <p className="detail">Nenhum módulo. Adicione um módulo manual ou volte à descoberta.</p>
      ) : (
        <ul className="builder-list">
          {draft.modules.map((module) => (
            <li key={module.name} className="builder-card">
              <div className="builder-grid">
                <label>
                  Nome
                  <input
                    value={module.name}
                    onChange={(event) => mutate((currentDraft) =>
                      renameDraftModule(currentDraft, module.name, event.target.value),
                    )}
                  />
                </label>
                <label>
                  Path
                  <input
                    value={module.path}
                    onChange={(event) => mutate((currentDraft) =>
                      updateDraftModule(currentDraft, module.name, { path: event.target.value }),
                    )}
                  />
                </label>
              </div>
              <label>
                Responsabilidade (texto livre)
                <input
                  value={module.responsibility}
                  onChange={(event) => mutate((currentDraft) =>
                    updateDraftModule(currentDraft, module.name, {
                      responsibility: event.target.value,
                    }),
                  )}
                />
              </label>
              <label className="choice">
                <input
                  type="checkbox"
                  checked={!module.needs_confirmation}
                  onChange={(event) => mutate((currentDraft) =>
                    updateDraftModule(currentDraft, module.name, {
                      needs_confirmation: !event.target.checked,
                    }),
                  )}
                />
                Confirmar módulo
              </label>
              {module.evidence.length > 0 && (
                <ul className="detail">
                  {module.evidence.map((evidence) => (
                    <li key={evidence}>evidência: {evidenceLine(evidence)}</li>
                  ))}
                </ul>
              )}
              <button type="button" onClick={() => mutate((currentDraft) => removeDraftModule(currentDraft, module.name))}>
                Remover
              </button>
            </li>
          ))}
        </ul>
      )}
      <button type="button" onClick={() => mutate(addDraftModule)}>
        Adicionar módulo
      </button>
    </>
  );

  const layersStep = () => {
    if (draft.profile !== 'layered_monolith') {
      return (
        <p className="detail">
          O perfil sem camadas não grava <code>architecture.layers</code>. Volte à etapa
          Organização para usar camadas.
        </p>
      );
    }
    return (
      <>
        {draft.layers.length === 0 && <p className="detail">Nenhuma camada declarada.</p>}
        <ul className="builder-list">
          {draft.layers.map((layer) => (
            <li key={layer.name} className="builder-card">
              <label>
                Nome da camada
                <input
                  value={layer.name}
                  onChange={(event) => mutate((currentDraft) =>
                    renameDraftLayer(currentDraft, layer.name, event.target.value),
                  )}
                />
              </label>
              <fieldset>
                <legend>Módulos da camada</legend>
                {draft.modules.map((module) => (
                  <label key={module.name} className="choice">
                    <input
                      type="checkbox"
                      checked={layer.modules.includes(module.name)}
                      onChange={() => mutate((currentDraft) =>
                        toggleLayerModule(currentDraft, layer.name, module.name),
                      )}
                    />
                    {module.name}
                  </label>
                ))}
              </fieldset>
              <label>
                Camadas permitidas como destino (uma por linha)
                <textarea
                  rows={2}
                  value={patternText(layer.allowed_dependencies)}
                  onChange={(event) => mutate((currentDraft) =>
                    updateDraftLayer(currentDraft, layer.name, {
                      allowed_dependencies: patternLines(event.target.value),
                    }),
                  )}
                />
              </label>
            </li>
          ))}
        </ul>
        <button type="button" onClick={() => mutate(addDraftLayer)}>
          Adicionar camada
        </button>
      </>
    );
  };

  const dependenciesStep = () => (
    <>
      <p className="hint">{warningText()}</p>
      {draft.observed_dependencies.length === 0 && (
        <p className="detail">Nenhuma relação observada; dependências podem ser digitadas nos módulos.</p>
      )}
      <ul className="builder-list">
        {observedDecisionItems(draft).map((item) => {
          const key = keyFor(item.source, item.target);
          const selected = item.selected ?? item.moduleName ?? '';
          const decide = (allow: boolean) => mutate((currentDraft) =>
            decideObservedDependency(currentDraft, item, selected, allow),
          );
          return (
            <li key={key} className="builder-card">
              <div>
                <code>{item.source}</code> → <code>{item.target}</code>
              </div>
              <div className="detail">{item.detail}</div>
              {item.evidence && (
                <div className="detail">evidência: {evidenceLine(item.evidence)}</div>
              )}
              {item.options.length > 0 ? (
                <label>
                  Origem ambígua — escolha o módulo
                  <select
                    value={selected}
                    onChange={(event) => mutate((currentDraft) =>
                      setResolution(
                        currentDraft,
                        item.ambiguitySubject ?? '',
                        item.ambiguityDetail ?? '',
                        event.target.value || null,
                      ),
                    )}
                  >
                    <option value="">sem escolha</option>
                    {item.options.map((option) => (
                      <option key={option} value={option}>
                        {option}
                      </option>
                    ))}
                  </select>
                </label>
              ) : (
                <div className="detail">Módulo de origem: {item.moduleName}</div>
              )}
              <div className="actions">
                <button type="button" disabled={!selected} onClick={() => decide(true)}>
                  Permitir
                </button>
                <button type="button" disabled={!selected} onClick={() => decide(false)}>
                  Rejeitar
                </button>
              </div>
            </li>
          );
        })}
      </ul>
      <h3>Ambiguidades</h3>
      {draft.discovered_ambiguities.length === 0 ? (
        <p className="detail">Nenhuma ambiguidade registrada na descoberta.</p>
      ) : (
        <ul className="builder-list">
          {draft.discovered_ambiguities.map((ambiguity) => {
            const resolution = draft.resolved_ambiguities.find(
              (entry) => entry.subject === ambiguity.subject && entry.detail === ambiguity.detail,
            );
            return (
              <li key={`${ambiguity.subject}-${ambiguity.detail}`}>
                <div>
                  {ambiguity.subject}: {ambiguity.detail}
                </div>
                <select
                  value={resolution?.choice ?? ''}
                  onChange={(event) => mutate((currentDraft) =>
                    setResolution(
                      currentDraft,
                      ambiguity.subject,
                      ambiguity.detail,
                      event.target.value || null,
                    ),
                  )}
                >
                  <option value="">sem escolha</option>
                  {ambiguity.options.map((option) => (
                    <option key={option} value={option}>
                      {option}
                    </option>
                  ))}
                </select>
              </li>
            );
          })}
        </ul>
      )}
    </>
  );

  const reviewStep = () => (
    <>
      {validation === null ? (
        <p className="detail">Valide o rascunho para ver a prévia e os efeitos das regras.</p>
      ) : (
        <>
          <p className={validation.valid ? 'status-success' : 'status-error'}>
            {validationMessage(validation)}
          </p>
          {!validation.valid && (
            <ul className="detail">
              {validation.errors.map((message) => (
                <li key={message}>{message}</li>
              ))}
            </ul>
          )}
        </>
      )}
      {effects !== null && (
        <>
          <h3>Efeitos das regras</h3>
          <p className="detail">{effectsLine(effects)}</p>
          <ul className="detail">
            {effects.notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        </>
      )}
      <h3>Prévia do contrato</h3>
      {preview === null ? (
        <p className="detail">Prévia indisponível enquanto houver decisões pendentes.</p>
      ) : (
        <pre className="preview">{preview}</pre>
      )}
      <p className="hint">
        Nenhum arquivo foi alterado. A gravação exigirá confirmação explícita na próxima etapa.
      </p>
      <button type="button" disabled={busy} onClick={review}>
        {busy ? 'Validando…' : 'Validar rascunho'}
      </button>
    </>
  );

  const stepContent: Record<string, () => ReactNode> = {
    descoberta: discoveryStep,
    escopo: scopeStep,
    organizacao: organizationStep,
    modulos: modulesStep,
    camadas: layersStep,
    dependencias: dependenciesStep,
    revisao: reviewStep,
  };

  return (
    <section className="panel wizard" id="painel-builder">
      <h2>Architecture Builder</h2>
      <p className="detail">Raiz: {root}</p>
      <ol className="wizard-steps">
        {BUILDER_STEPS.map((item, index) => (
          <li key={item.id} className={index === step ? 'atual' : undefined}>
            {item.label}
          </li>
        ))}
      </ol>
      <h3>
        Etapa {step + 1} de {BUILDER_STEPS.length} — {current.label}
      </h3>
      <p className="detail">{current.hint}</p>
      {stepContent[current.id]()}
      <div className="actions">
        <button type="button" disabled={step === 0} onClick={() => setStep(previousStep(step))}>
          Voltar
        </button>
        <button
          type="button"
          disabled={step === BUILDER_STEPS.length - 1}
          onClick={() => setStep(nextStep(step))}
        >
          Próxima
        </button>
        <button type="button" onClick={onClose}>
          Fechar
        </button>
      </div>
    </section>
  );
}
