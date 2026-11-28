# Vibe Coding Auditor

Ferramenta local para comparar a arquitetura pretendida de um projeto com a arquitetura realmente implementada. Cada achado deve apontar a regra violada, os arquivos envolvidos e, quando possível, as linhas relevantes. O contrato arquitetural (`auditor.toml`) pertence ao usuário e é versionado junto ao projeto auditado.

**Status atual:** Etapas 0–3 e a fase inicial 4–10 concluídas; na nova fase, a Etapa 4 (descoberta e Architecture Builder) e a Etapa 5 (suíte E2E) foram concluídas em 27/11/2026, com ajustes posteriores de nome da CLI, visibilidade da extensão e interface já validados. A Etapa 6 está em andamento: o baseline 6.0 está em [docs/validation/etapa-6/baseline.md](docs/validation/etapa-6/baseline.md), a reconciliação documental 6.1 foi aplicada e a descoberta de manifests multi-ecossistema 6.2 e a resolução determinística 6.3 foram concluídas em 27/11/2026; as subetapas 6.4–6.8 permanecem planejadas.

A validação automatizada está registrada em [docs/validation/etapa-4](docs/validation/etapa-4) e no run `46067832029` ([E2E Windows](https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/46067832029)); o usuário confirmou em 27/11/2026 a validação manual do desktop com os projetos VibeCodingAuditor e BackBet e, após reinstalação do `.vsix` e ajuste do `vibeCodingAuditor.cliPath` para `vibe-coding-auditor-cli`, confirmou a validação da extensão na host real do VS Code (descoberta sem `auditor.toml` sem violações de conformidade, status/resumo de descoberta e auditoria com contrato exibindo diagnósticos no Problems). A CLI cria um modelo de `auditor.toml`, inventaria arquivos, imprime o grafo de dependências, aplica regras determinísticas, gera relatórios em texto, JSON ou Markdown e registra execuções em SQLite. A aplicação desktop e a extensão VS Code reutilizam o mesmo núcleo. Regras personalizadas, sugestões com IA e auditoria agendada ficam fora desta fase por decisão registrada. Detalhes das fases anteriores em [docs/validation/etapa-3](docs/validation/etapa-3).

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

Modo automático: `scan`, `analyze` e `audit` escolhem o resultado pela presença de `auditor.toml` — contrato presente e válido seleciona `compliance`; ausência seleciona `discovery`; TOML existente e inválido é erro explícito (código `2`), sem fallback para descoberta. `history record` segue o mesmo modo. Os detalhes e os códigos de saída estão nas seções **Descoberta** e **Execução**.

No Windows, com a pasta do projeto sincronizada pelo OneDrive, o Cargo pode emitir avisos `Acesso negado` ao finalizar sessões de compilação incremental. Para uma execução limpa do Clippy nesse ambiente:

```sh
cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings
```

## Contrato arquitetural

O contrato de um projeto auditado usa `schema_version = 1` ou `2` e contém:

- `[project] name`: nome do projeto.
- `[scan] include` e `exclude`: padrões de caminho relativo, com `/` como separador.
- `[architecture] style`: nesta versão, apenas `modular_monolith`.
- `[[architecture.modules]]`: `name`, `path`, `responsibility` e `allowed_dependencies`.
- `[[architecture.layers]]` (somente v2): `name`, `modules` e `allowed_dependencies`.

Campos desconhecidos, nomes ou caminhos duplicados, referências a módulos inexistentes, caminhos absolutos e versões de schema não suportadas são rejeitados com mensagem explicativa que nomeia o campo. No v2, cada módulo deve pertencer a exatamente uma camada quando `layers` é declarado; nomes de camada duplicados, lista vazia, auto-dependência, destino desconhecido, dependência duplicada, ciclo entre camadas e paths de módulos sobrepostos também são rejeitados. `responsibility` continua sendo descrição para humanos: o auditor não interpreta texto livre como regra. O modelo gerado pelo subcomando `init` (`cargo run -p auditor-cli -- init <diretorio>`) continua em v1, está em [`auditor.example.toml`](auditor.example.toml) e **precisa ser revisado e confirmado pelo usuário** antes de ser considerado aprovado. O auditor lê v1 e v2 no mesmo núcleo e nunca reescreve um contrato existente. Um exemplo v2 validado está em [`tests/fixtures/projects/layer_boundaries/auditor.toml`](tests/fixtures/projects/layer_boundaries/auditor.toml).

## Descoberta

O comando `scan <raiz>` lê `<raiz>/auditor.toml`, percorre a árvore sem executar arquivos e imprime o inventário. Os padrões suportados são `**` como componente inteiro e `*` dentro de um componente; `?`, classes e chaves não são interpretados. Diretórios `.git`, `target`, `node_modules`, `dist`, `build`, `.venv`, `venv`, `.next`, `out`, `coverage` e `.turbo` são ignorados em qualquer profundidade, além de `scan.exclude`. Links simbólicos nunca são seguidos, e entradas inacessíveis são registradas sem interromper a varredura. Cada arquivo incluído registra caminho relativo, extensão e tamanho em bytes.

Desde a Etapa 4, `auditor.toml` é opcional para `scan`, `analyze` e `audit`: a presença do contrato seleciona `compliance` e a ausência seleciona `discovery`, sem nova flag. TOML presente e inválido continua sendo erro explícito, sem fallback para descoberta. Sem contrato, `scan` usa os defaults seguros com os mesmos motivos de exclusão; `analyze` mostra arestas `observada`, sem rótulos `permitida`, `proibida` ou `não resolvida`, e apresenta módulos candidatos, relações classificadas por evidência, ambiguidades e limitações. `audit` em descoberta usa os mesmos três formatos e expõe `mode: "discovery"`, `contract.status: "missing"` e `findings: []`; texto e Markdown começam com `Modo: descoberta (sem contrato)`. Em conformidade, o JSON ganha apenas `mode: "compliance"` e `contract` com `status` e `schema_version`; nenhum campo antigo foi removido ou renomeado. Com `--diff`, a descoberta valida o patch, mas não filtra achados e registra essa limitação explicitamente. `history record` segue o modo do projeto: sem contrato grava descoberta com modo explícito, `schema_version = 0`, zero achados e identidade ausente; nunca transforma observação em conformidade.

