# Vibe Coding Auditor

Ferramenta local para comparar a arquitetura pretendida de um projeto com a arquitetura realmente implementada. Cada achado deve apontar a regra violada, os arquivos envolvidos e, quando possível, as linhas relevantes. O contrato arquitetural (`auditor.toml`) pertence ao usuário e é versionado junto ao projeto auditado.

**Status atual:** primeira fase do projeto concluída; Etapa 2 de confiabilidade iniciada pelos itens 2.1 (cobertura honesta dos arquivos) e 2.2 (mapeamentos ambíguos sem conclusão definitiva). Etapas 0–10 fechadas, com TypeScript/JavaScript na Etapa 10 e os trabalhos posteriores `R1`–`R6` (revisão do plano, Python, concentração, duplicação, código sem referências e Git diff). A CLI cria um modelo de `auditor.toml`, inventaria arquivos, imprime o grafo de dependências, aplica regras determinísticas, gera relatórios em texto, JSON ou Markdown e registra execuções em SQLite. A aplicação desktop e a extensão VS Code reutilizam o mesmo núcleo. Regras personalizadas, sugestões com IA e auditoria agendada ficam fora desta fase por decisão registrada.

## Documentação

- Decisões técnicas e ambiente verificado: [docs/DECISIONS.md](docs/DECISIONS.md)
- Progresso das etapas: [docs/ROADMAP.md](docs/ROADMAP.md)
- Regras para agentes: [AGENTS.md](AGENTS.md)

## Direção técnica definida

- Núcleo de auditoria em Rust, reutilizado pela CLI, pela aplicação desktop e pela extensão VS Code.
- Interface desktop implementada com Tauri 2, React e TypeScript.
- Extensão VS Code implementada sobre a interface JSON da CLI, sem daemon e sem regras locais.
- Histórico local em SQLite; contrato dos projetos auditados em TOML.
- Processamento local e auditoria somente leitura por padrão.
- Regras determinísticas, sem dependência de IA ou de internet para o funcionamento básico.

## Ambiente

O resumo das versões encontradas e das referências oficiais consultadas está em [docs/DECISIONS.md](docs/DECISIONS.md). Nenhuma ferramenta do sistema foi instalada, atualizada ou removida; as dependências Rust do projeto passaram a ser resolvidas pelo Cargo a partir da Etapa 1.

## Comandos de desenvolvimento

```sh
cargo check
cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p auditor-cli -- --help
cargo run -p auditor-cli -- --version
cargo run -p auditor-cli -- init <diretorio>
cargo run -p auditor-cli -- scan <raiz>
cargo run -p auditor-cli -- analyze <raiz>
cargo run -p auditor-cli -- audit <raiz>
cargo run -p auditor-cli -- audit <raiz> --format json
cargo run -p auditor-cli -- audit <raiz> --format markdown --output relatorio.md
git diff > alteracoes.patch
cargo run -p auditor-cli -- audit <raiz> --diff alteracoes.patch
cargo run -p auditor-cli -- history record <raiz>
cargo run -p auditor-cli -- history list
cargo run -p auditor-cli -- history show <id>
cargo run -p auditor-cli -- history compare <id-anterior> <id-posterior>
cargo run -p auditor-cli -- history list --database /caminho/historico.db
```

No Windows, com a pasta do projeto sincronizada pelo OneDrive, o Cargo pode emitir avisos `Acesso negado` ao finalizar sessões de compilação incremental. Para uma execução limpa do Clippy nesse ambiente:

```sh
cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings
```

## Contrato arquitetural

O contrato de um projeto auditado usa `schema_version = 1` e contém:

- `[project] name`: nome do projeto.
- `[scan] include` e `exclude`: padrões de caminho relativo, com `/` como separador.
- `[architecture] style`: nesta versão, apenas `modular_monolith`.
- `[[architecture.modules]]`: `name`, `path`, `responsibility` e `allowed_dependencies`.

Campos desconhecidos, nomes ou caminhos duplicados, referências a módulos inexistentes, caminhos absolutos e versões de schema não suportadas são rejeitados com mensagem explicativa. O modelo gerado pelo subcomando `init` (`cargo run -p auditor-cli -- init <diretorio>`) está em [`auditor.example.toml`](auditor.example.toml) e **precisa ser revisado e confirmado pelo usuário** antes de ser considerado aprovado.

## Descoberta

