# Baseline — Etapa 6.0

Data: 27/11/2026
Branch: `main`
HEAD: `4146fcc8dd3554ee98923bd9df5fd39d00314d95` (`2026-11-27T18:39:01-03:00`, `docs: consolida status final do roadmap`)

Este documento congela o estado observado **antes** de qualquer implementação da Etapa 6. Nenhum código, manifest, workflow, contrato `auditor.toml`, dependência ou ferramenta foi alterado, instalado, atualizado ou removido. O único arquivo criado nesta subetapa é este relatório.

Os códigos de saída abaixo foram capturados no PowerShell com `Write-Output "EXITCODE=$LASTEXITCODE"` após cada comando; o comando listado é o comando efetivo executado.

## 1. Estado do Git e preservação do trabalho local

`git status --short --branch` retornou:

```text
## main...origin/main
 D PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md
?? PLANO_ETAPA_6_AMADURECIMENTO_ARQUITETURAL_E_DISTRIBUICAO.md
```

- `git rev-parse HEAD` → `4146fcc8dd3554ee98923bd9df5fd39d00314d95`.
- `git log -1 --format='%H %cI %s'` → `4146fcc8dd3554ee98923bd9df5fd39d00314d95 2026-11-27T18:39:01-03:00 docs: consolida status final do roadmap`.
- `git log -10 --oneline --decorate` → `4146fcc`, `5ecbe03`, `4ceeab2`, `f87fd09`, `193e238`, `3a83d89`, `7cf48f7`, `eec73a8`, `7ffcf22`, `836c967`, com `HEAD -> main, origin/main, origin/HEAD` no primeiro commit.
- `git remote -v` → `origin https://github.com/dioneyfroes-coder/VibeCodingAuditor.git` (fetch e push).
- `git status --short --branch` repetido ao final → exatamente as mesmas duas mudanças locais.

Nenhuma mudança local foi restaurada, limpa, commitada ou sobrescrita. Não houve `git reset`, `git clean`, checkout destrutivo ou `git add`. Os artefatos regenados (`apps/desktop/dist`, `apps/desktop/out`, `extensions/vscode/out`, `*.vsix`) são ignorados por `.gitignore` e não aparecem no status.

## 2. Ambiente verificado

| Ferramenta | Versão observada |
|---|---|
| Git | `2.55.0.windows.4` |
| rustc | `1.98.1 (48a229cea 2026-09-01)` |
| cargo | `1.98.1 (797e8a9bc 2026-08-05)` |
| Node.js | `v24.19.0` |
| npm | `11.17.0` |

Nenhuma ferramenta foi instalada, atualizada ou removida. O workspace declara `rust-version = 1.98.1`, o mesmo valor fixado no CI.

## 3. Inventário de componentes, manifests e workflows

| Componente | Caminhos conferidos | Manifesto/versão | Observação |
|---|---|---|---|
| Núcleo Rust | `crates/auditor-core` | `Cargo.toml`, versão `0.1.0` | Não depende de Tauri, React nem CLI. |
| CLI Rust | `crates/auditor-cli` | `Cargo.toml`, versão `0.1.0` | `[[bin]] name = "vibe-coding-auditor-cli"`. |
| Backend Tauri | `apps/desktop/src-tauri` | `Cargo.toml`, versão `0.1.0` | Adaptador fino sobre o núcleo. |
| Desktop React | `apps/desktop` | `package.json`, versão `0.1.0` | React 19.3.1, Vite 8.3.8, TypeScript 7.0.2. |
| Extensão VS Code | `extensions/vscode` | `package.json`, versão `0.1.0` | `@vscode/vsce` 4.1.1, host fixado em 1.146.0 no E2E. |
| Config Tauri | `apps/desktop/src-tauri/tauri.conf.json` | `version: "0.1.0"` | `bundle.active: false`. |
| Contrato próprio | `auditor.toml` | `schema_version = 1` | Escopo Rust-only: `core`, `cli`, `desktop-backend`. |
| Workspace Rust | `Cargo.toml` e `Cargo.lock` | `resolver = "2"`, `rust-version = 1.98.1` | 3 membros. |
| Lockfiles Node | `apps/desktop/package-lock.json`, `extensions/vscode/package-lock.json` | presentes | `npm ls --depth=0` limpo nas duas pastas. |