Módulos candidatos são sugestões com `needs_confirmation: true`: a categoria `direct` vem de manifesto Cargo reconhecido, `inferred_path` de diretório com código e `ambiguous` de evidências conflitantes. Nenhuma dependência observada é convertida em permissão sem decisão do usuário.

A descoberta também lê `package.json`, `pyproject.toml` e `go.mod` na raiz e em subprojetos, sem executar scripts, ambientes ou comandos do projeto. Cada arquivo presente vira uma observação em `candidates.manifests` com ecossistema, estado (`parsed`, `malformed` ou `unsupported`), pacotes locais, dependências separadas por papel (`runtime`, `development`, `peer`, `optional`, `optional_group`, `workspace`, `require`, `replace`, `exclude`) e destino (`external` ou `local_workspace`), sempre com caminho relativo e evidência curta. Workspaces npm reconhecem listas e `workspaces.packages` com diretórios literais ou `Nome/*`; padrões com `**`, `?`, `..` ou `*` em outra posição viram limitação localizada. PEP 621 é lido em `[project]`; formatos específicos de Poetry/uv e versões dinâmicas são registrados como limitações, não inferidos. `go.mod` cobre `module`, `require` (linha ou bloco), `replace`, `exclude` e comentários; diretivas não compreendidas viram limitação com número de linha. Arquivos `.go` continuam **sem analisador**, inclusive quando aparecem no inventário; a leitura de `go.mod` não implica análise do código Go. Manifest ausente não gera observação; manifest inválido ou parcialmente suportado não derruba a varredura. Esses manifests alimentam a resolução determinística da Etapa 6.3, descrita em `analyze` e nos relatórios.

Três categorias não se confundem: **observação** é uma relação encontrada no código, sem juízo arquitetural; **sugestão** é um candidato do núcleo que ainda exige confirmação; **violação** é um achado de conformidade produzido pelas regras do contrato aprovado. Em descoberta há observações e sugestões, nunca violações.

## Architecture Builder

O Architecture Builder fica na aplicação desktop e transforma a descoberta do núcleo em um rascunho editável. Ele percorre sete etapas fixas (perfil, projeto, módulos, responsabilidades, dependências, decisões pendentes e revisão), mantém módulos e relações como sugestões até decisão explícita e usa somente o validador e o serializador do núcleo. O rascunho é v2; contratos v1 existentes são lidos e convertidos para revisão sem reescrita automática.

A gravação é a única etapa com efeito colateral e exige duas ações separadas: visualizar o diff e confirmar a substituição. `Create` nunca sobrescreve um `auditor.toml` existente; `Replace` confere o texto atual outra vez antes da renomeação atômica. O arquivo temporário fica no mesmo diretório e é removido em caso de falha. Fechar ou cancelar o assistente não cria nem altera arquivo. Depois de salvar, o núcleo reabre o contrato e executa a conformidade, devolvendo o modo real ao desktop.

Limitações atuais: o Builder ainda não transforma as observações de `package.json`, `pyproject.toml` ou `go.mod` em sugestões editáveis de módulo; elas aparecem no relatório, mas não entram no rascunho. Ele não cria contrato na extensão VS Code, não há gravação automática nem aprovação implícita. `init` continua gerando contrato v1, e o Builder gera v2.

## Análise de dependências

O comando `analyze <raiz>` lê o contrato e o inventário, lê os manifests `Cargo.toml` do workspace — incluindo `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[target.*.dependencies]` (e equivalentes de dev/build por plataforma) e `[workspace.dependencies]` — e usa `syn` para interpretar declarações `mod` e importações `use` dos arquivos `.rs` incluídos. As crates declaradas aparecem em `declared_dependencies` no JSON e na saída do `analyze`, com nomes normalizados para identificadores Rust (`-` vira `_`). Aliases `package = "..."` e entradas herdadas com `workspace = true` são resolvidos sem executar o Cargo. Membros de `[workspace].members` com um `*` no último componente são expandidos para diretórios reais com `Cargo.toml`, em ordem determinística, e filtrados por `workspace.exclude` antes da leitura; membros literais mantêm o comportamento anterior. Outros curingas, `*` em componente intermediário, caminhos com `.`/`..`, diretório ausente e links simbólicos viram limitação explícita em `analysis.limitations` em vez de erro de leitura. Nenhum comando do Cargo ou do projeto analisado é executado. Dependências declaradas com `path` são resolvidas lexicalmente dentro da raiz: as herdadas de `[workspace.dependencies]` usam a raiz do workspace como base e as diretas usam o diretório do manifesto do membro. Para `.ts`, `.tsx`, `.js` e `.jsx`, um scanner próprio lê o arquivo como documento contínuo e reconhece `import`, `export ... from`, `require(...)` e `import(...)`, inclusive em declarações estáticas multilinha; comentários, strings, template literals e regex são mascarados, e `import.meta` não é confundido com declaração de importação. Para `.py`, reconhece `import` e `from ... import`. Comentários e strings são descartados antes da leitura, e especificadores relativos são resolvidos lexicalmente contra os arquivos do inventário. Um arquivo só é associado a um módulo quando exatamente um `path` do contrato corresponde ao caminho relativo. Sem correspondência, ele vira `file.unmapped`; com múltiplas correspondências, vira `file.ambiguous_module` com os candidatos listados em ordem alfabética, sem gerar arestas ou declarações `mod` atribuídas a um módulo escolhido.

