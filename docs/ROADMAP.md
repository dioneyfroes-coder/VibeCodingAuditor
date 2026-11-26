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

Etapa 2 (confiabilidade do motor): concluída em 26/11/2026 — itens 2.1 a 2.8 verificados localmente no Windows e cobertos pelo workflow de CI, cuja primeira execução remota (`CI Windows` #1, commit `7a07286`) terminou com sucesso.

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

## Etapa 2 — Confiabilidade do motor (concluída em 26/11/2026)

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

### Item 2.7 — Expor os motivos de exclusão e a cobertura no relatório — Concluído em 25/11/2026

- [x] Transportar `Inventory.skipped` para `AuditReport` em um campo aditivo `skip_summary`
- [x] Expor contagem total, contagens intencionais/de cobertura e contagem por motivo com rótulo estável
- [x] Distinguir exclusões intencionais de link simbólico, entrada inacessível e tipo não tratado
- [x] Limitar a saída a 3 exemplos por motivo, com contagem dos demais, em texto, JSON, Markdown e desktop
- [x] Preservar `auditor.toml` v1, `rule_id`, SQLite e o JSON consumido pela extensão; o campo no JSON é aditivo
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Revisão complementar: teste do núcleo com os seis motivos no mesmo construtor do pipeline, validando texto, JSON e Markdown; extração dos textos do resumo para `apps/desktop/src/skipSummary.ts` com 2 testes Node
- [x] Verificação: `cargo fmt --check`, `cargo test --workspace` (133 testes: 120 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes), `npm test` da desktop (2 testes) e `npm run build` da desktop passaram
- [x] Smoke: `simple` mostrou 14 entradas fora do inventário (10 ignoradas por padrão, 1 excluída, 3 fora do `include`), com 3 exemplos e `+7 outros` no maior motivo; `dependencies` manteve 5 achados e `typescript` manteve 8 arestas, 4 achados e 1 limitação

### Item 2.8 — Reforçar a verificação automatizada — Concluído em 26/11/2026

- [x] Criar `.github/workflows/ci.yml` para `windows-latest` em pushes para `main` e nos pull requests
- [x] Executar `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm test` e `npm run build` da desktop e `npm test` da extensão, sempre após `npm ci`
- [x] Declarar `rust-version = 1.98.1` no workspace e fixar Rust `1.98.1`/Node `24.19.0` no CI; documentar pré-requisitos reproduzíveis
- [x] Reforçar o teste de desempenho do SCC com 600 componentes desconectados e limite de 10 s, mantendo o caso denso de 80 módulos/6400 arestas com limite de 5 s
- [x] Adicionar testes de regressão combinados para os scanners TypeScript/JavaScript e Python
- [x] Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers
- [x] Não habilitar o bundle Tauri: `bundle.active` continua `false` e o CI valida frontend e crate, sem gerar instalador
- [x] Verificação local no Windows: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (135 testes: 122 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm ci` nas duas pastas Node, `npm test` e `npm run build` da desktop (2 testes) e `npm test` da extensão (4 testes) passaram
- [x] Smoke repetido nas fixtures anteriores sem mudança de contagens: `simple` com 14 não incluídos, `dependencies` com 5 achados e `typescript` com 8 arestas, 4 achados e 1 limitação
- [x] Acompanhamento remoto: execuções #1 e #2 passaram; o aviso de Node.js 20 nas actions foi corrigido com `@v5` e a execução #3 passou sem ele

### Fechamento da Etapa 2

- [x] Arquivos incluídos sem analisador geram cobertura parcial explícita
- [x] Mapeamentos ambíguos não originam violações definitivas
- [x] Dependências externas Rust declaradas são classificadas corretamente
- [x] `self::` e `super::` não são descartados sem resolução ou revisão explícita
- [x] Workspaces com glob são analisados ou recebem limitação clara e testada
- [x] Imports TypeScript multilinha e `import.meta` têm comportamento coberto por testes
- [x] Detecção de ciclos usa componentes fortemente conectados com testes densos e de muitos componentes
- [x] Motivos de arquivos ignorados aparecem no relatório
- [x] CLI, desktop e extensão continuam consumindo o mesmo modelo de achados
- [x] Suíte Rust completa, fmt e clippy passaram localmente no Windows; o workflow repete isso no Windows automatizado
- [x] Build TypeScript/React e testes da extensão passaram
- [x] README, roadmap e decisões refletem somente o que foi efetivamente validado

A cobertura estruturada e o resumo de motivos ainda não são gravados em colunas próprias no histórico; a limitação textual continua sendo persistida. Isso foi reavaliado no item 2.7 e mantido fora do escopo. A validação do desktop foi feita por tipos, build e testes dos helpers; a janela Tauri não foi inspecionada visualmente. Também não há fixture com symlink, permissão negada ou FIFO reais: esses motivos são exercitados com inventário em memória no construtor real do relatório. O empacotamento de instaladores continua desativado e o workflow não executa `tauri build`. A primeira execução do workflow no GitHub Actions foi observada em 26/11/2026: `CI Windows` #1, commit `7a07286`, `status: completed`, `conclusion: success`, com todas as etapas do job `verify` aprovadas em `windows-latest`.

## Etapa 3 — Validação arquitetural (em andamento; 3.0 a 3.5 e 3.7 concluídas; 3.6 pendente)

### Etapa 3.0 — Baseline reproduzível — Concluída em 26/11/2026

- [x] Alterações locais identificadas e submetidas como `fc2dd04` antes de qualquer mudança de código; branch `etapa-3/validacao-arquitetural` criada, sem push.
- [x] `git pull --ff-only` terminou com `Already up to date.`; versions: `git 2.55.0.windows.4`, `rustc/cargo 1.98.1`, `node v24.19.0` e `npm 11.17.0`.
- [x] Rust: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (135 testes: 122 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`.
- [x] Desktop: `npm ci`, `npm test` (2 testes) e `npm run build` retornaram `0`. Extensão: `npm ci` e `npm test` (4 testes) retornaram `0`; `npm run package` não foi executado no baseline.
- [x] Resultado integral, incluindo os avisos conhecidos do `npm ci` da extensão, em [docs/validation/etapa-3/baseline.md](validation/etapa-3/baseline.md).

### Etapa 3.1.1 — Dependências `path` do workspace — Concluída em 26/11/2026

- [x] Confirmado por leitura e teste que `collect_manifest_dependencies` incluía dependências `path` em `declared_dependencies`, enquanto `classify_use` descartava toda crate declarada: `auditor-core` da CLI e do backend desktop era ignorado silenciosamente.
- [x] Criada a fixture `tests/fixtures/projects/workspace_local_dependencies` com `core-lib`, `cli-app` e `tools-lib`, ordem de membros invertida, alias `core_api` e `path` herdado de `[workspace.dependencies]`; `serde` permanece como dependência externa.
- [x] Testes de regressão escritos antes da implementação: aresta local `cli -> core`, persistência do descarte de externa, revisão para pacote local sem módulo no contrato e independência da ordem dos módulos. Na fase vermelha, dois testes falharam pela lacuna e um por expectativa incorreta sobre `Analysis.modules`, que documentadamente segue a ordem do contrato; esse último foi ajustado para comparar a classificação após ordenar a listagem.
- [x] Implementada a separação entre dependências externas e `path`: resolução lexical dentro da raiz, sem Cargo, sem links e sem saída da raiz; base do `path` herdado é a raiz do workspace; associação ao contrato somente quando o diretório do pacote corresponde a um único módulo por prefixo literal; conflito de alias, pacote fora do workspace e ambiguidade viram revisão explícita.
- [x] O caso sem associação segura é representado pela aresta `Unresolved` já existente no modelo, usada para gerar `dep.unresolved`; nenhuma aresta `Allowed`/`Forbidden` é inventada. `declared_dependencies`, `rule_id`, `auditor.toml` v1, SQLite e o JSON de `findings` não mudaram.
- [x] Smoke revelou que o `path` herdado estava sendo resolvido a partir do manifesto do membro; a base foi corrigida e um teste focado foi adicionado para impedir a regressão.
- [x] Revisão de helpers: `resolve_dependency_directory`, `inherited_dependency_path`, `literal_path_prefix`, `module_is_inside_package`, `resolve_one_local_dependency`, `insert_local_target` e `resolve_local_dependencies` receberam testes diretos ou cobertura pela fixture; nomes e comentários foram revisados.
- [x] Verificação final: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (144 testes: 131 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; o último emitiu apenas o aviso LF/CRLF esperado do OneDrive.
- [x] Smoke na fixture: `analyze` retornou `0` com `cli -> core [permitida]` em `crates/cli/src/main.rs:1`, `cli -> tools [não resolvida]`, `serde` sem aresta e 2 revisões; `audit` retornou `1` por duas revisões (`dep.unresolved:tools` e `file.unmapped`), sem erros ou avisos.

### Etapa 3.1.2 — Contrato de autoauditoria — Concluída em 26/11/2026

- [x] Contrato raiz `auditor.toml` criado a partir da árvore real, com escopo Rust-only: `core` (`crates/auditor-core/src/**`), `cli` (`crates/auditor-cli/src/**`) e `desktop-backend` (`apps/desktop/src-tauri/src/**`).
- [x] `core` tem `allowed_dependencies = []`; `cli` e `desktop-backend` permitem somente `core`, refletindo os manifests e `AGENTS.md`. Cada módulo tem `responsibility` descritiva, sem valor semântico verificável.
- [x] `scan.include` cobre apenas os três diretórios de código Rust de produção; exclusões documentadas no próprio TOML removem fixtures, UI React, extensão, `target` e `node_modules`. A comunicação TypeScript ↔ Tauri e CLI ↔ extensão fica fora do grafo e não é declarada como verificada.
- [x] Contrato submetido como `0a1eac4` antes da correção de `crate::`.

### Etapa 3.1.3 — Autoauditoria e resolução de `crate::` — Concluída em 26/11/2026

- [x] Primeira autoauditoria com o contrato Rust-only expôs 54 achados `dep.unresolved`, dos quais 46 vinham de `use crate::analysis`, `crate::contract` etc. dentro do próprio `core`: o analisador usava apenas o primeiro segmento e confundia submódulos internos com fronteiras arquiteturais.
- [x] Criada a fixture `tests/fixtures/projects/crate_module_resolution` com um módulo que cobre o crate inteiro, imports `crate::analysis::...` internos e um `crate::unknown` que deve continuar não resolvido. Testes de regressão escritos antes da mudança: o teste interno falhou com `exit code 101`; o fallback e os testes de `super::`, `crate::self` e dependência `path` passaram.
- [x] Implementada a resolução de `crate::` pela raiz de fontes inferida do componente `src`, tentando os prefixos mais longos no inventário e associando o arquivo ao contrato. Destino interno não gera aresta; destino de outro módulo gera `permitida`/`proibida`; ambiguidade vira revisão; sem arquivo correspondente, o comportamento anterior do primeiro segmento é preservado. Nenhum `rule_id`, `auditor.toml` v1, SQLite ou JSON de `findings` mudou.
- [x] Reexecução: `analyze .` passou a mostrar 17 arestas (9 `permitidas`, 0 `proibidas`, 8 `não resolvidas`), com `cli -> core` e `desktop-backend -> core` presentes e nenhum `use crate::*` não resolvido. `audit .` terminou com 62 revisões (54 `code.duplicated` e 8 `dep.unresolved`), 0 erros, 0 avisos e 0 limitações; saída `1` é o comportamento normal quando há achados.
- [x] Limitações mantidas e documentadas: 8 `super::` em módulos de teste continuam sem associação; a evidência de alguns `use` agrupados aparece deslocada em uma linha; `code.duplicated` é heurística de linha exata, não prova de dívida. Esses sinais não foram transformados em erro ou em aresta inventada.
- [x] Relatório completo em [docs/validation/etapa-3/self-audit.md](validation/etapa-3/self-audit.md), com inventário, cobertura, arestas, achados, exclusões e avaliação humana dos módulos.
- [x] Verificação final: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (147 testes: 134 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm ci`/`npm test`/`npm run build` da desktop e `npm ci`/`npm test` da extensão retornaram `0`; o `git diff --check` emitiu apenas o aviso LF/CRLF esperado do OneDrive.

### Etapa 3.2 — Identidade dos achados e confiabilidade do histórico — Concluída em 26/11/2026

- [x] Auditoria do código atual confirmou que `dep.not_allowed` e `dep.unresolved` incluíam `edge.line` no `Finding.id`; os demais identificadores já eram estáveis. O `id` era usado no `UNIQUE (run_id, finding_id)` do SQLite, em `history compare` e na mensagem dos diagnósticos da extensão, mas não era reimplementado fora do núcleo.
- [x] Criada a fixture `tests/fixtures/projects/finding_identity` com três variantes: dependência proibida na linha 1, mesmas linhas deslocadas por comentários e evidência alterada com o mesmo vínculo arquitetural. Testes de regressão escritos antes da correção: deslocamento e duplicidade falharam com `exit code 101`; evidência alterada já passava e ficou protegida contra regressão.
- [x] Política aplicada: `id` identifica a ocorrência lógica por `rule_id`, origem, destino e arquivo; linha e evidência continuam no achado como localização e justificativa. Repetições do mesmo vínculo no mesmo arquivo são deduplicadas de forma determinística, mantendo a primeira ocorrência ordenada por `id`, caminho, linha e evidência. Nenhum `rule_id`, campo de `auditor.toml` v1, coluna SQLite ou campo do JSON de `findings` mudou.
- [x] IDs legados com sufixo de linha são normalizados apenas dentro de `history compare`, sem reescrever banco nem alterar o que `history show` devolve. Banco já migrado continua legível; schema futuro continua rejeitado com mensagem clara, conforme teste existente.
- [x] Smoke na CLI: `history record` nas variantes `before` e `after` gravou o mesmo `id` `dep.not_allowed:application:infrastructure:src/lib.rs`, com local atualizado de `src/lib.rs:1` para `src/lib.rs:4`; `history compare 1 2` retornou 0 novos, 0 resolvidos e 2 persistentes.
- [x] Verificação final: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (154 testes: 141 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; o último emitiu apenas o aviso LF/CRLF esperado do OneDrive.

### Etapa 3.3 — Validação em segundo projeto sintético — Concluída em 26/11/2026

- [x] Atendendo ao pedido explícito do usuário, criado o alvo `validation/targets/fake-order-service`, com quatro módulos Rust, manifests próprios, teste, configuração, documentação de intenção e três dependências proibidas plantadas. O contrato foi criado nesse diretório novo; nenhum `auditor.toml` preexistente foi substituído.
- [x] Classificação registrada: `domain` sem dependências, `application` depende de `domain`, `infrastructure` depende de `domain` e `application`, e `webapi` depende de `application`. `shared/legacy.rs`, `web/src/client.ts` e `tools/check.go` ficaram fora dos módulos por decisão explícita, para exercitar revisão e cobertura parcial.
- [x] `scan` retornou `0` com 12 arquivos incluídos e 8 não incluídos, todos intencionais; `analyze` retornou `0` com 6 arestas, das quais 3 proibidas, e 2 revisões; `audit` retornou `1` com 7 achados: 3 erros, 1 aviso e 3 revisões. Os quatro achados arquiteturais plantados foram detectados; não houve falso positivo nem violação plantada não detectada.
- [x] Censo manual dos 7 achados: 4 verdadeiros positivos, 3 revisões corretas, 0 falsos positivos. `11/12` arquivos de código analisados; o único não analisado foi `tools/check.go`, por ausência de analisador `.go`. Ruído `needs_review`: `3/7` (42,9%), todo esperado. Tempos medidos em uma execução cada: `scan` 1063 ms, `analyze` 875 ms, `audit --format json` 863 ms e `audit` texto 873 ms; não é benchmark estatístico.
- [x] Limitações documentadas: cobertura parcial por `.go`, `import()` dinâmico sem aresta, pacotes npm não enumerados, duplicação por linha exata e identidade que não sobrevive à renomeação de arquivo. Nenhuma alteração no motor foi justificada pela evidência; a prioridade seguinte é o contrato v2 para expressar fronteiras de `shared` e frontend, não adicionar parser Go ou adivinhar imports dinâmicos.
- [x] Integridade confirmada: SHA-256 dos 20 arquivos do alvo permaneceu idêntico antes e depois de segunda auditoria somente leitura. Relatório integral em [docs/validation/etapa-3/second-project-synthetic.md](validation/etapa-3/second-project-synthetic.md).
- [x] Ressalva explícita: o alvo é sintético e criado nesta sessão, não um projeto de terceiros. A validação sai da arquitetura do auditor, mas não substitui uma repetição futura em projeto real disponível.

### Etapa 3.4 — Especificação do contrato v2 — Concluída em 26/11/2026

- [x] Proposta escrita em [docs/architecture/CONTRACT_V2_PROPOSAL.md](architecture/CONTRACT_V2_PROPOSAL.md), antes de qualquer alteração de código. Problemas concretos dos dois pilotos: ausência de campo de camada na autoauditoria, dois arquivos `file.unmapped` no alvo sintético por falta de fronteira `shared`/frontend, risco de ampliar permissão globalmente para expressar exceção e paths sobrepostos que hoje só viram revisão em tempo de análise.
- [x] Escopo proposto: Leitura de v1 e v2 com normalização interna, sem reescrever arquivos; `architecture.layers[]` com `name`, `modules` e `allowed_dependencies`; nova regra `dep.layer_not_allowed`; validação de sobreposição de paths e de ciclo entre camadas. `responsibility` permanece descrição para humanos.
- [x] Capacidades avaliadas: camadas incluídas; domínios e exceções adiados por falta de evidência nos pilotos; linguagem genérica de regras, limiares de métrica, migração automática, novos parsers e dependências rejeitados. `module.concentration` continua sendo concentração no grafo, não complexidade ciclomática ou cognitiva.
- [x] Cada campo proposto tem semântica e teste de aceitação definidos na proposta, incluindo precedência de `dep.not_allowed` sobre a regra de camada para não duplicar erro na mesma aresta.
- [x] Usuário revisou e aprovou a proposta em 26/11/2026, com a resposta “Aprovar e iniciar 3.5 (Recommended)”. Somente depois disso os campos v2 foram implementados e documentados no README.

### Etapa 3.5 — Implementação incremental do contrato v2 — Concluída em 26/11/2026

- [x] Parser e validador v2 com `architecture.layers[]` (`name`, `modules`, `allowed_dependencies`) submetidos como `8850a67`; regra `dep.layer_not_allowed` submetida como `38943b2`; normalização entre versões e mensagens de validação com caminho do campo submetidas como `4dc1640`.
- [x] Testes de regressão escritos antes da implementação: schema zero com nome do campo, v1 com camadas, normalização v1/v2, ordem dos campos no TOML, ordem das camadas e mensagens que apontam `architecture.layers[i].name|modules|allowed_dependencies` e `architecture.modules[i].name|path`. Fase vermelha registrada com `exit code 101`.
- [x] `Contract::normalized_layers()` devolve `None` para v1 e `Some(...)` para v2, inclusive lista vazia. V1 com `architecture.layers` é rejeitado; `auditor init` continua gerando v1 em `auditor.example.toml`, sem reescrever contratos existentes.
- [x] `dep.not_allowed` tem precedência sobre `dep.layer_not_allowed` na mesma aresta; aresta não resolvida continua apenas `dep.unresolved`; ausência de mapeamento de camada em tempo de execução vira limitação explícita, sem `error` inventado. Identidade `dep.layer_not_allowed:{origem}:{destino}:{path}`, sem linha.
- [x] Validação v2 cobre paths sobrepostos, camada duplicada, módulo sem camada, módulo em duas camadas, auto-dependência, destino desconhecido, dependência duplicada e ciclo entre camadas.
- [x] Fixture ponta a ponta `tests/fixtures/projects/layer_boundaries`: 5 incluídos, 1 não incluído intencional, 5 arestas, 3 achados (2 erros e 1 revisão); os IDs de camada, de módulo e de não resolvida aparecem em texto, JSON e Markdown. `audit` retorna `1` pela presença de achados.
- [x] Histórico gravou o novo `rule_id` como texto com `schema_version = 2`, sem migração: `record` duas vezes, `show` e `compare` com achado persistente. Desktop e extensão consomem o achado sem regra própria.
- [x] Contratos v2 equivalentes ao contrato raiz e ao alvo sintético da Etapa 3.3 produziram o mesmo resumo, achados, limitações e cobertura; os arquivos originais não foram alterados. Relatório em [docs/validation/etapa-3/contract-v2.md](validation/etapa-3/contract-v2.md).
- [x] Verificação completa: `cargo fmt --all`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (185 testes: 171 no núcleo, 10 na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (5 testes) e `git diff --check` retornaram `0`; o último emitiu apenas o aviso LF/CRLF esperado do OneDrive.

### Etapa 3.6 — Cobertura estruturada no histórico — Pendente

- [ ] Somente se a necessidade de comparar cobertura entre execuções for confirmada; nenhuma migração SQLite foi feita nesta entrega.

### Etapa 3.7 — Validação ponta a ponta do desktop e da extensão — Concluída em 26/11/2026

- [x] Relatório real da CLI sobre `layer_boundaries` capturado em UTF-8 via `--output` (`cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format json --output ...`); saída `1` esperada por haver achados.
- [x] Apresentação do desktop extraída para `apps/desktop/src/reportView.ts` e coberta por 5 testes Node que leem o mesmo JSON: resumo, filtros, opções, cobertura e lista de arestas (7 testes no total com os 2 de exclusões). Fase vermelha registrada com `ERR_MODULE_NOT_FOUND`.
- [x] Extensão ganhou 2 testes de paridade sobre o mesmo JSON (7 testes no total), comparando quantidade, IDs, caminhos, linhas e severidades dos diagnósticos com o relatório da CLI.
- [x] Build do desktop passou após corrigir `TS7053` no retorno de `availableSeverities`; comandos Tauri já eram comparados com o núcleo nos testes Rust, incluindo a regra de camada. `npm run package` da extensão gerou `.vsix` com 7 arquivos, sem fontes de teste.
- [x] Após o relato do usuário sobre a janela, a interface ganhou área de status com o resultado das seis ações, mensagem explícita para histórico vazio, dica do `auditor.toml` na raiz, rolagem até o painel gerado, badges de severidade, legenda do grafo e CSS com estados de botão. Helpers puros em `apps/desktop/src/status.ts` com 5 testes novos; fase vermelha registrada com `ERR_MODULE_NOT_FOUND`. O total de testes do desktop passou para 12.
- [x] Relatório com a comparação e os checklists em [docs/validation/etapa-3/desktop-vscode.md](validation/etapa-3/desktop-vscode.md), incluindo o diagnóstico do erro de contrato ausente e as correções de feedback.
- [x] Checklist visual da janela Tauri confirmado pelo usuário em 26/11/2026: todos os botões executam e o visual ficou mais agradável. Executável release recompilado em `target/release/auditor-desktop.exe` e auditoria do alvo sintético repetida pela CLI (`audit` = `1` com 7 achados; `analyze` = `0` com 6 arestas).
- [x] Checklist da host do VS Code confirmado pelo usuário em 26/11/2026: `.vsix` instalado com `code --install-extension ... --force` (`0`), CLI release compilada (`0`) e `vibeCodingAuditor.cliPath` configurado fora do repositório. Comando executado pela paleta e pelo menu de contexto; 7 diagnósticos no painel Problems com 3 erros, 1 aviso de ciclo e 3 revisões; clique abriu arquivo/linha; barra de status mostrou `VCA: 7 achados`; código `1` da CLI aceito sem erro operacional. O estado da janela registrou o mesmo relatório da CLI (7 achados, 3 erros, 1 aviso, 3 revisões). Etapa 3.7 concluída.