O comando `scan <raiz>` lê `<raiz>/auditor.toml`, percorre a árvore sem executar arquivos e imprime o inventário. Os padrões suportados são `**` como componente inteiro e `*` dentro de um componente; `?`, classes e chaves não são interpretados. Diretórios `.git`, `target`, `node_modules`, `dist`, `build`, `.venv`, `venv`, `.next`, `out`, `coverage` e `.turbo` são ignorados em qualquer profundidade, além de `scan.exclude`. Links simbólicos nunca são seguidos, e entradas inacessíveis são registradas sem interromper a varredura. Cada arquivo incluído registra caminho relativo, extensão e tamanho em bytes.

## Análise de dependências

O comando `analyze <raiz>` lê o contrato e o inventário, lê os manifests `Cargo.toml` do workspace e usa `syn` para interpretar declarações `mod` e importações `use` dos arquivos `.rs` incluídos. Para `.ts`, `.tsx`, `.js` e `.jsx`, um scanner próprio reconhece `import`, `export ... from`, `require(...)` e `import(...)`. Para `.py`, reconhece `import` e `from ... import`. Comentários e strings são descartados antes da leitura, e especificadores relativos são resolvidos lexicalmente contra os arquivos do inventário. Um arquivo só é associado a um módulo quando exatamente um `path` do contrato corresponde ao caminho relativo. Sem correspondência, ele vira `file.unmapped`; com múltiplas correspondências, vira `file.ambiguous_module` com os candidatos listados em ordem alfabética, sem gerar arestas ou declarações `mod` atribuídas a um módulo escolhido.

As arestas são classificadas assim:

- `permitida`: o destino está em `allowed_dependencies` do módulo de origem.
- `proibida`: o destino é um módulo declarado no contrato, mas não está em `allowed_dependencies`.
- `não resolvida`: o destino não pôde ser associado de forma inequívoca a um módulo declarado — nome ausente do contrato ou múltiplos candidatos — e também gera item de revisão.

A origem de cada aresta guarda arquivo relativo, linha e a linha de código encontrada. Declarações `mod` também são registradas com arquivo e linha; o conteúdo de `mod` inline é percorrido. `use` de `std`, `core`, `alloc`, `test`, `self`, `super` e dependências do próprio módulo não geram arestas arquiteturais. Em TS/JS, especificadores sem `./` ou `../` são ignorados como pacote externo, exceto quando o nome coincide com um módulo do contrato — nesse caso viram revisão, nunca erro. Em Python, nomes absolutos só viram aresta quando começam com um módulo do contrato e possuem sufixo de arquivo encontrado no inventário; pacotes externos e `import *` são ignorados. Se o arquivo resolvido pertence a mais de um módulo do contrato, a dependência vira `dep.unresolved` com os candidatos ordenados, nunca `dep.not_allowed`.

Limitações explícitas: em Rust, imports dentro de corpos de funções, macros complexos, código gerado e nomes resolvidos dinamicamente não são classificados; arquivos que não fazem parse viram limitação. Em TS/JS, não há suporte a `tsconfig.paths`, workspaces npm, resolução de pacotes, template literals ou imports construídos dinamicamente; `import type` é contado como acoplamento. Em Python, apenas `import` e `from ... import` em uma linha são reconhecidos; nomes absolutos sem sufixo de arquivo, imports relativos fora da raiz, `import *` e resolução dinâmica não geram arestas. O contador `packages` continua se referindo somente a pacotes Cargo, com limitation explícita quando há arquivos TS/JS ou Python.

## Cobertura da análise

A auditoria distingue arquivos incluídos no inventário de arquivos de código efetivamente processados. O relatório expõe um campo estruturado `coverage` com `code_files_included`, `code_files_analyzed`, `code_files_not_analyzed`, `unsupported_code_extensions` (extensão e quantidade) e `partial`. Não há nota geral de qualidade: `partial` indica apenas que pelo menos um arquivo de código incluído não foi analisado.

Extensões de código reconhecidas sem analisador — por exemplo `.go`, `.java`, `.cs`, `.cpp` e `.rb` — são contadas por extensão e geram uma limitação visível em texto, Markdown, JSON e desktop. Documentos e dados (`.md`, `.json`, `.toml`, imagens) não são tratados como código. Extensões fora da lista conhecida também não são classificadas como código; essa é uma limitação explícita, não uma suposição de cobertura. Falhas de leitura ou parse em arquivos com analisador entram em `code_files_not_analyzed` e `partial`, além de manter a limitação específica do arquivo.

O campo `coverage` é aditivo no JSON: a extensão VS Code continua consumindo `findings` como antes. O histórico SQLite continua gravando as limitações em texto, sem coluna nova para a cobertura estruturada nesta etapa. A fixture `tests/fixtures/projects/partial_coverage` exerce o caso: 5 arquivos incluídos, 4 de código, 2 analisados, `.go` (1) e `.rb` (1) sem analisador, 1 limitação e 0 achados. O comando `analyze <raiz>` também mostra a cobertura.

