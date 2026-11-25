# Roadmap

Fonte integral: [PLANO_VIBE_CODING_AUDITOR.md](../PLANO_VIBE_CODING_AUDITOR.md). Uma etapa só avança depois de concluída, testada e revisada.

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
| 9 | Criar a extensão VS Code | Não iniciada |
| 10 | Expandir linguagens e recursos | Não iniciada (após MVP estável) |

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