As quatro versões de produto (`Cargo.toml` do workspace, `apps/desktop/package.json`, `tauri.conf.json` e `extensions/vscode/package.json`) são `0.1.0`.

### Workflows

`.github/workflows/` contém **um único arquivo**: `.github/workflows/ci.yml` (`name: CI Windows`). Nenhum workflow de release, empacotamento, publicação, Linux ou macOS foi encontrado.

| Workflow | Gatilhos | Job | Plataforma | Finalidade |
|---|---|---|---|---|
| `ci.yml` | `push` em `main`, `pull_request`, `workflow_dispatch` | `verify` | `windows-latest` | fmt, check, test e clippy do Rust; test e build da desktop; test da extensão. |
| `ci.yml` | idem | `E2E Windows` (`needs: verify`, 45 min) | `windows-latest` | build release da CLI e do app, E2E da CLI, da extensão e do desktop, artefatos em falha. |

Permissões declaradas: `contents: read` no nível do workflow. Actions: `checkout@v5`, `setup-node@v5`, `cache@v4`, `upload-artifact@v4`.

Scripts E2E locais lidos antes de qualquer execução (nenhum nome de script foi inventado):

- CLI: `node --test tests/e2e/cli/cli.e2e.test.mjs` (raiz; usa binário release ou debug).
- Extensão: `npm --prefix extensions/vscode run test:e2e` → `npm run compile && node e2e/runTests.mjs`; exige CLI release/debug e host VS Code `1.146.0` por `VCA_VSCODE_VERSION`.
- Desktop: `npm --prefix apps/desktop run test:e2e` → `wdio run e2e/wdio.conf.mjs`; exige `target/release/auditor-desktop.exe`, `tauri-driver`, MSEdgeDriver compatível com WebView2 e, em runner elevado, `scripts/windows/run-deelevated.ps1`.
- Bootstrap Windows: `scripts/windows/install-msedgedriver.ps1` e `scripts/windows/run-deelevated.ps1`.

## 4. Comandos executados e resultados

### Rust e Git

| Comando | Resultado | Código |
|---|---|---:|
| `cargo fmt --all --check` | sem diferenças | `0` |
| `cargo check --workspace` | `Finished dev profile in 1.82s` | `0` |
| `cargo test --workspace` | 277 testes aprovados, 0 falhas | `0` |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | sem avisos | `0` |
| `git diff --check` | sem problemas de whitespace | `0` |
| `cargo run -q -p auditor-cli -- --version` | `vibe-coding-auditor-cli 0.1.0` | `0` |
| `cargo run -q -p auditor-cli -- --help` | apresentação e 5 subcomandos (`init`, `scan`, `analyze`, `audit`, `history`) | `0` |

Contagem detalhada de `cargo test --workspace`:

| Destino | Testes |
|---|---:|
| `auditor-cli` (unit tests, `src/main.rs`) | 11 |
| `auditor-cli` (`tests/cli.rs`) | 7 |
| `auditor-cli` (`tests/fixtures.rs`) | 5 |
| `auditor-core` (unit tests) | 239 |
| `auditor-desktop` (unit tests, `src/lib.rs`) | 15 |
| `auditor-desktop` (`src/main.rs`) | 0 |
| Doc-tests (`auditor_core`, `auditor_desktop`) | 0 |
| **Total** | **277** |

### Node

`npm ci` **não foi executado** nesta subetapa: o prompt autoriza somente o baseline e proíbe instalar dependências; `node_modules` já existe nas duas pastas e os lockfiles estão presentes. Como substituto somente leitura, `npm --prefix <pasta> ls --depth=0` retornou `0` nas duas pastas, com todas as dependências diretas coincidindo com os manifests.

