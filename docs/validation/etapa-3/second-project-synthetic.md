# Validação em segundo projeto — Etapa 3.3

Data: 26/11/2026
Branch: `etapa-3/validacao-arquitetural`
Alvo: `validation/targets/fake-order-service`

## Ressalva de escopo

O usuário solicitou explicitamente a criação de um projeto falso para esta validação. Portanto, o alvo é **sintético e criado por esta sessão**, com violações plantadas e documentadas em seu `README.md`. Ele não é um projeto de terceiros nem uma substituição silenciosa por fixture: esta etapa valida o contrato e as regras fora da arquitetura do próprio auditor, mas **não prova** o comportamento em código real não controlado. Se surgir um projeto real apropriado, a Etapa 3.3 deve ser repetida nele.

Nenhum arquivo do alvo foi modificado durante a auditoria. Os 20 arquivos foram hashed com SHA-256 antes e depois de uma segunda execução; o resultado foi idêntico.

## Escolha e classificação do alvo

O alvo tem quatro módulos Rust com responsabilidades distintas, manifests próprios, um teste, configuração em `config/app.toml`, documentação de intenção e três dependências proibidas plantadas.

| Módulo | Path do contrato | Responsabilidade declarada | Dependências permitidas |
|---|---|---|---|
| `domain` | `src/domain/**` | Modelos e regras de pedido; não conhece adaptadores | nenhuma |
| `application` | `src/application/**` | Casos de uso e coordenação; depende somente do domínio | `domain` |
| `infrastructure` | `src/infrastructure/**` | Persistência e adaptadores externos | `domain`, `application` |
| `webapi` | `src/webapi/**` | Rotas HTTP; não acessa infraestrutura diretamente | `application` |

Decisões de classificação:

- `shared/legacy.rs` foi incluído na varredura, mas não recebeu módulo: não há responsabilidade arquitetural declarada para ele. Isso exercita a revisão `file.unmapped`.
- `web/src/client.ts` também ficou fora dos módulos: o contrato v1 não declara fronteira de frontend. A review é correta; o `import()` com variável não virou aresta.
- `tools/check.go` é código incluído, mas esta versão não tem analisador Go. A cobertura parcial é explícita.
- `config/app.toml`, manifests, `README.md` e `auditor.toml` ficaram fora do `scan.include`; nenhum desses arquivos reduz a cobertura de código.

O contrato usado está no próprio alvo, em `validation/targets/fake-order-service/auditor.toml`. Como o diretório foi criado nesta etapa, nenhum `auditor.toml` preexistente foi substituído.

## Comandos e tempo

Tempo medido com `Stopwatch` ao redor de cada `cargo run -q`; o build já estava pronto. São medições únicas, não um benchmark estatístico.

| Comando | Código de saída | Tempo |
|---|---:|---:|
| `cargo run -q -p auditor-cli -- scan validation/targets/fake-order-service` | `0` | 1063 ms |
| `cargo run -q -p auditor-cli -- analyze validation/targets/fake-order-service` | `0` | 875 ms |
| `cargo run -q -p auditor-cli -- audit validation/targets/fake-order-service --format json --output <temp>\audit.json` | `1` | 863 ms |
| `cargo run -q -p auditor-cli -- audit validation/targets/fake-order-service` | `1` | 873 ms |
| `cargo run -q -p auditor-cli -- audit validation/targets/fake-order-service --format json --output <temp>\audit-repeat.json` | `1` | não medido; usado para o hash |

Código `1` em `audit` é o contrato existente: há achados. Não houve código `2`.

Resumo do JSON: `12` arquivos incluídos, `8` não incluídos, `4` módulos, `4` pacotes Cargo, `6` arestas, `7` achados (`3` erros, `1` aviso, `0` informações, `3` revisões). Todas as `8` entradas não incluídas foram classificadas como intencionais; `0` reduzem cobertura.

Arestas do `analyze`: `application -> domain` permitida, `application -> infrastructure` proibida, `domain -> application` proibida, `infrastructure -> domain` permitida, `webapi -> application` permitida e `webapi -> infrastructure` proibida.

## Classificação manual de todos os 7 achados

A amostra é o censo completo dos achados, não uma subamostra.

