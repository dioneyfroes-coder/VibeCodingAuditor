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
| 4 (nova fase) | Descoberta automática e Architecture Builder | Concluída em 27/11/2026 — descoberta, Builder, consistência, validação automatizada e validação manual desktop/VS Code confirmadas; ressalvas e UX registrados |
| 5 (nova fase) | Suíte de testes E2E dedicada | Concluída em 27/11/2026 — CLI, extensão, desktop e CI Windows verdes no run `46067832029` |
| 6 (nova fase) | Amadurecimento arquitetural e distribuição | Em andamento — 6.0 baseline registrado em `docs/validation/etapa-6/baseline.md`; 6.1 documentação coerente, 6.2 descoberta multi-ecossistema e 6.3 resolução determinística concluídas em 27/11/2026; 6.4–6.8 planejadas |

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

## Etapa 5 da nova fase — Suíte de testes E2E dedicada

Objetivo: cobrir os fluxos que os testes atuais não exercitam em host real, sem substituir a suíte determinística existente. A Etapa 5 só começa implementação após a 5.0 aprovada; dependências novas só entram no item que as usar.

### 5.0 — Escopo, ferramentas e critérios (detalhado em 27/11/2026)

- [x] Escopo separado dos testes determinísticos:
  - **CLI E2E**: executa `vibe-coding-auditor-cli`/`.cmd` como processo real, com diretório temporário, fixtures e asserções sobre stdout/stderr/código de saída/arquivos gerados. Não chama funções Rust internas.
  - **Desktop E2E**: executa o app Tauri empacotado/dev em janela real, por automação WebDriver. Valida fluxo de usuário; não substitui os testes Node puros de `viewState/reportView/status/builderFlow`.
  - **Extensão E2E**: executa a extensão em host real do VS Code via `@vscode/test-electron`. Valida ativação, comandos, menu/contexto, Problems, status bar e canal de saída.
  - **Fora do escopo 5.x inicial**: macOS/Linux E2E, teste visual pixel-perfect, fuzzing, carga/performance, testes que dependam de rede externa, e reescrita dos testes determinísticos existentes.
- [x] Ferramentas escolhidas e justificativa:

  | Camada | Ferramenta | Justificativa | Impacto |
  | --- | --- | --- | --- |
  | CLI | Node `node:test` + `node:child_process` + `node:fs/os/path` | Já existe no repo, sem dependência nova, roda bem no Windows/CI. | Baixo; reaproveita padrão dos testes da extensão/desktop. |
  | Extensão VS Code | `@vscode/test-electron` + `@vscode/test-cli` (ou runner mínimo com `@vscode/test-electron`) + `mocha`/runner compatível | API oficial para host real do VS Code; cobre ativação, comandos, diagnostics e workspace. `@vscode/test-electron` já é o caminho suportado pelo ecossistema VS Code. | Adiciona devDependencies na extensão; download de VS Code em CI/cache; execução mais lenta que testes puros. |
  | Desktop Tauri | `tauri-driver` + `WebdriverIO` + `@wdio/local-runner` + `@wdio/mocha-framework` + `@wdio/spec-reporter` + `edgedriver/msedgedriver` conforme suporte Windows/WebView2 | Caminho oficial Tauri v2 para Windows/Linux; WebView2 no Windows usa Edge WebDriver. Permite clicar/abrir pasta/validar DOM real. | Maior custo: dependências npm, binário `tauri-driver`/driver compatível, flakiness de timing, necessidade de build do app antes do teste. |
  | Diagnóstico CI | artefatos do GitHub Actions (`actions/upload-artifact`) | Logs, stdout/stderr, screenshots WebDriver e logs do VS Code ajudam depuração sem rerun local cego. | Pequeno no YAML; cuidado com tamanho/retention. |

- [x] Fluxos críticos cobertos por etapa:
  - **5.1 CLI E2E**: `--help`, `--version`, sem args (mensagem de apresentação), `init` em pasta vazia, `audit` em fixture com contrato, `audit` em pasta sem `auditor.toml` (descoberta), `audit --diff`, `history record/list/show/compare`, execução pelo `vibe-coding-auditor-cli.cmd` sem argumentos não deve travar/fechar silenciosamente no contexto de processo (validar saída/código; duplo clique real continua smoke manual).
  - **5.2 Extensão E2E (host real)**: ativação; comando no projeto sem `auditor.toml` gera observações no canal e um único aviso com ação “Ver observações”; projeto com contrato gera diagnósticos no Problems com severidade/código/arquivo; `Ocultar diagnósticos` limpa; `Abrir achado...` navega; ausência da CLI mostra erro orientando `vibeCodingAuditor.cliPath`.
  - **5.3 Desktop E2E (janela real)**: abrir app, selecionar pasta, detectar projeto, auditar/descobrir, trocar projeto limpa resultados, alternar abas sem empilhar painéis, abrir contrato, registrar/atualizar histórico, smoke do layout dark sem fundo branco por CSS computado.
  - **5.4 CI Windows E2E**: job separado ou etapa separada após build; cache de downloads; artefatos em falha; retries limitados apenas para boot de host gráfico/driver, nunca para asserção funcional.
- [x] Riscos e mitigação:
  - **Flakiness**: usar esperas por condição observável, nunca `sleep` fixo grande; timeouts explícitos; retries só no bootstrap do host.
  - **Custo CI**: separar E2E do job determinístico rápido; rodar E2E em `windows-latest`; permitir execução manual `workflow_dispatch` durante estabilização se necessário.
  - **Desktop Tauri**: WebDriver no Windows depende de Edge WebDriver compatível com WebView2; fixar versão/estratégia no 5.3. Se bloqueado, registrar limitação e não fingir cobertura.
  - **Extensão VS Code**: download do VS Code pode falhar; usar cache e versão fixa. Testes devem criar workspace temporário, não poluir o repo.
  - **CLI `.cmd`**: duplo clique é comportamento do shell/Windows Explorer; E2E automatizado valida invocação por processo/`cmd /c`; confirmação de duplo clique permanece checklist manual.
- [x] Critérios de aceite da 5.0:
  - Decisão E5.0 registrada em `docs/DECISIONS.md`.
  - Roadmap detalha 5.1–5.4, ferramentas, riscos, fora de escopo e artefatos.
  - Nenhuma dependência nova adicionada nesta etapa de planejamento.
  - Suíte determinística atual continua verde (não alterada).

### 5.1 — CLI E2E (concluída em 27/11/2026)
- [x] Adicionar harness Node em `tests/e2e/cli/cli.e2e.test.mjs`, usando `node:test` + `child_process`, sem dependência nova.
- [x] Cobrir cenários 5.0/CLI com fixtures/binário reais: ajuda, versão, execução sem argumentos, atalho `.cmd` com argumentos, `init` sem sobrescrever, descoberta JSON, conformidade com achados, `--diff` e histórico.
- [x] Asserções incluem stdout/stderr/código de saída; logs são exibidos pelo runner do Node em falha.
- [x] Verificação: `node --test tests/e2e/cli/cli.e2e.test.mjs` = `0` (8 testes). O harness usa `target/release` quando presente, senão `target/debug`; exige build prévio da CLI.

### 5.2 — Extensão VS Code E2E (concluída em 27/11/2026)
- [x] Adicionar `@vscode/test-electron@3.1.0` (devDependency fixada) e `npm run test:e2e`, isolado do `npm test` puro.
- [x] Runner em `extensions/vscode/src/e2e/runner.ts` ativa a extensão em host real, usa workspace temporário com fixtures reais e cobre: descoberta sem contrato (observações no canal e zero diagnósticos), contrato com diagnósticos no Problems preserving código da regra, `Abrir achado...` com navegação no editor, `Ocultar diagnósticos` e CLI ausente com orientação de `vibeCodingAuditor.cliPath`.
- [x] Expor API mínima de observação (`getOutputLog`, `getLastRunMode`) apenas para testes; nenhuma regra é avaliada na superfície de teste.
- [x] Falhas incluem o conteúdo do canal `Vibe Coding Auditor`; logs do Extension Host são herdados pelo stdout e capturados pelo CI.
- [x] Verificação: `npm --prefix extensions/vscode test` = `0` (18 testes) e `npm --prefix extensions/vscode run test:e2e` = `0` (host VS Code 1.146.0 fixo por `VCA_VSCODE_VERSION`).
- [x] Limitação conhecida: contagem exata de popups (`um único aviso` com ação “Ver observações”) não é observável pela API do host; permanece no checklist manual já validado pelo usuário.

