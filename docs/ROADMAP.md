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
| 6 | Registrar histórico local em SQLite | Não iniciada |
| 7 | Provar o núcleo antes da interface gráfica | Não iniciada |
| 8 | Criar a aplicação desktop com Tauri | Não iniciada |
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