A regra `module.concentration` usa apenas arestas resolvidas entre módulos declarados, deduplicadas por par origem/destino. Ela conta origens e destinos distintos e sinaliza revisão quando um deles chega a 3; o limite é fixo para manter a regra determinística e não vira parâmetro do contrato. O custo é `O(M + E)` por auditoria, sem leitura adicional de arquivos; a decisão sobre refatorar permanece com o usuário.

A regra `code.duplicated` compara linhas já lidas pela análise, ignorando comentários e declarações de import/export/use. Somente linhas com 40 ou mais caracteres e repetidas em mais de um arquivo viram revisão; o identificador usa um hash FNV-1a estável do trecho, sem dependência externa. O custo é linear no total de linhas incluídas. Não há detecção semântica, normalização de variáveis nem comparação de blocos: apenas repetição exata de uma linha.

A regra `module.unreferenced` usa o mesmo grafo resolvido: módulos com arquivos incluídos, mas sem aresta de entrada ou saída, viram revisão. É um proxy conservador para código isolado, não uma prova de código morto: o grafo não registra o arquivo de destino, apenas o módulo de destino. Módulos sem arquivos continuam sendo tratados somente por `module.no_files`. O custo é `O(M + E)`.

## Motor de regras e relatórios

O comando `audit <raiz>` reutiliza o mesmo pipeline do núcleo e aplica estas regras com identificadores estáveis:

| Regra | Severidade | Situação |
|---|---|---|
| `dep.not_allowed` | `error` | Destino é módulo declarado, mas ausente de `allowed_dependencies`. |
| `dep.unresolved` | `needs_review` | Destino não pôde ser associado de forma inequívoca a um módulo do contrato. |
| `file.unmapped` | `needs_review` | Arquivo de código incluído não corresponde a nenhum módulo. |
| `file.ambiguous_module` | `needs_review` | Arquivo corresponde a mais de um módulo do contrato. |
| `dep.cycle` | `warning` | Grafo contém ciclo simples entre módulos, incluindo autoimportação. |
| `module.no_files` | `warning` | Módulo declarado sem arquivo de código correspondente. |
| `module.concentration` | `needs_review` | Módulo com 3 ou mais origens ou destinos distintos no grafo resolvido. |
| `code.duplicated` | `needs_review` | Linha idêntica com 40 ou mais caracteres em dois ou mais arquivos. |
| `module.unreferenced` | `needs_review` | Módulo com arquivos incluídos e zero arestas resolvidas incidentes. |

Cada achado contém `id`, `rule_id`, `severity`, `title`, `message`, `path`, `line_start`, `line_end`, `evidence`, `expected`, `suggestion` e `confidence`. Todas as regras são determinísticas, então `confidence` é sempre nulo; o auditor não inventa precisão numérica nem aplica sugestões automaticamente. Dependências não resolvidas, destinos ambíguos e arquivos não mapeados ou ambíguos são sinais para decisão humana, não violações comprovadas.

As fixtures `tests/fixtures/projects/ambiguous_mapping_a` e `ambiguous_mapping_b` descrevem o mesmo projeto com a ordem dos módulos invertida. Ambas produzem o mesmo relatório: um arquivo ambíguo com `use crate::infrastructure::Client;` não gera `dep.not_allowed`, e um import Python cujo destino pertence a dois módulos gera `dep.unresolved` com os candidatos `domain, entities`.

Os formatos disponíveis são `text` (padrão), `json` e `markdown`. Sem `--output`, o relatório vai para o terminal; com `--output <arquivo>`, o arquivo indicado é gravado e sobrescrito se já existir. O projeto auditado nunca é modificado.

## Auditoria incremental com `git diff`

`audit <raiz> --diff <arquivo>` lê um patch unificado salvo de `git diff`, sem executar o Git e sem alterar arquivos. O núcleo converte cada bloco `@@ ... @@` em intervalos de linhas adicionadas e mantém somente os achados cujo caminho corresponde a um arquivo alterado. Achados com linha conhecida entram apenas se a linha estiver em um desses intervalos; achados sem linha e achados de módulo com padrão `path` entram quando o arquivo alterado corresponde ao caminho. As contagens do resumo são recalculadas, e uma limitação diz explicitamente que achados fora do diff foram omitidos.

