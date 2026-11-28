# Proposta de contrato arquitetural v3

Data: 27/11/2026

Status: proposta formal da Etapa 6.4.0 **aprovada e implementada na Etapa 6.4.1** (27/11/2026)

Branch/HEAD de referência da proposta: `main` @ `5087fc4`

Escopo original da subetapa 6.4.0: documentação. Nenhum código, contrato de projeto, manifesto, workflow ou schema SQLite foi alterado na proposta. A Etapa 6.4.1 implementou o desenho abaixo sem alterar dependências, workflows, manifests existentes, nome da CLI, `when` da extensão ou `bundle.active`.

Este documento descreve o desenho aprovado para `schema_version = 3`, agora efetivo no núcleo, na CLI, no desktop e na extensão. Contratos v1 e v2 continuam sendo lidos exatamente como antes; nenhum arquivo existente é reescrito automaticamente.

## 1. Motivo e estado atual

O contrato v2 resolvido na Etapa 3.5 acrescentou `architecture.layers[]` e a regra `dep.layer_not_allowed`. Isso cobriu a direção entre camadas, mas deixou quatro lacunas que a Etapa 6.4 pede para endereçar:

1. `architecture.style` aceita somente `modular_monolith`; perfis como camadas e Clean/Hexagonal vivem apenas no Builder (`BuilderProfile`) e não são gravados nem fiscalizados como tal.
2. Um módulo tem um único `path`. Não há agrupamento por domínio/contexto delimitado.
3. Não há exceção local: para liberar uma aresta é preciso ampliar a lista `allowed_dependencies` de um módulo ou de uma camada inteira.
4. Não há regra personalizada declarativa; qualquer necessidade fora das listas fixas exigiria interpretar texto livre ou escrever código, ambos vedados.

Estado corrente confirmado no código:

- `SUPPORTED_SCHEMA_VERSIONS = [1, 2]` em `contract::mod`.
- `architecture.style` é o enum `ArchitectureStyle::ModularMonolith`.
- `architecture.modules[].path` é um `String`.
- `architecture.layers[]` tem `name`, `modules` e `allowed_dependencies`, com validação de cobertura, duplicatas, ciclos e sobreposição de paths.
- `responsibility` é obrigatório e usado apenas como descrição; nenhuma regra o interpreta.
- `BuilderProfile` (`ModularMonolith`, `LayeredMonolith`) pertence ao rascunho do Builder e não vira campo do contrato.
- `contract_identity()` é o hash FNV-1a de 64 bits do TOML canônico produzido por `render_contract()`.
- `audit_runs.schema_version` já armazena a versão do contrato usada na execução de conformidade (`0` em descoberta), e `audit_runs.contract_identity` já guarda o identificador do contrato.

## 2. Princípios

1. O núcleo Rust continua sendo a única fonte de validação e de regras.
2. Um perfil é um **preset do Builder**, não uma afirmação de que o motor entende um estilo arquitetural. As regras efetivas vêm somente dos campos declarativos gravados no contrato.
3. Toda regra nova tem campo explícito, semântica determinística, ID estável e evidência.
4. `responsibility` continua sendo texto para humanos. Não é regra, não é evidência de conformidade e não entra na identidade canônica v3.
5. Exceção não é aprovação silenciosa: ela é localizada, justificada e visível no relatório e no histórico.
6. Regras personalizadas são declarativas e fechadas. Não há JavaScript, Lua, Python, WASM, `eval`, regex executável nem linguagem de expressões.
7. Contratos v1 e v2 continuam sendo lidos por igual. Nenhum arquivo existente é reescrito automaticamente.
8. Descoberta continua produzindo observações, nunca violações.
9. Não anunciar microsserviços como suportados. O modelo não representa serviços implantáveis, comunicação remota nem contratos de runtime.
10. Nenhuma dependência nova é necessária para esta proposta.

## 3. Decisões resumidas