As arestas são classificadas assim:

- `permitida`: o destino está em `allowed_dependencies` do módulo de origem.
- `proibida`: o destino é um módulo declarado no contrato, mas não está em `allowed_dependencies`.
- `não resolvida`: o destino não pôde ser associado de forma inequívoca a um módulo declarado — nome ausente do contrato ou múltiplos candidatos — e também gera item de revisão.

A origem de cada aresta guarda arquivo relativo, linha e a linha de código encontrada. Declarações `mod` também são registradas com arquivo e linha; o conteúdo de `mod` inline é percorrido. `use` de `std`, `core`, `alloc`, `test` e dependências do próprio módulo não geram arestas arquiteturais; uma crate externa declarada no Cargo também não vira aresta nem revisão. Uma dependência `path` que aponta para membro do workspace vira aresta `permitida` ou `proibida` quando o pacote corresponde a um único módulo do contrato; sem associação segura, ela aparece no `analyze` como aresta `não resolvida` e gera a revisão `dep.unresolved`, sem aresta `permitida` ou `proibida`. Caminhos `self::` e `super::` são examinados: quando o destino é outro módulo do contrato, a aresta vira `permitida` ou `proibida`; quando o destino é símbolo do próprio módulo ou crate externa declarada, não há aresta; sem evidência suficiente, há revisão explícita em vez de descarte silencioso. `crate::self` também é tratado como destino interno. Caminhos `crate::...` são resolvidos contra a raiz de fontes inferida do componente `src`: o prefixo mais longo encontrado no inventário é associado ao contrato; arquivos do mesmo módulo não geram aresta, arquivos de outro módulo geram `permitida`/`proibida` e a ausência de arquivo mantém a regra anterior do primeiro segmento. Em TS/JS, a ordem de resolução é: caminho relativo, alias local de `tsconfig*.json`, pacote local de workspace, nome de módulo do contrato, built-in Node, pacote externo declarado e, por fim, pacote externo não confirmado. Um alias com um único arquivo no inventário gera aresta classificada; com zero ou vários destinos, gera revisão explícita, nunca escolha silenciosa. Em Python, nomes absolutos viram aresta quando o primeiro segmento é módulo do contrato e há sufixo de arquivo no inventário; fora disso, o módulo é relacionado a uma distribuição declarada somente quando a equivalência é determinística por caso/separador. `import *` continua sem aresta. Se o arquivo resolvido pertence a mais de um módulo do contrato, a dependência vira `dep.unresolved` com os candidatos ordenados, nunca `dep.not_allowed`.

Relações com pacotes aparecem em `external_packages`, separadas das arestas entre módulos, com estado (`builtin`, `declared`, `unconfirmed` ou `workspace_local`), origem, evidência, arquivo e linha. Nenhum desses estados vira violação por si só: somente uma regra declarada no contrato pode elevar uma relação a achado. A descoberta sem contrato reutiliza o mesmo contexto para classificar `observed_dependencies` e expõe as mesmas relações em `analysis.external_packages`, sempre como observação ou sugestão.

Limitações explícitas: em Rust, imports dentro de corpos de funções, macros complexos, código gerado e nomes resolvidos dinamicamente não são classificados; arquivos que não fazem parse viram limitação. A associação de um pacote local ao contrato usa o prefixo literal do `path` do módulo; curinga em componente intermediário, pacote fora do workspace, alias conflitante e caminho que tenta sair da raiz não geram aresta definitiva e viram revisão explícita. A resolução de `super::` usa o diretório do arquivo e os arquivos do inventário, não a árvore completa de módulos Rust: quando um nome coincide com módulo do contrato ele é classificado pelo contrato; quando não há arquivo de destino no inventário, a revisão é explícita. quando nenhum arquivo do inventário corresponde a um caminho `crate::`, o primeiro segmento após a raiz volta a ser usado como destino, para não inventar associação; `super::` que aponta para um símbolo do próprio módulo ainda gera revisão, porque o analisador não resolve símbolos Rust. A evidência textual de alguns `use` agrupados pode aparecer deslocada em uma linha em relação ao destino classificado; a aresta e a revisão usam o número de linha localizado, mas essa diferença é uma limitação conhecida, não uma violação. A expansão de membros do workspace aceita somente um `*` no último componente: `**`, `?`, classes de caracteres e `*` em componente intermediário geram limitação acionável, não uma falha ou uma suposição sobre os pacotes. Em TS/JS, `tsconfig*.json` é lido somente no subconjunto local: `compilerOptions.baseUrl`, `paths` com zero ou um `*` e `extends` apontando para arquivo dentro da raiz. `extends` por pacote externo vira limitação e não é carregado; destino de `paths` que tenta sair da raiz também é limitação. Workspaces e dependências de `package.json` identificam a origem do pacote, mas não aprovam dependência nem criam aresta: `node_modules`, registry, scripts e ferramentas do projeto nunca são consultados. `import type` é contado como acoplamento. `import()` e `require()` com literal estático continuam gerando aresta, mas um especificador de template literal ou uma chamada sem literal vira limitação explícita. A detecção de regex é heurística: quando o contexto permite, a regex é mascarada, mas uma barra ambígua pode ser tratada como divisão. Em Python, apenas `import` e `from ... import` em uma linha são reconhecidos; nomes absolutos sem sufixo de arquivo, imports relativos fora da raiz, `import *` e resolução dinâmica não geram arestas, mas podem virar relação externa com estado explícito. Nomes de distribuição sem equivalência determinística — por exemplo `Pillow` para `PIL` — ficam como `unconfirmed`, com limitação localizada; nenhuma instalação ou ambiente é consultado. O contador `packages` continua se referindo somente a pacotes Cargo, com limitação explícita quando há arquivos TS/JS ou Python.

## Cobertura da análise