| Comando | Resultado | Código |
|---|---|---:|
| `npm --prefix apps/desktop ls --depth=0` | 16 dependências diretas, sem inconsistência | `0` |
| `npm --prefix extensions/vscode ls --depth=0` | 5 dependências diretas, sem inconsistência | `0` |
| `npm --prefix apps/desktop test` | 39 testes aprovados, 0 falhas | `0` |
| `npm --prefix apps/desktop run build` | 25 módulos; `index.html` 0,49 kB, CSS 8,39 kB, JS 259,54 kB | `0` |
| `npm --prefix extensions/vscode test` | 18 testes aprovados, 0 falhas | `0` |
| `npm --prefix extensions/vscode run package` | `.vsix` com 10 arquivos, 15,23 kB | `0` |

Aviso não falho no empacotamento da extensão: `LICENSE, LICENSE.md, or LICENSE.txt not found`. A contagem de arquivos do `.vsix` (10) difere da registrada em documentação anterior (7 arquivos, 8,98 kB): o pacote atual inclui `e2e/runTests.mjs` e `out/e2e/runner.js`; nenhuma regra ou código de produção mudou por isso.

### Avisos do ambiente

- `cargo fmt --all --check`, `cargo check`, `cargo test` e `cargo clippy --config 'build.incremental=false'` terminaram sem avisos. O modo incremental sem essa opção não foi reexecutado neste baseline porque a documentação já registra o aviso `Acesso negado (os error 5)` do OneDrive (`D1.6`).
- `npm --prefix extensions/vscode run package` emitiu o aviso de `LICENSE` ausente.
- Nenhum teste determinístico falhou; nenhuma correção foi aplicada.

## 5. Validação E2E remota existente

A documentação local (`docs/ROADMAP.md` e `docs/DECISIONS.md`) registra a Etapa 5 concluída com a execução remota:

- Workflow: `CI Windows` (`ci.yml`), execução **#18**.
- URL: <https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/46067832029>.
- Commit: `4ceeab2` (`ci: executa E2E desktop sem elevacao no runner`).
- Gatilho: push em `main`, 27/11/2026 21:04.
- Resultado observado na página HTML: **Status: Success**, duração total `12m 57s`.
- Jobs: `verify` (`6m 22s`) e `E2E Windows` (`6m 27s`), ambos concluídos com sucesso.
- Anotação: 1 aviso — `actions/cache@v4` ainda executa em Node.js 20 e está sendo forçado para Node.js 24 pelo runner. `checkout@v5` e `setup-node@v5` não aparecem no aviso.

A API pública REST retornou HTTP `403` (limite de requisições); a confirmação foi feita pela página HTML pública do run. O HEAD atual (`4146fcc`) está dois commits à frente do commit validado (`5ecbe03` e `4146fcc`, ambos documentais), e este baseline não identificou uma execução remota posterior. As suítes E2E **não foram reexecutadas localmente** nesta subetapa: seus pré-requisitos foram lidos e a execução remota bem-sucedida está registrada acima; não se alega validação E2E local do HEAD atual.

## 6. Vulnerabilidades npm atuais

Comandos: `npm --prefix apps/desktop audit --json` e `npm --prefix extensions/vscode audit --json`, ambos com código `1` por encontrarem alertas. Nenhuma correção foi aplicada (`npm audit fix`, `--force` e atualizações ficam fora da Etapa 6.0).

| Projeto | Total | Crítico | Alto | Moderado | Baixo/Info | Produção (`audit --omit=dev`) |
|---|---:|---:|---:|---:|---:|---|
| `apps/desktop` | 25 | 0 | 14 | 11 | 0 | 0 alertas, código `0` |
| `extensions/vscode` | 2 | 0 | 0 | 2 | 0 | 0 alertas, código `0` |

Classificação dos alertas atuais:

