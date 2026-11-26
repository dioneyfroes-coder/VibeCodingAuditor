# Autoauditoria da Etapa 3.1

Data: 26/11/2026  
Branch: `etapa-3/validacao-arquitetural`  
Contrato: `auditor.toml` na raiz, submetido como `0a1eac4`  
Escopo decidido pelo usuário: somente código Rust de produção.

## Contrato e fronteiras

| Módulo | `path` | `allowed_dependencies` | Responsabilidade registrada |
|---|---|---|---|
| `core` | `crates/auditor-core/src/**` | `[]` | Contrato, descoberta, análise, regras, relatórios e histórico local; não conhece adaptadores. |
| `cli` | `crates/auditor-cli/src/**` | `["core"]` | Comandos auditáveis que delegam toda regra ao núcleo. |
| `desktop-backend` | `apps/desktop/src-tauri/src/**` | `["core"]` | Comandos Tauri que delegam contrato, análise, auditoria e histórico ao núcleo. |

`scan.include` cobre apenas os três diretórios acima. `scan.exclude` remove `tests/fixtures/**`, `apps/desktop/src/**`, `extensions/vscode/src/**`, `target/**` e `node_modules/**`. A interface React e a extensão VS Code não são módulos deste contrato: o grafo atual não observa a comunicação entre TypeScript e o backend Tauri nem a chamada da extensão à CLI, portanto essas fronteiras **não** são declaradas como verificadas. `responsibility` é descrição humana, não regra semanticamente avaliada.

## Comandos executados

Executados na raiz do repositório, no PowerShell:

| Comando | Saída | Código |
|---|---|---:|
| `cargo run -p auditor-cli -- scan .` | 15 arquivos incluídos; 47 entradas não incluídas | `0` |
| `cargo run -p auditor-cli -- analyze .` | 3 módulos; 3 pacotes; 17 arestas; 8 revisões; cobertura 15/15; 0 limitações | `0` |
| `cargo run -p auditor-cli -- audit .` | 62 achados, todos `needs_review` | `1` |
| `cargo run -p auditor-cli -- audit . --format json --output "$env:TEMP\vca-self-audit.json"` | JSON gravado (56.295 bytes) | `1` |
| `cargo run -p auditor-cli -- audit . --format markdown --output "$env:TEMP\vca-self-audit.md"` | Markdown gravado (46.996 bytes) | `1` |

As saídas textuais foram capturadas em `$env:TEMP\vca-self-audit-scan.txt`, `vca-self-audit-analyze.txt` e `vca-self-audit.txt`. O código `1` é o contrato da CLI: há pelo menos um achado. Não houve falha operacional (`2`).

A primeira execução, antes da correção de `crate::`, produziu 108 achados: 54 `dep.unresolved` e 54 `code.duplicated`. Desse total, 46 `dep.unresolved` vinham de `use crate::analysis`, `crate::contract` e outros submódulos internos de `core`. A correção está registrada em `docs/DECISIONS.md` (`E3.3`).

## Inventário

Raiz canonicalizada: `\\?\C:\Users\dratz\OneDrive\Documentos\workspace\VibeCodingAuditor`.

Arquivos incluídos (15):

- `apps/desktop/src-tauri/src/lib.rs` (4.567 bytes)
- `apps/desktop/src-tauri/src/main.rs` (110 bytes)
- `crates/auditor-cli/src/main.rs` (18.394 bytes)
- `crates/auditor-core/src/analysis/mod.rs` (83.809 bytes)
- `crates/auditor-core/src/analysis/python.rs` (19.724 bytes)
- `crates/auditor-core/src/analysis/typescript.rs` (25.916 bytes)
- `crates/auditor-core/src/audit.rs` (31.987 bytes)
- `crates/auditor-core/src/contract/mod.rs` (12.935 bytes)
- `crates/auditor-core/src/diff.rs` (18.441 bytes)
- `crates/auditor-core/src/discovery/mod.rs` (16.448 bytes)
- `crates/auditor-core/src/findings/mod.rs` (3.149 bytes)
- `crates/auditor-core/src/history/mod.rs` (37.680 bytes)
- `crates/auditor-core/src/lib.rs` (776 bytes)
- `crates/auditor-core/src/report/mod.rs` (29.005 bytes)
- `crates/auditor-core/src/rules/mod.rs` (38.043 bytes)