| Tema | Decisão proposta |
|---|---|
| Perfil no contrato | **Não gravar.** Permanece no Builder e apenas expande campos declarativos. |
| Estilos aceitos | `architecture.style` continua com `modular_monolith`. Camadas e Clean/Hexagonal são presets que geram `architecture.layers[]`. |
| Domínios | `architecture.domains[]` opcional. Cada módulo pertence a **no máximo um** domínio; zero ou um são válidos. |
| Múltiplos domínios por módulo | **Rejeitado.** Tornaria a origem da regra ambígua. |
| Múltiplos paths por módulo | **Rejeitado nesta versão.** Nenhuma fixture prova necessidade; domínios agrupam módulos sem duplicar identidade. |
| Precedência | módulo → camada → domínio → regra personalizada. A primeira negação vence e encerra a cadeia daquela aresta. |
| Exceções | Por `rule`, `from`, `to`, `path` opcional e `justification` obrigatório. Suprimem o achado, mas permanecem visíveis. |
| Expiração | `expires_on` é metadado visível, não aplicador automático de silêncio. |
| Identidade | v1/v2 mantêm o hash legado; v3 usa projeção semântica canônica e ordenada. |
| SQLite | Nenhuma migração necessária para v3. Exceções aplicadas são persistidas como limitações textuais já suportadas. |

## 4. Formato proposto

### 4.1 Exemplo válido

```toml
schema_version = 3

[project]
name = "payments"

[scan]
include = ["src/**/*.rs", "web/**/*.ts"]
exclude = []

[architecture]
style = "modular_monolith"

[[architecture.modules]]
name = "domain"
path = "src/domain/**"
responsibility = "Modelos e regras de pagamento"
allowed_dependencies = []

[[architecture.modules]]
name = "application"
path = "src/application/**"
responsibility = "Casos de uso de pagamento"
allowed_dependencies = ["domain"]

[[architecture.modules]]
name = "infrastructure"
path = "src/infrastructure/**"
responsibility = "Persistência e integrações externas"
allowed_dependencies = ["domain", "application"]

[[architecture.modules]]
name = "web"
path = "web/src/**"
responsibility = "Cliente e rotas de apresentação"
allowed_dependencies = ["application"]

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
modules = ["web"]
allowed_dependencies = ["application"]

[[architecture.domains]]
name = "payments"
modules = ["domain", "application", "infrastructure"]
allowed_dependencies = []

[[architecture.domains]]
name = "checkout"
modules = ["web"]
allowed_dependencies = ["payments"]

[[architecture.exceptions]]
rule = "dep.layer_not_allowed"
from = "web"
to = "infrastructure"
path = "web/src/legacy-client.ts"
justification = "Ponte temporária durante a migração do cliente legado."
expires_on = "2027-03-31"

[[architecture.rules]]
id = "web-sem-integracao-direta"
scope = "layer"
from = "presentation"
to = "infrastructure"
severity = "error"
justification = "Apresentação não deve acessar infraestrutura diretamente."
```

Neste exemplo:

- `web -> infrastructure` é permitido pela lista do módulo `web`, mas negado pela camada `presentation`; a exceção local, quando o arquivo for `web/src/legacy-client.ts`, registra a dispensa em vez de transformar a regra em permissão geral.
- `checkout -> payments` é permitido pelo domínio; o contrário exigiria declaração explícita.
- A regra personalizada `web-sem-integracao-direta` é um reforço declarativo sobre camadas; ela não substitui a regra de camada nem cria exceção.

### 4.2 Campos novos

| Campo | Tipo | Semântica |
|---|---|---|
| `architecture.domains[].name` | `String` | Identificador único do domínio/contexto delimitado. |
| `architecture.domains[].modules` | `Vec<String>` | Módulos do domínio. Não vazio, sem duplicatas; cada módulo aparece em no máximo um domínio. |
| `architecture.domains[].allowed_dependencies` | `Vec<String>` | Domínios permitidos como destino. Sem auto-referência, duplicatas ou ciclo. |
| `architecture.exceptions[].rule` | `String` | `dep.not_allowed`, `dep.layer_not_allowed`, `dep.domain_not_allowed` ou o `id` de uma regra personalizada declarada. |
| `architecture.exceptions[].from` | `String` | Origem exata da aresta que pode ser dispensada. |
| `architecture.exceptions[].to` | `String` | Destino exato da aresta que pode ser dispensada. |
| `architecture.exceptions[].path` | `String?` | Caminho relativo exato do arquivo de origem. Sem curinga e sem `..`. Quando ausente, vale para qualquer arquivo do par `from -> to`. |
| `architecture.exceptions[].justification` | `String` | Obrigatório, não vazio. Aparece no relatório e no histórico. |
| `architecture.exceptions[].expires_on` | `String?` | Data `YYYY-MM-DD` para revisão humana. Não é aplicada automaticamente. |
| `architecture.rules[].id` | `String` | Identificador único da regra personalizada. |
| `architecture.rules[].scope` | enum | `module`, `layer` ou `domain`. Define onde `from` e `to` são resolvidos. |
| `architecture.rules[].from` | `String` | Nome de módulo, camada ou domínio de origem, conforme `scope`. |
| `architecture.rules[].to` | `String` | Nome de módulo, camada ou domínio de destino, conforme `scope`. |
| `architecture.rules[].severity` | enum | `error` ou `needs_review`; padrão `error`. |
| `architecture.rules[].justification` | `String` | Obrigatório, não vazio. |

