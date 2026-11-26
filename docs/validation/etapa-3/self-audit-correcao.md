# Correção da confiabilidade da autoauditoria

Data: 26/11/2026
Branch: `etapa-3/validacao-arquitetural`
Contrato: `auditor.toml` na raiz, sem alteração
Escopo decidido pelo usuário: somente código Rust de produção.

Este relatório complementa `self-audit.md` e registra a revalidação pedida após a Etapa 3. A pergunta respondida não foi “quantos achados a ferramenta gera?”, mas “os achados apontam para o código correto e respeitam o escopo declarado?”.

## Defeitos confirmados antes da correção

| Defeito | Evidência observada | Causa |
|---|---|---|
| Linha e evidência deslocadas | `cli -> core` aparecia em `crates/auditor-cli/src/main.rs:344` com evidência `use super::dependency_kind_label;`; os imports reais estavam em 345–347. Sete `dep.unresolved` também tinham linha errada. | `locate_source_line()` percorria o texto com um cursor de linhas `use`, mas o visitante de itens do `syn` não desce ao corpo de funções. O `use auditor_core::history::{self, History};` na linha 291 consumia uma posição do cursor. |
| Escopo de produção violado | Os 8 `dep.unresolved` vinham de `#[cfg(test)] mod tests`; boa parte dos 63 `code.duplicated` eram asserções e helpers idênticos de teste. | `visit_items()` entrava em módulos inline `#[cfg(test)]` e `collect_duplicate_lines()` lia o arquivo inteiro. |
| Falso negativo revelado | Depois de excluir testes, `module.unreferenced:cli` apareceu apesar das relações reais com o núcleo. | O CLI usa `auditor_core::...` qualificado em produção; o analisador só interpretava declarações `use`. Antes, os `use` de teste sustentavam a aresta. |

Estado inicial medido: `analyze .` → `0`, 17 arestas, 8 revisões, 15/15 arquivos analisados, 0 limitações; `audit .` → `1`, 71 achados (63 `code.duplicated` e 8 `dep.unresolved`).

## Incrementos aprovados e implementados

| Incremento | Commit | Mudança |
|---|---|---|
| A — localização real | `1a42a97` | `use` e `mod` usam spans do `proc_macro2` com `span-locations`; o cursor textual foi removido. |
| B — escopo de produção | `14ed6e8` | Módulos `#[cfg(test)]` não geram arestas, declarações `mod` nem candidatos a duplicação. |
| C — caminhos qualificados | `04c976a` | O visitante do `syn` classifica caminhos qualificados em expressões e tipos quando a raiz é `crate`, `self`, `super`, módulo do contrato ou dependência local declarada. |

Nenhum `rule_id` novo, nenhum campo novo em `findings`, nenhuma alteração em `auditor.toml`, comandos/códigos da CLI, desktop, extensão ou SQLite. Nenhum pacote novo entrou em `Cargo.lock`: `proc-macro2` já era transitivo e `syn` já estava no workspace. `proc-macro2` passou a ser dependência direta do núcleo com `span-locations`, e `syn` passou a declarar a feature `visit`.

## Fase vermelha

Os testes de regressão foram escritos antes da implementação de cada incremento:

| Teste | Falha antes da correção |
|---|---|
| `module_level_use_keeps_its_own_source_line` | `Some(2)` no lugar de `Some(6)`; `exit code 101`. |
| `cfg_test_modules_are_outside_the_production_scope` | aresta `use domain::Other;` de `#[cfg(test)]` presente; `exit code 101`. |
| `qualified_paths_in_production_become_edges` | nenhuma aresta para `domain::Value` em expressão; `exit code 101`. |

Durante o incremento C, a primeira versão classificava qualquer caminho de dois segmentos e produziu 444 revisões falsas (`Vec::new`, `fmt::Result`, `fs::read`). O teste existente `unresolved_dependency_becomes_review` falhou com `crates/application/src/lib.rs` onde era esperado `infra/src/unresolved.rs`. O filtro de raiz foi adicionado antes de qualquer commit do incremento C.

Fixtures novas, usadas somente por testes e fora do escopo da autoauditoria:

- `tests/fixtures/projects/source_lines` — `use` dentro de função antes de imports de módulo.
- `tests/fixtures/projects/production_scope` — dependência e duplicação apenas em `#[cfg(test)]` convivendo com código de produção.
- `tests/fixtures/projects/qualified_paths` — caminho qualificado em expressão e tipo, biblioteca padrão e código de teste.

## Estado final medido

| Comando | Resultado | Código |
|---|---|---:|
| `cargo run -q -p auditor-cli -- analyze .` | 3 módulos; 3 pacotes; 22 arestas; 0 revisões; 15/15 arquivos analisados; 0 limitações | `0` |
| `cargo run -q -p auditor-cli -- audit . --format json --output <temporário>` | 32 achados, todos `code.duplicated`; 0 erros, 0 avisos, 0 informações | `1` |

Evolução: 71 → 33 → 32 achados; 8 → 0 revisões; 17 → 22 arestas. O código `1` continua significando “há achados”, não falha operacional.

Exemplos de evidência corrente no grafo:

| Origem | Destino | Arquivo | Linha | Evidência |
|---|---|---|---:|---|
| `cli` | `core` | `crates/auditor-cli/src/main.rs` | 100 | `impl From<OutputFormat> for auditor_core::report::ReportFormat {` |
| `cli` | `core` | `crates/auditor-cli/src/main.rs` | 183 | `let analysis = auditor_core::analysis::analyze(root, &contract, &inventory)?;` |
| `desktop-backend` | `core` | `apps/desktop/src-tauri/src/lib.rs` | 10 | `use auditor_core::analysis::{self, Analysis};` |

As 32 duplicações restantes são heurísticas sobre linhas idênticas de produção; não foram silenciadas porque não há evidência de que sejam falsos positivos. A decisão sobre elas permanece com o usuário.

## Verificação completa

Executada na raiz do repositório, no PowerShell, sobre o estado final:

| Comando | Resultado | Código |
|---|---|---:|
| `cargo fmt --all` | formatação aplicada | `0` |
| `cargo fmt --all --check` | sem diferenças | `0` |
| `cargo check --workspace` | sem erros | `0` |
| `cargo test --workspace` | 194 testes (180 núcleo, 10 CLI, 4 desktop) | `0` |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | sem avisos | `0` |
| `git diff --check` | sem problemas de whitespace; apenas avisos LF/CRLF do OneDrive | `0` |
| `npm --prefix apps/desktop test` | 12 testes | `0` |
| `npm --prefix apps/desktop run build` | CSS 3,99 kB; JS 230,99 kB | `0` |
| `npm --prefix extensions/vscode test` | 7 testes | `0` |

## Limitações que permanecem explícitas

- O visitante do `syn` não interpreta tokens dentro de atributos: `#[arg(long, default_value = auditor_core::history::DEFAULT_DATABASE, global = true)]` continua fora do grafo, embora existam outras evidências da mesma dependência.
- Caminhos qualificados cuja raiz é um apelido local ou pacote externo não declarado como dependência local não são classificados. Isso evita confundir `Vec::new()` e `fmt::Result` com fronteiras arquiteturais.
- `code.duplicated` continua detectando repetição exata de linha, não semântica.
- Fronteiras React ↔ Tauri e CLI ↔ extensão continuam fora do grafo e não são declaradas como verificadas.
- A comparação de histórico pode mostrar achados antigos de teste como resolvidos; a identidade dos achados e o schema do banco não foram alterados.

A decisão técnica está registrada em `docs/DECISIONS.md` (`E3.13`).
