# Roadmap

Estado consolidado das Etapas 0–10 e dos trabalhos posteriores. Decisões e escopo em [docs/DECISIONS.md](DECISIONS.md). Uma etapa só avança depois de concluída, testada e revisada.

| Etapa | Nome | Status |
|---|---|---|
| 0 | Verificar o ambiente e preparar o repositório | Concluída em 25/11/2026 — documentação criada e repositório Git inicializado |
| 1 | Criar o workspace Rust e a CLI mínima | Concluída em 25/11/2026 — `cargo check`, `test`, `fmt --check` e `clippy` passaram |
| 2 | Criar o modelo do contrato arquitetural | Concluída em 25/11/2026 — 10 testes passaram; `init` validado |
| 3 | Descobrir e inventariar o projeto | Concluída em 25/11/2026 — 16 testes passaram; `scan` validado em fixture |
| 4 | Implementar a primeira análise de dependências Rust | Concluída em 25/11/2026 — 36 testes passaram; `analyze` validado em fixture |
| 5 | Criar o motor de regras e os relatórios | Concluída em 25/11/2026 — 60 testes passaram; `audit` validado em três formatos |
| 6 | Registrar histórico local em SQLite | Concluída em 25/11/2026 — 73 testes passaram; `record`, `list`, `show` e `compare` validados |
| 7 | Provar o núcleo antes da interface gráfica | Concluída em 25/11/2026 — três projetos fictícios auditados pela CLI; 75 testes passaram |
| 8 | Criar a aplicação desktop com Tauri | Concluída em 25/11/2026 — Tauri/React compilados e iniciados; 78 testes passaram |
| 9 | Criar a extensão VS Code | Concluída em 25/11/2026 — `.vsix` gerado; diagnósticos conferem com a CLI; 82 testes passaram |
| 10 | Expandir linguagens e recursos | Concluída em 25/11/2026; extensões adicionais nesta rodada: TypeScript/JavaScript e Python no núcleo. Demais opções avaliadas e diferidas |

## Critérios da Etapa 0

- [x] Verificar `git`, `rustc`, `cargo`, `rustup`, `node` e `npm`
- [x] Verificar pré-requisitos oficiais do Tauri para Windows sem instalar nada
- [x] Consultar documentação oficial das versões estáveis disponíveis
- [x] Criar o repositório Git (`git init`, autorizado; branch `main`)
- [x] Criar os arquivos mínimos de documentação (`AGENTS.md`, `README.md`, `docs/DECISIONS.md`, `docs/ROADMAP.md`)
- [x] Preservar o plano e não criar código de funcionalidade

## Critérios da Etapa 1

- [x] Criar um Cargo workspace com `auditor-core` e `auditor-cli`
- [x] Fazer o executável responder a `--help` e `--version`
- [x] Configurar `cargo fmt` e `cargo clippy`
- [x] Criar testes unitários mínimos (2 na CLI, 1 no núcleo)
- [x] Documentar os comandos de desenvolvimento no `README.md` e no `AGENTS.md`
- [x] `cargo check`, `cargo test`, `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings` passaram

## Critérios da Etapa 2

- [x] Definir o schema v1 e rejeitar versões não suportadas com mensagem clara
- [x] Criar modelos Rust para projeto, módulos, estilo e exclusões
- [x] Validar campos obrigatórios, caminhos e referências; rejeitar propriedades desconhecidas
- [x] Criar `auditor init` com modelo que exige revisão e sem sobrescrever contrato existente
- [x] Criar fixtures válidas e inválidas com testes reproduzíveis
- [x] `cargo check`, `cargo test`, `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings` passaram

## Critérios da Etapa 3

- [x] Receber a raiz do projeto como argumento explícito (`scan <raiz>`)
- [x] Respeitar `include` e `exclude` do contrato
- [x] Ignorar por padrão diretórios gerados e de controle de versão
- [x] Não seguir links simbólicos
- [x] Registrar caminho relativo, extensão, tamanho e motivo das entradas ignoradas
- [x] Lidar com entradas inacessíveis sem interromper a varredura
- [x] Criar árvore de diretórios controlada em `tests/fixtures/projects/simple`
- [x] `cargo check`, `cargo test`, `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings` passaram

## Critérios da Etapa 4

