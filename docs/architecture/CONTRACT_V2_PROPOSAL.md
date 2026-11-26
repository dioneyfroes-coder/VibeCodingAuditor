# Proposta de contrato arquitetural v2

Data: 26/11/2026
Status: aprovada pelo usuário em 26/11/2026; implementada e verificada na Etapa 3.5
Branch: `etapa-3/validacao-arquitetural`

Este documento descreve o que foi implementado na Etapa 3.5. Os campos aqui listados só passaram a ser considerados disponíveis depois da aprovação, da implementação e da passagem dos testes de aceitação.

## 1. Objetivo e princípios

Permitir que `auditor.toml` expresse camadas de forma declarativa e verificável, sem interpretar prosa livre como regra de máquina.

Princípios:

1. O contrato descreve a arquitetura pretendida; ele não executa código do projeto auditado.
2. Toda regra implementada tem campo explícito, semântica determinística e teste de aceitação.
3. `responsibility` continua sendo descrição para humanos. O auditor não alega entender português, intenção ou semântica de responsabilidade escrita em texto livre.
4. `error` só quando há aresta resolvida e violação demonstrada. Falta de evidência vira `needs_review` ou limitação.
5. Nenhuma abstração de regra genérica, linguagem de expressões, IA, parser adicional ou dependência nova entra nesta proposta sem evidência dos pilotos.
6. V1 e v2 são lidos pelo mesmo núcleo e normalizados internamente. Nenhum arquivo é reescrito automaticamente.

## 2. Problemas concretos observados nos dois pilotos

### 2.1 Autoauditoria do próprio auditor

Fonte: `docs/validation/etapa-3/self-audit.md`.

- O contrato raiz tem três módulos Rust (`core`, `cli`, `desktop-backend`) e nenhuma aresta proibida. Não houve falha de detecção. A limitação é de expressividade: a direção "adaptadores dependem do núcleo; o núcleo não depende de adaptadores" existe apenas como fora de `allowed_dependencies` dos adaptadores e na prosa de `responsibility`. Não há campo que represente a camada nem a razão da direção.
- React e extensão VS Code ficaram fora do grafo, e o relatório diz isso explicitamente. Hoje não há como declarar uma fronteira de apresentação e dizer que ela pode depender de `application`, mas não de `infrastructure`.
- Houve 8 achados `dep.unresolved` vindos de `super::` em módulos de teste e 54 achados `code.duplicated` por linha exata. Isso reforça que heurística não pode virar `error` e que a proposta não deve usar esses sinais como regra de camada.

### 2.2 Projeto falso `fake-order-service`

Fonte: `docs/validation/etapa-3/second-project-synthetic.md`.

- `shared/legacy.rs` e `web/src/client.ts` geraram `file.unmapped`. O contrato v1 não tem lugar para declarar uma fronteira `shared` nem uma camada de frontend. A revisão foi correta, mas a intenção "web é apresentação" só existe no `README.md`, que o auditor não interpreta.
- `webapi -> infrastructure` foi detectado como `dep.not_allowed` porque `webapi.allowed_dependencies = ["application"]`. A regra de camada correspondente — apresentação não acessa infraestrutura — não é expressável como tal. Se uma exceção fosse necessária, a única forma atual seria acrescentar `infrastructure` à lista do módulo inteiro, ampliando a permissão para todos os arquivos de `webapi`.
- `web/src/client.ts` contém `import(moduleName)`. Como o destino é dinâmico, não virou aresta. A política de v2 precisa manter isso como limitação, não como violação de camada.
- `tools/check.go` ficou sem analisador. Isso não justifica campo de contrato novo; cobertura parcial continua sendo informada pelo relatório.
- A validação atual rejeita `path` duplicado literal, mas não paths sobrepostos por padrão. A análise em memória já trata múltiplos módulos correspondentes como `file.ambiguous_module`; v2 deve validar a sobreposição antes da auditoria para não depender de decisão em tempo de execução.

## 3. Escopo proposto

Entram no v2:

- `schema_version = 2`;
- campos existentes de `project`, `scan`, `architecture.style` e `architecture.modules`, com os mesmos tipos e significado do v1;
- `architecture.layers[]`, com `name`, `modules` e `allowed_dependencies`;
- uma nova regra determinística `dep.layer_not_allowed`;
- leitura de v1 e v2 no mesmo núcleo, com normalização interna.

Não entram nesta proposta:

- domínios/subdomínios;
- exceções locais;
- limiares de métricas;
- linguagem genérica de regras;
- migração automática de arquivos;
- novos analisadores de linguagem;
- dependências externas;
- alteração de schema SQLite;
- interpretação de `responsibility` por IA ou por processo de linguagem natural.

Esses itens foram avaliados na seção 7.

## 4. Formato proposto

### 4.1 Exemplo válido

```toml
schema_version = 2

[project]
name = "fake-order-service"

[scan]
include = ["src/**/*.rs", "web/src/**/*.ts"]
exclude = []

[architecture]
style = "modular_monolith"

[[architecture.modules]]
name = "domain"
path = "src/domain/**"
responsibility = "Modelos e regras de pedido; não conhece adaptadores"
allowed_dependencies = []

[[architecture.modules]]
name = "application"
path = "src/application/**"
responsibility = "Casos de uso e coordenação; depende somente do domínio"
allowed_dependencies = ["domain"]

[[architecture.modules]]
name = "infrastructure"
path = "src/infrastructure/**"
responsibility = "Persistência e adaptadores externos"
allowed_dependencies = ["domain", "application"]

[[architecture.modules]]
name = "webapi"
path = "src/webapi/**"
responsibility = "Rotas HTTP; não acessa infraestrutura diretamente"
allowed_dependencies = ["application"]

[[architecture.modules]]
name = "web"
path = "web/src/**"
responsibility = "Cliente frontend; comunica-se com webapi"
allowed_dependencies = []

[[architecture.layers]]
name = "domain"
modules = ["domain"]
allowed_dependencies = []

[[architecture.layers]]
name = "application"
modules = ["application"]
allowed_dependencies = ["domain"]

[[architecture.layers]]
name = "infrastructure"
modules = ["infrastructure"]
allowed_dependencies = ["domain", "application"]

[[architecture.layers]]
name = "presentation"
modules = ["webapi", "web"]
allowed_dependencies = ["application"]
```

Neste exemplo, `webapi -> infrastructure` continua proibido pela lista do módulo e também pela camada. Como a lista do módulo já proíbe a aresta, o auditor emite `dep.not_allowed` e não duplica o achado com `dep.layer_not_allowed`.

### 4.2 Exemplo onde a camada acrescenta informação

```toml
[[architecture.modules]]
name = "web"
path = "web/src/**"
responsibility = "Cliente frontend"
allowed_dependencies = ["application", "infrastructure"]

[[architecture.layers]]
name = "presentation"
modules = ["web"]
allowed_dependencies = ["application"]
```

A lista do módulo permite `web -> infrastructure`, mas a camada `presentation` não permite `infrastructure`. Com a aresta resolvida, o resultado é `dep.layer_not_allowed`, com severidade `error`. Isso mostra que a camada não é apenas repetição da lista do módulo.

### 4.3 Exemplos inválidos

Módulo sem camada quando layers foi declarado:

```toml
[[architecture.layers]]
name = "domain"
modules = ["domain"]
allowed_dependencies = []
# application, infrastructure, webapi e web ficaram sem camada: inválido.
```

Módulo em duas camadas:

```toml
[[architecture.layers]]
name = "application"
modules = ["application", "web"]
allowed_dependencies = ["domain"]

[[architecture.layers]]
name = "presentation"
modules = ["web"]
allowed_dependencies = ["application"]
```

Paths sobrepostos:

```toml
[[architecture.modules]]
name = "core"
path = "src/**"
responsibility = "Núcleo"
allowed_dependencies = []

[[architecture.modules]]
name = "domain"
path = "src/domain/**"
responsibility = "Domínio"
allowed_dependencies = []
```

Referência a camada inexistente e ciclo de camadas:

```toml
[[architecture.layers]]
name = "presentation"
modules = ["web"]
allowed_dependencies = ["missing"]

[[architecture.layers]]
name = "application"
modules = ["application"]
allowed_dependencies = ["presentation"]
```

Campo com efeito não implementado:

```toml
[architecture]
style = "modular_monolith"
forbidden_phrases = ["nunca chamar infraestrutura"]
# inválido: campo desconhecido; prosa não vira regra.
```

## 5. Campos, tipos e semântica