A auditoria distingue arquivos incluídos no inventário de arquivos de código efetivamente processados. O relatório expõe um campo estruturado `coverage` com `code_files_included`, `code_files_analyzed`, `code_files_not_analyzed`, `unsupported_code_extensions` (extensão e quantidade) e `partial`. Não há nota geral de qualidade: `partial` indica apenas que pelo menos um arquivo de código incluído não foi analisado.

Extensões de código reconhecidas sem analisador — por exemplo `.go`, `.java`, `.cs`, `.cpp` e `.rb` — são contadas por extensão e geram uma limitação visível em texto, Markdown, JSON e desktop. Documentos e dados (`.md`, `.json`, `.toml`, imagens) não são tratados como código. Extensões fora da lista conhecida também não são classificadas como código; essa é uma limitação explícita, não uma suposição de cobertura. Falhas de leitura ou parse em arquivos com analisador entram em `code_files_not_analyzed` e `partial`, além de manter a limitação específica do arquivo.

O campo `coverage` é aditivo no JSON: a extensão VS Code continua consumindo `findings` como antes. Desde a Etapa 3.6, o histórico SQLite também grava cobertura estruturada em tabelas próprias; desde a Etapa 4.7, o schema v3 acrescenta modo de execução e identidade do contrato. `history show` e `history compare` apresentam esses dados. A fixture `tests/fixtures/projects/partial_coverage` exerce o caso: 5 arquivos incluídos, 4 de código, 2 analisados, `.go` (1) e `.rb` (1) sem analisador, 1 limitação e 0 achados. O comando `analyze <raiz>` também mostra a cobertura.

O relatório também expõe `skip_summary`, com `total`, `intentional`, `coverage_gaps` e `reasons`. Cada motivo traz `reason`, `label`, `count`, até 3 `examples` e `additional` para os caminhos omitidos. Diretórios ignorados por padrão, exclusões do contrato e arquivos fora de `include` são intencionais; link simbólico, entrada inacessível e tipo não tratado são marcados como `reduces_coverage`. A fixture `tests/fixtures/projects/simple` tem 14 entradas fora do inventário: 10 ignoradas por padrão, 1 excluída e 3 fora do `include`. O resumo aparece em texto, Markdown, JSON e desktop sem despejar a lista completa. Um teste do núcleo cobre os seis motivos no construtor real do relatório, incluindo link simbólico, entrada inacessível e tipo não tratado como redutores de cobertura; não há fixture com esses três casos reais porque não são reproduzíveis de forma igual no Windows. Após reavaliar o item 2.7, `coverage` e `skip_summary` continuaram fora do esquema SQLite até a Etapa 3.6, quando o usuário confirmou a necessidade de comparar cobertura entre execuções. Passaram a ser gravadas apenas contagens agregadas e rótulos estáveis, sem os exemplos de caminho exibidos no relatório.

A regra `module.concentration` usa apenas arestas resolvidas entre módulos declarados, deduplicadas por par origem/destino. Ela conta origens e destinos distintos e sinaliza revisão quando um deles chega a 3; o limite é fixo para manter a regra determinística e não vira parâmetro do contrato. O custo é `O(M + E)` por auditoria, sem leitura adicional de arquivos; a decisão sobre refatorar permanece com o usuário.

A regra `code.duplicated` compara linhas já lidas pela análise, ignorando comentários e declarações de import/export/use. Somente linhas com 40 ou mais caracteres e repetidas em mais de um arquivo viram revisão; o identificador usa um hash FNV-1a estável do trecho, sem dependência externa. O custo é linear no total de linhas incluídas. Não há detecção semântica, normalização de variáveis nem comparação de blocos: apenas repetição exata de uma linha.

A regra `module.unreferenced` usa o mesmo grafo resolvido: módulos com arquivos incluídos, mas sem aresta de entrada ou saída, viram revisão. É um proxy conservador para código isolado, não uma prova de código morto: o grafo não registra o arquivo de destino, apenas o módulo de destino. Módulos sem arquivos continuam sendo tratados somente por `module.no_files`. O custo é `O(M + E)`.

## Motor de regras e relatórios

O comando `audit <raiz>` reutiliza o mesmo pipeline do núcleo e aplica estas regras com identificadores estáveis:

| Regra | Severidade | Situação |
|---|---|---|
| `dep.not_allowed` | `error` | Destino é módulo declarado, mas ausente de `allowed_dependencies`. |
| `dep.layer_not_allowed` | `error` | Aresta resolvida permitida pelo módulo, mas não pela camada de origem (contrato v2). |
| `dep.unresolved` | `needs_review` | Destino não pôde ser associado de forma inequívoca a um módulo do contrato. |
| `file.unmapped` | `needs_review` | Arquivo de código incluído não corresponde a nenhum módulo. |
| `file.ambiguous_module` | `needs_review` | Arquivo corresponde a mais de um módulo do contrato. |
| `dep.cycle` | `warning` | Grafo contém um componente fortemente conectado entre módulos, incluindo autoimportação; há um achado por componente. |
| `module.no_files` | `warning` | Módulo declarado sem arquivo de código correspondente. |
| `module.concentration` | `needs_review` | Módulo com 3 ou mais origens ou destinos distintos no grafo resolvido. |
| `code.duplicated` | `needs_review` | Linha idêntica com 40 ou mais caracteres em dois ou mais arquivos. |
| `module.unreferenced` | `needs_review` | Módulo com arquivos incluídos e zero arestas resolvidas incidentes. |

Em contratos v2, `dep.not_allowed` tem precedência: se a lista do módulo já proíbe a aresta, a camada não gera segundo erro para a mesma ligação. O identificador de `dep.layer_not_allowed` é `dep.layer_not_allowed:{origem}:{destino}:{path}`, sem linha; a linha e a evidência continuam fora da identidade. Dependência não resolvida continua sendo apenas `dep.unresolved`, sem conclusão de camada. Se um mapeamento de camada faltar em tempo de execução apesar da validação, a ausência vira limitação explícita, nunca `error`.