- [x] Ler os manifests `Cargo.toml` do workspace e dos membros declarados, sem executar o Cargo
- [x] Analisar declarações `mod` e importações `use` com `syn`, incluindo módulos inline
- [x] Relacionar arquivos `.rs` aos módulos do contrato pelo mesmo matcher da descoberta
- [x] Construir um grafo direcionado entre módulos
- [x] Classificar arestas permitidas, proibidas e não resolvidas
- [x] Registrar dependências não resolvidas e arquivos não mapeados como revisão necessária
- [x] Guardar arquivo, linha e texto de evidência de cada aresta
- [x] Criar a fixture `tests/fixtures/projects/dependencies` com os três tipos de aresta
- [x] Adicionar o subcomando `analyze <raiz>` somente para inspeção do grafo
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes diretos dos helpers
- [x] `cargo check`, `cargo test` (36 testes), `cargo fmt --check` e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` passaram

## Critérios da Etapa 5

- [x] Separar descoberta, análise, avaliação de regras e geração de relatórios em módulos do núcleo
- [x] Atribuir identificadores estáveis às seis regras implementadas
- [x] Produzir achados com todos os campos do modelo da seção 6 do plano
- [x] Classificar dependência proibida, dependência não resolvida, arquivo fora de módulo, arquivo ambíguo, ciclo entre módulos e módulo sem arquivos
- [x] Gerar saída legível no terminal e exportar JSON e Markdown
- [x] Definir códigos de saída `0` (sem achados), `1` (com achados) e `2` (falha operacional)
- [x] Garantir que os mesmos dados gerem relatórios idênticos e que cada achado traga arquivo, linha e evidência quando disponíveis
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] `cargo check`, `cargo test` (60 testes: 51 no núcleo e 9 na CLI), `cargo fmt --check` e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` passaram

## Critérios da Etapa 6

- [x] Salvar data/hora UTC, raiz canonicalizada, nome do projeto, versão do contrato e resumo da execução
- [x] Salvar achados associados à execução, com severidade, local, evidência e sugestão
- [x] Permitir listar execuções com filtro por projeto e limite, e consultar uma execução pelo identificador
- [x] Comparar duas execuções e classificar achados novos, resolvidos e persistentes pelo identificador estável
- [x] Definir política de migração versionada, sem downgrade automático e com rejeição explícita de schema mais novo
- [x] Não armazenar conteúdo integral dos arquivos do usuário no banco
- [x] Manter a renderização de texto do histórico no núcleo; a CLI apenas imprime
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] `cargo check`, `cargo test` (73 testes: 63 no núcleo e 10 na CLI), `cargo fmt --check` e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` passaram

## Critérios da Etapa 7

- [x] Criar pelo menos três projetos fictícios: `clean` (arquitetura correta), `dependencies` (violações intencionais) e `partial` (estrutura parcialmente desconhecida)
- [x] Executar a auditoria completa pela CLI nos três projetos, em texto, JSON e Markdown
- [x] Verificar o pipeline com `history record` e `history compare` entre um projeto correto e um parcial
- [x] Avaliar falsos positivos: nenhum no projeto correto; ciclos sobrepostos documentados como limitação conhecida
- [x] Revisar mensagens e evidências, mantendo revisões e limitações explícitas quando a análise não decide
- [x] Manter o schema v1 do contrato, sem campos sem implementação
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] `cargo check`, `cargo test` (75 testes: 65 no núcleo e 10 na CLI), `cargo fmt --check` e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` passaram

## Critérios da Etapa 8