| Campo | Tipo | Semântica |
|---|---|---|
| `schema_version` | `u32` | Deve ser `2` para contratos v2. Outros valores são rejeitados com mensagem clara; nenhum arquivo é reescrito. |
| `project.name` | `String` | Nome legível do projeto. Inalterado em relação ao v1; não participa de regras. |
| `scan.include` | `Vec<String>` | Padrões que precisam corresponder para entrar no inventário. Inalterado. |
| `scan.exclude` | `Vec<String>` | Padrões que removem caminhos mesmo quando incluídos. Inalterado. |
| `architecture.style` | enum `modular_monolith` | Inalterado. Novo valor de enum não é adicionado sem efeito implementado. |
| `architecture.modules[].name` | `String` | Identificador do módulo nas arestas e nas listas de dependências. Inalterado. |
| `architecture.modules[].path` | `String` | Padrão relativo que associa arquivos ao módulo. Inalterado, com validação de sobreposição reforçada no v2. |
| `architecture.modules[].responsibility` | `String` | Descrição para humanos. Nunca usada como regra, classificador ou evidência de violação. |
| `architecture.modules[].allowed_dependencies` | `Vec<String>` | Lista fechada de módulos permitidos como destino. Inalterado; continua sendo a primeira checagem. |
| `architecture.layers[].name` | `String` | Identificador único da camada, usado em `allowed_dependencies` da própria camada. |
| `architecture.layers[].modules` | `Vec<String>` | Nomes de módulos que pertencem à camada. Não vazio, sem duplicatas e sem módulo repetido em outra camada. |
| `architecture.layers[].allowed_dependencies` | `Vec<String>` | Lista fechada de camadas permitidas como destino. Não permite a própria camada, nomes desconhecidos, duplicatas ou ciclo no grafo de camadas. |

Não há campo `priority`, `rank`, `level`, `threshold`, `forbidden_dependencies`, `exception` nem equivalente. Dependência entre camadas é sempre `origem -> destino`.

## 6. Validação de paths e sobreposições

O v2 mantém todas as validações do v1: campos obrigatórios não vazios, padrões relativos com `/`, sem `..`, sem prefixo absoluto, nomes de módulo únicos, paths literais únicos, dependências conhecidas, não duplicadas e sem auto-dependência.

Acrescenta:

1. Se `architecture.layers` existe, todo módulo declarado deve aparecer em exatamente uma camada.
2. Nomes de camada são únicos e não vazios.
3. `layers[].modules` não é vazio, não repete nome e só referencia módulos declarados.
4. `layers[].allowed_dependencies` pode ser vazio; quando preenchido, não referencia a si mesma, não repete nome e só referencia camadas declaradas.
5. O grafo de camadas não contém ciclo.
6. Paths de módulos não podem se sobrepor.

Definição operacional de sobreposição para o validador:

- normalizar cada `path` em componentes separados por `/`;
- remover o componente final `**` antes de comparar;
- dois paths se sobrepõem quando um dos prefixos de componentes é igual ao outro, incluindo o caso de um ser exatamente o outro;
- quando há `*` ou `?` no mesmo prefixo, a comparação é conservadora: se os prefixos literais até o primeiro curinga forem iguais, os paths são considerados sobrepostos;
- `src/domain/**` e `src/webapi/**` são válidos porque divergem no segundo componente;
- `src/**` e `src/domain/**` são inválidos porque `src` é prefixo do outro;
- `src/**/*.rs` e `src/domain/**` são inválidos pela mesma regra.

Essa checagem é conservadora: pode rejeitar padrões que na prática seriam disjuntos. Isso é preferível a decisão ambígua em tempo de auditoria. Mensagens de erro devem indicar o campo, os dois módulos e os dois paths.

## 7. Semântica da direção das dependências

Para uma aresta resolvida `origem -> destino`:

1. Se `destino` não está em `origem.allowed_dependencies`, emitir `dep.not_allowed` com a semântica atual e encerrar a avaliação de camada para essa aresta. Isso evita dois erros para a mesma violação.
2. Caso contrário, determinar as camadas de `origem` e `destino`.
3. Se algum dos dois não tiver camada por configuração inválida, o contrato já foi rejeitado na validação. Se, ainda assim, o mapeamento faltar em tempo de execução, emitir limitação; nunca inventar `error`.
4. Se `destino` não estiver em `camada_origem.allowed_dependencies`, emitir `dep.layer_not_allowed` com severidade `error`.
5. Permissão não é transitiva: se `presentation -> application` e `application -> domain` são permitidas, isso não autoriza `presentation -> domain` sem declaração explícita.
6. Ciclo entre camadas é erro de contrato, não achado de auditoria. O `dep.cycle` existente continua medindo o grafo de módulos, não o grafo de camadas.