Cada achado contém `id`, `rule_id`, `severity`, `title`, `message`, `path`, `line_start`, `line_end`, `evidence`, `expected`, `suggestion` e `confidence`. Todas as regras são determinísticas, então `confidence` é sempre nulo; o auditor não inventa precisão numérica nem aplica sugestões automaticamente. Dependências não resolvidas, destinos ambíguos e arquivos não mapeados ou ambíguos são sinais para decisão humana, não violações comprovadas.

As fixtures `tests/fixtures/projects/ambiguous_mapping_a` e `ambiguous_mapping_b` descrevem o mesmo projeto com a ordem dos módulos invertida. Ambas produzem o mesmo relatório: um arquivo ambíguo com `use crate::infrastructure::Client;` não gera `dep.not_allowed`, e um import Python cujo destino pertence a dois módulos gera `dep.unresolved` com os candidatos `domain, entities`.

A fixture `tests/fixtures/projects/rust_dependencies` cobre a classificação do item 2.3: imports de crates declaradas em tabelas normais, de desenvolvimento, de build, específicas por plataforma e no workspace — inclusive com alias e herança `workspace = true` — não viram `dep.unresolved`. Um `super::` que atravessa uma fronteira gera uma aresta `proibida` (`dep.not_allowed`) ou `permitida` (sem achado) conforme `allowed_dependencies`, e um `super::` sem destino identificável gera revisão. O relatório textual da fixture tem 1 erro, 1 aviso de ciclo e 3 revisões, sem achados para as crates declaradas.

A fixture `tests/fixtures/projects/workspace_local_dependencies` cobre a Etapa 3.1.1: `cli-app` declara `core-lib` por `path` direto e `tools-lib` por `path` herdado, ambos com alias, e também importa `serde`. `analyze` mostra `cli -> core [permitida]` e `cli -> tools [não resolvida]`; `serde` continua ignorado como pacote externo. `audit` termina com 2 revisões (`dep.unresolved:tools` e `file.unmapped`), 0 erros e 0 avisos. A ordem dos membros e dos módulos no TOML não altera a classificação.

A fixture `tests/fixtures/projects/crate_module_resolution` cobre a Etapa 3.1.3: um módulo cobre o crate inteiro, `crate::analysis::config::Settings` e `crate::analysis::Analysis` são internos e não geram aresta, enquanto `crate::unknown::Thing` continua `dep.unresolved`. O teste de regressão falhou antes da correção e passou depois dela; os testes de `super::`, `crate::self` e dependência `path` continuaram verdes.

A fixture `tests/fixtures/projects/workspace_glob` cobre o item 2.4: `members = ["crates/*", "legacy"]` com `exclude = ["crates/omitted"]` descobre `alpha`, `beta` e `legacy`; diretórios sem `Cargo.toml` são ignorados e o resultado é determinístico. `analyze` mostra os três pacotes, `serde` e a aresta `application -> domain`; `audit` termina com 0 achados e 0 limitações. A fixture `tests/fixtures/projects/workspace_glob_unsupported` reúne `crates/**`, `crates/?lpha`, `../outside/*` e um glob sem correspondência: todos viram limitações explícitas, com 0 pacotes descobertos e nenhum erro de leitura.

A fixture `tests/fixtures/projects/typescript_scanner` cobre o item 2.5: um `import` multilinha gera a única aresta `application -> domain`, com evidência reconstruída em uma linha; `import.meta`, regex e template literal não geram aresta nem achado; `import()` sem literal e com template literal geram duas limitações específicas. O relatório tem 0 achados e 3 limitações, incluindo a de pacotes npm. Um teste de regressão reúne comentário, string, regex, `import.meta`, `import` multilinha, reexportação e `require` em uma única varredura e espera apenas as três arestas reais. A fixture `typescript` anterior mantém 8 arestas, 4 achados e 1 limitação.

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

A fixture `tests/fixtures/projects/typescript` exercita imports permitidos e proibidos, `require`, `import()` dinâmico, reexportação, pacote externo ignorado, nome de módulo ambíguo e comentário/string que não podem virar aresta. A fixture `tests/fixtures/projects/python` cobre `import`, `from ... import`, comentários, strings triplas e pacote externo; um teste de regressão garante que texto dentro de comentário ou string tripla não vire aresta. CLI, desktop e extensão VS Code consomem o mesmo relatório, sem regras locais. Nenhuma dependência Rust foi adicionada ao núcleo.

## Histórico local

O banco padrão é `data/auditor.db`, relativo ao diretório de execução da CLI; esse diretório é ignorado pelo Git. `--database <arquivo>` pode ser usado antes ou depois da operação `history` para escolher outro local.

| Comando | Finalidade |
|---|---|
| `history record <raiz>` | executa o projeto no modo determinado pelo núcleo e salva uma nova execução |
| `history list` | lista execuções da mais recente para a mais antiga |
| `history list --project <nome> --limit <n>` | filtra por nome do contrato e limita a quantidade |
| `history show <id>` | mostra uma execução com achados, limitações e cobertura estruturada |
| `history compare <antes> <depois>` | classifica achados como novos, resolvidos ou persistentes e mostra o delta de cobertura somente entre execuções compatíveis |