Entradas não incluídas (47), todas intencionais: 7 por padrão ignorado (`.git`, `apps/desktop/dist`, `apps/desktop/node_modules` e +4), 3 por `scan.exclude` (`apps/desktop/src`, `extensions/vscode/src`, `tests/fixtures`) e 37 fora do `include` (`.github/workflows/ci.yml`, `.gitignore`, `AGENTS.md` e +34). Nenhuma reduziu a cobertura.

## Cobertura do código

| Campo | Valor |
|---|---:|
| `code_files_included` | 15 |
| `code_files_analyzed` | 15 |
| `code_files_not_analyzed` | 0 |
| `unsupported_code_extensions` | vazio |
| `partial` | `false` |

`analysis.limitations` ficou vazio. Declarações `mod` registradas: 25.

## Grafo e dependências

Pacotes Cargo reconhecidos: `auditor-cli`, `auditor-core` e `auditor-desktop`. A lista `declared_dependencies` contém 10 nomes normalizados: `auditor_core`, `clap`, `rusqlite`, `serde`, `serde_json`, `syn`, `tauri`, `tauri_build`, `tauri_plugin_dialog` e `toml`. Desses, somente `auditor_core` é dependência local `path` e foi associado ao módulo `core`; os outros nove são externos e **não** viraram módulos internos, arestas ou revisões.

Arestas: 17 no total — 9 `permitidas`, 0 `proibidas`, 8 `não resolvidas`.

Relações internas reconhecidas:

| Origem | Destino | Arquivo | Linha reportada | Evidência reportada |
|---|---|---|---:|---|
| `cli` | `core` | `crates/auditor-cli/src/main.rs` | 344 | `use super::dependency_kind_label;` |
| `cli` | `core` | `crates/auditor-cli/src/main.rs` | 345 | `use auditor_core::analysis::{AnalysisCoverage, DependencyKind};` |
| `cli` | `core` | `crates/auditor-cli/src/main.rs` | 346 | `use auditor_core::report::ReportFormat;` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 10 | `use auditor_core::analysis::{self, Analysis};` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 11 | `use auditor_core::audit;` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 12 | `use auditor_core::contract::load_contract;` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 13 | `use auditor_core::discovery::discover;` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 14 | `use auditor_core::history::{History, RunSummary};` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 15 | `use auditor_core::report::AuditReport;` |

As três arestas `cli -> core` são classificadas corretamente, mas suas linhas/evidências aparecem deslocadas: os imports `auditor_core` reais estão nas linhas 345–347. Esse descompasso é uma limitação conhecida do pareamento textual de `use` agrupados e está registrada em `docs/DECISIONS.md` (`E3.3`) e no `README.md`; ele não altera a classificação `permitida`, mas impede usar a evidência como local exato sem conferência humana.

As 8 arestas `não resolvidas` são revisões, não violações comprovadas:

| Arquivo | Linha | Destino | Evidência |
|---|---:|---|---|
| `crates/auditor-cli/src/main.rs` | 336 | `Args` | `use clap::error::ErrorKind;` |
| `crates/auditor-cli/src/main.rs` | 338 | `Commands` | `use super::Args;` |
| `crates/auditor-cli/src/main.rs` | 339 | `HistoryCommands` | `use super::Commands;` |
| `crates/auditor-cli/src/main.rs` | 340 | `OutputFormat` | `use super::HistoryCommands;` |
| `crates/auditor-cli/src/main.rs` | 341 | `Path` | `use super::OutputFormat;` |
| `crates/auditor-cli/src/main.rs` | 342 | `audit_exit_code` | `use super::Path;` |
| `crates/auditor-cli/src/main.rs` | 343 | `dependency_kind_label` | `use super::audit_exit_code;` |
| `crates/auditor-core/src/lib.rs` | 23 | `CORE_NAME` | `use super::CORE_NAME;` |

