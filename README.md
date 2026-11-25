# Vibe Coding Auditor

Ferramenta local para comparar a arquitetura pretendida de um projeto com a arquitetura realmente implementada. Cada achado deve apontar a regra violada, os arquivos envolvidos e, quando possível, as linhas relevantes. O contrato arquitetural (`auditor.toml`) pertence ao usuário e é versionado junto ao projeto auditado.

**Status atual:** Etapa 2 — modelo do contrato arquitetural concluído. A CLI já cria um modelo de `auditor.toml`; ainda não há varredura de arquivos nem regras de auditoria.

## Documentação

- Plano completo e critérios de cada etapa: [PLANO_VIBE_CODING_AUDITOR.md](PLANO_VIBE_CODING_AUDITOR.md)
- Decisões técnicas e ambiente verificado: [docs/DECISIONS.md](docs/DECISIONS.md)
- Progresso das etapas: [docs/ROADMAP.md](docs/ROADMAP.md)
- Regras para agentes: [AGENTS.md](AGENTS.md)

## Direção técnica definida no plano

- Núcleo de auditoria em Rust, reutilizado pela CLI e pela futura aplicação desktop.
- Interface desktop planejada com Tauri 2, React e TypeScript.
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

Campos desconhecidos, nomes ou caminhos duplicados, referências a módulos inexistentes, caminhos absolutos e versões de schema não suportadas são rejeitados com mensagem explicativa. O modelo gerado por `auditor init` está em [`auditor.example.toml`](auditor.example.toml) e **precisa ser revisado e confirmado pelo usuário** antes de ser considerado aprovado.

## Execução

O executável `vibe-coding-auditor` responde a `--help`, `--version` e `init <diretorio>`. O comando `init` cria `auditor.toml` no diretório informado e nunca sobrescreve um arquivo existente. Nenhum comando de auditoria foi implementado ainda.