São gravados data/hora em UTC, raiz canonicalizada do projeto, modo de execução, identidade do contrato quando conhecida, nome e versão do schema do contrato, resumo da execução, achados, limitações e cobertura estruturada. Descoberta entra com `schema_version = 0`, modo `discovery`, zero achados e identidade ausente. A cobertura fica em três tabelas: contagens de arquivos de código incluídos, analisados e não analisados com o indicador `partial`; extensões sem analisador com quantidade de arquivos; e motivos de exclusão com posição estável, rótulo, contagem e impacto na cobertura. **O conteúdo integral dos arquivos auditados nunca é gravado**: de cada achado ficam apenas metadados e a linha de evidência já apresentada no relatório, e os motivos não guardam exemplos de caminho. A comparação usa o `id` estável de cada achado, não texto livre, e preserva a ordem dos achados gravados. Execuções gravadas antes da migração v2 não recebem números inventados: aparecem como `sem dados`. A comparação de cobertura não verifica se as duas execuções pertencem ao mesmo projeto; desde a migração v3, porém, `history compare` recusa modos diferentes, duas descobertas e conformidades com identidade diferente ou desconhecida, com mensagem explícita.

A identidade segue o vínculo arquitetural, não a posição no arquivo: `dep.not_allowed` usa `rule_id`, origem, destino e arquivo, e `dep.unresolved` usa `rule_id`, destino e arquivo. Inserir linhas ou mudar a evidência não cria outro achado; linha e evidência continuam sendo a localização e a justificativa atuais do mesmo achado. Duas importações com o mesmo vínculo no mesmo arquivo são apresentadas uma única vez, na primeira ocorrência determinística. Alterar origem, destino ou regra produz identidade diferente. Bancos antigos não são reescritos: `history compare` normaliza apenas IDs legados de dependência que terminavam em `:<linha>`; `history show` continua devolvendo o ID gravado. A identidade não é preservada após renomear um arquivo, porque o caminho participa do vínculo.

Política de migração: a tabela `schema_migrations` registra cada versão aplicada do schema do histórico. A versão atual é `3`; bancos v1 e v2 recebem `execution_mode` e `contract_identity` na mesma transação das migrações pendentes ao abrir, e um banco com versão superior à suportada é rejeitado com mensagem explícita, sem downgrade ou exclusão automática. Linhas legadas ficam com modo `compliance` pela origem histórica e identidade desconhecida; nenhum contrato é inventado para elas. Se alguma migração falhar, a versão anterior permanece registrada. Não há comando de limpeza nesta etapa: o banco cresce com novas execuções e deve ser gerenciado pelo usuário.

## Validação do núcleo (Etapa 7)

Três projetos fictícios em `tests/fixtures/projects/` exercitam o pipeline completo sem interface gráfica. A fixture `simple` continua servindo apenas à descoberta de arquivos.

| Fixture | Cenário | Resultado observado |
|---|---|---|
| `clean` | arquitetura correta, dependências permitidas e workspace Cargo coerente | 0 achados, 0 limitações, saída `0` |
| `dependencies` | dependência proibida, componente cíclico, destino não resolvido e arquivo não mapeado | 5 achados (2 erros, 1 aviso, 2 revisões), 1 limitação, saída `1` |
| `partial` | estrutura parcialmente desconhecida: dependência externa, path sobreposto, arquivo fora de módulo, módulo sem arquivos e arquivo com erro de parse | 4 achados (1 aviso, 3 revisões), 1 limitação, saída `1` |

```sh
cargo run -p auditor-cli -- audit tests/fixtures/projects/clean
cargo run -p auditor-cli -- audit tests/fixtures/projects/dependencies --format json
cargo run -p auditor-cli -- audit tests/fixtures/projects/partial --format markdown
cargo run -p auditor-cli -- history record tests/fixtures/projects/clean --database /caminho/temporario.db
cargo run -p auditor-cli -- history record tests/fixtures/projects/partial --database /caminho/temporario.db
cargo run -p auditor-cli -- history compare 1 2 --database /caminho/temporario.db
```

A fixture `tests/fixtures/projects/finding_identity` cobre a Etapa 3.2 com três variantes: `before` tem a dependência proibida na linha 1, `after` insere comentários e move a mesma dependência para a linha 4, e `evidence_changed` mantém o vínculo com evidência diferente. As três produzem o mesmo `id` `dep.not_allowed:application:infrastructure:src/lib.rs`; `history compare 1 2` entre `before` e `after` retorna 0 novos, 0 resolvidos e 2 persistentes.

A avaliação não encontrou falso positivo no projeto correto. Os três módulos de `dependencies` formam um único componente fortemente conectado: antes havia três avisos sobre caminhos fechados sobrepostos; agora há um aviso `dep.cycle:application->domain->infrastructure` com evidência ordenada das cinco arestas internas. A busca usa Tarjan iterativo em `O(V + E)`, e a evidência é limitada a 8 arestas com contagem do restante para grafos densos. Testes de desempenho cobrem um grafo denso de 80 módulos e 6400 arestas (limite de 5 s) e 600 componentes desconectados (limite de 10 s), ambos com achados e evidências limitados. Em `partial`, o que não pôde ser decidido foi classificado como revisão necessária, e o arquivo com sintaxe inválida aparece somente em `Limitações`, sem virar erro inventado. Mensagens e evidências foram revisadas: todo achado traz `id`, quando há linha ela é apontada, e a evidência é a linha de código observada ou uma explicação explícita para achados de arquivo. O formato do contrato v1 foi suficiente para os três cenários; nenhum campo novo foi introduzido.

## Aplicação desktop (Etapa 8)

A aplicação está em `apps/desktop`: Tauri 2, React 19, TypeScript 7 e Vite 8. O crate `apps/desktop/src-tauri` é um adaptador fino — ele traduz chamadas da interface para `auditor-core` e devolve modelos serializados (`AuditReport`, `Analysis` e `RunSummary`). A interface apenas filtra e renderiza; nenhuma regra de auditoria foi movida para o frontend.

