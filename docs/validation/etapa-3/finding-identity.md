# Identidade dos achados — Etapa 3.2

Data: 26/11/2026  
Branch: `etapa-3/validacao-arquitetural`

## Cenário

A fixture `tests/fixtures/projects/finding_identity` tem três variantes do mesmo projeto:

| Variante | `src/lib.rs` | Objetivo |
|---|---|---|
| `before` | `use infrastructure::Client;` na linha 1 | linha de base |
| `after` | dois comentários e linha em branco antes do mesmo `use`, now na linha 4 | deslocamento por comentário |
| `evidence_changed` | `use infrastructure::{Client, Config};` na linha 1 | evidência diferente, mesmo vínculo |

As três variantes declaram `application` com `allowed_dependencies = ["domain"]` e `infrastructure` como módulo separado. Portanto, a dependência é proibida nas três.

## Fase vermelha

| Teste | Código de saída | Falha observada |
|---|---:|---|
| `inserted_comment_lines_do_not_create_a_new_finding` | `101` | ID `dep.not_allowed:application:infrastructure:src/lib.rs:1` contra `:4` |
| `repeated_forbidden_binding_is_reported_once` | `101` | 2 achados em vez de 1 |
| `legacy_line_suffixed_ids_compare_as_persistent` | `101` | `new_findings` não vazio |
| `changed_evidence_without_new_binding_keeps_identity` | `0` | já protegia o comportamento desejado |

O teste de ID legado também falhou uma vez após a correção por erro no gerador do relatório antigo: ele acrescentava `:<linha>` a todas as regras, inclusive às que nunca tiveram linha no ID. O gerador foi limitado a `dep.not_allowed` e `dep.unresolved`.

## IDs e localização após a correção

Comando executado:

```sh
cargo run -q -p auditor-cli -- audit tests/fixtures/projects/finding_identity/before --format json --output "$env:TEMP\vca-identity-before.json"
```

Código de saída: `1` (há achados). Trecho real do JSON:

```json
"id": "dep.not_allowed:application:infrastructure:src/lib.rs",
"line_start": 1,
"evidence": "use infrastructure::Client;"
```

Na variante `after`, o mesmo `id` aparece com `line_start: 4`; na variante `evidence_changed`, o mesmo `id` aparece com evidência `use infrastructure::{Client, Config};`.

## Histórico ponta a ponta

Comandos executados com banco temporário `%TEMP%\vca-etapa-3-2-identity.db`:

| Comando | Código | Resultado |
|---|---:|---|
| `cargo run -q -p auditor-cli -- history record tests/fixtures/projects/finding_identity/before --database $db` | `1` | execução #1 gravada; `id` sem linha e `local: src/lib.rs:1` |
| `cargo run -q -p auditor-cli -- history record tests/fixtures/projects/finding_identity/after --database $db` | `1` | execução #2 gravada; mesmo `id` e `local: src/lib.rs:4` |
| `cargo run -q -p auditor-cli -- history compare 1 2 --database $db` | `0` | 0 novos, 0 resolvidos, 2 persistentes |

O código `1` em `history record` é o contrato existente da CLI: a gravação também devolve o código da auditoria quando há achados.

## Verificação

| Comando | Código |
|---|---:|
| `cargo test -p auditor-core --lib inserted_comment_lines_do_not_create_a_new_finding` | `0` |
| `cargo test -p auditor-core --lib changed_evidence_without_new_binding_keeps_identity` | `0` |
| `cargo test -p auditor-core --lib identity_fixture_audit_is_deterministic` | `0` |
| `cargo test -p auditor-core --lib repeated_forbidden_binding_is_reported_once` | `0` |
| `cargo test -p auditor-core --lib identity_changes_with_source_target_or_rule` | `0` |
| `cargo test -p auditor-core --lib identity_key_ignores_only_legacy_dependency_line_suffix` | `0` |
| `cargo test -p auditor-core --lib legacy_line_suffixed_ids_compare_as_persistent` | `0` |
| `cargo fmt --check` | `0` |
| `cargo check --workspace` | `0` |
| `cargo test --workspace` | `0` — 154 testes: 141 no núcleo, 10 na CLI e 3 no desktop |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | `0` |
| `git diff --check` | `0`, apenas aviso LF/CRLF do OneDrive |
| `npm --prefix extensions/vscode test` | `0` — 4 testes |

## Limitações

- A identidade não sobrevive à renomeação de arquivo, pois o caminho é parte do vínculo.
- Duas importações do mesmo destino no mesmo arquivo são uma única ocorrência lógica; a primeira ocorrência determinística fornece localização e evidência.
- `history show` preserva IDs gravados no passado; a normalização ocorre somente na comparação.
- Nenhuma coluna, tabela ou versão de schema SQLite foi alterada nesta etapa.