Novo `rule_id`:

- `dep.layer_not_allowed`
- ID proposto: `dep.layer_not_allowed:{origem}:{destino}:{path}`
- Linha e evidência continuam fora da identidade, conforme E3.4.
- Mensagem inclui as camadas envolvidas e a lista permitida da camada de origem.
- Se a aresta não estiver resolvida, o resultado continua sendo `dep.unresolved`, sem conclusão de camada.

## 8. Política para elementos não resolvidos e ambíguos

| Situação | Resultado |
|---|---|
| Import não resolvido | `dep.unresolved` como revisão; nenhum `dep.layer_not_allowed`. |
| Arquivo sem módulo correspondente | `file.unmapped`; nenhuma conclusão de camada. |
| Arquivo que corresponde a mais de um módulo | `file.ambiguous_module`; nenhum `error`. No v2 o contrato é rejeitado se a sobreposição for detectável estaticamente. |
| Camada ausente por contrato incompleto | Contrato inválido antes da auditoria. |
| Camada ausente em tempo de execução por falha interna | Limitação explícita, nunca `error`. |
| Import dinâmico com destino variável | Limitação de resolução; sem aresta e sem conclusão de camada. |
| Arquivo de código sem analisador | Cobertura parcial; sem aresta inventada. |
| `responsibility` contradiz o código | Nenhuma regra automática; o texto continua apenas descritivo. |

## 9. Compatibilidade com v1

Política escolhida: **opção 1 do plano** — o núcleo lê v1 e v2 e normaliza ambos para um modelo interno comum, sem reescrever arquivos.

- Modelo interno proposto: `NormalizedContract { project, scan, modules, layers }`.
- Contrato v1 normaliza para `layers = None`. Nenhuma regra de camada é avaliada.
- Contrato v2 normaliza para `layers = Some(...)` e mantém os mesmos módulos.
- `auditor init` continua gerando v1 até que a implementação v2 e seus testes estejam prontos. Não é acrescentada opção de versão ao CLI nesta etapa.
- Histórico guarda `schema_version` lido; registros antigos continuam legíveis. Nenhuma coluna ou índice SQLite muda.
- O `id` de achados existentes não muda. O novo `dep.layer_not_allowed` acrescenta um `rule_id`, mas não altera contratos v1 nem IDs legados.
- CLI, desktop e extensão continuam consumindo o modelo de `findings`; nenhuma delas implementa regra de camada.
- Migração manual, se o usuário quiser, é uma edição inspecionável do TOML. O auditor nunca a executa silenciosamente.

## 10. Impacto por componente

| Componente | Impacto previsto |
|---|---|
| Núcleo | Novos tipos internos, parser/validador v2, normalizador e avaliação de camada. Nenhuma dependência externa nova. |
| Regras | Novo `rule_id` `dep.layer_not_allowed`; precedência da regra de módulo para evitar duplicação na mesma aresta. |
| CLI | Nenhum comando novo; `audit`, `scan`, `analyze` e `history` passam a aceitar contrato v2 pela mesma leitura do núcleo. |
| Desktop | Nenhuma lógica nova. Exibe o novo achado porque já renderiza `findings` genéricos. |
| Extensão VS Code | Nenhuma lógica nova. `rule_id` vira código do diagnóstico e mensagem continua vinda do núcleo. |
| SQLite | Nenhuma migração. `findings` já são texto; o novo `rule_id` é armazenado como qualquer outro. |
| Documentação | README e exemplo TOML só devem descrever campos depois de implementados e testados. |

## 11. Capacidades candidatas avaliadas

| Capacidade | Evidência dos pilotos | Decisão | Consequência |
|---|---|---|---|
| Camadas | Autoauditoria não consegue declarar direção como campo; alvo falso não consegue dizer que `web` é apresentação | **Incluir**, limitado a `allowed_dependencies` por camada | Implementar validação, normalização e `dep.layer_not_allowed` na Etapa 3.5. |
| Domínios/subdomínios | Nenhum dos dois pilotos tem agrupamento por domínio que o auditor tenha deixado de verificar | **Adiar** | Introduziria segundo eixo de agrupamento e ambiguidade sem evidência. Reavaliar com projeto real. |
| Restrições estruturais gerais | Casos reais são listas fechadas de módulos e camadas | **Não incluir linguagem genérica** | `allowed_dependencies` resolve os casos observados; expressões e regex ampliariam superfície sem necessidade. |
| Exceções justificadas | Nenhuma exceção real foi plantada ou observada; o alvo mostra apenas o risco de ampliar lista de módulo | **Adiar** | Quando surgir evidência, o mecanismo deve ser localizado por arquivo/caminho, com razão explícita e revisável. Nunca silenciamento global. |
| Limites mensuráveis | `module.concentration` já existe e mede concentração no grafo; não há complexidade ciclomática/cognitiva implementada | **Não incluir novos limites** | Não renomear nem descrever a métrica existente como complexidade. Nenhum limiar sem métrica real e teste. |
| Migração automática | Preferência explícita do plano é leitura dupla sem reescrita | **Não incluir** | Evita alteração silenciosa de contrato do usuário. |