Campos existentes de `project`, `scan`, `architecture.style`, `architecture.modules` e `architecture.layers` mantêm os tipos e significado atuais. `architecture.modules[].path` continua sendo um único `String`.

## 5. Perfis e presets

O perfil continua sendo estado do Builder e **não é gravado no `auditor.toml`**. Ele só escolhe quais campos declarativos o usuário revisará antes de salvar.

| Perfil do Builder | Expansão proposta | Regra efetiva |
|---|---|---|
| `modular_monolith` | Módulos sem `layers`, sem `domains`, sem exceções e sem regras extras | Somente `allowed_dependencies` de módulo |
| `layered_monolith` | Módulos + `architecture.layers[]`; cada módulo em exatamente uma camada | Módulo e camada |
| `clean_hexagonal` | Módulos + camadas `domain`, `application`, `adapters`, `framework`, com direções explícitas | Módulo e camada; domínios e exceções continuam opcionais |

Direções do preset `clean_hexagonal`:

| Camada | `allowed_dependencies` |
|---|---|
| `domain` | `[]` |
| `application` | `["domain"]` |
| `adapters` | `["application", "domain"]` |
| `framework` | `["adapters", "application", "domain"]` |

A atribuição de módulos às camadas continua sendo decisão humana revisável. O auditor não infere que um diretório é `adapters` apenas pelo nome. O ponto central é que o motor continua avaliando listas fechadas; o preset não introduce uma nova semântica de estilo.

Microsserviços ficam explicitamente fora do escopo: nenhum perfil, campo ou `rule_id` os anuncia como suportados.

## 6. Domínios

Um domínio é um agrupamento declarativo de módulos, ortogonal às camadas. A decisão proposta é **no máximo um domínio por módulo**, com três estados possíveis na validação:

- módulo com exatamente um domínio;
- módulo sem domínio, quando `architecture.domains` não está declarado ou quando o módulo é transversal;
- contrato inválido se o mesmo módulo aparecer em dois domínios.

Justificativa: domínio é a origem de uma regra. Pertencer a dois domínios tornaria a precedência dependente da ordem das declarações, o mesmo problema já rejeitado para módulos sobrepostos em `E2.2`. Já a ausência de domínio não é uma violação: módulos compartilhados podem continuar sujeitos apenas às regras de módulo e camada.

Quando `architecture.domains` é declarado:

1. nomes de domínio são únicos e não vazios;
2. `modules` não é vazio e só referencia módulos declarados;
3. um módulo não aparece em dois domínios;
4. `allowed_dependencies` só referencia domínios declarados, sem auto-referência, duplicatas ou ciclo;
5. uma aresta resolvida só recebe conclusão de domínio quando **origem e destino** possuem domínio;
6. se algum dos dois não possui domínio, a avaliação de domínio é omitida e uma limitação localizada é registrada, nunca um `error`.

## 7. Regras e precedência

Para uma aresta resolvida `origem -> destino`, a avaliação proposta é:

1. **Lista do módulo:** se `destino` não está em `origem.allowed_dependencies`, emitir `dep.not_allowed` e encerrar.
2. **Camada:** se há camadas e a camada de origem não permite a camada de destino, emitir `dep.layer_not_allowed` e encerrar.
3. **Domínio:** se há domínios e ambos os lados possuem domínio, mas o domínio de origem não permite o domínio de destino, emitir `dep.domain_not_allowed` e encerrar.
4. **Regra personalizada:** avaliar as regras declaradas em ordem canônica por `id`; cada correspondência gera um achado com o `id` da regra. Regras personalizadas só apertam a política, nunca a relaxam.
5. **Exceções:** aplicar depois da avaliação. Uma exceção exata suprime o achado correspondente e registra a dispensa.