Pré-requisitos verificados no ambiente: WebView2 `158.0.4370.33`, MSVC `14.44.35207` (Visual Studio 2022 Community), Node `24.19.0`, npm `11.17.0` e Rust `1.98.1`. O workspace declara `rust-version = 1.98.1`; para reproduzir o verificador, use `rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy`, confirme Node/npm nas versões acima e rode `npm ci` em `apps/desktop` e `extensions/vscode` antes dos comandos abaixo. Nenhuma ferramenta do sistema foi instalada, atualizada ou removida pelo projeto.

## Verificação automatizada

O workflow `.github/workflows/ci.yml` é o único do repositório e roda em `windows-latest` nos pushes para `main`, nos pull requests e em disparos manuais, com Node `24.19.0` e Rust `1.98.1`. Ele possui dois jobs: `verify` (`cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm test` e `npm run build` da desktop, e `npm test` da extensão — sempre após `npm ci`) e `E2E Windows` (`needs: verify`, com build release, suítes reais de CLI, extensão e desktop e upload de artefatos em falha). A execução remota corrente é `CI Windows` #18, run `46067832029`, commit `4ceeab2`, em 27/11/2026: `verify` e `E2E Windows` terminaram com sucesso em `12m 57s`, com o E2E desktop aprovando 5 testes usando WebView2 `157.0.4341.60`. O run registra um aviso: `actions/cache@v4` ainda executa em Node.js 20. As execuções antigas #1 (commit `7a07286`, [execução](https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/45811082223)), #2 (commit `14cbc12`) e #3 (commit `f5fc270`, [execução](https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/45813011665)) permanecem como histórico. O job E2E compila o executável com `tauri build`, mas `bundle.active = false`: nenhum instalador é gerado e nenhuma release é publicada por este workflow (ver Etapa 6.7).

```sh
cd apps/desktop
npm install
npm test
npm run build
npm run tauri dev
npm run tauri build
```

`npm run tauri build` gera `target/release/auditor-desktop.exe` para execução local e para o E2E; com `bundle.active = false` ele **não** produz instalador, bundle NSIS/MSI nem artefato de release. A preparação de distribuição está na Etapa 6.7.

O `npm test` compila `src/skipSummary.ts`, `src/reportView.ts`, `src/status.ts`, `src/viewState.ts`, `src/builderFlow.ts` e `src/BuilderWizard.tsx` e executa 39 testes Node puros, sem dependência nova, sobre resumo de exclusões, apresentação do relatório, status das ações, limpeza da tela e assistente. Os testes de apresentação e status usam o mesmo JSON real produzido pela CLI.

Telas disponíveis: seleção nativa da pasta, área de status com a última mensagem e um resumo recolhido das anteriores, painel de configurações do banco e abas para os resultados: Descoberta, Relatório, Mapa, Contrato e Histórico. Só o resultado ativo aparece; trocar a pasta limpa todos os resultados, mensagens e filtros anteriores. O relatório mostra primeiro cinco problemas e oferece “Ver mais”; detalhes como referência, trecho encontrado, expectativa e sugestão ficam recolhidos. O mapa de módulos usa SVG com as classificações feitas pelo núcleo e esconde a lista longa de ligações atrás de um resumo. O contrato é somente leitura e o histórico mantém modo e contrato em texto corrente. Desde a Etapa 4, `Auditar / descobrir` usa o modo devolvido pelo núcleo: sem contrato, abre observações em vez de violações. Após cada ação, a tela rola até as abas de resultado. TOML existente e inválido continua sendo erro explícito do núcleo, sem fallback para descoberta.

Segurança: apenas `core:default` e `dialog:allow-open` foram concedidos à janela principal; não há plugin de filesystem, CSP foi definido explicitamente e não há execução automática em segundo plano. O contrato nunca é editado pela interface — uma futura edição exigiria confirmação explícita.

Limitações conhecidas: o caminho do banco é relativo ao diretório de execução do aplicativo (use um caminho absoluto no painel para fixá-lo); o empacotamento de instaladores está desativado (`bundle.active = false`): `tauri build` gera somente `auditor-desktop.exe`, não um instalador, bundle ou artefato de release; isso não é uma release distribuível e será tratado na Etapa 6.7. O ícone incluído é técnico e substituível. A comparação de execuções continua disponível pela CLI, não pela interface. A verificação do resumo de exclusões foi feita por tipos, build e testes dos helpers de apresentação. A janela Tauri foi conferida manualmente pelo usuário em 26/11/2026 na Etapa 3 e novamente em 27/11/2026 na Etapa 4 com os projetos VibeCodingAuditor e BackBet; ele confirmou o funcionamento, mas relatou excesso de informações, rolagem longa, resíduos ao trocar de projeto e linguagem técnica, corrigidos na Etapa 4.9. A extensão foi ajustada para não depender de `auditor.toml` no menu e para localizar a CLI no repositório da extensão; o usuário confirmou em 27/11/2026 a validação na host real do VS Code após reinstalar o `.vsix` e ajustar `vibeCodingAuditor.cliPath` para `vibe-coding-auditor-cli`. Após o ajuste pós-4.9, as observações de descoberta são registradas no canal `Vibe Coding Auditor` e a extensão mostra um único aviso informativo; o usuário confirmou esse comportamento em 27/11/2026.

O critério de conclusão foi verificado por testes de adaptação: o comando `audit_project` devolve exatamente o mesmo `AuditReport` produzido por `auditor-core`, e o binário de release foi gerado e iniciado com sucesso.

## Extensão VS Code (Etapa 9)

A extensão em `extensions/vscode` oferece auditoria manual pela paleta de comandos ou pelo menu de contexto, converte o JSON da CLI em diagnósticos do painel **Problems** e abre o arquivo/linha de cada achado. A barra de status mostra a última execução e o estado é restaurado ao reabrir o workspace. Desde a Etapa 4, o campo `mode` do JSON distingue os resultados: em conformidade, os diagnósticos preservam `rule_id`, severidade e evidência da CLI; em descoberta, a extensão limpa diagnósticos antigos e mostra as observações e limitações do núcleo como mensagens informativas, com orientação para usar o Architecture Builder no desktop. Nenhuma regra é reavaliada na extensão. JSON legado sem `mode` continua sendo tratado como conformidade. O item do menu de contexto aparece em qualquer pasta, inclusive sem `auditor.toml` (`when: explorerResourceIsFolder`); a paleta também funciona sem contrato.