- [x] Criar `apps/desktop` com Tauri 2, React 19, TypeScript 7 e Vite 8
- [x] Manter o backend como adaptador fino que chama somente funções de `auditor-core`
- [x] Selecionar e abrir projeto pelo diálogo nativo
- [x] Executar a auditoria e apresentar resumo e achados filtráveis por severidade e regra
- [x] Visualizar o grafo de dependências classificado pelo núcleo
- [x] Visualizar o contrato somente leitura e operar o histórico local com registro explícito
- [x] Incluir configuração local do caminho do banco
- [x] Restringir permissões a `core:default` e `dialog:allow-open`, com CSP definido e sem plugin de filesystem
- [x] Não implementar execução automática em segundo plano nem alteração silenciosa de contrato
- [x] `cargo check`, `cargo test` (78 testes: 65 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check` e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` passaram
- [x] `npm install`, `npm run build` e `npm run tauri build` passaram; `auditor-desktop.exe` foi gerado e iniciado
- [x] Não antecipar a extensão VS Code da Etapa 9

## Critérios da Etapa 9

- [x] Criar `extensions/vscode` consumindo a interface JSON da CLI, sem daemon e sem regras na extensão
- [x] Detectar `auditor.toml` na pasta aberta e publicar o estado no contexto do editor
- [x] Acionar auditoria manual pela paleta de comandos e pelo menu de contexto da pasta
- [x] Mostrar achados no painel Problems com severidade, código da regra e mensagem rastreável
- [x] Abrir arquivo e linha do achado selecionado
- [x] Mostrar e restaurar o estado da última auditoria na barra de status
- [x] Aceitar os códigos de saída `0` e `1` da CLI como resultados válidos
- [x] `npm test` com 4 testes puros de conversão e `npm run package` gerando `vibe-coding-auditor-0.1.0.vsix`
- [x] Smoke CLI → JSON → diagnósticos: `clean` 0/0, `dependencies` 7/7, `partial` 4/4
- [x] `cargo test` permaneceu com 78 testes; total verificado: 82 testes (78 Rust e 4 da extensão)
- [x] Limitações registradas: sem testes na host do VS Code, alerta moderado apenas no `@vscode/vsce` de desenvolvimento e sem histórico na extensão
- [x] Não iniciar a Etapa 10 antes de fechar a Etapa 9

## Critérios da Etapa 10

- [x] Avaliar todas as opções do plano e escolher TypeScript/JavaScript por caso de uso real e menor risco arquitetural
- [x] Manter o schema v1 do contrato: ativação por globs explícitos de extensão, sem campos novos
- [x] Adicionar analisador determinístico no núcleo, sem nova dependência Rust e sem regras na CLI, na UI ou na extensão
- [x] Reconhecer `import`, `export ... from`, `require(...)` e `import(...)` em `.ts`, `.tsx`, `.js` e `.jsx`
- [x] Ignorar comentários e strings; resolver imports relativos contra o inventário e classificar pela mesma regra do Rust
- [x] Transformar especificador ambíguo em `needs_review`, nunca em `error`; limitar pacotes npm não enumerados
- [x] Ignorar diretórios gerados do ecossistema frontend (`.next`, `out`, `coverage`, `.turbo`) com teste na descoberta
- [x] Criar a fixture `tests/fixtures/projects/typescript` com imports permitidos, proibidos, dinâmicos, reexportação e ruído
- [x] Preservar contagens das fixtures anteriores e o contrato JSON consumido por desktop e extensão
- [x] `cargo check`, `cargo test` (84 testes: 71 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check` e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` passaram
- [x] Smoke na CLI: `typescript` = 8 arestas, 4 achados (2 erros e 2 revisões), 1 limitação, saída `1`
- [x] Smoke CLI → JSON → extensão: 4 achados produziram 4 diagnósticos (2 `error` e 2 `warning`); `npm test` e `npm run build` da desktop passaram
- [x] Limitações registradas: sem `tsconfig.paths`, workspaces npm, resolução de pacotes, template literals ou imports dinâmicos construídos em tempo de execução; `import type` conta como acoplamento; `packages` continua sendo Cargo
- [x] Python, complexidade, duplicação, regras personalizadas, IA, Git diff e auditoria agendada avaliados e diferidos por falta de caso de uso validado nesta rodada

## Critérios da extensão para Python (após a Etapa 10)

- [x] Analisador determinístico para `.py`, sem nova dependência, sem mudança no schema e sem regras fora do núcleo
- [x] Reconhecer `import` e `from ... import`; mascarar comentários `#` e strings triplas
- [x] Resolver nomes absolutos contra módulos do contrato e sufixos de arquivo do inventário; nomes ambíguos viram revisão
- [x] Criar a fixture `tests/fixtures/projects/python` com imports permitidos, proibidos, ruído e arquivo fora dos módulos
- [x] Registrar limitações: pacotes pip não enumerados, `import *`, nomes absolutos sem sufixo, imports relativos fora da raiz e resolução dinâmica
- [x] Verificação completa: `cargo check`, `cargo test` (89 testes: 76 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` passaram
- [x] Smoke na CLI: `python` = 4 arestas, 3 achados (2 erros e 1 revisão), 1 limitação, saída `1`; fixtures anteriores mantiveram as contagens

## Critérios de complexidade e concentração (após a Etapa 10)

- [x] Regra `module.concentration` no núcleo, usando arestas resolvidas e deduplicadas por par origem/destino
- [x] Limite explícito de 3 origens ou destinos distintos, sem parâmetro novo no contrato
- [x] Severidade `needs_review`: concentração é sinal para decisão humana, não violação comprovada
- [x] Custo `O(M + E)` documentado, sem leitura adicional de arquivos
- [x] Criar a fixture `tests/fixtures/projects/concentration` e teste de integração
- [x] `cargo check`, `cargo test` (90 testes: 77 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` passaram
- [x] Smoke na CLI: `concentration` = 3 arestas, 1 revisão, 0 limitações, saída `1`; conversão CLI→JSON→extensão gerou 1 diagnóstico `warning`; `npm test` da extensão passou
- [x] Fixtures `clean`, `dependencies`, `partial`, `typescript` e `python` mantiveram as contagens anteriores

## Critérios de duplicação exata (após a Etapa 10)

- [x] Regra `code.duplicated` no núcleo, sem nova dependência e sem campo novo no contrato
- [x] Coleta de linhas já lidas pela análise, ignorando comentários e declarações de import/export/use
- [x] Limite explícito de 40 caracteres e repetição em mais de um arquivo, com identificador FNV-1a estável
- [x] Fixture `tests/fixtures/projects/duplication` e teste de integração
- [x] Limitações registradas: somente repetição exata de linha, sem semântica ou blocos
- [x] `cargo check`, `cargo test` (92 testes: 79 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` passaram
- [x] Smoke na CLI: `duplication` = 1 aresta, 1 revisão, 0 limitações, saída `1`; conversão CLI→JSON→extensão gerou 1 diagnóstico `warning`

## Critérios de código sem referências (após a Etapa 10)

- [x] Regra `module.unreferenced` no núcleo, sem nova dependência e sem campo novo no contrato
- [x] Aplicação somente a módulos com arquivos incluídos e zero arestas resolvidas incidentes
- [x] Fixture `tests/fixtures/projects/unreferenced` e teste de integração
- [x] Limitações registradas: proxy por módulo, não prova de código morto; módulos sem arquivos continuam com `module.no_files`
- [x] Contagem da fixture `partial` atualizada de 4 para 5 achados pela inclusão legítima de `module.unreferenced:shared`; demais fixtures inalteradas
- [x] `cargo check`, `cargo test` (93 testes: 80 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` passaram
- [x] Smoke na CLI: `unreferenced` = 1 aresta, 1 revisão, 0 limitações, saída `1`; conversão CLI→JSON→extensão gerou 1 diagnóstico `warning`; `npm test` da extensão e `npm run build` da desktop passaram

## Critérios de integração com Git diff (após a Etapa 10)

- [x] Opção `audit <raiz> --diff <arquivo>` consome patch unificado salvo de `git diff`, sem executar o Git
- [x] Parser de diffs no núcleo, sem nova dependência, com linhas adicionadas fundidas por arquivo
- [x] Recorte de achados por arquivo e linha alterada, incluindo achados sem linha e caminhos de módulo
- [x] Resumo recalculado e limitação explícita sobre achados omitidos fora do diff
- [x] Esquema JSON inalterado; desktop e extensão continuam consumindo o mesmo relatório
- [x] Limitações registradas: diffs combinados, caminhos citados com escapes, renomeações puras e linhas removidas
- [x] Correções de revisão: `--- `/`+++ ` dentro de hunk não são confundidos com cabeçalhos; arquivos sem linhas adicionadas não incluem achados com linha
- [x] `cargo check`, `cargo test` (101 testes: 88 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` passaram
- [x] `cargo run -p auditor-cli -- audit --help` mostrou `--diff <ARQUIVO>`; `npm test` da extensão e `npm run build` da desktop passaram
- [x] Smoke com `git diff` real: `dependencies` caiu de 7 para 3 achados no arquivo alterado; CLI→JSON→extensão gerou 3 diagnósticos (1 `error` e 2 `warning`)

## Fechamento da primeira fase

- [x] Decisão registrada: `decisões aprovadas` ficam fora do escopo; sem mudança de schema, contrato ou banco
- [x] Decisão registrada: limite de tamanho de arquivo permanece fora do MVP (D5.2), sem campo novo no contrato
- [x] Decisão registrada: regras personalizadas, sugestões com IA e auditoria agendada ficam fora desta fase; IA no núcleo continua vedada
- [x] `README.md` atualizado com o status da primeira fase e o nome correto do subcomando `init`
- [x] Árvore da seção 4 do plano atualizada com `history/`
- [x] `docs/DECISIONS.md` registra a decisão `R7`; `docs/ROADMAP.md` registra o fechamento
- [x] Verificação final repetida após os ajustes documentais; resultados em `docs/DECISIONS.md` (`R7`)

## Etapa 2 — Confiabilidade do motor (em andamento)

Ordem e critérios vêm de `RELATORIO_ETAPA_1_E_PLANO_ETAPA_2.md`, seção 8. Um item só é marcado como concluído depois que seus critérios de aceite são testados.

### Item 2.1 — Cobertura honesta dos arquivos — Concluído em 25/11/2026

- [x] Diferenciar arquivos inventariados de arquivos de código analisados, sem nota geral de qualidade
- [x] Detectar extensões de código incluídas sem analisador e contá-las por extensão
- [x] Gerar limitação explícita quando houver código incluído sem analisador
- [x] Expor `coverage` no `Analysis` e no `AuditReport`, de forma aditiva no JSON
- [x] Mostrar a cobertura em texto, Markdown, JSON, `analyze` e desktop
- [x] Criar a fixture `tests/fixtures/projects/partial_coverage` e testes de regressão antes da implementação
- [x] Preservar `auditor.toml` v1, o esquema do SQLite e o consumo de `findings` pela extensão
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (105 testes: 92 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram
- [x] Smoke na fixture `partial_coverage`: texto, JSON, Markdown e `analyze` mostram 2 de 4 arquivos de código analisados, `.go (1)` e `.rb (1)` sem analisador, 1 limitação, 0 achados

### Item 2.2 — Eliminar conclusões definitivas baseadas em mapeamentos ambíguos — Concluído em 25/11/2026

- [x] `module_for_file()` não escolhe o primeiro módulo quando há mais de uma correspondência
- [x] Ambiguidade de origem gera apenas revisão, com candidatos ordenados, sem arestas ou declarações `mod` atribuídas
- [x] Seleção ambígua de destino no Python foi substituída por `dep.unresolved` com candidatos ordenados, sem `dep.not_allowed`
- [x] Resultado independe da ordem dos módulos em `auditor.toml`, provado pelas fixtures `ambiguous_mapping_a` e `ambiguous_mapping_b`
- [x] Preservar `auditor.toml` v1, os `rule_id` existentes e o consumo de `findings` pela extensão
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (107 testes: 94 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram
- [x] Smoke nas duas ordens do contrato: saída JSON idêntica, 0 erros, nenhum `dep.not_allowed` nos arquivos ambíguos e revisões com candidatos `domain, overlap` e `domain, entities`

### Item 2.3 — Classificar dependências externas Rust corretamente e resolver `self::`/`super::` — Concluído em 25/11/2026

- [x] Ler `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[target.*.dependencies]` (e equivalentes de dev/build por plataforma) e `[workspace.dependencies]` sem executar o Cargo
- [x] Resolver aliases `package = "..."` e herança `workspace = true`, normalizando nomes para identificadores Rust
- [x] Expor crates declaradas em `WorkspaceInfo.declared_dependencies` e no `analyze`, sem alterar `ReportSummary.packages`
- [x] Imports de crates declaradas não geram `dep.unresolved`; raízes ausentes do contrato e dos manifests continuam em revisão
- [x] `self::`/`super::` entre módulos do contrato viram arestas `Allowed`/`Forbidden`; destinos internos seguem sem aresta; destinos sem evidência viram revisão explícita
- [x] `crate::self`/`crate::super` não viram dependência
- [x] Criar a fixture `tests/fixtures/projects/rust_dependencies` e testes de regressão antes da implementação
- [x] Preservar `auditor.toml` v1, os `rule_id` existentes e o consumo de `findings` pela extensão
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (115 testes: 102 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram
- [x] Smoke na fixture `rust_dependencies`: `analyze` exibiu 8 crates declaradas e nenhuma aresta para elas; `audit` produziu 1 erro, 1 aviso de ciclo e 3 revisões, com `dep.not_allowed` via `super::` e revisão para o destino relativo sem evidência

### Item 2.4 — Tratar `workspace.members` com glob — Concluído em 25/11/2026

- [x] Expandir `*` no último componente de `workspace.members` usando apenas diretórios reais com `Cargo.toml`, sem executar o Cargo
- [x] Aplicar `workspace.exclude` também aos membros expandidos, com ordem estável e deduplicação
- [x] Manter membros literais com o comportamento anterior
- [x] Converter `**`, `?`, classes, `*` em componente intermediário, `.`/`..` e prefixo ausente ou simbólico em limitação acionável, não erro de leitura
- [x] Reutilizar `matches_pattern` da descoberta e não adicionar dependência externa nem campo ao contrato
- [x] Criar as fixtures `workspace_glob` e `workspace_glob_unsupported` e testes de regressão antes da implementação
- [x] Preservar `auditor.toml` v1, os `rule_id` existentes, o JSON consumido pela extensão e as contagens das fixtures anteriores
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (120 testes: 107 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram
- [x] Smoke nas fixtures: `workspace_glob` descobriu `alpha`, `beta` e `legacy` com 1 aresta permitida, 0 achados e 0 limitações; `workspace_glob_unsupported` produziu 4 limitações e 0 pacotes sem erro de leitura

### Item 2.5 — Corrigir a análise TypeScript/JavaScript — Concluído em 25/11/2026

- [x] Ler o código como documento contínuo e delimitar declarações por profundidade de parênteses, colchetes e chaves
- [x] Reconhecer `import` estático multilinha preservando linha de início e evidência reconstruída
- [x] Diferenciar `import.meta` de declaração de import, sem capturar literais posteriores
- [x] Mascarar comentários, strings, template literals e regex; testar cada caso sem aresta inventada
- [x] Manter `import()`/`require()` com literal estático como aresta e converter template literal ou chamada sem literal em limitação explícita
- [x] Criar a fixture `typescript_scanner` e testes de regressão antes da implementação
- [x] Preservar `auditor.toml` v1, os `rule_id` existentes, o JSON consumido pela extensão e a fixture `typescript` com 8 arestas, 4 achados e 1 limitação
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (126 testes: 113 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram
- [x] Smoke: `typescript` manteve 8 arestas, 4 achados e 1 limitação; `typescript_scanner` produziu 1 aresta multilinha, 0 achados e 3 limitações

### Item 2.6 — Substituir enumeração de ciclos por componentes fortemente conectados — Concluído em 25/11/2026

- [x] Substituir `find_cycles()` por Tarjan iterativo, com pilha explícita e complexidade `O(V + E)`
- [x] Emitir um achado `dep.cycle` por componente cíclico, com `id` determinístico formado pelos membros ordenados
- [x] Indexar arestas por componente em uma única passagem, ordenar as internas e limitar a evidência a 8 itens com contagem das demais
- [x] Testar grafo grande e denso, muitos componentes, desconectado, acíclico e sem ciclo, além da fixture `dependencies`
- [x] Preservar `rule_id`, `auditor.toml` v1, JSON consumido pela extensão e SQLite; apenas o `id` dos componentes com 3+ módulos mudou
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (129 testes: 116 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram
- [x] Smoke: `dependencies` passou de 7 para 5 achados (2 erros, 1 aviso, 2 revisões), com um `dep.cycle:application->domain->infrastructure` e 5 arestas na evidência; `rust_dependencies` manteve 5 achados e 1 aviso de ciclo; teste denso com 80 módulos e 6400 arestas terminou em menos de 5 s com 1 achado

### Itens seguintes — Não iniciados

- [ ] 2.7 — Expor os motivos de exclusão e a cobertura no relatório
- [ ] 2.8 — Reforçar a verificação automatizada

A cobertura estruturada ainda não é gravada em coluna própria no histórico; a limitação textual continua sendo persistida. Isso será reavaliado no item 2.7, não neste item.