A primeira negação vence; uma dimensão posterior não pode contradizer ou desfazer uma negação anterior. Permissão não é transitiva em nenhuma dimensão. Arestas não resolvidas continuam apenas em `dep.unresolved`, sem conclusão de camada, domínio ou regra personalizada.

Novos `rule_id` propostos:

- `dep.domain_not_allowed`
- `dep.custom_not_allowed`

Ambos seguem o padrão já adotado: ID determinístico com origem, destino e caminho; linha e evidência fora da identidade; `error` por padrão. `dep.custom_not_allowed` inclui também o `id` da regra personalizada.

Situações especiais:

| Situação | Resultado |
|---|---|
| Módulo sem camada quando há camadas | Contrato inválido, como no v2 |
| Módulo sem domínio quando há domínios | Sem conclusão de domínio; limitação localizada |
| Aresta não resolvida | `dep.unresolved`; nenhuma regra de dimensão |
| Regra personalizada com `scope = "layer"` e participante sem camada | Contrato inválido na validação das referências |
| Exceção sem justificativa | Contrato inválido |
| `responsibility` contradiz o código | Nenhuma regra automática |

## 8. Exceções

Uma exceção é uma dispensa **localizada**, não uma mudança de política global. Chave de identidade: (`rule`, `from`, `to`, `path`); chaves duplicadas são rejeitadas.

Validação proposta:

- `rule` precisa ser um `rule_id` conhecido ou um `id` de regra personalizada declarada;
- `from` e `to` precisam existir no escopo da regra;
- `path`, quando presente, precisa ser um caminho relativo exato com `/`, sem `..` e sem curinga;
- `justification` é obrigatório;
- `expires_on`, quando presente, precisa ser uma data `YYYY-MM-DD` válida;
- exceções não podem apagar `dep.unresolved`, `file.unmapped`, `dep.cycle` nem achados de cobertura.

`expires_on` é informação para revisão humana. Para manter a auditoria determinística, o auditor **não** muda o resultado conforme a data do sistema: ele exibe a data, e a renovação ou remoção da exceção continua sendo uma edição consciente do contrato. Se, no futuro, expiração automática for desejada, isso deve vir com uma proposta separada e injeção explícita da data de referência nos testes.

Quando uma exceção é aplicada:

- o achado correspondente não aparece em `findings`;
- `AuditReport.exceptions_applied` registra `rule`, `from`, `to`, `path`, `justification`, `expires_on`, o `id` que seria emitido e a linha/arquivo da evidência;
- texto e Markdown ganham a seção `Exceções aplicadas`, com o motivo;
- uma linha estável em `limitations` registra a mesma dispensa, para ser persistida pelo histórico atual;
- `history show` passa a exibir essas linhas; `history compare` ganha campos aditivos `before_waivers` e `after_waivers`, para que um achado que desapareceu por exceção não pareça simplesmente resolvido.

Assim, o histórico não é apagado: a dispensa é tão visível quanto o achado que deixou de ser contado.

## 9. Regras personalizadas declarativas

`architecture.rules[]` é uma lista fechada de proibições por par. Não há campo para código, expressão, regex executável, condição arbitrária ou precedência numérica.

Validação proposta:

- `id` único e não vazio;
- `scope` em `module`, `layer` ou `domain`;
- `from` e `to` existem no escopo escolhido e não são iguais;
- `severity` em `error` ou `needs_review`;
- `justification` obrigatório;
- campo desconhecido, como `script` ou `expression`, é rejeitado por `deny_unknown_fields`.

Regras personalizadas são avaliadas depois das dimensões nativas. Uma regra pode coincidir com uma proibição já emitida por módulo, camada ou domínio? Nesse caso, a primeira negação da cadeia encerra a avaliação e a regra personalizada não duplica o achado; ela continua disponível para exceção futura apenas se for a primeira negação aplicável. Para evitar dependência da ordem, a precedência é fixa nativa → personalizada, e as regras personalizadas são ordenadas por `id`.

## 10. Múltiplos paths por módulo

Decisão proposta: **não incluir `paths[]` no v3**.

