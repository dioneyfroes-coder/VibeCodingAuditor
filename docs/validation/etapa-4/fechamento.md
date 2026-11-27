# Fechamento da Etapa 4 — validação em projetos de teste

Data: 26/11/2026

Este registro cobre a Etapa 4.8 do plano. Nenhuma dependência Rust ou npm foi adicionada. Nenhum comando executou código dos projetos analisados, acessou rede ou gravou `auditor.toml` nas fixtures de descoberta.

## Fixtures criadas

Sob `tests/fixtures/projects/`:

| Fixture | Cenário |
|---|---|
| `discovery_rust_no_contract` | Rust sem contrato, com import interno e externo observado |
| `discovery_ts_no_contract` | TypeScript com `package.json` e imports locais, sem contrato |
| `discovery_python_no_contract` | Python com `import`/`from ... import`, sem contrato |
| `discovery_workspace` | workspace Cargo com dois membros e `package.json`, sem contrato |
| `discovery_ambiguous` | dois manifests com o mesmo nome de pacote, sem contrato |
| `discovery_empty_project` | diretório sem código; marcador `.gitkeep` fora do `include` |
| `builder_existing_project` | projeto descoberto com módulos, sem contrato |
| `builder_contract_v1` | cópia de `clean`, contrato v1 |
| `builder_contract_v2` | cópia de `layer_boundaries`, contrato v2 com camadas |
| `builder_write_safety` | cópia de `clean` para tentativas de Create/Replace e falha de temporário |

## Integridade e determinismo

O teste `crates/auditor-cli/tests/fixtures.rs` executa a CLI como processo real duas vezes por fixture, compara byte a byte a saída JSON e o conteúdo completo da árvore antes e depois. Também foi feita uma verificação externa com SHA-256 agregado por árvore, antes e depois de `audit --format json`:

| Fixture | Código da CLI | Hash agregado igual |
|---|---:|---|
| `discovery_rust_no_contract` | 0 | sim (`72b7de5302d35a9d…`) |
| `discovery_ts_no_contract` | 0 | sim (`4a9e309440dbf388…`) |
| `discovery_python_no_contract` | 0 | sim (`ff6596dbf380274f…`) |
| `discovery_workspace` | 0 | sim (`29aaf7125221e82a…`) |
| `discovery_ambiguous` | 0 | sim (`f75de9ba5584bfe4…`) |
| `discovery_empty_project` | 0 | sim (`88b2de425c435ffe…`) |
| `builder_existing_project` | 0 | sim (`29aaf7125221e82a…`) |

Todas as saídas trouxeram `"mode": "discovery"`, `"status": "missing"` e `"findings": []`; nenhuma trouxe `dep.not_allowed` ou `dep.layer_not_allowed`.

## Smoke test da CLI

Comandos executados com `cargo run -q -p auditor-cli -- …`:

| Comando | Código | Resultado observado |
|---|---:|---|
| `scan tests/fixtures/projects/discovery_rust_no_contract` | 0 | `Modo: descoberta (sem contrato)` |
| `analyze tests/fixtures/projects/discovery_rust_no_contract` | 0 | `Modo: descoberta (sem contrato)` |
| `audit tests/fixtures/projects/discovery_rust_no_contract --format json` | 0 | `mode: discovery`, `status: missing`, `findings: []` |
| `audit tests/fixtures/projects/clean --format json` | 0 | `schema_version: 1`, `mode: compliance`, 0 achados |
| `audit tests/fixtures/projects/layer_boundaries --format json` | 1 | `schema_version: 2`, `mode: compliance`, 3 achados (2 erros e 1 revisão) |
| `audit tests/fixtures/projects/discovery_invalid_contract --format json` | 2 | falha explícita, sem fallback para descoberta |

Contratos v1 e v2 foram lidos e não foram reescritos: os testes de integração comparam os bytes de `auditor.toml` antes e depois da auditoria, e os testes do Builder comparam o rascunho gerado sem gravar.

## Histórico

Com banco temporário em `C:\Users\dratz\AppData\Local\Temp\opencode\vca-etapa48.db`, removido ao final:

| Comando | Código | Resultado observado |
|---|---:|---|
| `history --database <temp> record tests/fixtures/projects/discovery_rust_no_contract` | 0 | registro `#1`, `Modo: descoberta (sem contrato)` |
| `history --database <temp> record tests/fixtures/projects/clean` | 0 | registro `#2`, `Modo: conformidade`, identidade `943758ce7d0ac0db` |
| `history --database <temp> list` | 0 | `#1` mostra `modo descoberta | sem contrato | schema v0`; `#2` mostra modo, identidade e schema v1 |
| `history --database <temp> compare 1 2` | 2 | `execuções #1 e #2 não são comparáveis: as execuções usam modos diferentes` |

## Segurança do Builder

Testes focados em `crates/auditor-core/src/builder.rs`:

- `empty_project_fixture_accepts_a_planned_contract`: projeto vazio aceita contrato planejado e reabre em conformidade.
- `cancelling_before_save_creates_no_contract`: descobrir, montar rascunho e fechar não cria nem altera arquivo.
- `v1_contract_fixture_is_read_without_rewrite` e `v2_contract_fixture_keeps_layers_in_the_review_draft`: leitura/diff sem gravação.
- `write_safety_fixture_refuses_overwrite_and_failed_temporary_creation`: `Create` recusa existente, `Replace` exige texto exato, falha no temporário preserva o contrato e não deixa resíduo `.tmp`.

