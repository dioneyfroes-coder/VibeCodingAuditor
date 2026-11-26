# Validação — Etapa 3.6: cobertura estruturada no histórico

Data: 26/11/2026
Escopo: gravar e comparar cobertura entre execuções históricas, com migração SQLite versionada. Sem mudança em contrato, `rule_id`, formato de `findings`, desktop ou extensão.

## Desenho verificado

Proposta aprovada antes da implementação em [docs/architecture/HISTORY_COVERAGE_V2_PROPOSAL.md](../../architecture/HISTORY_COVERAGE_V2_PROPOSAL.md); decisão final em `E3.12`. O histórico passa a ter `SCHEMA_VERSION = 2` e três tabelas agregadas por execução:

- `audit_run_coverage`: `code_files_included`, `code_files_analyzed`, `code_files_not_analyzed` e `partial`;
- `audit_coverage_unsupported_extensions`: extensão e quantidade de arquivos;
- `audit_coverage_skip_reasons`: posição estável de `SkipReason::ALL`, rótulo, contagem e impacto na cobertura.

Nenhum exemplo de caminho e nenhum conteúdo de arquivo é gravado. `history show` apresenta cobertura e motivos; `history compare` apresenta as duas coberturas, deltas de analisados/não analisados, extensões sem analisador e mudança nos motivos. Execuções anteriores à migração aparecem como `sem dados`.

## Testes

| Comando | Código de saída | Resultado |
|---|---:|---|
| `cargo test -p auditor-core history::tests` (antes da implementação) | `101` | fase vermelha: `RunDetail` e `ExecutionComparison` ainda não tinham os campos de cobertura |
| `cargo test -p auditor-core history::tests` (após a implementação) | `0` | 22 testes aprovados, incluindo migração v1→v2, banco já migrado, rollback transacional, versão futura rejeitada, gravação/recarga, comparação e 2 testes diretos dos helpers |
| `cargo test --workspace` | `0` | 193 testes aprovados: 179 no núcleo, 10 na CLI e 4 no desktop |
| `cargo fmt --all` | `0` | formatação aplicada |
| `cargo fmt --all --check` | `0` | sem diferenças |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | `0` | sem avisos |
| `git diff --check` e `git diff --cached --check` | `0` | apenas avisos LF/CRLF esperados do OneDrive |

Durante a implementação, duas asserções precisaram de ajuste: o helper de banco v1 passou a criar explicitamente `schema_migrations` (antes, essa criação só acontecia em `History::migrate()`), e a comparação passou a ser verificada como uma linha estável `Cobertura: antes -> depois`, em vez de uma continuação indentada. A mudança foi de formato esperado, não de comportamento.

## Smoke com banco temporário (build de depuração)

Banco: `C:\Users\dratz\AppData\Local\Temp\opencode\vca-history-v2-smoke.db`, criado vazio para este teste.

| Comando | Código de saída | Resultado |
|---|---:|---|
| `cargo run -q -p auditor-cli -- history --database <temp> record tests/fixtures/projects/partial_coverage` | `0` | execução #1; cobertura `2 de 4` parcial, `.go (1)` e `.rb (1)`, 1 motivo intencional |
| `cargo run -q -p auditor-cli -- history --database <temp> record tests/fixtures/projects/clean` | `0` | execução #2; cobertura `3 de 3` completa, 5 motivos intencionais |
| `cargo run -q -p auditor-cli -- history --database <temp> show 1` | `0` | mesmos dados da gravação, sem perda ao reabrir |
| `cargo run -q -p auditor-cli -- history --database <temp> compare 1 2` | `0` | `2 de 4 ... -> 3 de 3`, analisados `+1`, não analisados `-2`, sem analisador `.go (1), .rb (1) -> (nenhum)`, motivos `fora do include 1 -> 5` |

## Binários release recompilados

| Comando | Código de saída | Resultado |
|---|---:|---|
| `cargo build --release -p auditor-cli` | `0` | `target\release\vibe-coding-auditor.exe`: 5.786.112 bytes (antes: 5.760.512) |
| `npm run tauri build` (em `apps/desktop`) | `0` | `npm run build` com `tsc --noEmit` e Vite (CSS 3,99 kB; JS 230,99 kB), seguido do build Rust; `target\release\auditor-desktop.exe`: 14.241.792 bytes (antes: 14.235.136) |

Uma conferência de tamanho com caminho relativo executada dentro de `apps/desktop` falhou (`Get-Item` não encontrou `apps\desktop\target\release\...`); o caminho correto é a raiz do workspace e a verificação com caminho absoluto terminou com `0`. O build em si não falhou.

## Smoke com o binário release

Banco: `C:\Users\dratz\AppData\Local\Temp\opencode\vca-history-v2-release.db`.

| Comando | Código de saída | Resultado |
|---|---:|---|
| `vibe-coding-auditor.exe history --database <temp> record tests/fixtures/projects/partial_coverage` | `0` | execução #1 com cobertura estruturada |
| `vibe-coding-auditor.exe history --database <temp> record tests/fixtures/projects/clean` | `0` | execução #2 com cobertura completa |
| `vibe-coding-auditor.exe history --database <temp> compare 1 2` | `0` | mesmos deltas do smoke de depuração |

## Migração de um banco v1 real pelo binário release

O banco `C:\Users\dratz\AppData\Local\Temp\opencode\vca-history-v1-release.db` foi criado por `python C:\Users\dratz\AppData\Local\Temp\opencode\make_v1_history_db.py <db>` (`0`), com o DDL original de `MIGRATION_V1`, `schema_migrations` na versão `1` e uma execução gravada. O script temporário não faz parte do repositório.

| Comando | Código de saída | Resultado |
|---|---:|---|
| `vibe-coding-auditor.exe history --database <v1> show 1` | `0` | execução legada preservada; `Cobertura do código: sem dados (execução anterior à migração v2)` |
| `vibe-coding-auditor.exe history --database <v1> list` | `0` | execução #1 listada normalmente |
| `python C:\Users\dratz\AppData\Local\Temp\opencode\inspect_history_db.py <v1>` | `0` | versões `[(1,), (2,)]`; tabelas `audit_run_coverage`, `audit_coverage_skip_reasons`, `audit_coverage_unsupported_extensions` presentes |
| `python ...\inspect_history_db.py <v2>` | `0` | mesmas versões e tabelas no banco criado diretamente no schema v2 |

## Limitações mantidas

- `history compare` não verifica se as duas execuções pertencem ao mesmo projeto; a decisão permanece com o usuário.
- A comparação de cobertura está disponível apenas na CLI. Desktop e extensão não ganharam comandos ou painéis novos.
- O banco continua sem comando de limpeza e cresce com novas execuções.
- Não há fixture com symlink, permissão negada ou FIFO reais; os motivos correspondentes são exercitados com inventário em memória no construtor real do relatório.
- A instalação da extensão VS Code e a configuração `vibeCodingAuditor.cliPath` permanecem fora do repositório; o `.vsix` não precisa ser regenerado porque a extensão apenas invoca a CLI externa e não consome o histórico.

## Commits desta etapa

- `fabc84f` — proposta e testes de regressão (fase vermelha)
- `1e5cd00` — implementação do schema v2 e dos helpers de apresentação
- `ef5df40` — documentação e fechamento da Etapa 3