```sh
cd extensions/vscode
npm install
npm test
npm run package
```

O comando `npm test` compila o TypeScript e executa 18 testes puros de conversão, resolução do executável da CLI, manifesto e paridade; `npm run package` gera `vibe-coding-auditor-0.1.0.vsix`. O executável usado vem de `vibeCodingAuditor.cliPath` (padrão `vibe-coding-auditor-cli`: procura no `PATH`, depois em `target/release`/`target/debug` da pasta aberta e também no repositório que contém a extensão). O menu de contexto aparece em qualquer pasta, mesmo sem `auditor.toml`, e o comando executa descoberta quando não há contrato. A extensão aceita os códigos de saída `0` e `1` da CLI; somente códigos maiores que `1` são falha.

Mapeamento: `error` → `Error`, `warning` e `needs_review` → `Warning`, `info` → `Information`. O `rule_id` vira código do diagnóstico e a mensagem inclui `id`, evidência, expectativa e sugestão. Nenhuma regra é avaliada na extensão e nenhum daemon é criado.

Limitações: não há testes automatizados dentro da host do VS Code; a verificação foi feita com testes de conversão, empacotamento e um smoke CLI→modelo nas três fixtures. O estado local guarda metadados e linhas de evidência já presentes no relatório, não conteúdo integral de arquivos. A extensão não acessa o histórico SQLite. `npm audit` aponta 2 alertas moderados apenas em dependência de desenvolvimento do `@vscode/vsce` (`0` em runtime); a versão anterior sugerida pelo auditor era pior, com 3 alertas e um módulo nativo adicional.

## Autoauditoria (Etapa 3.1)

O `auditor.toml` da raiz audita somente o código Rust de produção: `core`, `cli` e `desktop-backend`. `cli` e `desktop-backend` podem depender de `core`; o núcleo não depende de adaptadores. React e extensão VS Code estão fora do grafo, e a comunicação entre TypeScript e Tauri/CLI não é apresentada como verificada. O relatório completo está em [docs/validation/etapa-3/self-audit.md](docs/validation/etapa-3/self-audit.md).

Na execução corrente, `scan .` inventaria 15 arquivos Rust e 47 entradas não incluídas, todas intencionais. `analyze .` mostra 9 arestas `permitidas`, nenhuma `proibida` e 8 `não resolvidas`; `cli -> core` e `desktop-backend -> core` aparecem, e nenhuma crate externa declarada é falsamente classificada como módulo interno. `audit .` termina com 62 revisões (54 `code.duplicated` e 8 `dep.unresolved`), 0 erros, 0 avisos e 0 limitações, com cobertura 15/15. As 8 revisões remotas são `super::` em módulos de teste; a heurística de duplicação compara apenas linhas exatas.

## Validação em alvo sintético (Etapa 3.3)

O diretório `validation/targets/fake-order-service` é um projeto falso criado a pedido, com contrato v1 próprio e quatro módulos: `domain`, `application`, `infrastructure` e `webapi`. Três dependências proibidas e um ciclo foram plantados. A auditoria é somente leitura: os 20 arquivos do alvo tiveram o mesmo SHA-256 antes e depois de duas execuções.

`scan` inventaria 12 arquivos incluídos e 8 entradas intencionais; `analyze` encontra 6 arestas, das quais 3 proibidas; `audit` gera 7 achados (3 erros, 1 aviso e 3 revisões). O censo manual classificou 4 verdadeiros positivos e 3 revisões corretas, sem falso positivo e sem violação plantada não detectada. `tools/check.go` não foi analisado por falta de analisador `.go`, e o `import()` dinâmico em `web/src/client.ts` não virou aresta. O relatório completo está em [docs/validation/etapa-3/second-project-synthetic.md](docs/validation/etapa-3/second-project-synthetic.md). Este alvo não substitui uma futura validação em projeto real de terceiros.

## Execução

O executável `vibe-coding-auditor-cli` responde a `--help`, `--version`, `init <diretorio>`, `scan <raiz>`, `analyze <raiz>`, `audit <raiz>` e `history <operacao>`. Sem argumentos, imprime uma apresentação e a ajuda e termina com código `0`; é uma CLI de terminal, não uma janela — para a interface gráfica use `auditor-desktop.exe`. No Windows, o atalho `vibe-coding-auditor-cli.cmd` na raiz executa a CLI e, quando aberto sem argumentos, pausa antes de fechar a janela. `init` cria `auditor.toml` sem sobrescrever arquivo existente; `scan` e `analyze` são somente leitura e servem para inspeção. `audit` produz achados e relatórios no formato escolhido. `history record` reutiliza a auditoria e grava o banco; `list`, `show` e `compare` apenas leem o banco.

Códigos de saída: `0` quando a operação termina sem achados de conformidade — inclusive descoberta com observações e candidatos —, `1` quando há pelo menos um achado de conformidade e `2` para falha operacional, contrato inválido ou argumento inválido. Ausência de `auditor.toml` nunca produz `2` em `scan`, `analyze`, `audit` ou `history record`; TOML existente e inválido produz `2` sem cair em descoberta. `audit` e `history record` usam `0`/`1` conforme os achados da execução; `init`, `scan`, `analyze`, `history list`, `history show` e `history compare` retornam `0` em sucesso e `2` em falha. `history record` sem contrato registra descoberta e retorna `0`. `history compare` recusa comparações entre modos diferentes, duas descobertas ou conformidades com identidade desconhecida/diferente e retorna `2`.
