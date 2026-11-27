# Proposta de migração do histórico para o schema v3

## 1. Contexto

A Etapa 4.7 precisa manter CLI, desktop e extensão coerentes quanto ao modo de
execução (`discovery` ou `compliance`). O histórico SQLite v2 grava apenas a
versão do schema do contrato em `audit_runs.schema_version`; não há coluna para
modo nem para identidade determinística do contrato. Consequentemente, uma
execução de descoberta poderia ser comparada com uma de conformidade como se
fossem a mesma métrica.

Esta proposta é registrada antes de qualquer implementação, conforme exigido
pela Etapa 4.7 e pela seção 8 de `DISCOVERY_MODE_PROPOSAL.md`.

## 2. Decisão

Elevar `SCHEMA_VERSION` de `2` para `3` com duas colunas aditivas em
`audit_runs`:

- `execution_mode TEXT NOT NULL DEFAULT 'compliance'` — modo explícito da
  execução, com os valores serializados pelo núcleo (`discovery`, `compliance`);
- `contract_identity TEXT` — identidade curta e determinística do contrato
  aprovado, calculada por `contract::contract_identity()`. `NULL` significa
  **identidade desconhecida**, nunca “contrato vazio”.

A migração é aplicada por `ALTER TABLE` dentro da mesma transação que aplica as
migrações pendentes v1/v2, com registro em `schema_migrations`. Nenhuma linha
existente é apagada, reescrita ou reclassificada além do preenchimento feito pelo
próprio `DEFAULT`.

## 3. Compatibilidade

- **Banco v1:** recebe v1, v2 e v3 na mesma transação; a execução legada fica
  com `execution_mode = 'compliance'` e `contract_identity = NULL`.
- **Banco v2:** recebe somente v3; o mesmo preenchimento se aplica.
- **Banco v3:** nenhuma migração pendente.
- **Banco com versão maior que 3:** continua sendo recusado sem alteração, com
  `HistoryError::UnsupportedSchema`, como já ocorre hoje.
- **Leitura:** `RunSummary` passa a expor `execution_mode` e
  `contract_identity`. Execuções gravadas antes da v3 mostram identidade
  desconhecida; nenhum valor é inferido a partir do nome do projeto ou da raiz.

Todas as execuções anteriores só podiam ser gravadas por `audit_project()`,
que exige contrato válido. Por isso o valor legado `compliance` é uma
afirmação sobre a origem do registro, não uma adivinhança sobre o arquivo.

## 4. Regras de comparação

`History::compare()` passa a recusar, com `HistoryError::Incompatible` e
mensagem explícita:

1. execuções de modos diferentes;
2. duas execuções de descoberta (não há achados de conformidade para comparar);
3. execuções de conformidade com identidades diferentes — uma conhecida e outra
   desconhecida, ou ambas conhecidas e diferentes.

Somente são comparadas execuções de conformidade com a **mesma identidade
conhecida**. Nenhuma comparação é silenciosamente reclassificada.

## 5. Gravação

- `History::record(root, report)` continua disponível e passa a preencher modo e
  identidade: o modo vem de `report.mode`; a identidade é calculada lendo
  `auditor.toml` na raiz quando o relatório diz `compliance`. Se o arquivo não
  puder ser lido nesse ponto, a identidade fica desconhecida e a comparação será
  bloqueada — nunca é inventada.
- `History::record_run(root, &ProjectRun)` é o caminho usado por CLI e desktop.
  Para `Compliance`, delega a gravação do relatório. Para `Discovery`, grava
  contagens derivadas do inventário/candidatos/análise, `findings` vazio e
  `schema_version = 0` (explicitamente “sem contrato”).
- CLI e desktop usam `project_run()` como fonte única. Nenhum arquivo do projeto
  é modificado; apenas o banco local recebe metadados, contagens, achados,
  limitações e cobertura — nunca conteúdo integral de arquivos.

## 6. Testes de banco legado

- Banco v1 com execução existente é migrado para v3 sem perder a linha.
- Banco v2 com execução existente é migrado para v3 sem perder a linha.
- Versão maior que 3 continua recusada sem alteração destrutiva.
- Execução legada migrada tem modo `compliance` e identidade desconhecida;
  compará-la com uma execução atual produz erro explícito.
- Duas execuções com a mesma identidade conhecida continuam comparáveis.
- Gravação de descoberta registra modo `discovery`, identidade ausente e zero
  achados.

## 7. Alternativas rejeitadas

- **Codificar o modo em `schema_version`:** reutilizaria um campo com outro
  significado e impediria distinguir “sem contrato” de “contrato inválido”.
- **Tabela paralela de modo/identidade:** duplicaria a chave de execução e
  permitiria registros órfãos, tornando `compare` mais frágil.
- **Não registrar descoberta:** atenderia ao histórico antigo, mas deixaria de
  registrar o tipo de execução exigido pela Etapa 4.7.
- **Inventar identidade para linhas legadas:** faria comparações parecerem
  seguras sem evidência; a identidade permanece desconhecida.

## 8. Limitações registradas

- Comparação entre duas execuções de descoberta não é suportada nesta etapa; a
  recusa é explícita.
- Execuções legadas com identidade desconhecida não podem ser comparadas com
  execuções novas, mesmo quando o projeto é o mesmo.
- O modo legado `compliance` é inferido da origem histórica do registro, não de
  um campo gravado naquele momento.

## 9. Verificação planejada

Testes focados no núcleo (`history::tests`), no adaptador Tauri, na CLI e nas
suítes npm da extensão/desktop, seguidos da verificação completa descrita em
`AGENTS.md`. Nenhuma dependência Rust ou npm é adicionada.
