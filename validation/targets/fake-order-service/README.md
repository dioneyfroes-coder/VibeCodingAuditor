# Fake Order Service

Projeto sintético criado somente para a validação da Etapa 3.3. Ele não é executável e nenhum script dele deve ser rodado. Os arquivos abaixo descrevem uma arquitetura intencionalmente descumprida para que o auditor tenha violações conhecidas.

## Intenção arquitetural

| Módulo | Caminho | Responsabilidade | Dependências permitidas |
|---|---|---|---|
| `domain` | `src/domain/**` | Modelos e regras de pedido | nenhuma |
| `application` | `src/application/**` | Casos de uso e coordenação | `domain` |
| `infrastructure` | `src/infrastructure/**` | Persistência e adaptadores | `domain`, `application` |
| `webapi` | `src/webapi/**` | Rotas HTTP | `application` |

## Violações plantadas

1. `domain` importa `application` em `src/domain/model.rs`: proibida e fecha um ciclo com `application -> domain`.
2. `application` importa `infrastructure` em `src/application/use_cases.rs`: proibida.
3. `webapi` importa `infrastructure` em `src/webapi/routes.rs`: proibida.
4. `shared/legacy.rs` está fora de todos os paths do contrato: revisão de configuração.
5. A mesma linha `ORDER_POLICY` aparece em dois arquivos: revisão heurística de duplicação, intencional neste projeto.
6. `web/src/client.ts` usa `import()` com variável: limitação de linguagem/resolução, não deve virar aresta.
7. `tools/check.go` usa uma extensão reconhecida como código, mas sem analisador nesta versão: redução de cobertura.

## Observações

- `Cargo.toml` e os manifests dos crates existem apenas para dar configuração própria ao projeto; nenhum comando Cargo é executado pelo auditor.
- `config/app.toml` é documentação de configuração, não código.
- O contrato definitivo usado na validação está em `auditor.toml`.