## Comandos e resultados

| Comando | Código |
|---|---:|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo check --workspace` | 0 |
| `cargo test --workspace` | 0 |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | 0 |
| `cargo test -p auditor-core --lib builder::tests` | 0 — 29 testes |
| `cargo test -p auditor-cli --test fixtures` | 0 — 5 testes |
| `npm --prefix apps/desktop ci` | 0 |
| `npm --prefix apps/desktop test` | 0 — 33 testes |
| `npm --prefix apps/desktop run build` | 0 |
| `npm --prefix extensions/vscode ci` | 0 |
| `npm --prefix extensions/vscode test` | 0 — 12 testes |
| `npm --prefix extensions/vscode run package` | 0 — `vibe-coding-auditor-0.1.0.vsix` |
| `git diff --check` | 0 — somente avisos LF/CRLF do OneDrive |

`cargo test --workspace` executou 275 testes Rust: 239 no núcleo, 10 unitários na CLI, 6 de integração na CLI, 5 de fixtures na CLI e 15 no desktop. Nenhum teste falhou.

## Validação manual pendente

Não foi executada validação manual da interface desktop nem da extensão dentro da host real do VS Code nesta etapa, e não há confirmação explícita do usuário. Os testes npm, o build do desktop e o empacotamento da extensão foram executados, mas não substituem essa validação. Esta ressalva permanece registrada até confirmação.

## Ajuste pós-fechamento — 27/11/2026

A pedido do usuário, o executável da CLI foi renomeado para `vibe-coding-auditor-cli` e a execução sem argumentos passou a imprimir apresentação e ajuda com código `0`.

| Comando | Código | Resultado |
|---|---:|---|
| `cargo build --release -p auditor-cli` | 0 | `target/release/vibe-coding-auditor-cli.exe` (6.302.720 bytes) |
| `vibe-coding-auditor-cli.exe` sem argumentos | 0 | apresentação + `Usage: vibe-coding-auditor-cli [COMMAND]` |
| `vibe-coding-auditor-cli.exe --version` | 0 | `vibe-coding-auditor-cli 0.1.0` |
| `vibe-coding-auditor-cli.exe --help` | 0 | ajuda do Clap com todos os subcomandos |

Os binários antigos `target/release/vibe-coding-auditor.exe` e `target/debug/vibe-coding-auditor.exe` foram removidos. A suíte completa foi reexecutada: `cargo test --workspace` = `0` (277 testes: 239 no núcleo, 11 unitários na CLI, 7 de integração na CLI, 5 de fixtures na CLI e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `npm --prefix extensions/vscode test` = `0` (12) e `npm --prefix extensions/vscode run package` = `0`. Os testes e o build do desktop também permaneceram em `0`. Nenhuma regra, schema, contrato ou dependência foi alterado.

## Validação manual — 27/11/2026

O usuário executou manualmente o desktop com o próprio VibeCodingAuditor e o projeto BackBet e confirmou que os fluxos examinados funcionaram. Em pergunta explícita, respondeu que a validação foi **somente do desktop**; a extensão na host real do VS Code não foi executada nesta rodada e continua pendente. Ele relatou quatro ressalvas de apresentação: excesso de informações, página enorme com barra de rolagem, resíduos de um projeto ao trocar para outro e linguagem muito técnica. A Etapa 4 não é declarada totalmente concluída enquanto a validação da extensão estiver pendente.

## Etapa 4.9 — Interface mais limpa — 27/11/2026

A pedido do usuário, a tela passou a limpar todos os resultados ao trocar de pasta, a exibir apenas uma aba de resultado por vez, a limitar listas longas com “Ver mais” e detalhes recolhidos, e a traduzir rótulos técnicos para linguagem corrente. Adicionados `apps/desktop/src/viewState.ts` e `apps/desktop/tests/viewState.test.mjs`; nenhuma regra, schema, contrato ou dependência mudou.

| Comando | Código | Resultado |
|---|---:|---|
| `cargo fmt --all` | 0 | formatação aplicada |
| `cargo fmt --all --check` | 0 | sem diferenças |
| `cargo check --workspace` | 0 | workspace compila |
| `cargo test --workspace` | 0 | 277 testes Rust, nenhuma falha |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | 0 | sem avisos |
| `npm --prefix apps/desktop test` | 0 | 39 testes Node |
| `npm --prefix apps/desktop run build` | 0 | build Vite concluído |
| `npm --prefix extensions/vscode test` | 0 | 12 testes Node |
| `npm --prefix extensions/vscode run package` | 0 | `vibe-coding-auditor-0.1.0.vsix` |
| `npm --prefix apps/desktop run tauri build` | 0 | `target/release/auditor-desktop.exe` |
| `git diff --check` | 0 | somente avisos LF/CRLF do OneDrive |

Smoke da janela: `auditor-desktop.exe` iniciou com `RESPONDING=True`, `MainWindowHandle` não nulo e título `Vibe Coding Auditor`. Não houve nova validação visual pelo usuário após esta mudança de interface; os testes cobrem os helpers de apresentação e limpeza de estado, não a aparência final.
