# Roadmap

Fonte integral: [PLANO_VIBE_CODING_AUDITOR.md](../PLANO_VIBE_CODING_AUDITOR.md). Uma etapa só avança depois de concluída, testada e revisada.

| Etapa | Nome | Status |
|---|---|---|
| 0 | Verificar o ambiente e preparar o repositório | Concluída em 25/11/2026 — documentação criada e repositório Git inicializado |
| 1 | Criar o workspace Rust e a CLI mínima | Concluída em 25/11/2026 — `cargo check`, `test`, `fmt --check` e `clippy` passaram |
| 2 | Criar o modelo do contrato arquitetural | Não iniciada |
| 3 | Descobrir e inventariar o projeto | Não iniciada |
| 4 | Implementar a primeira análise de dependências Rust | Não iniciada |
| 5 | Criar o motor de regras e os relatórios | Não iniciada |
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