Evidência considerada: as fixtures de módulos, camadas, TypeScript, Python e resolução da Etapa 6.3 associam cada módulo a um prefixo de diretório; nenhuma delas exige que um mesmo módulo reúna dois prefixos disjuntos. Quando dois diretórios precisam ser tratados juntos, a proposta é usar um domínio para agrupar módulos distintos, não duplicar a identidade de um módulo.

Aceitar vários paths criaria uma segunda fonte de ambiguidade: a validação de sobreposição do v2 teria que ser refeita para combinações de pares, e `file.ambiguous_module` passaria a depender da ordem das listas. Se um projeto real demonstrar a necessidade, a mudança deve vir com fixture própria e nova versão de schema, não por conveniência de escrita.

## 11. Identidade canônica

A identidade atual é estável, mas reflete a ordem de declaração e inclui `responsibility`, que não é regra. A proposta mantém compatibilidade sem quebrar registros antigos:

- **v1 e v2:** `contract_identity()` continua calculando exatamente o hash legado do TOML renderizado. Testes de regressão devem fixar os hashes atuais das fixtures antes de qualquer alteração.
- **v3:** novo ramo da mesma função calcula o hash de uma projeção semântica:

  1. `schema_version = 3`;
  2. `project.name` e `scan` exatamente como declarados;
  3. módulos ordenados por `name`, sem `responsibility`;
  4. camadas, domínios, exceções e regras ordenados por suas chaves estáveis;
  5. listas internas de dependências ordenadas para que reordenar uma lista não mude a identidade;
  6. nenhum campo de perfil, porque ele não é gravado.

`responsibility` fica fora da identidade v3 porque é descrição, não política. Uma mudança de texto descritivo não invalida comparações de histórico; uma mudança de módulo, path, lista de dependência, domínio, exceção ou regra altera a identidade. Duas declarações v3 com a mesma semântica, mas em ordem diferente, produzem o mesmo identificador.

O hash continua sendo FNV-1a de 64 bits, sem dependência externa. Nenhum registro antigo é recalculado.

## 12. Compatibilidade e SQLite

- `SUPPORTED_SCHEMA_VERSIONS` passa a `[1, 2, 3]`.
- Contratos v1 e v2 continuam sendo lidos e avaliados com as regras atuais; nenhum campo novo é exigido deles.
- Contratos v3 podem omitir `domains`, `exceptions` e `rules`; listas vazias têm o mesmo significado de ausência.
- Nenhum contrato existente é reescrito. O Builder pode propor um rascunho v3, mas continua exigindo prévia de diff e confirmação separada antes de gravar.
- `auditor init` passa a gerar contrato v3 mínimo quando a implementação for aprovada, mantendo a regra de nunca sobrescrever `auditor.toml` existente.
- CLI, desktop e extensão continuam consumindo os mesmos `findings`; os novos campos de relatório são aditivos.
- `audit_runs.schema_version` já aceita o valor `3` e `audit_runs.contract_identity` já aceita o novo hash. **Não é necessária migração SQLite para suportar v3.**
- Exceções aplicadas são gravadas nas linhas já existentes de `audit_limitations`, portanto também não exigem nova tabela. Uma representação estruturada de exceções no banco exigiria proposta de migração separada, com testes de preservação.

## 13. Impacto por componente

| Componente | Impacto previsto após aprovação |
|---|---|
| `auditor-core::contract` | Novos tipos, `schema_version = 3`, validações, projeção canônica e compatibilidade de leitura. |
| `auditor-core::rules` | Novos `rule_id`, precedência por dimensão, avaliação de regras personalizadas e aplicação de exceções. |
| `auditor-core::builder` | Preset `clean_hexagonal`, novas dimensões editáveis; perfil continua fora do TOML. |
| `auditor-core::history` | Campos aditivos de waivers na comparação; nenhuma migração. |
| `report` | Novo campo `exceptions_applied`, seções em texto/Markdown e JSON. |
| CLI | Aceita v3; `init` gera v3 mínimo; saídas continuam nos três formatos atuais. |
| Desktop | Exibe novos campos vindos do núcleo, sem validação própria. |
| Extensão | Sem regra nova; novos `rule_id` viram códigos de diagnóstico como qualquer outro. |
| SQLite | Nenhuma mudança de schema. |
| Dependências | Nenhuma. |

## 14. Testes planejados e critérios de aceite

A implementação da Etapa 6.4.1 deverá começar por testes que falhem antes do código:

