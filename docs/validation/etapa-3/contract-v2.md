# Etapa 3.5 — Implementação do contrato arquitetural v2

Data: 26/11/2026
Branch: `etapa-3/validacao-arquitetural`
Proposta aprovada: [docs/architecture/CONTRACT_V2_PROPOSAL.md](../../architecture/CONTRACT_V2_PROPOSAL.md)

## Escopo implementado

- Leitura de `schema_version = 1` e `2` no mesmo núcleo, sem reescrita de arquivos.
- `architecture.layers[]` com `name`, `modules` e `allowed_dependencies`, aceito somente no v2.
- Normalização explícita por `Contract::normalized_layers()`: v1 devolve `None`; v2 devolve a lista de camadas, possivelmente vazia.
- Validação v2 de paths sobrepostos, camada duplicada, módulo sem camada, módulo em duas camadas, auto-dependência de camada, destino desconhecido, dependência duplicada e ciclo entre camadas. Mensagens nomeiam o campo, por exemplo `architecture.layers[1].allowed_dependencies`.
- Nova regra `dep.layer_not_allowed`, com ID `dep.layer_not_allowed:{origem}:{destino}:{path}`. `dep.not_allowed` tem precedência na mesma aresta; aresta não resolvida continua apenas como `dep.unresolved`.
- Limitação explícita quando uma aresta resolvida não pode ser associada a camadas em tempo de execução, sem inventar `error`.
- Nenhuma alteração de schema SQLite, nenhuma dependência nova, nenhum parser novo e nenhuma regra na CLI, no desktop ou na extensão.

## Evidência de aceite

- Contratos v1 anteriores continuaram com os mesmos resultados nas fixtures existentes.
- Contrato v2 válido com camadas foi aceito; v2 sem camadas também foi aceito.
- `schema_version = 0` e `3` foram rejeitados; v1 com `architecture.layers` foi rejeitado com mensagem que cita `schema_version = 2`.
- Ordem dos campos no TOML e ordem das camadas não alteraram o contrato normalizado nem os achados.
- Contrato v2 equivalente à autoauditoria e ao alvo sintético da Etapa 3.3 produziu o mesmo resumo, os mesmos achados, as mesmas limitações e a mesma cobertura do contrato v1 original. Os arquivos `auditor.toml` existentes não foram modificados; as versões equivalentes estão em `self-audit-v2.toml` e `fake-order-service-v2.toml` neste diretório.
- Fixture ponta a ponta `tests/fixtures/projects/layer_boundaries`: 5 arquivos incluídos, 1 intencionalmente não incluído, 5 arestas, 3 achados (2 erros e 1 revisão). Texto, JSON e Markdown contêm o mesmo ID de camada.
- O comando `audit` retornou `1` na fixture v2, como esperado quando há achados; falha operacional continuaria retornando `2`.
- `history record` gravou o novo `rule_id` com `schema_version = 2`; `history compare` manteve o achado como persistente e `history show` o exibiu sem migração de banco.
- Desktop e extensão consumiram o achado do núcleo sem implementar regra própria.
- `auditor init` continua gerando v1. O README descreve v2 somente depois da implementação e dos testes.

## Comandos executados e resultados

| Comando | Código | Resultado |
|---|---:|---|
| `cargo fmt --all` | `0` | formatação aplicada |
| `cargo fmt --check` | `0` | sem diferenças |
| `cargo check --workspace` | `0` | todos os crates compilados |
| `cargo test --workspace` | `0` | 185 testes Rust: 171 no núcleo, 10 na CLI e 4 no desktop |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | `0` | sem avisos |
| `npm test` em `apps/desktop` | `0` | 2 testes passaram |
| `npm run build` em `apps/desktop` | `0` | frontend TypeScript/React compilado |
| `npm test` em `extensions/vscode` | `0` | 5 testes passaram |
| `git diff --check` | `0` | apenas avisos LF/CRLF do OneDrive |

Todos os comandos da tabela foram repetidos depois da atualização de `README.md`, `docs/ROADMAP.md` e `docs/DECISIONS.md`, com os mesmos códigos de saída e contagens.
| `cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format text` | `1` | 3 achados, incluindo `dep.layer_not_allowed:application:infrastructure:src/application/rules.rs` |
| `cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format json` | `1` | `"schema_version": 2` e o mesmo `rule_id` |
| `cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format markdown` | `1` | mesmo ID e mesma regra no Markdown |
| `cargo run -q -p auditor-cli -- history --database <temporário> record tests/fixtures/projects/layer_boundaries` (duas vezes) | `1` | duas execuções gravadas; código `1` é esperado porque há achados |
| `cargo run -q -p auditor-cli -- history --database <temporário> compare 1 2` | `0` | 3 achados persistentes, incluindo a regra de camada |
| `cargo run -q -p auditor-cli -- history --database <temporário> show 2` | `0` | `Schema do contrato: v2` e a regra de camada presentes |

Uma execução de `npm test` na raiz do repositório retornou `-4058` porque não há `package.json` nesse nível. O comando foi repetido no diretório correto da extensão e passou, conforme a tabela.

## Limitações que permanecem

- Domínios/subdomínios, exceções locais, limiares de métrica e linguagem genérica de regras continuam fora do contrato.
- A checagem de sobreposição de paths é conservadora e pode rejeitar padrões disjuntos na prática.
- A ausência de mapeamento de camada em tempo de execução é tratada como limitação; a validação impede que isso venha de um contrato v2 válido.
- `responsibility` continua sendo prosa para revisão humana, sem interpretação automática.
- Os contratos v2 equivalentes são evidência de compatibilidade sobre o mesmo código; não substituem validação futura em projeto real de terceiros.