### 5.3 — Desktop Tauri E2E (concluída em 27/11/2026)
- [x] Adicionar WebDriverIO 9.33.1, `@wdio/tauri-service` 1.5.1 e script `npm --prefix apps/desktop run test:e2e`, separado do teste determinístico.
- [x] Cobrir cenários 5.0/Desktop com o executável real `target/release/auditor-desktop.exe`: tema escuro, sem painéis iniciais, detecção/descoberta sem contrato, limpeza ao trocar raiz, auditoria com contrato, abas sem empilhamento, contrato e histórico local.
- [x] Publicar screenshot em `apps/desktop/e2e/artifacts/` quando um teste falha; diretório ignorado pelo Git.
- [x] Usar `driverProvider: 'external'` com `tauri-driver` 2.1.0 instalado via cargo, sem `tauri-plugin-wdio-webdriver` no app; `TAURI_DRIVER_PATH` permite ambientes onde o cargo não pode instalar no PATH.
- [x] Evitar o loop de foco do serviço (plugin de foco ausente no modo externo) e desativar captura de logs que sobrecarregava o driver; selects e inputs são lidos da UI, sem regras reimplementadas.
- [x] Limitação conhecida: o diálogo nativo de pasta é smoke manual; o E2E troca a raiz pelo campo de texto. A instalação WDIO reportou 25 vulnerabilidades em devDependencies transitivas, registradas sem `npm audit fix --force`.
- [x] Verificação: `npm --prefix apps/desktop run test:e2e` = `0` com `TAURI_DRIVER_PATH` apontando para o driver instalado fora do PATH (5 testes).

### 5.4 — CI Windows E2E (concluída em 27/11/2026)
- [x] Adicionar job `e2e` em `.github/workflows/ci.yml`, posterior ao job determinístico `verify`, em `windows-latest`.
- [x] Compilar CLI release e aplicativo Tauri antes das três suítes E2E (CLI, extensão, desktop).
- [x] Cachear host VS Code fixado, `tauri-driver` 2.1.0 e MSEdgeDriver pela versão do WebView2 detectada.
- [x] Instalar o MSEdgeDriver via `scripts/windows/install-msedgedriver.ps1` antes do E2E: extração compatível com Windows PowerShell 5.1 e driver no `GITHUB_PATH`. Isso corrige a falha observada no primeiro run remoto, onde o download interno do `@wdio/tauri-service` usou `Expand-Archive -Force` e uma sobrecarga de `ExtractToDirectory` inexistente no .NET Framework.
- [x] Após o segundo run remoto ainda falhar, o `VCA_EDGEDRIVER_DIR` passa a ser repassado explicitamente ao ambiente do `tauri-driver` pela configuração do WebdriverIO; uma etapa verifica `where msedgedriver.exe` e a versão do driver antes do E2E.
- [x] Após o quarto run remoto, o E2E desktop passa a rodar em integridade média via `scripts/windows/run-deelevated.ps1` (`runas /trustlevel:0x20000`): runners elevados fazem o WebView2 ≥ 150 ignorar `--remote-debugging-port` recebido por `WEBVIEW2_*`/HKCU, que é o canal usado pelo `msedgedriver`.
- [x] Limitar retries a duas tentativas apenas na instalação do driver; asserções funcionais rodam uma vez. Timeout do job: 45 minutos.
- [x] Publicar screenshots/logs em falha por 7 dias com `actions/upload-artifact@v4`.
- [x] Verificação local: YAML interpretado com `js-yaml` (2 jobs, `needs: verify`, 19 passos); `cargo install tauri-driver --version 2.1.0 --locked` = `0` em root temporário; script de instalação do MSEdgeDriver = `0` para as versões 157 e 158 (inclusive reaproveitamento); E2E desktop = `0` (5 testes) com `VCA_EDGEDRIVER_DIR` explícito, execução de integridade média e download automático forçado a falhar; helper propagou códigos `0` e `7`.
- [x] Verificação remota: run `46067832029` (commit `4ceeab2`) com `verify` e `E2E Windows` verdes; E2E desktop = 5 testes com WebView2 157.0.4341.60 em `windows-latest`.

### Fechamento da Etapa 5 (concluída em 27/11/2026)
- [x] Suítes 5.1–5.4 verde localmente e no job `E2E Windows`; nenhuma regra de auditoria foi movida para a interface.
- [x] Limitações mantidas: contagem exata do popup da extensão e diálogo nativo de pasta permanecem smoke manual; vulnerabilidades transitivas em devDependencies registradas, sem `npm audit fix --force`.

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

A cobertura estruturada e o resumo de motivos passaram a ser gravados em tabelas próprias na Etapa 3.6 (schema SQLite v2), sem exemplos de caminho; a limitação textual continua sendo persistida. Isso havia sido reavaliado no item 2.7 e mantido fora de escopo até o usuário confirmar, em 26/11/2026, que comparar cobertura entre execuções históricas é requisito real. A validação do desktop foi feita por tipos, build e testes dos helpers, além da inspeção visual confirmada em `E3.10`. Também não há fixture com symlink, permissão negada ou FIFO reais: esses motivos são exercitados com inventário em memória no construtor real do relatório. O empacotamento de instaladores continua desativado e o workflow não executa `tauri build`. A primeira execução do workflow no GitHub Actions foi observada em 26/11/2026: `CI Windows` #1, commit `7a07286`, `status: completed`, `conclusion: success`, com todas as etapas do job `verify` aprovadas em `windows-latest`.

## Etapa 3 — Validação arquitetural (concluída em 26/11/2026; subetapas 3.0 a 3.7 concluídas)

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
- [x] Limitações mantidas e documentadas naquele momento: 8 `super::` em módulos de teste continuam sem associação; a evidência de alguns `use` agrupados aparece deslocada em uma linha; `code.duplicated` é heurística de linha exata, não prova de dívida. Esses sinais não foram transformados em erro ou em aresta inventada. A limitação de evidência deslocada e a presença de testes no escopo foram substituídas pela correção registrada em `E3.13`.
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

### Etapa 3.6 — Cobertura estruturada no histórico — Concluída em 26/11/2026

- [x] O usuário confirmou em 26/11/2026 que comparar cobertura entre execuções é requisito real, não curiosidade. Proposta com motivo, alternativas e impacto registrada antecipadamente em `docs/architecture/HISTORY_COVERAGE_V2_PROPOSAL.md` e aprovada com “Aprovar e iniciar (Recommended)”.
- [x] Testes de regressão escritos antes da implementação falharam na fase vermelha com `exit code 101` por campos e tabelas inexistentes; a suíte focada terminou com 22 testes aprovados após a implementação (incluindo 2 testes diretos dos helpers de renderização).
- [x] Migração SQLite v2 aplicada em transação única: `audit_run_coverage` (contagens e `partial`), `audit_coverage_unsupported_extensions` (extensão e arquivos) e `audit_coverage_skip_reasons` (posição estável, rótulo, contagem e impacto). Nenhum exemplo de caminho ou conteúdo de arquivo é gravado.
- [x] `migrate()` aplica versões pendentes em ordem e registra cada versão aplicada; bancos v1 são migrados preservando execuções, bancos v2 reabertos sem alteração, versões futuras continuam rejeitadas e uma falha de DDL reverte a versão registrada.
- [x] `history show` exibe cobertura e motivos; `history compare` mostra coberturas, deltas de analisados/não analisados, extensões sem analisador e mudança nos motivos. Execuções anteriores à migração aparecem como `sem dados`, sem números inventados.
- [x] Nenhuma mudança em `auditor.toml`, `rule_id` existente, formato de `findings`, comandos/códigos de saída da CLI, desktop ou extensão. Nenhuma dependência nova.
- [x] Smoke real pela CLI com banco temporário: `record` nas fixtures `partial_coverage` e `clean`, `show 1` e `compare 1 2` retornaram `0` e exibiram os deltas esperados. Verificação completa: `cargo fmt --all`, `cargo fmt --check`, `cargo test --workspace` (193 testes: 179 no núcleo, 10 na CLI e 4 no desktop) e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` retornaram `0`.
- [x] Binários release recompilados com o schema v2: `cargo build --release -p auditor-cli` (`0`, 5.786.112 bytes) e `npm run tauri build` (`0`, 14.241.792 bytes). Smoke com o executável release e migração de um banco v1 real confirmaram `show`/`list`/`compare` com `0` e versões `1, 2` no `schema_migrations`. Relatório integral em [docs/validation/etapa-3/history-coverage.md](validation/etapa-3/history-coverage.md); decisão em `E3.12`.

### Etapa 3.7 — Validação ponta a ponta do desktop e da extensão — Concluída em 26/11/2026

- [x] Relatório real da CLI sobre `layer_boundaries` capturado em UTF-8 via `--output` (`cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format json --output ...`); saída `1` esperada por haver achados.
- [x] Apresentação do desktop extraída para `apps/desktop/src/reportView.ts` e coberta por 5 testes Node que leem o mesmo JSON: resumo, filtros, opções, cobertura e lista de arestas (7 testes no total com os 2 de exclusões). Fase vermelha registrada com `ERR_MODULE_NOT_FOUND`.
- [x] Extensão ganhou 2 testes de paridade sobre o mesmo JSON (7 testes no total), comparando quantidade, IDs, caminhos, linhas e severidades dos diagnósticos com o relatório da CLI.
- [x] Build do desktop passou após corrigir `TS7053` no retorno de `availableSeverities`; comandos Tauri já eram comparados com o núcleo nos testes Rust, incluindo a regra de camada. `npm run package` da extensão gerou `.vsix` com 7 arquivos, sem fontes de teste.
- [x] Após o relato do usuário sobre a janela, a interface ganhou área de status com o resultado das seis ações, mensagem explícita para histórico vazio, dica do `auditor.toml` na raiz, rolagem até o painel gerado, badges de severidade, legenda do grafo e CSS com estados de botão. Helpers puros em `apps/desktop/src/status.ts` com 5 testes novos; fase vermelha registrada com `ERR_MODULE_NOT_FOUND`. O total de testes do desktop passou para 12.
- [x] Relatório com a comparação e os checklists em [docs/validation/etapa-3/desktop-vscode.md](validation/etapa-3/desktop-vscode.md), incluindo o diagnóstico do erro de contrato ausente e as correções de feedback.
- [x] Checklist visual da janela Tauri confirmado pelo usuário em 26/11/2026: todos os botões executam e o visual ficou mais agradável. Executável release recompilado em `target/release/auditor-desktop.exe` e auditoria do alvo sintético repetida pela CLI (`audit` = `1` com 7 achados; `analyze` = `0` com 6 arestas).
- [x] Checklist da host do VS Code confirmado pelo usuário em 26/11/2026: `.vsix` instalado com `code --install-extension ... --force` (`0`), CLI release compilada (`0`) e `vibeCodingAuditor.cliPath` configurado fora do repositório. Comando executado pela paleta e pelo menu de contexto; 7 diagnósticos no painel Problems com 3 erros, 1 aviso de ciclo e 3 revisões; clique abriu arquivo/linha; barra de status mostrou `VCA: 7 achados`; código `1` da CLI aceito sem erro operacional. O estado da janela registrou o mesmo relatório da CLI (7 achados, 3 erros, 1 aviso, 3 revisões). Etapa 3.7 concluída.

### Fechamento da Etapa 3

- [x] Todas as subetapas 3.0 a 3.7 estão concluídas. A Etapa 3 é declarada concluída em 26/11/2026 com base nas evidências registradas nas seções anteriores e nas decisões `E3.1` a `E3.12`.
- [x] Limitações que permanecem explícitas: a inspeção visual do desktop e da extensão dependeu da confirmação manual do usuário, sem captura de tela; não há fixture com symlink, permissão negada ou FIFO reais; empacotamento de instaladores segue desativado; o histórico cresce sem comando de limpeza; a comparação de cobertura não valida se as duas execuções pertencem ao mesmo projeto e continua disponível apenas na CLI.
- [x] Revalidação do baseline no estado final (`e54d233`), a pedido do usuário: mesmas versões de ferramentas, `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` (193 testes), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm ci`/`npm test`/`npm run build` do desktop (12 testes) e `npm ci`/`npm test` da extensão (7 testes), todos com código `0`. Relatório em [docs/validation/etapa-3/baseline-revalidacao.md](validation/etapa-3/baseline-revalidacao.md); nenhum código alterado.