## 12. Testes de aceitação definidos por campo

| Campo ou comportamento | Teste de aceitação |
|---|---|
| `schema_version = 2` | Contrato v2 válido é aceito; `3` e `0` são rejeitados com mensagem que nomeia o campo. |
| `project.name`, `scan.*`, `architecture.modules.*` vindo do v1 | Fixtures v1 existentes produzem exatamente o mesmo inventário, arestas, achados e IDs de antes. |
| `layers[].name` | Nome vazio, duplicado ou ausente rejeitado com caminho do campo. |
| `layers[].modules` | Lista vazia, duplicada, módulo desconhecido, módulo em duas camadas e módulo sem camada são rejeitados. |
| `layers[].allowed_dependencies` | Destino desconhecido, self, duplicata e ciclo entre camadas são rejeitados. |
| Ordem do TOML | Reordenar campos, módulos e camadas não altera resultado nem mensagem de erro estável. |
| Paths | Sobreposição `src/**` + `src/domain/**` rejeitada; paths irmãos aceitos. |
| Dependência permitida por módulo e camada | Aresta resolvida não gera achado de camada. |
| Dependência proibida pela camada, permitida pelo módulo | `dep.layer_not_allowed` com severidade `error`, camadas na mensagem e ID sem linha. |
| Dependência proibida pelo módulo | Continua apenas `dep.not_allowed`; nenhum segundo achado pela camada. |
| Aresta não resolvida | Continua `dep.unresolved`; nenhum `dep.layer_not_allowed`. |
| Mapeamento ambíguo | Revisão ou rejeição de contrato, nunca `error` definitivo por camada. |
| Saídas | Texto, JSON e Markdown mostram o mesmo `rule_id`, severidade e mensagem, em ordem determinística. |
| Desktop e extensão | Consomem o achado do núcleo sem novo `match`, regra ou tradução de severidade própria. |
| SQLite | `history record` e `compare` preservam o novo `rule_id` sem migração de schema. |

## 13. Critérios para aceitar ou rejeitar a proposta

A proposta é **aceita** somente se:

1. Usuário revisar e aprovar este documento antes de qualquer alteração de código.
2. Cada campo listado tiver semântica e teste de aceitação definidos, como acima.
3. V1 continuar produzindo os mesmos resultados nas fixtures e nos dois relatórios de validação.
4. Nenhuma regra depender de interpretação de texto livre, IA ou serviço externo.
5. O escopo caberá nos incrementos da Etapa 3.5 sem nova dependência, sem parser adicional e sem mudança de schema SQLite.
6. `needs_review` e limitações continuarem separados de `error`.

A proposta é **rejeitada ou reduzida** se:

1. Exigir que o auditor interprete `responsibility` ou prosa em português.
2. Precisar de mecanismo global para silenciar erros.
3. Introduzir domínios, exceções ou métricas sem evidência e teste correspondente.
4. Alterar silenciosamente contratos v1 existentes.
5. Duplicar regras na CLI, desktop ou extensão.
6. Não couber em incrementos testáveis.

## 14. Plano de implementação dependente da aprovação

Somente após aprovação, seguir a ordem da Etapa 3.5:

1. Tipos internos normalizados, sem mudança de análise.
2. Parser e validador v2.
3. Testes de campos obrigatórios, desconhecidos, duplicados, paths inválidos e referências inexistentes.
4. Normalização de v1 e v2 para o modelo comum.
5. Validação de camadas e regra `dep.layer_not_allowed`.
6. Integração das saídas texto, JSON, Markdown, desktop e extensão.
7. Repetição da autoauditoria e da auditoria do alvo sintético com contrato v2 equivalente.

Domínios e exceções ficam fora deste plano até que haja evidência de projeto real que os exija.
