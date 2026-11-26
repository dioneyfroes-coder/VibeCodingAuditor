# Proposta — cobertura estruturada no histórico (schema v2)

Data: 26/11/2026
Status: aprovada pelo usuário em 26/11/2026

## Motivo

A Etapa 3.6 ficou condicionada à confirmação de que comparar cobertura entre execuções históricas é um requisito real. O usuário confirmou. Hoje `audit_runs` guarda apenas `files_included` e `files_skipped`; `coverage` e `skip_summary` são descartados no `record`, e a única pista de cobertura parcial é uma mensagem livre em `audit_limitations`.

Comparar apenas arquivos incluídos/não incluídos não responde ao requisito: não distingue arquivos de código analisados dos não analisados, não mostra extensões sem analisador e não mostra motivos de exclusão.

## Alternativas avaliadas

1. **Interpretar a limitação textual.** Rejeitada: mensagem não é contrato de dados, muda com o texto e não cobre `skip_summary` nem falhas de leitura/parse sem extensão desconhecida. Repete a alternativa já recusada em `E2.1`.
2. **Gravar `AuditReport` inteiro em uma coluna JSON.** Rejeitada em `D6.2`: aumenta o volume do banco e aproxima o histórico do conteúdo dos arquivos auditados, sem necessidade para a comparação.
3. **Comparar só `files_included`/`files_skipped`, sem migração.** Rejeitada: atende apenas parte do requisito e esconderia a perda de cobertura real.
4. **Migração versionada com tabelas agregadas.** Escolhida: guarda o necessário para a comparação, sem exemplos de caminhos e sem conteúdo de arquivos.

## Desenho aprovado

`SCHEMA_VERSION` passa de `1` para `2`. A migração v2 cria:

- `audit_run_coverage`: `run_id`, `code_files_included`, `code_files_analyzed`, `code_files_not_analyzed`, `partial`;
- `audit_coverage_unsupported_extensions`: `run_id`, `extension`, `files`;
- `audit_coverage_skip_reasons`: `run_id`, `position`, `label`, `count`, `reduces_coverage`.

Rótulos vêm de `SkipReason::label()` e a posição vem de `SkipReason::ALL`, preservando a ordem estável dos relatórios sem duplicar o enum no banco. Exemplos de arquivos não são gravados.

`migrate()` aplica versões pendentes em ordem, registra o número de cada migração no `schema_migrations` e continua rejeitando apenas bancos escritos por versão posterior à suportada. A migração é transacional: se qualquer DDL falhar, a versão anterior permanece.

`history show` passa a exibir cobertura e motivos. `history compare` passa a exibir as duas coberturas, o delta de arquivos analisados/não analisados e os motivos de exclusão. Execuções gravadas antes da migração não ganham números inventados: aparecem como `sem dados`.

## Impacto

- Nenhuma mudança em `auditor.toml`, `rule_id`, formato de `findings`, comandos ou códigos de saída da CLI.
- Nenhuma mudança no desktop ou na extensão VS Code; a comparação continua disponível pela CLI. Isso evita adicionar comandos Tauri e painéis não pedidos.
- Bancos v1 são migrados em uma transação; bancos v2 são reabertos sem alteração; bancos com versão superior continuam falhando com `HistoryError::UnsupportedSchema`.
- A gravação de cobertura entra na transação já usada por `record`, sem nova dependência.

## Verificação exigida antes de concluir

Testes escritos antes da implementação: migração v1→v2 preservando execuções, banco já migrado, rollback transacional em falha, versão futura rejeitada, gravação e recarga de cobertura/motivos, comparação com coberturas diferentes e execução antiga sem dados. Depois da implementação: testes focados, suíte Rust completa, `cargo fmt`, `cargo clippy`, `git diff --check` e documentação atualizada.