### Correção da autoauditoria — concluída em 26/11/2026 (após o fechamento da Etapa 3)

- [x] Pedido do usuário para que o Vibe Coding Auditor fiscalize corretamente a si próprio. Diagnóstico: `audit .` devolvia 71 achados, todos `needs_review`, com 7 linhas/evidências deslocadas por causa do cursor textual de `use` e 8 `dep.unresolved` originados de `#[cfg(test)]`, fora do escopo de produção declarado em `auditor.toml`.
- [x] Incremento A (`1a42a97`): `use` e `mod` passaram a usar spans reais do `proc_macro2` com `span-locations`; teste de regressão em `tests/fixtures/projects/source_lines` falhou antes com `Some(2)` no lugar de `Some(6)`.
- [x] Incremento B (`14ed6e8`): módulos inline `#[cfg(test)]` deixaram de produzir arestas, declarações `mod` e duplicações; teste em `tests/fixtures/projects/production_scope` falhou antes por manter `use domain::Other;` de teste.
- [x] Incremento C (`04c976a`): caminhos qualificados em expressões e tipos passaram a ser classificados quando a raiz é `crate`, `self`, `super`, módulo do contrato ou dependência local declarada. Sem esse incremento, excluir testes revelou falso negativo (`module.unreferenced:cli`) porque o CLI usa `auditor_core::...` sem `use`. Teste em `tests/fixtures/projects/qualified_paths` falhou antes por ausência da aresta. Uma versão intermediária sem filtro de raiz gerou 444 revisões falsas e foi corrigida antes do commit.
- [x] Estado final: `analyze .` = `0` com 22 arestas, 0 revisões, 15/15 arquivos analisados e 0 limitações; `audit .` = `1` com 32 achados, todos `code.duplicated`, sem erros, avisos ou informações. Nenhum `rule_id`, `auditor.toml`, formato de `findings`, comando da CLI, desktop, extensão ou schema SQLite mudou; nenhum pacote novo foi instalado.
- [x] Verificação final: `cargo fmt --all`, `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` (194 testes: 180 no núcleo, 10 na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm --prefix apps/desktop test` (12), `npm --prefix apps/desktop run build` e `npm --prefix extensions/vscode test` (7) retornaram `0`. Decisão em `E3.13`; relatório em [docs/validation/etapa-3/self-audit-correcao.md](validation/etapa-3/self-audit-correcao.md).

## Etapa 4 — Descoberta automática e Architecture Builder

Nova fase iniciada em 26/11/2026 a partir de `PLANO_ETAPA_4_DESCOBERTA_E_ARCHITECTURE_BUILDER.md`. O número 4 é reutilizado para a nova fase; a Etapa 4 original da primeira fase permanece registrada na tabela e nos critérios acima como histórico. Regra de andamento: uma subetapa por vez, com aprovação explícita antes de implementar.

### Etapa 4.0 — Baseline e proposta técnica — Concluída em 26/11/2026

- [x] Estado real conferido: `main` em `84192bf`, duas mudanças locais do usuário preservadas (`PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md` deletado e `PLANO_ETAPA_4_DESCOBERTA_E_ARCHITECTURE_BUILDER.md` sem tracking). Nada foi restaurado, commitado ou sobrescrito.
- [x] Documentos e scripts lidos: `AGENTS.md`, `README.md`, `docs/ROADMAP.md`, `docs/DECISIONS.md`, plano da Etapa 4, `package.json` da desktop e da extensão, `.github/workflows/ci.yml` e pontos públicos do núcleo, CLI, Tauri, React e extensão.
- [x] Baseline verde registrado: `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` (194 testes), Clippy com e sem `--config 'build.incremental=false'`, `npm ci`/`npm test`/`npm run build` da desktop (12 testes), `npm ci`/`npm test`/`npm run package` da extensão (7 testes) e `git diff --check`, todos com código `0`. Relatório em [docs/validation/etapa-4/baseline.md](validation/etapa-4/baseline.md).
- [x] Decisão `E4.0` registrada antes de qualquer mudança arquitetural: contrato opcional, modos separados, TOML inválido como erro, descoberta sem falsas violações, nenhuma aprovação automática de dependências, perfis limitados ao schema suportado e núcleo como fonte única da verdade. A implementação depende de aprovação.
- [x] Proposta técnica escrita em [docs/architecture/DISCOVERY_MODE_PROPOSAL.md](architecture/DISCOVERY_MODE_PROPOSAL.md), incluindo APIs afetadas, formato de descoberta, agrupamento de candidatos, compatibilidade CLI/JSON/SQLite, mudanças previstas no desktop e riscos.

### Etapa 4.1 — Descoberta sem contrato no núcleo — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, a implementação ficou restrita ao núcleo; CLI, Tauri, React, extensão, SQLite e contratos não foram alterados em comportamento.
- [x] `ScanOptions` separa escopo de varredura de regras arquiteturais. `discover()` continua usando `scan.include`/`scan.exclude` do contrato; `discover_without_contract()` usa `DEFAULT_DISCOVERY_INCLUDE` (`.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`) e nenhuma exclusão adicional. `DEFAULT_IGNORED_DIRS` e `SkippedEntry` permanecem os mesmos.
- [x] `DependencyKind::Observed` representa relação vista no código sem classificação. `analyze_observations()` roda sem módulos declarados e não produz `reviews`, arestas `Allowed`/`Forbidden` nem aprovação automática. `analyze()` preserva o caminho de conformidade.
- [x] `project_run()` escolhe o modo pela presença de `auditor.toml`: `ProjectRun::Discovery` sem contrato, `ProjectRun::Compliance` com contrato válido. TOML existente e inválido retorna `AuditError::Contract`; não há fallback silencioso. `ExecutionMode` e `ContractStatus` tornam o modo explícito no modelo e no JSON.
- [x] Relações observadas guardam arquivo, linha, evidência e detalhe. Rust registra `use` e caminhos qualificados; TypeScript/JavaScript e Python registram especificadores estáticos lidos pelos mascaradores existentes, com limitações para especificadores dinâmicos. `skip_summary` e `coverage` continuam presentes no inventário e na análise.
- [x] Testes escritos antes da implementação falharam na fase vermelha por funções e tipos inexistentes. A primeira suíte completa após a implementação encontrou uma regressão em `ambiguous_module_becomes_review_without_definitive_mapping`: as novas ramificações de observação também rodavam com contrato válido. Elas foram restritas a `contract.is_none()` e a inclusão de `workspace_names` em `is_root` foi revertida; a suíte passou depois disso.
- [x] Verificação final: `cargo fmt --all` = `0`, `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (202 testes: 188 no núcleo, 10 na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (apenas avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (12), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (7).
- [x] Limitações explícitas: ainda não há módulos candidatos nem agrupamento por evidência (4.2); os comandos da CLI e o contrato do relatório não expõem o novo modo (4.3); `auditor init` continua gerando v1 (risco já registrado na proposta); descoberta cobre somente os analisadores existentes e não resolve pacotes externos.

### Etapa 4.2 — Descoberta de módulos candidatos e relações — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, a implementação ficou restrita ao núcleo. CLI, Tauri, React, extensão, SQLite e contratos não mudaram em comportamento; nenhuma dependência nova foi adicionada.
- [x] `crates/auditor-core/src/discovery/candidates.rs` agrupa candidatos por evidência: `direct` (pacote de manifesto Cargo reconhecido), `inferred_path` (diretório, apenas heurística) e `ambiguous` (conflito preservado). Todo candidato tem `needs_confirmation: true`, nenhum é decisão aprovada.
- [x] Pacotes/manifests são priorizados: limites `direct` são criados antes dos heurísticos, e arquivos já cobertos por um limite `direct` não geram candidato por diretório. Nomes de pacotes do workspace são excluídos da lista de dependências externas antes da classificação.
- [x] Relações observadas reusam arquivo, linha e evidência das arestas existentes e são classificadas sem regras de conformidade em `resolved`, `unresolved`, `external` e `ambiguous`. Ambiguidades viram itens explícitos com descrição e opções distinguíveis pelo caminho.
- [x] Ordenação determinística com `BTreeMap`/`BTreeSet` e ordenação explícita de candidatos, arestas, ambiguidades e limitações. O teste `candidate_building_is_deterministic` falhou na fase vermelha por evidências vindas de `HashSet`; a troca por `BTreeSet` corrigiu a causa.
- [x] `DiscoveryReport` expõe `candidates`, com `candidate_modules`, `observed_dependencies`, `ambiguities`, `languages` e `limitations`. `languages` informa a extensão, a quantidade de arquivos e o analisador disponível; limitações registram arquivos na raiz sem candidato, escopo padrão de `scan.include` e manifests `package.json`/`pyproject.toml`/`go.mod` ainda não interpretados.
- [x] Fixtures novas sem `auditor.toml`: `tests/fixtures/projects/discovery_candidates` (workspace Cargo, código TypeScript e `package.json`) e `tests/fixtures/projects/discovery_candidates_ambiguous` (dois pacotes com o mesmo nome). Nenhuma foi gravada sobre contrato existente.
- [x] Verificação final: `cargo fmt --all` = `0`, `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (211 testes: 197 no núcleo, 10 na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (apenas avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (12), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (7). Testes focados: `cargo test -p auditor-core --lib discovery::candidates` = `0` (8), `cargo test -p auditor-core --lib project_run` = `0` (5) e `cargo test -p auditor-core --lib discovery_report_exposes_candidates_and_analyzer_limits` = `0` (1).
- [x] Limitações explícitas: a descoberta ainda não lê `package.json`, `pyproject.toml` nem `go.mod` para extrair candidatos; não resolve pacotes externos nem usa analisadores além de Rust, TypeScript/JavaScript e Python; a CLI, o texto/Markdown, o histórico e a interface ainda não mostram o modo nem os candidatos (4.3); `auditor init` continua gerando v1 (risco já registrado na proposta).

### Etapa 4.3 — CLI e contrato do relatório — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, as mudanças ficaram restritas ao núcleo, à CLI e à documentação. Tauri/React, extensão VS Code, SQLite e contratos não mudaram em comportamento; nenhuma dependência nova foi adicionada.
- [x] Seleção de modo automática pela presença de `auditor.toml` em `scan_run()`, `analyze_run()` e `project_run()`. TOML existente e inválido continua retornando `AuditError::Contract`, sem fallback para descoberta.
- [x] `scan` e `analyze` funcionam sem contrato e exibem `Modo: descoberta (sem contrato)`. `analyze` em descoberta mostra arestas `observada`, sem rótulos de conformidade, além de candidatos, relações classificadas por evidência, ambiguidades e limitações.
- [x] `audit` sem contrato renderiza descoberta nos três formatos, com `mode: "discovery"`, `contract.status: "missing"`, `findings: []`, `skip_summary` e cobertura. Com contrato, `AuditReport` ganha apenas os campos aditivos `mode: "compliance"` e `contract` com `status` e `schema_version`; campos antigos permanecem.
- [x] Texto e Markdown indicam o modo: `Modo: conformidade (contrato vN)` ou `Modo: descoberta (sem contrato)`. `ProjectRun` passou a serializar sem tag externa porque cada ramo já traz `mode`; um teste garante uma única chave `mode` no JSON.
- [x] `--diff` em descoberta valida o patch, não filtra achados e registra a limitação `--diff não filtrou achados: descoberta não produz achados de conformidade`. Em conformidade, o comportamento anterior foi preservado.
- [x] Códigos de saída documentados no README: `0` para sucesso inclusive descoberta com observações, `1` para conformidade com achados e `2` para falha operacional ou contrato inválido. Ausência de TOML não produz `2`; `history record` sem contrato falha com `2` e não grava descoberta.
- [x] Testes escritos antes da implementação: renderização/serialização no núcleo e cinco testes de integração que executam o binário real. A fase vermelha falhou por métodos e campos inexistentes. Durante a implementação, o Clippy apontou `large_enum_variant` em `ProjectRun` (536 bytes contra 280), resolvido com exceção justificada no código, e um teste revelou `mode` duplicado no JSON, corrigido com `#[serde(untagged)]` e o campo `mode` em `ComplianceRun`.
- [x] Verificação final: `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (220 testes: 201 no núcleo, 10 unitários na CLI, 5 de integração na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (apenas avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (12), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (7).
- [x] Limitações explícitas: desktop, extensão e histórico ainda não mostram modo nem candidatos (Etapas 4.5 e 4.7); `init` continua gerando v1; os candidatos ainda não leem `package.json`, `pyproject.toml` nem `go.mod`; `--diff` em descoberta apenas valida o patch; `ProjectRun` mantém uma exceção documentada do Clippy para variantes grandes.

### Etapa 4.4 — Modelo do Architecture Builder — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, as mudanças ficaram restritas ao núcleo e à documentação. Nenhuma dependência Rust ou npm foi adicionada; Tauri, React, CLI, extensão, SQLite e contratos de exemplo não mudaram em comportamento.
- [x] Novo módulo `builder` no núcleo, sem dependência de Tauri, com `ContractDraft`, `DraftModule`, `DraftLayer`, `PendingDecision` e `AmbiguityResolution`. O rascunho cobre nome do projeto, `include`/`exclude`, estilo, módulos com um `path`, responsabilidade descritiva, dependências permitidas e rejeitadas, camadas e decisões pendentes.
- [x] `empty_draft()` atende projeto novo ou vazio: sem módulos, sem camadas e sem pendências. `draft_from_discovery()` importa candidatos como sugestões editáveis (`needs_confirmation`), preserva evidências e cria decisões para cada relação observada e ambiguidade, sem aprovar dependências.
- [x] `validate_draft()` usa os fatos da descoberta como fonte de verdade: módulos precisam ser confirmados, ambiguidades e relações sem módulo de origem único exigem escolha registrada, e cada relação observada exige `allowed_dependencies` ou `rejected_dependencies`. Limpar `pending_decisions` não burla a validação.
- [x] Perfis: monólito modular sem camadas e em camadas. O perfil em camadas exige exatamente uma camada por módulo; o perfil sem camadas rejeita `layers`. Sobreposição de paths, nomes duplicados e demais regras estruturais continuam sendo fiscalizadas pelo validador oficial do contrato.
- [x] Novas configurações usam `schema_version = 2` por padrão. `render_contract()` deriva `Serialize` dos tipos existentes e produz TOML determinístico; `contract_identity()` gera hash estável de 16 dígitos da forma canônica. `validate_draft()` valida o TOML gerado com `parse_contract()`, garantindo round-trip rascunho → TOML → contrato equivalente. Nenhum campo exclusivo do rascunho (`needs_confirmation`, `rejected_dependencies`, `pending_decisions`, `evidence`) é gravado no contrato.
- [x] Projetos vazios e paths planejados ainda inexistentes são válidos: o validador do contrato já aceita módulos sem arquivos no inventário, e o Builder não exige que o path exista no disco.
- [x] Testes escritos antes da implementação falharam na fase vermelha por tipos e funções inexistentes. Durante a revisão, o Clippy apontou import não usado, `if` aninhável e comparação de tamanho; a validação foi reforçada para não depender da lista de pendências.
- [x] Limitações explícitas: não há interface, backend Tauri ou comando CLI para o Builder nesta etapa (4.5); `auditor init` continua gerando v1; `responsibility` é texto livre, sem interpretação por regras; um módulo aceita apenas um `path`; não há resolução automática de ambiguidades nem leitura de `package.json`, `pyproject.toml` ou `go.mod`.

### Etapa 4.5 — Interface visual do Builder — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, as mudanças ficaram restritas ao núcleo, ao adaptador Tauri e à interface desktop. CLI, extensão VS Code, SQLite e contratos de exemplo não mudaram em comportamento; nenhuma dependência Rust ou npm foi adicionada.
- [x] O núcleo expõe `validate_draft_result()`, `draft_effects()` e `draft_from_contract()`. `PendingDecisionKind::DecideDependency` passa a informar o `module` já identificado pelo núcleo, para que a interface não reimplemente correspondência de globs. O rascunho e as decisões passam a derivar `Serialize`/`Deserialize` com o formato marcado externamente; `ContractDraft` preserva o perfil `snake_case`.
- [x] Novos comandos Tauri finos e somente leitura: `load_project`, `discover_project`, `start_draft`, `validate_contract_draft`, `preview_contract` e `draft_effects`. Todos delegam ao núcleo; não há comando de gravação nesta etapa.
- [x] “Detectar projeto” distingue `modo: descoberta (sem contrato)` de `modo: conformidade (contrato vN)` e habilita “Definir arquitetura” ou “Revisar arquitetura”. A detecção inclui TOML inválido como erro explícito, sem fallback silencioso.
- [x] O assistente cobre sete etapas fixas: descoberta, escopo, organização, módulos, camadas, dependências e revisão. Voltar e avançar preserva o rascunho; qualquer edição invalida validação, prévia e efeitos anteriores.
- [x] A descoberta mostra candidatos com categoria, path e evidências limitadas, relações observadas com arquivo/linha/evidência, cobertura por linguagem/analisador, ambiguidades e limitações. Estados vazio e de ausência de analisador são explícitos.
- [x] Módulos podem ser confirmados, renomeados (com atualização de referências em dependências, camadas e escolhas), adicionados e removidos (com limpeza de referências órfãs). Camadas podem ser criadas, renomeadas e receber módulos; trocar de perfil ajusta as camadas ao schema.
- [x] Cada relação observada exige decisão humana: origem única vem do núcleo; origem ambígua exige escolha do módulo antes de permitir ou rejeitar. Nenhuma dependência observada é aprovada automaticamente, e o aviso aparece na descoberta e nas dependências.
- [x] A revisão mostra validação feita pelo parser oficial, prévia TOML determinística do núcleo, identidade do contrato quando válido, contagens de módulos/camadas/dependências permitidas e identificadores das regras que passam a valer. Nenhum arquivo é alterado; a gravação fica para a Etapa 4.6.
- [x] Estados explícitos: carregamento, falha operacional, ausência de candidatos/relações, decisões pendentes, rascunho inválido e rascunho válido com prévia. Helpers puros (`builderFlow.ts`) concentram navegação, edição e formatação; regras, validação e serialização continuam apenas no núcleo.
- [x] Testes escritos antes da implementação falharam na fase vermelha por funções/types inexistentes e pelo campo `module` ausente em `DecideDependency`. Durante a revisão, um teste revelou que renomear módulo deixava referências órfãs e que o painel de dependências não conseguia decidir origem ambígua sem duplicar globs; ambos foram corrigidos no núcleo e nos helpers puros.
- [x] Verificação final: `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (247 testes: 221 no núcleo, 10 unitários na CLI, 5 de integração na CLI e 11 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (somente avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (28), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (7). Testes focados: `cargo test -p auditor-core --lib builder::tests` = `0` (18) e `cargo test -p auditor-desktop --lib` = `0` (11).
- [x] Limitações explícitas: não há gravação, confirmação final nem segunda auditoria nesta etapa; o agrupamento automático em camadas cria uma camada inicial com todos os módulos, sem inferir responsabilidades; `auditor init` continua gerando v1; a descoberta ainda não lê `package.json`, `pyproject.toml` ou `go.mod` para candidatos; a interface ainda não mostra modo/candidatos na extensão nem no histórico (Etapa 4.7).

### Etapa 4.6 — Gravação, validação final e segunda auditoria — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, as mudanças ficaram restritas ao núcleo, ao adaptador Tauri e à interface desktop. CLI, extensão VS Code, SQLite e contratos de exemplo não mudaram em comportamento; nenhuma dependência Rust ou npm foi adicionada.
- [x] O núcleo passa a expor `SaveAuthorization`, `SaveOutcome`, `SaveError`, `contract_diff()` e `save_draft()`. A gravação revalida o rascunho, renderiza a prévia, reparseia o texto, usa arquivo temporário no mesmo diretório com `create_new`, `sync_all` e `fs::rename`, reabre o arquivo pelo parser oficial e executa `project_run()`, que agora termina em modo de conformidade.
- [x] Criação exige `Create` e falha se `auditor.toml` existir. Substituição exige `Replace { expected_text }` conferido com o texto lido e relido antes da renomeação; aprovação obsoleta é recusada sem tocar o arquivo anterior. Falha ao criar o temporário remove o que foi criado e preserva o contrato anterior.
- [x] `contract_diff()` gera comparação determinística linha a linha no núcleo, sem crate novo. O comando Tauri `contract_diff` devolve também o texto atual e a indicação de existência, usados pelo assistente para mostrar o diff e construir a autorização conferida.
- [x] Novos comandos Tauri finos: `contract_diff` e `save_contract`. O segundo delega integralmente a `builder::save_draft()` e devolve o contrato gravado, a identidade e a auditoria de conformidade; nenhuma regra foi movida para o adaptador.
- [x] A revisão passa a mostrar o diff entre o contrato atual e a prévia, com resumo de linhas adicionadas/removidas. Criar ou substituir exige duas ações separadas: abrir a confirmação com o texto explícito e confirmar a gravação. Qualquer edição posterior invalida diff, confirmação, validação, prévia, efeitos e resultado anterior.
- [x] Depois da gravação, o assistente exibe lado a lado as observações da descoberta (sem julgamento) e as violações/revisões da conformidade (contrato aprovado), com identidade, versão e modo explícitos. A aplicação atualiza a detecção e o relatório externo sem fechar o assistente.
- [x] Testes escritos antes da implementação falharam na fase vermelha por tipos/funções inexistentes no núcleo, comandos ausentes no adaptador e exports ausentes em `builderFlow.ts`. A verificação inclui criação, recusa de sobrescrita, texto obsoleto, rascunho inválido, falha de escrita, reparso, modo de conformidade, serialização da autorização e delegação dos comandos.
- [x] Verificação final: `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (256 testes: 227 no núcleo, 10 unitários na CLI, 5 de integração na CLI e 14 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (somente avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (32), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (7). Testes focados: `cargo test -p auditor-core --lib builder::tests` = `0` (24) e `cargo test -p auditor-desktop --lib` = `0` (14).
- [x] Limitações explícitas: a checagem do texto atual reduz, mas não elimina, a janela entre conferência e renomeação em cenários de escrita concorrente; se a auditoria posterior falhar, o contrato permanece gravado e o erro é exibido, sem rollback silencioso; o diff mostra o arquivo inteiro, sem agrupamento em hunks; não há comando CLI para o Builder, nem mudança em extensão/histórico nesta etapa; `auditor init` continua gerando v1; a descoberta ainda não lê `package.json`, `pyproject.toml` ou `go.mod` para candidatos.

### Etapa 4.7 — Consistência entre CLI, desktop, extensão e histórico — Concluída em 26/11/2026

- [x] Após o usuário responder “prossiga”, a proposta de migração SQLite foi registrada em `docs/architecture/HISTORY_SCHEMA_V3_PROPOSAL.md` antes de qualquer mudança de schema. Nenhuma dependência Rust ou npm foi adicionada.
- [x] Schema do histórico elevado para v3: `execution_mode TEXT NOT NULL DEFAULT 'compliance'` e `contract_identity TEXT`, aplicados por `ALTER TABLE` na mesma transação das migrações pendentes v1/v2. Bancos v1/v2 preservam as linhas; versões superiores a 3 continuam recusadas sem alteração destrutiva.
- [x] Linhas legadas recebem modo `compliance` pela origem histórica, mas identidade desconhecida; nenhum valor é inventado. `RunSummary`, renderização de lista e detalhe exibem modo e identidade sem tratar ausência como contrato válido.
- [x] `History::record_run()` grava qualquer `ProjectRun`. Descoberta gravada com `schema_version = 0`, modo `discovery`, identidade ausente, zero achados e cobertura/limitações/motivos de exclusão do núcleo; conformidade continua com relatório e identidade determinística calculada pelo núcleo.
- [x] `History::compare()` recusa modos diferentes, duas descobertas e conformidades com identidades diferentes ou desconhecidas, com `HistoryError::Incompatible` e mensagem explícita. Somente execuções de conformidade com a mesma identidade conhecida continuam comparáveis.
- [x] CLI e Tauri usam `project_run()` em `history record`/`history_record`; projeto sem `auditor.toml` agora registra descoberta. Testes de integração cobrem o modo explícito na saída e a recusa da comparação entre modos.
- [x] Extensão VS Code discrimina o JSON da CLI por `mode`. Em conformidade, diagnósticos, IDs e severidades permanecem intactos. Em descoberta, diagnósticos antigos são limpos, observações e limitações do núcleo aparecem como mensagens informativas, e a orientação abre o Architecture Builder no desktop. JSON legado sem `mode` continua sendo tratado como conformidade.
- [x] Desktop usa `load_project()` para escolher o painel: relatório de conformidade ou observações de descoberta. Histórico exibe modo e identidade; o status da gravação informa o modo devolvido pelo núcleo.
- [x] Testes escritos antes da implementação falharam na fase vermelha por campos/métodos ausentes no núcleo, comandos/adaptadores ainda presos a `audit_project()` e exports ausentes em `model.ts`. Durante a revisão, um teste de migração legada precisou abrir o banco após a conversão para atualizar a identidade conhecida usada apenas naquele cenário de cobertura.
- [x] Verificação final: `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (265 testes: 234 no núcleo, 10 unitários na CLI, 6 de integração na CLI e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (somente avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (33), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (12). Testes focados: `cargo test -p auditor-core --lib history::tests` = `0` (29), `cargo test -p auditor-cli --test cli history_` = `0` (2) e `cargo test -p auditor-desktop --lib` = `0` (15).
- [x] Limitações explícitas: comparação entre duas descobertas é recusada; execuções legadas com identidade desconhecida não são comparáveis; o modo legado `compliance` decorre da origem histórica do registro; a extensão mostra observações como mensagens, sem painel próprio; `auditor init` continua gerando v1; os candidatos ainda não leem `package.json`, `pyproject.toml` ou `go.mod`.

### Etapa 4.8 — Validação automatizada e manual concluídas em 27/11/2026

- [x] Após o usuário responder “prossiga”, dez fixtures foram criadas sob `tests/fixtures/projects/`: `discovery_rust_no_contract`, `discovery_ts_no_contract`, `discovery_python_no_contract`, `discovery_workspace`, `discovery_ambiguous`, `discovery_empty_project`, `builder_existing_project`, `builder_contract_v1`, `builder_contract_v2` e `builder_write_safety`. Nenhuma dependência Rust ou npm foi adicionada.
- [x] `crates/auditor-cli/tests/fixtures.rs` executa a CLI como processo real duas vezes por fixture. Todas as saídas são determinísticas, trazem `mode: discovery`, `status: missing` e `findings: []`, não contêm `dep.not_allowed`/`dep.layer_not_allowed`, e a árvore de arquivos permanece idêntica. Hash SHA-256 agregado foi recalculado externamente antes/depois da auditoria nas sete fixtures de descoberta, com igualdade em todas.
- [x] Workspace com dois pacotes e estrutura ambígua mostram pacotes, relações observadas e ambiguidades sem classificação arquitetural. O diretório vazio usa `.gitkeep` fora do `include`, não gera erro falso e aceita contrato planejado gravado pelo núcleo.
- [x] Contratos v1 e v2 foram lidos e não foram reescritos. A auditoria v1 termina com código `0`; a v2 com camadas preserva 3 achados (2 erros e 1 revisão) e código `1`. TOML inválido continua retornando `2` sem cair em descoberta.
- [x] Segurança do Builder: gravação cria contrato v2 e reabre em conformidade; cancelar não cria arquivo; `Create` recusa sobrescrita; `Replace` exige texto exato; falha na criação do temporário preserva o contrato e não deixa resíduo `.tmp`.
- [x] `README.md` descreve os dois modos, o Architecture Builder, observação/sugestão/violação, a discriminação por `mode` na extensão, o histórico v3 e as comparações recusadas. `AGENTS.md` trata `auditor.toml` como opcional e exige aprovação explícita para gravação. O status da proposta original deixou de dizer que nada foi implementado. O registro detalhado está em `docs/validation/etapa-4/fechamento.md`.
- [x] Verificação final: `cargo fmt --all` = `0`, `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (275 testes: 239 no núcleo, 10 unitários na CLI, 6 de integração na CLI, 5 de fixtures na CLI e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `cargo test -p auditor-core --lib builder::tests` = `0` (29), `cargo test -p auditor-cli --test fixtures` = `0` (5), `npm --prefix apps/desktop ci` = `0`, `npm --prefix apps/desktop test` = `0` (33), `npm --prefix apps/desktop run build` = `0`, `npm --prefix extensions/vscode ci` = `0`, `npm --prefix extensions/vscode test` = `0` (12), `npm --prefix extensions/vscode run package` = `0` e `git diff --check` = `0` (somente avisos LF/CRLF do OneDrive). Smoke tests: `scan`/`analyze`/`audit` em `discovery_rust_no_contract` = `0`, `audit clean --format json` = `0`, `audit layer_boundaries --format json` = `1`, TOML inválido = `2`; `history record` sem contrato = `0`, `history list` = `0` e `history compare` entre modos diferentes = `2` com mensagem explícita. Fase vermelha documentada em `docs/DECISIONS.md` (`E4.8`).
- [x] Limitações explícitas: candidatos ainda não leem `package.json`, `pyproject.toml` ou `go.mod`; pacotes externos e `tsconfig.paths` não são resolvidos; `discovery_empty_project` existe com `.gitkeep` fora do include; fixtures v1/v2 são cópias de casos já validados, não projetos de terceiros.
- [x] Validação manual do fluxo completo no desktop — confirmada pelo usuário em 27/11/2026 com o próprio VibeCodingAuditor e o projeto BackBet. Ele examinou os fluxos e relatou que funcionaram, com ressalvas de apresentação (excesso de informações, rolagem longa, resíduos ao trocar de projeto e linguagem técnica), tratadas na Etapa 4.9.
- [x] Validação da extensão na host real do VS Code — confirmada pelo usuário em 27/11/2026 após reinstalar `extensions/vscode/vibe-coding-auditor-0.1.0.vsix` e atualizar `vibeCodingAuditor.cliPath` para `vibe-coding-auditor-cli` (o valor antigo apontava para `vibe-coding-auditor.exe`, removido). O usuário confirmou: comando aparece sem `auditor.toml`; descoberta sem violações de conformidade no Problems; barra de status/resumo de descoberta; auditoria com contrato válido exibe diagnósticos no Problems. Ressalva de UX: observações de descoberta são exibidas como múltiplos popups informativos.
- [x] Etapa 4 declarada concluída com ressalvas documentadas, após confirmação explícita das validações manuais desktop e VS Code.

### Ajuste pós-4.9 — Ruído de popups na extensão — 27/11/2026

- [x] Motivo: na validação manual da host real do VS Code, a descoberta sem contrato gerou vários popups informativos. Impacto: apenas `extensions/vscode`; nenhuma regra, schema, contrato, SQLite, CLI Rust ou desktop alterado.
- [x] Descoberta passou a escrever observações/limitações no canal de saída `Vibe Coding Auditor` e mostrar um único popup com ação “Ver observações”. Falhas operacionais também oferecem “Ver detalhes” e registram no canal.
- [x] `npm --prefix extensions/vscode test` = `0` (18) e `npm --prefix extensions/vscode run package` = `0`. Usuário confirmou em 27/11/2026, após reinstalar o `.vsix` e recarregar a janela, que a descoberta mostra um único aviso e as observações no canal de saída `Vibe Coding Auditor`.

### Ajuste pós-4.9 — UI dark e layout horizontal do desktop — 27/11/2026

- [x] Motivo: usuário relatou fundo branco irritante, rolagem vertical excessiva e acúmulo de informação. Impacto: apenas apresentação no `apps/desktop` (HTML/CSS/React); nenhuma regra, contrato, schema, SQLite, CLI, extensão ou dependência alterada.
- [x] Dark mode padrão (`color-scheme: dark`, fundo escuro em `html/body/#root`), workspace mais horizontal, barra lateral sticky, ações em grade, configurações locais recolhíveis, toolbar de resultado com abas no mesmo eixo e rolagem interna do painel de resultado.
- [x] `npm --prefix apps/desktop test` = `0` (39), `npm --prefix apps/desktop run build` = `0`, `npm --prefix apps/desktop run tauri build` = `0`. Executável atualizado em `target/release/auditor-desktop.exe`.

### Ajuste pós-4.8 — Nome da CLI e ajuda sem argumentos — 27/11/2026

- [x] Após o usuário relatar que o executável “não abria”, foi confirmado que ele abriu `vibe-coding-auditor.exe`, uma CLI de terminal que terminava em silêncio sem argumentos. O binário foi renomeado para `vibe-coding-auditor-cli`, no `Cargo.toml`, no nome do Clap, nos testes `CARGO_BIN_EXE_*`, na documentação e no padrão `vibeCodingAuditor.cliPath` da extensão. Os binários antigos de `target/release` e `target/debug` foram removidos após o novo build.
- [x] Sem argumentos, a CLI imprime apresentação, ajuda do Clap e linha em branco, com código `0`. `--help`, `--version`, `init`, `scan`, `analyze`, `audit` e `history` permanecem inalterados. A apresentação informa que a interface gráfica é `auditor-desktop.exe`.
- [x] Dois testes novos cobrem o nome do comando e a saída sem argumentos. A extensão e o README foram atualizados; quem configurou `cliPath` com o nome antigo precisa ajustar a configuração. Nenhuma regra, schema, contrato ou dependência mudou.
- [x] Verificação: `cargo fmt --all` = `0`, `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (277 testes: 239 no núcleo, 11 unitários na CLI, 7 de integração na CLI, 5 de fixtures na CLI e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `npm --prefix apps/desktop test` = `0` (33), `npm --prefix apps/desktop run build` = `0`, `npm --prefix extensions/vscode test` = `0` (12), `npm --prefix extensions/vscode run package` = `0` e `cargo build --release -p auditor-cli` = `0`. Smoke: sem argumentos = `0` com apresentação e `Usage: vibe-coding-auditor-cli [COMMAND]`; `--version` = `0` com `vibe-coding-auditor-cli 0.1.0`; `--help` = `0`.

### Validação manual parcial — 27/11/2026

- [x] O usuário executou manualmente o desktop com o próprio VibeCodingAuditor e o projeto BackBet e confirmou que os fluxos funcionaram. Relatou quatro problemas de apresentação: acúmulo rápido de informações, página enorme com barra de rolagem, resíduos de um projeto ao trocar para outro e linguagem muito técnica.
- [x] A validação da extensão na host real do VS Code foi confirmada pelo usuário em 27/11/2026, após reinstalar o `.vsix` e ajustar `vibeCodingAuditor.cliPath` para `vibe-coding-auditor-cli`. O usuário confirmou os quatro pontos do checklist: comando disponível sem `auditor.toml`, sem violações de conformidade no Problems em descoberta, status/resumo de descoberta e diagnósticos no Problems em projeto com contrato. Observações de descoberta geraram vários popups informativos; registrado como ressalva de UX.

### Etapa 4.9 — Interface mais limpa — 27/11/2026

- [x] Motivo: os relatórios longos e os painéis acumulados tornavam a janela difícil de usar, e a troca de projeto mantinha dados antigos na tela. Alternativas consideradas: apenas documentar o comportamento — não resolveria o problema; criar abas para cada resultado sem limpar o estado — manteria os resíduos; alterar o núcleo para resumir relatórios — violaria o limite de responsabilidade e alteraria contratos de saída. Impacto: mudança restrita ao frontend do desktop e a helpers de apresentação; nenhuma regra, severidade, schema, contrato ou dependência foi alterado.
- [x] Trocar a raiz — pelo diálogo ou pelo campo de texto — agora limpa relatório, descoberta, mapa, contrato, histórico exibido, filtros, mensagens, assistente e estado de expansão. O helper puro `projectViewReset()` fornece o estado inicial e `isProjectChange()` evita limpar ao escolher a mesma pasta; a configuração do banco permanece.
- [x] Os resultados passaram a abas `Descoberta`, `Relatório`, `Mapa`, `Contrato` e `Histórico`: apenas a aba ativa é renderizada. A área de status mostra a última mensagem e deixa as anteriores em um resumo recolhido. Achados e observações são limitados a cinco e seis itens com botão “Ver mais”; todas as listas longas de detalhes, limitações e ligações ficam recolhidas. O contrato exibido tem altura máxima com rolagem própria.
- [x] Linguagem mais simples: “problemas encontrados” no resumo, rótulos amigáveis para as dez regras conhecidas, “ligações” no lugar de “arestas”, histórico sem `identidade` crua e painéis com títulos como “O que foi encontrado” e “Como resolver”. Identificadores, códigos de regra desconhecidos e dados do núcleo permanecem disponíveis nos detalhes, sem reclassificação.
- [x] `apps/desktop/src/viewState.ts` e `apps/desktop/tests/viewState.test.mjs` foram adicionados; `reportView.ts`, `status.ts`, `App.tsx`, `styles.css`, testes existentes, `package.json`, `tsconfig.test.json` e documentação foram ajustados. Nenhuma dependência Rust ou npm foi adicionada.
- [x] Verificação: `cargo fmt --all` = `0`, `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (277 testes: 239 no núcleo, 11 unitários na CLI, 7 de integração na CLI, 5 de fixtures na CLI e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `npm --prefix apps/desktop test` = `0` (39), `npm --prefix apps/desktop run build` = `0`, `npm --prefix extensions/vscode test` = `0` (12), `npm --prefix extensions/vscode run package` = `0`, `npm --prefix apps/desktop run tauri build` = `0` e `git diff --check` = `0`. Smoke: `auditor-desktop.exe` iniciou com `RESPONDING=True` e título `Vibe Coding Auditor`.

### Ajuste pós-4.9 — CLI clicável e extensão sem contrato — 27/11/2026

- [x] Motivo: o usuário tentou abrir `vibe-coding-auditor-cli.exe` por duplo clique e a janela fechou imediatamente; também tentou usar a extensão em um projeto antigo sem `auditor.toml` e o comando não aparecia. Alternativas consideradas: manter a CLI bloqueando ao final de qualquer execução — atrapalharia automação e testes; pedir que o usuário configurasse `cliPath` manualmente — não resolve o caso comum; manter a exigência de `auditor.toml` — contradiz o modo de descoberta da Etapa 4. Impacto: atalho `.cmd` só na raiz do repositório, ajustes no manifesto/estado da extensão e documentação; nenhuma regra, schema, contrato, SQLite ou núcleo mudou.
- [x] `vibe-coding-auditor-cli.cmd` procura `target/release` e `target/debug`, executa a CLI com os argumentos recebidos e pausa apenas quando é chamado sem argumentos. Chamadas com argumentos preservam o comportamento de terminal.
- [x] A extensão deixou de esconder o menu de contexto por `projectConfigured`. O comando “Auditar ou descobrir projeto” aparece em qualquer pasta; sem `auditor.toml`, o status mostra descoberta disponível e a execução usa o modo discovery da CLI. `cliCandidates` também procura a CLI no repositório que contém a extensão, cobrindo o caso de abrir outro projeto sem `target/`.
- [x] Testes novos: `extensions/vscode/tests/cliPath.test.mjs` (4) e `extensions/vscode/tests/manifest.test.mjs` (2). A extensão passou a 18 testes Node puros e o `.vsix` foi reempacotado. Validação manual na host real do VS Code confirmada pelo usuário em 27/11/2026; necessário reinstalar o `.vsix` e ajustar `vibeCodingAuditor.cliPath` quando apontar para o nome antigo da CLI.

### Etapa 6 — Amadurecimento arquitetural e distribuição — Em andamento

#### Etapa 6.0 — Baseline, inventário e reconciliação — Concluída em 27/11/2026

- [x] Branch `main`, HEAD `4146fcc8dd3554ee98923bd9df5fd39d00314d95` e alterações locais preservadas (`D PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md` e `?? PLANO_ETAPA_6_AMADURECIMENTO_ARQUITETURAL_E_DISTRIBUICAO.md`).
- [x] Inventário completo: um único workflow (`ci.yml`, jobs `verify` e `E2E Windows` em `windows-latest`); manifests, lockfiles, config Tauri e contrato conferidos.
- [x] Seis divergências da seção 2.3 do plano classificadas: README desatualizado, `bundle.active = false` e CI somente Windows confirmados; `cliPath` e menu sem contrato descartados; contagens de vulnerabilidades atualizadas sem correção.
- [x] Baseline com comandos, códigos e contagens em `docs/validation/etapa-6/baseline.md`: Rust 277 testes, desktop 39, extensão 18; `cargo fmt/check/test/clippy`, `git diff --check`, testes/builds Node e empacotamento `.vsix` com código `0`.
- [x] `npm audit`: desktop 25 alertas de desenvolvimento (14 altos, 11 moderados) e extensão 2 moderados de desenvolvimento; produção `0` nos dois; nenhuma correção aplicada.
- [x] Limitação explícita: E2E não reexecutado localmente no HEAD atual; o run remoto `46067832029` (commit `4ceeab2`) registra `verify` e `E2E Windows` verdes.

#### Etapa 6.1 — Documentação e configuração coerentes — Concluída em 27/11/2026

- [x] `README.md` passou a descrever Etapas 4 e 5 da nova fase, ajustes posteriores e Etapa 6 em andamento, com link para o baseline.
- [x] Modo automático documentado junto aos comandos da CLI: contrato válido = `compliance`, ausência = `discovery`, TOML inválido = erro `2`, sem fallback; `history record` segue o modo do núcleo.
- [x] Executável `vibe-coding-auditor-cli` confirmado em `Cargo.toml`, `--version`, `--help`, manifesto da extensão, `src/cliPath.ts`, testes e documentação. Nenhum nome ou caminho foi alterado.
- [x] Visibilidade do comando confirmada por `when: explorerResourceIsFolder` e pelo teste `o menu de pasta não exige auditor.toml`; contradição no README removida.
- [x] `bundle.active = false` documentado como ausência de instalador/bundle/release; `tauri build` gera apenas `auditor-desktop.exe`. Distribuição permanece na Etapa 6.7.
- [x] `AGENTS.md` ajustado somente para explicitar o modo automático no comando `audit`; regras de segurança, somente leitura e aprovação humana preservadas.
- [x] Decisões `E6.0` e `E6.1` registradas em `docs/DECISIONS.md`.
- [x] Verificação: `git diff --check` = `0` (apenas aviso LF/CRLF esperado do OneDrive em `AGENTS.md`), `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (277 testes), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `npm --prefix apps/desktop test` = `0` (39), `npm --prefix apps/desktop run build` = `0`, `npm --prefix extensions/vscode test` = `0` (18) e `npm --prefix extensions/vscode run package` = `0` (`.vsix` com 10 arquivos, 15,23 kB).
- [x] Limitações explícitas: nenhuma dependência, workflow, contrato, manifesto ou regra foi alterado; vulnerabilidades e `actions/cache@v4` aguardam 6.6; bundle/instalador aguardam 6.7; E2E remoto continua no commit anterior aos dois commits documentais do HEAD.

#### Etapa 6.2 — Descoberta de manifests multi-ecossistema — Concluída em 27/11/2026

- [x] Módulo `crates/auditor-core/src/discovery/manifests.rs` lê `package.json`, `pyproject.toml` e `go.mod` na raiz e em subprojetos, sem executar scripts, ambientes, instaladores ou comandos do projeto; diretórios padrão ignorados e links simbólicos não são seguidos.
- [x] `package.json`: extrai `name`, `version` e `dependencies`, `devDependencies`, `peerDependencies` e `optionalDependencies` sem misturar os papéis; reconhece workspaces em array ou `workspaces.packages` com diretórios literais e padrão final `/*`; manifestos malformados viram limitação localizada.
- [x] Relações `workspace:` são registradas com `destination: local_workspace` e ligadas ao pacote local quando possível; padrões com `**`, `?`, `..` ou `*` fora da posição final viram limitação explícita. Nenhum script é executado.
- [x] `pyproject.toml`: lê somente `[project]` no padrão PEP 621, com `name`, `version`, `dependencies` e `optional-dependencies` agrupados; Poetry/uv sem `[project]`, seções de gerenciador e versão dinâmica são limitações, não inferências.
- [x] `go.mod`: lê `module`, `require` em linha ou bloco, `replace`, `exclude` e comentários; diretivas não compreendidas e blocos não fechados geram limitação com linha quando disponível. Nenhum arquivo `.go` passa a ter analisador.
- [x] Tipos explícitos `ManifestObservation`, `ManifestEcosystem`, `ManifestStatus`, `LocalPackage`, `ManifestDependency`, `DependencyScope` e `DependencyDestination`, com caminho relativo, manifesto de origem e evidência curta; tudo em `candidates.manifests`, sem aprovar dependências ou gerar achados.
- [x] Relatórios Texto, JSON e Markdown e a saída `analyze` passaram a mostrar os manifests; as limitações locais também aparecem na lista geral de limitações usada por desktop e extensão.
- [x] Fixtures novas cobrem npm simples, workspace npm, `package.json` inválido, PEP 621 simples e com opcionais, Poetry fora de escopo, `go.mod` válido/parcial e projeto misto com manifests aninhados; `discovery_no_contract` cobre ausência. Fixtures são determinísticas e permanecem imutáveis após `audit`.
- [x] Decisão `E6.2` registrada em `docs/DECISIONS.md` e documentação alinhada em `README.md`.
- [x] Verificação: `git diff --check` = `0` (apenas avisos LF/CRLF esperados do OneDrive em arquivos Rust), `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (288 testes: 11 + 7 + 6 na CLI, 249 no núcleo e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `npm --prefix apps/desktop test` = `0` (39), `npm --prefix apps/desktop run build` = `0`, `npm --prefix extensions/vscode test` = `0` (18) e `npm --prefix extensions/vscode run package` = `0` (`.vsix` com 10 arquivos, 15,23 kB).
- [x] Limitações explícitas: nenhuma dependência, lockfile, workflow, `auditor.toml`, manifesto de projeto ou regra de conformidade foi alterado; o Builder ainda não transforma essas observações em sugestões editáveis, a resolução de imports/aliases permanece na 6.3, e arquivos Go continuam sem analisador.

#### Etapa 6.3 — Resolução de imports, aliases e dependências externas — Concluída em 27/11/2026

- [x] Módulo `crates/auditor-core/src/resolution.rs` reúne, somente dentro da raiz, `tsconfig*.json` local e os manifests da Etapa 6.2 em um contexto determinístico. `baseUrl` e `paths` aceitam zero ou um `*`; `extends` local é mesclado da configuração mais externa para a filha, com ciclo e padrão fora do subconjunto virando limitação; `extends` por pacote externo nunca é carregado.
- [x] Ordem de resolução documentada e aplicada: relativo → alias local → pacote local de workspace → nome de módulo do contrato → built-in → externo declarado → externo não confirmado. Alias com vários destinos gera revisão com opções ordenadas; alias sem arquivo no inventário gera revisão explícita; nenhuma ambiguidade é resolvida por escolha silenciosa.
- [x] TypeScript/JavaScript resolve aliases contra o inventário e registra arestas `permitida`/`proibida` quando o arquivo pertence a um único módulo do contrato. Built-ins Node (inclusive `node:`), scoped packages com subpath, workspaces locais e pacotes declarados/não confirmados viram relações separadas, nunca aresta de módulo.
- [x] Python associa dependência declarada somente quando o nome importável tem equivalência determinística com a distribuição; `Pillow`/`PIL` permanece `unconfirmed` com limitação. Biblioteca padrão conhecida vira `builtin`, sem manifesto e sem juízo de conformidade. Módulos do contrato continuam sendo resolvidos por sufixo de arquivo no inventário.
- [x] `Analysis.external_packages`, `AuditReport.external_packages` e `analysis.external_packages` na descoberta expõem estado, origem, evidência, arquivo e linha em JSON, texto, Markdown e `analyze`. `discovery::candidates` reutiliza o mesmo contexto para classificar `observed_dependencies`; nenhum estado externo gera `dep.not_allowed` ou aprovação de dependência.
- [x] Fixtures novas `resolution_typescript` e `resolution_python` cobrem alias com zero/um/muitos destinos, `baseUrl`/`paths`, `extends` local e externo, workspace local, scoped package com subpath, built-in Node, equivalência e divergência de nomes Python, além das regressões existentes.
- [x] Decisão `E6.3` registrada em `docs/DECISIONS.md` e documentação alinhada em `README.md`.
- [x] Verificação: `git diff --check` = `0` (apenas avisos LF/CRLF esperados do OneDrive em arquivos Rust), `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (298 testes: 11 + 7 + 6 na CLI, 259 no núcleo e 15 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `npm --prefix apps/desktop test` = `0` (39), `npm --prefix apps/desktop run build` = `0`, `npm --prefix extensions/vscode test` = `0` (18) e `npm --prefix extensions/vscode run package` = `0` (`.vsix` com 10 arquivos, 15,23 kB).
- [x] Limitações explícitas: nenhuma dependência, lockfile, workflow, `auditor.toml`, manifesto analisado, regra de conformidade, nome da CLI, `when` da extensão ou `bundle.active` foi alterado. `tsconfig` fora do subconjunto suportado, nomes Python sem equivalência determinística e ausência de analisador Go permanecem como limitações; nenhuma E2E foi reexecutada nesta subetapa.

#### Subetapas seguintes — planejadas, não iniciadas

- [ ] 6.4 — Contrato v3: proposta de schema aprovada antes de qualquer implementação.
- [ ] 6.5 — Autoauditoria multi-stack do próprio repositório.
- [ ] 6.6 — Segurança, dependências e consistência do CI.
- [ ] 6.7 — Empacotamento e estratégia de instalação.
- [ ] 6.8 — Fechamento, documentação e release candidata.

Esta seção será atualizada somente com evidência de testes e comportamentos realmente implementados.