Todas são importações de símbolos do próprio módulo dentro de `#[cfg(test)] mod tests`. O analisador não resolve símbolos Rust, apenas arquivos e módulos do contrato, então elas permanecem como `dep.unresolved` em vez de serem descartadas silenciosamente. A primeira linha da tabela também sofre o deslocamento de evidência descrito acima: o destino `Args` vem de `use super::Args;` na linha 338.

## Achados e severidades

Resumo do JSON: `files_included: 15`, `files_skipped: 47`, `modules: 3`, `packages: 3`, `edges: 17`, `findings: 62`, `errors: 0`, `warnings: 0`, `infos: 0`, `needs_review: 62`.

| `rule_id` | Quantidade | Interpretação |
|---|---:|---|
| `code.duplicated` | 54 | Repetição exata de linhas com 40 ou mais caracteres em mais de um arquivo; exemplos incluem `let inventory = discover(&root, &contract).expect("discovery must succeed");` e linhas de asserção em testes. É heurística, não prova de dívida. |
| `dep.unresolved` | 8 | Imports `super::` de símbolos internos em testes, listados acima. |

Não houve `dep.not_allowed`, `file.unmapped`, `file.ambiguous_module`, `dep.cycle`, `module.no_files`, `module.concentration` nem `module.unreferenced`. Os três módulos possuem arquivos incluídos e arestas resolvidas incidentes: `core` recebe de `cli` e `desktop-backend`; `cli` e `desktop-backend` apontam para `core`. Isso é um proxy estrutural, não prova de ausência de código morto; a decisão de refatorar continua humana.

## Limitações visíveis

- `analysis.limitations` e o resumo de limitações do relatório estão vazios: nenhum arquivo incluído deixou de ser lido ou interpretado.
- As 8 revisões `dep.unresolved` são símbolos `super::` em testes e não representam fronteiras arquiteturais decididas.
- A heurística `code.duplicated` compara linhas exatas, sem normalização, semântica ou blocos; repetições intencionais precisam de avaliação humana.
- A evidência de alguns `use` agrupados está deslocada em uma linha, conforme detalhado no grafo.
- React, Tauri frontend, extensão VS Code e seus pacotes npm não entram nesta autoauditoria; nenhuma aresta entre TypeScript e Rust foi inventada.
- Nenhuma dependência externa comum foi classificada como módulo interno, e o núcleo não recebeu aresta proibida da CLI ou do backend.

## Verificação final após a correção

| Comando | Resultado | Código |
|---|---|---:|
| `cargo fmt --check` | sem diferenças | `0` |
| `cargo check --workspace` | três crates verificados | `0` |
| `cargo test --workspace` | 147 testes: 134 no núcleo, 10 na CLI, 3 no desktop; 0 falhas | `0` |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | sem avisos | `0` |
| `git diff --check` | sem erros; apenas aviso LF/CRLF esperado do OneDrive | `0` |
| `npm --prefix apps/desktop ci` | instalado | `0` |
| `npm --prefix apps/desktop test` | 2 testes | `0` |
| `npm --prefix apps/desktop run build` | build concluído | `0` |
| `npm --prefix extensions/vscode ci` | instalado com avisos conhecidos: preset `secretlint` deprecado, `allow-scripts` pendente para `@vscode/vsce-sign@2.1.1` e 2 vulnerabilidades moderadas | `0` |
| `npm --prefix extensions/vscode test` | 4 testes | `0` |

A fase vermelha da correção de `crate::` foi executada antes da implementação: `cargo test -p auditor-core crate_paths_to_internal_modules_stay_internal` terminou com `exit code 101`; após a correção, esse teste e os de fallback, `crate::self`, `super::` e dependência `path` retornaram `0`.

## Conclusão da Etapa 3.1

Os critérios da Etapa 3.1 foram atendidos com as limitações acima: `cli -> core` e `desktop-backend -> core` aparecem no grafo quando o contrato os define; dependências externas não são falsamente internalizadas; o núcleo não tem aresta proibida vinda dos adaptadores; cobertura e exclusões ficam visíveis; o contrato documenta cada módulo e dependência permitida; e nenhuma fronteira não observável é apresentada como verificada. As Etapas 3.2 a 3.7 permanecem pendentes.