- **Desktop — todos em `devDependencies` e transitivos da suíte WebdriverIO usada no E2E.** Diretos afetados: `@wdio/cli` (alto), `@wdio/local-runner` (moderado), `@wdio/mocha-framework` (alto), `@wdio/tauri-service` (moderado) e `webdriverio` (alto). Transitivos incluem `@wdio/config`, `@wdio/globals`, `@wdio/runner`, `@wdio/utils`, `@puppeteer/browsers`, `archiver`, `archiver-utils`, `basic-ftp`, `expect-webdriverio`, `extract-zip`, `get-uri`, `glob`, `minimatch`, `mocha`, `pac-proxy-agent`, `proxy-agent`, `readdir-glob`, `serialize-javascript`, `webdriver` e `zip-stream`. As correções oferecidas são **majors** (por exemplo `@wdio/local-runner` 8.6.1 e `@wdio/mocha-framework` 10.1.0), com risco de quebrar o E2E; `@wdio/tauri-service` não tem correção disponível. `webdriver` e `zip-stream` têm correção não-major informada.
- **Extensão — 1 alerta direto em dev:** `@vscode/vsce` 4.1.1 (moderado), por `minimatch` transitivo. A correção sugerida é downgrade major para `@vscode/vsce` 3.9.1, explicitamente rejeitado em `D9.4` porque aumentaria os alertas e traria módulo nativo.
- Nenhuma dependência de produção (React, Tauri API, plugin de diálogo, TypeScript) apresenta alerta.

Os números históricos não foram reutilizados como estado atual: a documentação anterior registrava "25 vulnerabilidades" no E2E desktop e "2 moderadas" no `@vscode/vsce` de desenvolvimento. O total permanece, mas a composição atual é **14 altas e 11 moderadas** no desktop; é essa composição que deve ser citada daqui em diante.

## 7. Divergências da seção 2.3 do plano

| Item | Evidência observada | Classificação |
|---|---|---|
| 2.3.1 — README desatualizado | `README.md:5` diz "Etapas 0–3 concluídas e Etapa 4 concluída com ressalvas"; `docs/ROADMAP.md:18-19` registra as novas Etapas 4 e 5 concluídas em 27/11/2026 e ajustes posteriores. | **Confirmada.** |
| 2.3.2 — `cliPath` padrão | `extensions/vscode/package.json:51` → `vibe-coding-auditor-cli`; `extensions/vscode/src/cliPath.ts:8` → mesmo nome; `crates/auditor-cli/Cargo.toml:8` → mesmo binário; `--version` imprime `vibe-coding-auditor-cli 0.1.0`. | **Descartada.** Já está alinhado. |
| 2.3.3 — menu de contexto exige contrato | `extensions/vscode/package.json:41` → `"when": "explorerResourceIsFolder"`; teste `o menu de pasta não exige auditor.toml` passou. | **Descartada.** Já aparece sem `auditor.toml`. |
| 2.3.4 — `bundle.active = false` | `tauri.conf.json:25-27`; `target/release/bundle` não existe; somente `auditor-desktop.exe` e `vibe-coding-auditor-cli.exe` em `target/release`. | **Confirmada.** |
| 2.3.5 — CI apenas Windows | Único workflow é `ci.yml`; os dois jobs usam `windows-latest`. Não há matriz Linux/macOS nem workflow de release. | **Confirmada** (com a ressalva de que a existência de outros workflows não foi presumida: o inventário é completo). |
| 2.3.6 — números de vulnerabilidades | `npm audit` atual: desktop 25 (14 altas/11 moderadas, dev); extensão 2 moderadas (dev); produção 0 nos dois. | **Atualizada, não confirmada como divergência:** os totais antigos coincidem, mas a composição por severidade precisa ser a atual. Nenhuma correção aplicada. |

Achados adicionais (fora da lista 2.3, registrados para 6.1/6.6):

- `docs/validation/` contém apenas `etapa-3/` e `etapa-4/`; não existe `etapa-5/`. A evidência da Etapa 5 está no roadmap, nas decisões e no run remoto.
- O run `46067832029` tem aviso de Actions em Node.js 20 (`actions/cache@v4`).
- O `.vsix` gerado hoje tem 10 arquivos/15,23 kB, contra 7 arquivos/8,98 kB registrados em documento anterior.

## 8. Build do executável vs. bundle vs. instalador