O filtro é uma visão parcial do relatório, não uma nova regra: ele não muda severidades, identificadores nem o esquema JSON. O relatório completo continua sendo `audit <raiz>` sem `--diff`. Limitações conhecidas: diffs combinados (`@@@`) e caminhos citados com escapes não são interpretados; renomeações puro sem blocos são tratadas como mudança do caminho de destino; linhas removidas não têm linha correspondente na versão atual.

## TypeScript e JavaScript (Etapa 10)

Para auditar TS/JS, declare extensões explícitas em `scan.include` — chaves como `*.{ts,tsx}` não são curingas suportadas:

```toml
[scan]
include = ["src/**/*.ts", "src/**/*.tsx", "src/**/*.js", "src/**/*.jsx"]
```

A fixture `tests/fixtures/projects/typescript` exercita imports permitidos e proibidos, `require`, `import()` dinâmico, reexportação, pacote externo ignorado, nome de módulo ambíguo e comentário/string que não podem virar aresta. A fixture `tests/fixtures/projects/python` cobre `import`, `from ... import`, comentários, strings triplas e pacote externo. CLI, desktop e extensão VS Code consomem o mesmo relatório, sem regras locais. Nenhuma dependência Rust foi adicionada ao núcleo.

## Histórico local

O banco padrão é `data/auditor.db`, relativo ao diretório de execução da CLI; esse diretório é ignorado pelo Git. `--database <arquivo>` pode ser usado antes ou depois da operação `history` para escolher outro local.

| Comando | Finalidade |
|---|---|
| `history record <raiz>` | executa a auditoria e salva uma nova execução |
| `history list` | lista execuções da mais recente para a mais antiga |
| `history list --project <nome> --limit <n>` | filtra por nome do contrato e limita a quantidade |
| `history show <id>` | mostra uma execução com achados e limitações |
| `history compare <antes> <depois>` | classifica achados como novos, resolvidos ou persistentes |

São gravados data/hora em UTC, raiz canonicalizada do projeto, nome e versão do schema do contrato, resumo da execução, achados e limitações. **O conteúdo integral dos arquivos auditados nunca é gravado**: de cada achado ficam apenas metadados e a linha de evidência já apresentada no relatório. A comparação usa o `id` estável de cada achado, não texto livre, e preserva a ordem dos achados gravados.

Política de migração: a tabela `schema_migrations` registra a versão do schema do histórico. A versão atual é `1`; aberturas em bancos já migrados não alteram dados, e um banco com versão superior à suportada é rejeitado com mensagem explícita, sem downgrade ou exclusão automática. Não há comando de limpeza nesta etapa: o banco cresce com novas execuções e deve ser gerenciado pelo usuário.

## Validação do núcleo (Etapa 7)

Três projetos fictícios em `tests/fixtures/projects/` exercitam o pipeline completo sem interface gráfica. A fixture `simple` continua servindo apenas à descoberta de arquivos.

| Fixture | Cenário | Resultado observado |
|---|---|---|
| `clean` | arquitetura correta, dependências permitidas e workspace Cargo coerente | 0 achados, 0 limitações, saída `0` |
| `dependencies` | dependência proibida, ciclos, destino não resolvido e arquivo não mapeado | 7 achados (2 erros, 3 avisos, 2 revisões), 1 limitação, saída `1` |
| `partial` | estrutura parcialmente desconhecida: dependência externa, path sobreposto, arquivo fora de módulo, módulo sem arquivos e arquivo com erro de parse | 4 achados (1 aviso, 3 revisões), 1 limitação, saída `1` |

```sh
cargo run -p auditor-cli -- audit tests/fixtures/projects/clean
cargo run -p auditor-cli -- audit tests/fixtures/projects/dependencies --format json
cargo run -p auditor-cli -- audit tests/fixtures/projects/partial --format markdown
cargo run -p auditor-cli -- history record tests/fixtures/projects/clean --database /caminho/temporario.db
cargo run -p auditor-cli -- history record tests/fixtures/projects/partial --database /caminho/temporario.db
cargo run -p auditor-cli -- history compare 1 2 --database /caminho/temporario.db
```

A avaliação não encontrou falso positivo no projeto correto. Os três ciclos de `dependencies` são caminhos fechados reais no grafo, embora se sobreponham; essa enumeração já era uma limitação conhecida da Etapa 5. Em `partial`, o que não pôde ser decidido foi classificado como revisão necessária, e o arquivo com sintaxe inválida aparece somente em `Limitações`, sem virar erro inventado. Mensagens e evidências foram revisadas: todo achado traz `id`, quando há linha ela é apontada, e a evidência é a linha de código observada ou uma explicação explícita para achados de arquivo. O formato do contrato v1 foi suficiente para os três cenários; nenhum campo novo foi introduzido.

