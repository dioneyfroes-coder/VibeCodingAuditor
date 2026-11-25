# Vibe Coding Auditor

Ferramenta local para comparar a arquitetura pretendida de um projeto com a arquitetura realmente implementada. Cada achado deve apontar a regra violada, os arquivos envolvidos e, quando possível, as linhas relevantes. O contrato arquitetural (`auditor.toml`) pertence ao usuário e é versionado junto ao projeto auditado.

**Status atual:** Etapa 5 — motor de regras e relatórios concluído. A CLI cria um modelo de `auditor.toml`, inventaria arquivos, imprime o grafo de dependências e aplica regras determinísticas, gerando saída em texto, JSON ou Markdown. Ainda não há histórico em SQLite nem interface desktop.

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
cargo run -p auditor-cli -- scan <raiz>
cargo run -p auditor-cli -- analyze <raiz>
cargo run -p auditor-cli -- audit <raiz>
cargo run -p auditor-cli -- audit <raiz> --format json
cargo run -p auditor-cli -- audit <raiz> --format markdown --output relatorio.md
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

## Descoberta

O comando `scan <raiz>` lê `<raiz>/auditor.toml`, percorre a árvore sem executar arquivos e imprime o inventário. Os padrões suportados são `**` como componente inteiro e `*` dentro de um componente; `?`, classes e chaves não são interpretados. Diretórios `.git`, `target`, `node_modules`, `dist`, `build`, `.venv` e `venv` são ignorados em qualquer profundidade, além de `scan.exclude`. Links simbólicos nunca são seguidos, e entradas inacessíveis são registradas sem interromper a varredura. Cada arquivo incluído registra caminho relativo, extensão e tamanho em bytes.

## Análise de dependências

O comando `analyze <raiz>` lê o contrato e o inventário, lê os manifests `Cargo.toml` do workspace e usa `syn` para interpretar declarações `mod` e importações `use` dos arquivos `.rs` incluídos. Cada arquivo é associado ao primeiro módulo do contrato cujo `path` corresponde ao caminho relativo; múltiplas correspondências e arquivos sem correspondência viram itens de revisão.

As arestas são classificadas assim:

- `permitida`: o destino está em `allowed_dependencies` do módulo de origem.
- `proibida`: o destino é um módulo declarado no contrato, mas não está em `allowed_dependencies`.
- `não resolvida`: o destino não corresponde a nenhum módulo declarado; também gera item de revisão.

A origem de cada aresta guarda arquivo relativo, linha e a linha de código encontrada. Declarações `mod` também são registradas com arquivo e linha; o conteúdo de `mod` inline é percorrido. `use` de `std`, `core`, `alloc`, `test`, `self`, `super` e dependências do próprio módulo não geram arestas arquiteturais.

Limitações explícitas desta primeira versão: são analisados apenas arquivos `.rs` que entraram no inventário; imports dentro de corpos de funções, macros complexos, código gerado e nomes resolvidos dinamicamente não são classificados. Arquivos que não fazem parse viram limitação e não interrompem a análise. Quando a linha de uma declaração não pode ser localizada no texto, a evidência fica vazia e a limitação é registrada.

## Motor de regras e relatórios

O comando `audit <raiz>` reutiliza o mesmo pipeline do núcleo e aplica estas regras com identificadores estáveis:

| Regra | Severidade | Situação |
|---|---|---|
| `dep.not_allowed` | `error` | Destino é módulo declarado, mas ausente de `allowed_dependencies`. |
| `dep.unresolved` | `needs_review` | Destino não corresponde a nenhum módulo do contrato. |
| `file.unmapped` | `needs_review` | Arquivo Rust incluído não corresponde a nenhum módulo. |
| `file.ambiguous_module` | `needs_review` | Arquivo corresponde a mais de um módulo do contrato. |
| `dep.cycle` | `warning` | Grafo contém ciclo simples entre módulos, incluindo autoimportação. |
| `module.no_files` | `warning` | Módulo declarado sem arquivo Rust correspondente. |

Cada achado contém `id`, `rule_id`, `severity`, `title`, `message`, `path`, `line_start`, `line_end`, `evidence`, `expected`, `suggestion` e `confidence`. Todas as regras são determinísticas, então `confidence` é sempre nulo; o auditor não inventa precisão numérica nem aplica sugestões automaticamente. Dependências não resolvidas e arquivos não mapeados são sinais para decisão humana, não violações comprovadas.

Os formatos disponíveis são `text` (padrão), `json` e `markdown`. Sem `--output`, o relatório vai para o terminal; com `--output <arquivo>`, o arquivo indicado é gravado e sobrescrito se já existir. O projeto auditado nunca é modificado.

## Execução

O executável `vibe-coding-auditor` responde a `--help`, `--version`, `init <diretorio>`, `scan <raiz>`, `analyze <raiz>` e `audit <raiz>`. `init` cria `auditor.toml` sem sobrescrever arquivo existente; `scan` e `analyze` são somente leitura e servem para inspeção. `audit` produz achados e relatórios no formato escolhido.

Códigos de saída: `0` quando a auditoria não gera achados, `1` quando há pelo menos um achado e `2` para falha operacional ou argumento inválido. Esses códigos valem para todos os subcomandos; `init`, `scan` e `analyze` retornam `0` em sucesso e `2` em falha.