1. contratos v1 e v2 mantêm exatamente os hashes de identidade atuais;
2. v3 com domínios, exceções e regras é aceito; v3 com campos desconhecidos é rejeitado com nome do campo;
3. módulo em dois domínios, domínio órfão, ciclo de domínio e auto-dependência são rejeitados;
4. exceção sem justificativa, com regra desconhecida, com chave duplicada ou com `path` inválido é rejeitada;
5. script, expressão, regex executável ou `eval` em regra personalizada é rejeitado como campo desconhecido;
6. precedência módulo → camada → domínio → personalizada produz um único achado na primeira negação;
7. exceção suprime o achado, mas aparece em `exceptions_applied`, em `limitations`, em texto/Markdown e em `history show`;
8. `history compare` mostra waivers antes/depois e não declara silenciosamente resolvido um achado dispensado;
9. dois contratos v3 semanticamente iguais, com declarações reordenadas, têm a mesma identidade; mudança de regra muda a identidade;
10. perfis do Builder geram as camadas esperadas e não são gravados no TOML;
11. `auditor init` gera v3 e nunca sobrescreve contrato existente;
12. fixtures v1/v2 existentes produzem exatamente os mesmos achados de antes.

Critérios de aceite da Etapa 6.4 como um todo permanecem os do plano: proposta aprovada antes do código, leitura de v1/v2 sem reescrita, erro v3 com campo/regra identificado, domínios distintos de camadas, exceções visíveis, nenhum texto livre executável e paridade entre CLI, desktop e extensão.

## 15. Alternativas rejeitadas

| Alternativa | Motivo da rejeição |
|---|---|
| Gravar `profile` no contrato | Criaria uma afirmação de estilo não fiscalizada pelo motor. |
| Novos valores de `architecture.style` para Clean/Hexagonal | O motor não interpreta estilos; apenas listas declarativas. |
| Múltiplos paths por módulo | Sem fixture que prove necessidade; complicaria sobreposição e ambiguidade. |
| Módulo em vários domínios | Tornaria a precedência dependente da ordem das declarações. |
| Exceção global ou por regra sem origem/destino | Silenciaria violações fora do escopo revisado. |
| Exceção sem justificativa | Impediria auditoria da decisão humana. |
| Expiração automática pela data do sistema | Tornaria a saída dependente do relógio e difícil de testar; viraria proposta separada com data injetada. |
| Script, Lua, Python, WASM ou `eval` | Quebra determinismo, segurança e a regra de não executar código do projeto. |
| Migração SQLite v4 para exceções estruturadas | Desnecessária: `audit_limitations` já persiste o motivo. |
| Migração automática v1/v2 → v3 | Alteraria contrato do usuário sem inspeção. |
| Suporte a microsserviços | Fora de escopo; ausência de modelo de runtime e comunicação remota. |

## 16. Gate

Gate concluído: o desenho foi aprovado explicitamente com `Aprovar e implementar`, cobrindo domínio opcional por módulo, exceções com metadado de expiração não aplicado, regras personalizadas declarativas, não inclusão de múltiplos paths e `init` gerando v3. A Etapa 6.4.1 implementou exatamente esse conjunto.

## 17. Resultado da Etapa 6.4.1

- `SUPPORTED_SCHEMA_VERSIONS = [1, 2, 3]`; contratos v3 com `domains`, `exceptions` e `rules` opcionais são aceitos, e campos desconhecidos são rejeitados pelo nome.
- `contract_identity()` preserva os hashes legados de v1/v2 e usa projeção semântica ordenada para v3; `responsibility` não entra na identidade v3.
- Precedência módulo → camada → domínio → regra personalizada, primeira negação vence, com `dep.domain_not_allowed` e `dep.custom_not_allowed`.
- Exceções localizadas e justificadas suprimem o achado, mas aparecem em `exceptions_applied`, `limitations`, texto, Markdown, desktop, extensão e histórico; `expires_on` é apenas data de revisão.
- Builder ganha o preset `clean_hexagonal`, sem gravar perfil no contrato; as camadas vêm do núcleo via comando Tauri, não de regra duplicada no React.
- `auditor init` gera v3 sem sobrescrever contrato existente.
- Fixtures v3, relatório/histórico e documentação alinhados. Nenhuma migração SQLite foi necessária: dispensas aplicadas viram linhas em `audit_limitations`, e a comparação ganhou `before_waivers`/`after_waivers`.