- `cargo check`/`cargo test` e `npm run build` validam código e frontend; `npm --prefix apps/desktop run tauri build` (executado no CI) gera `auditor-desktop.exe`.
- `bundle.active: false` em `tauri.conf.json` significa que **nenhum instalador é produzido**: não há `target/release/bundle`, nem artefatos NSIS ou MSI.
- Portanto, o binário atual **não é uma release instalável**. Habilitar bundle, escolher NSIS/MSI, definir local do SQLite e estratégia da CLI para a extensão pertence à Etapa 6.7, não a este baseline.

## 9. Riscos e prioridades

1. **Documentação (P1):** README desatualizado e ausência de registro local da Etapa 5; risco de o usuário acreditar que etapas posteriores não existem.
2. **Distribuição (P1):** sem bundle, sem instalador e sem estratégia documentada para CLI/extensão fora do repositório; bloqueia qualquer anúncio de release.
3. **Descoberta multi-ecossistema (P1):** `package.json`, `pyproject.toml` e `go.mod` ainda não geram candidatos; limitação confirmada em `README.md` e nos testes da Etapa 4.
4. **Segurança de dependências (P2):** 25 alertas de desenvolvimento no desktop, 14 altos; nenhum em produção, mas correções sugeridas são breaking changes.
5. **CI (P2):** aviso de `actions/cache@v4` em Node.js 20 e toolchain fixado em `1.98.1` sem reavaliação nesta subetapa.
6. **Autoauditoria (P3):** `auditor.toml` cobre apenas Rust de produção; React, extensão e integrações Tauri/CLI ficam fora do grafo e não são apresentadas como verificadas.

## 10. Proposta técnica para a Etapa 6.1 (aguardando aprovação)

**Motivo:** eliminar contradições comprovadas entre README, roadmap, manifests e configuração, sem alterar comportamento. As divergências 2.3.2 e 2.3.3 já estão resolvidas no código; a 6.1 deve documentar o estado real, não "corrigir" o que já funciona.

**Alternativas consideradas:**

- Rejeitada: alterar `cliPath`, o `when` do menu ou o nome do binário — já alinhados e com testes verdes.
- Rejeitada: habilitar `bundle.active` agora — decisão de distribuição pertence à 6.7.
- Rejeitada: rodar `npm audit fix` ou downgrade do `@vscode/vsce` — breaking changes e decisão já registrada em `D9.4`.
- Rejeitada: adicionar dependência ou gerar nova migração SQLite — nenhuma necessidade comprovada.

**Impacto previsto:** apenas documentação (`README.md`, `docs/ROADMAP.md`, `docs/DECISIONS.md` e, se necessário, `AGENTS.md`); nenhuma regra, contrato, manifesto, workflow ou dependência muda.

**Arquivos previstos:** `README.md` (status das Etapas 4/5 e ajustes posteriores; modo automático já descrito no parágrafo de descoberta; `bundle.active = false` e ausência de instalador em "Limitações conhecidas"), `docs/ROADMAP.md` (Etapa 6 como planejada/em andamento, sem marcar subetapas futuras como concluídas), `docs/DECISIONS.md` (registro `E6.0` do baseline e decisões da 6.1), `AGENTS.md` somente se as regras atuais ficarem insuficientes.

**Estratégia de teste:** verificação somente leitura de consistência (nome do binário, `cliPath`, `when`, versões 0.1.0, `bundle.active`), seguida de `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace`, clippy, `git diff --check`, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, `npm --prefix extensions/vscode test` e `npm --prefix extensions/vscode run package`. Nenhuma E2E será reexecutada sem aprovação e configuração explícita de pré-requisitos.

## 11. Conclusão

Todos os comandos do baseline disponível foram executados e terminaram com código `0`, exceto `npm audit` (código `1` esperado pela presença de alertas de desenvolvimento) e a consulta à API do GitHub (HTTP `403`, contornada pela página HTML). As seis divergências da seção 2.3 foram reproduzidas ou descartadas com evidência. Nenhuma alteração local foi perdida.

A Etapa 6.0 está pronta para revisão humana. **A Etapa 6.1 não foi iniciada** e depende de aprovação explícita.