| # | `id` | Classificação manual | Evidência e justificativa |
|---|---|---|---|
| 1 | `code.duplicated:4ad69f7ddabff0cb:src/application/use_cases.rs\|src/domain/model.rs` | revisão correta por falta de evidência | A linha `ORDER_POLICY` foi repetida de propósito em dois arquivos. A regra é heurística de linha exata; o achado pede revisão, não afirma dívida ou violação. |
| 2 | `dep.cycle:application->domain->infrastructure` | verdadeiro positivo | `application -> domain`, `domain -> application`, `application -> infrastructure` e `infrastructure -> domain` formam o componente fortemente conectado plantado. |
| 3 | `dep.not_allowed:application:infrastructure:src/application/use_cases.rs` | verdadeiro positivo | `use infrastructure::Database;` em `src/application/use_cases.rs:2`; o contrato permite apenas `domain` para `application`. |
| 4 | `dep.not_allowed:domain:application:src/domain/model.rs` | verdadeiro positivo | `use application::CreateOrder;` em `src/domain/model.rs:1`; `domain` não permite dependências. |
| 5 | `dep.not_allowed:webapi:infrastructure:src/webapi/routes.rs` | verdadeiro positivo | `use infrastructure::Database;` em `src/webapi/routes.rs:2`; `webapi` permite apenas `application`. |
| 6 | `file.unmapped:shared/legacy.rs` | problema de configuração do contrato / revisão correta | Arquivo incluído e fora dos quatro paths. O contrato poderia declarar um módulo `shared` ou o `scan.include` poderia excluí-lo; o auditor não inventou módulo nem aresta. |
| 7 | `file.unmapped:web/src/client.ts` | problema de configuração do contrato / revisão correta | O arquivo está fora dos módulos. Além disso, `import(moduleName)` é dinâmico e não resolveria para um módulo estático; a limitação é de linguagem/resolução, não um falso positivo. |

Falso positivo: `0`. Falso negativo entre as violações plantadas: `0`.

## Métricas

As métricas são mantidas separadas para não produzir uma nota única sem metodologia.

| Métrica | Valor | Definição |
|---|---:|---|
| Achados classificados manualmente | `7/7` (100%) | Censo completo, um rótulo por `id`. |
| Achados confirmados como problema arquitetural | `4/7` (57,1%) | Três `dep.not_allowed` e um `dep.cycle`. |
| Achados confirmados como úteis/corretos | `7/7` (100%) | Inclui os quatro problemas e as três revisões corretas. |
| Violações conhecidas detectadas | `4/4` (100%) | Três dependências proibidas e um ciclo plantados. |
| Violações conhecidas não detectadas | `0/4` (0%) | Nenhuma violação plantada passou sem achado. |
| Arquivos de código analisados | `11/12` (91,7%) | `tools/check.go` não foi analisado por falta de analisador `.go`. |
| Ruído `needs_review` | `3/7` (42,9%) | Uma duplicação e dois arquivos fora do contrato; todos esperados e corretos. |
| Falsos positivos | `0/7` (0%) | Nenhum achado classificado como erro de diagnóstico. |

## Limitações e falsos negativos conhecidos

- `tools/check.go` foi incluído, mas não analisado: `cobertura parcial: 1 arquivo(s) de código incluído(s) não foram analisados; extensões sem analisador observadas: .go (1)`. O arquivo está vazio, então não há violação Go perdida neste alvo; a limitação é real para projetos com código Go.
- `web/src/client.ts` usa `import(moduleName)` com variável. Não virou aresta e não deve ser tratado como dependência resolvida sem analisador de TypeScript.
- O contador de pacotes cobre apenas Cargo: `pacotes npm não são enumerados`.
- `code.duplicated` compara linhas exatas; duplicação semanticamente igual com formatação diferente não é detectada. Aqui a repetição foi plantada exatamente.
- A identidade não cobre renomeação de arquivo, conforme decidido na Etapa 3.2.
- Estes resultados vêm de um alvo sintético pequeno; não permitem estimar precisão em projetos médios reais.

## Priorização pela evidência

Nenhuma alteração no motor foi feita nesta etapa. A evidência obtida prioriza assim:

1. **Contrato v2 (Etapa 3.4), não um novo analisador imediato:** dois achados `file.unmapped` mostram que o contrato v1 não expressa fronteiras de `shared` nem de frontend. Isso é mais relevante que inventar arestas para TypeScript.
2. **Cobertura Go: baixa prioridade:** há apenas um arquivo vazio sem analisador. Adicionar parser Go exigiria dependência e evidência de dependências perdidas; esta fixture não fornece essa evidência.
3. **Resolução dinâmica: baixa prioridade:** um `import()` com variável no alvo não tem destino determinístico. Transformá-lo em aresta exigiria heuristicamente adivinhar o módulo, contrariando a política de não inventar arestas.
4. **Regra de ciclo e deduplicação: manter:** ambas acertaram todos os casos plantados, inclusive o ciclo envolvendo três módulos.
5. **Heurística de duplicação: manter como revisão:** o único achado foi correto e não elevado a erro.

## Integridade do alvo

Comando usado para comparar hashes (SHA-256 de todos os 20 arquivos, ordenados por caminho) antes e depois da segunda auditoria:

```powershell
function Get-TargetHash {
  (Get-ChildItem -LiteralPath 'validation/targets/fake-order-service' -Recurse -File | Sort-Object FullName | ForEach-Object {
    "$($_.FullName)=$((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash)"
  }) -join "`n"
}
```

Resultado: `TARGET_UNCHANGED=TRUE`; segunda auditoria com `exit code 1` esperado.