## Aplicação desktop (Etapa 8)

A aplicação está em `apps/desktop`: Tauri 2, React 19, TypeScript 7 e Vite 8. O crate `apps/desktop/src-tauri` é um adaptador fino — ele traduz chamadas da interface para `auditor-core` e devolve modelos serializados (`AuditReport`, `Analysis` e `RunSummary`). A interface apenas filtra e renderiza; nenhuma regra de auditoria foi movida para o frontend.

Pré-requisitos verificados no ambiente: WebView2 `158.0.4370.33`, MSVC `14.44.35207` (Visual Studio 2022 Community), Node `24.19.0`, npm `11.17.0` e Rust `1.98.1`. Nenhuma ferramenta do sistema foi instalada, atualizada ou removida.

```sh
cd apps/desktop
npm install
npm run tauri dev
npm run tauri build
```

Telas disponíveis: seleção nativa da pasta do projeto, auditoria com resumo e filtros por severidade/regra, grafo de dependências em SVG com as classificações feitas pelo núcleo, visualizador somente leitura do `auditor.toml`, histórico local com registro explícito e painel de configurações do caminho do banco.

Segurança: apenas `core:default` e `dialog:allow-open` foram concedidos à janela principal; não há plugin de filesystem, CSP foi definido explicitamente e não há execução automática em segundo plano. O contrato nunca é editado pela interface — uma futura edição exigiria confirmação explícita.

Limitações conhecidas: o caminho do banco é relativo ao diretório de execução do aplicativo (use um caminho absoluto no painel para fixá-lo); o empacotamento de instaladores está desativado (`bundle.active = false`), então `tauri build` gera somente o executável; o ícone incluído é técnico e substituível. A comparação de execuções continua disponível pela CLI, não pela interface.

O critério de conclusão foi verificado por testes de adaptação: o comando `audit_project` devolve exatamente o mesmo `AuditReport` produzido por `auditor-core`, e o binário de release foi gerado e iniciado com sucesso.

## Extensão VS Code (Etapa 9)

A extensão em `extensions/vscode` detecta `auditor.toml` na pasta aberta, oferece auditoria manual pelo paleta de comandos ou menu de contexto, converte o JSON da CLI em diagnósticos do painel **Problems** e abre o arquivo/linha de cada achado. A barra de status mostra a última auditoria e o estado é restaurado ao reabrir o workspace.

```sh
cd extensions/vscode
npm install
npm test
npm run package
```

O comando `npm test` compila o TypeScript e executa 4 testes puros de conversão; `npm run package` gera `vibe-coding-auditor-0.1.0.vsix`. O executável usado vem de `vibeCodingAuditor.cliPath` (padrão: procurar no `PATH` e depois em `target/release`/`target/debug` da pasta aberta). A extensão aceita os códigos de saída `0` e `1` da CLI; somente códigos maiores que `1` são falha.

Mapeamento: `error` → `Error`, `warning` e `needs_review` → `Warning`, `info` → `Information`. O `rule_id` vira código do diagnóstico e a mensagem inclui `id`, evidência, expectativa e sugestão. Nenhuma regra é avaliada na extensão e nenhum daemon é criado.

Limitações: não há testes automatizados dentro da host do VS Code; a verificação foi feita com testes de conversão, empacotamento e um smoke CLI→modelo nas três fixtures. O estado local guarda metadados e linhas de evidência já presentes no relatório, não conteúdo integral de arquivos. A extensão não acessa o histórico SQLite. `npm audit` aponta 2 alertas moderados apenas em dependência de desenvolvimento do `@vscode/vsce` (`0` em runtime); a versão anterior sugerida pelo auditor era pior, com 3 alertas e um módulo nativo adicional.

## Execução

O executável `vibe-coding-auditor` responde a `--help`, `--version`, `init <diretorio>`, `scan <raiz>`, `analyze <raiz>`, `audit <raiz>` e `history <operacao>`. `init` cria `auditor.toml` sem sobrescrever arquivo existente; `scan` e `analyze` são somente leitura e servem para inspeção. `audit` produz achados e relatórios no formato escolhido. `history record` reutiliza a auditoria e grava o banco; `list`, `show` e `compare` apenas leem o banco.

Códigos de saída: `0` quando a auditoria não gera achados, `1` quando há pelo menos um achado e `2` para falha operacional ou argumento inválido. `audit` e `history record` usam `0`/`1` conforme os achados da execução; `init`, `scan`, `analyze`, `history list`, `history show` e `history compare` retornam `0` em sucesso e `2` em falha.
