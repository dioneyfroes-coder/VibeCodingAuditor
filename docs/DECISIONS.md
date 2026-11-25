# Decisões técnicas

Este arquivo registra decisões explícitas do projeto. Conforme a regra 11 do plano, alterar uma decisão registrada exige documentar a substituição e sua justificativa.

## D0.1 — Executar somente a Etapa 0 neste trabalho
- **Data:** 2026-11-25
- **Decisão:** preservar `PLANO_VIBE_CODING_AUDITOR.md` sem reescrever o escopo, sem criar código de funcionalidade e sem antecipar etapas posteriores.
- **Motivo:** o plano exige que cada etapa seja concluída, testada e revisada antes da seguinte.
- **Impacto:** nesta etapa existem apenas documentos; nenhum crate, comando de auditoria ou contrato de exemplo foi criado.

## D0.2 — Nenhuma instalação, atualização ou remoção de ferramentas
- **Decisão:** não instalar, atualizar ou remover git, Rust, Node.js, npm, Tauri ou componentes do Windows sem autorização explícita.
- **Estado:** nenhuma ferramenta foi modificada. Todas as verificações foram somente leitura.

## D0.3 — Ambiente verificado

Coletado em 25/11/2026 com comandos de consulta de versão e leitura de registro do Windows; nada foi instalado.

| Item | Valor encontrado | Observação |
|---|---|---|
| Sistema operacional | Windows 11 Pro, versão 10.0.26200, build 26200 | Tauri 2 exige Windows 7 ou superior. |
| Git | 2.55.0.windows.4 | Identidade global configurada: `dioneyfroes` / `dioney.froes@hotmail.com`. |
| rustup | 1.29.1 | Toolchain ativa e padrão: `stable-x86_64-pc-windows-msvc`. |
| rustc / cargo | 1.98.1 | Site oficial indica Rust estável **1.100.0** nesta data; ver D0.5. |
| Componentes Rust | `cargo`, `clippy`, `rust-docs`, `rust-src`, `rust-std`, `rustc`, `rustfmt` | `cargo fmt` e `cargo clippy` já disponíveis para a Etapa 1. |
| Host Rust | `x86_64-pc-windows-msvc` | Compilação usa a toolchain MSVC. |
| MSVC Build Tools | MSVC 14.44.35207 em `C:\Program Files\Microsoft Visual Studio\2022\Community` | `cl.exe` não aparece no PATH de um shell comum; isso é esperado fora do Developer Command Prompt. |
| WebView2 Runtime | 158.0.4370.33 | Pré-requisito do Tauri no Windows; já instalado. |
| Node.js | v24.19.0 | Linha 24 é LTS segundo a documentação oficial; LTS mais recente é v26.13.1. |
| npm | 11.17.0 | Gerenciador padrão; sem alteração. |
| Tauri CLI | Não instalada | Release mais recente encontrado: `tauri-cli-v2.12.6` (18/11/2026). |

Referências oficiais consultadas:

- Instalação do Rust e considerações no Windows: <https://www.rust-lang.org/tools/install>
- Site oficial do Rust (versão estável vigente): <https://www.rust-lang.org/>
- Referência do Cargo: <https://doc.rust-lang.org/cargo/>
- Pré-requisitos do Tauri 2 (Windows, MSVC, WebView2, Node.js): <https://tauri.app/start/prerequisites/>
- Início do Tauri 2: <https://tauri.app/start/>
- Release do Tauri CLI: <https://github.com/tauri-apps/tauri/releases/latest>
- Calendário de releases do Node.js: <https://nodejs.org/en/about/previous-releases>
- Instruções persistentes do OpenCode: <https://opencode.ai/docs/instructions>

## D0.4 — Estrutura mínima criada

Foram criados apenas:

- `AGENTS.md` (regras do projeto)
- `README.md` (visão e estado atual)
- `docs/DECISIONS.md` (este arquivo)
- `docs/ROADMAP.md` (status das etapas, sem implementação antecipada)

Além disso, `git init` criou o diretório `.git` nesta pasta (repositório local, sem commits).

Não foram criados `Cargo.toml`, `auditor.example.toml`, `.gitignore`, diretórios `apps/`, `crates/`, `tests/` ou `data/`, pois não são necessários para a Etapa 0. O plano original foi mantido intacto.

## D0.5 — Decisões pendentes

| Decisão | Situação | Impacto |
|---|---|---|
| Inicializar o repositório Git nesta pasta (`git init`) | Concluído em 25/11/2026, após autorização; branch inicial `main` | Repositório local criado; naquele momento ainda não havia commits (os commits `295bc2e` e `4bba68c` vieram depois). |
| Atualizar Rust de 1.98.1 para 1.100.0 | Não atualizado; reavaliado na Etapa 1 (D1.3) | 1.98.1 atende ao workspace e ao `clap` 4.6.10. |
| Atualizar Node.js da linha 24 para a LTS 26 | Não autorizado | Só relevante quando a interface desktop for implementada (Etapa 8). |
| Fixar versão do Tauri CLI e do bundle @tauri-apps/cli | Não autorizado | Nada a instalar nesta etapa; avaliar na Etapa 8. |
| Criar `.gitignore` | Criado na Etapa 1 (D1.4) | Ignora `/target/` e `data/`. |

## D1.1 — Workspace Rust com dois crates
- **Decisão:** `Cargo.toml` virtual na raiz, `resolver = "2"`, membros `crates/auditor-core` (biblioteca) e `crates/auditor-cli` (binário `vibe-coding-auditor`), com `version = "0.1.0"` e `edition = "2024"` herdados do workspace.
- **Motivo:** estabelece a separação exigida pelo plano sem criar crates adicionais, Tauri ou plugins.
- **Impacto:** `Cargo.lock` passa a fazer parte do repositório, como recomendado para projetos com executável.

## D1.2 — Única dependência externa: `clap` 4.6.10
- **Decisão:** usar `clap = { version = "4.6.10", features = ["derive"] }` somente em `auditor-cli`.
- **Justificativa:** o plano define Clap como tecnologia da CLI; ele fornece `--help`, `--version`, mensagens de erro e sugestões padronizadas. A biblioteca padrão resolveria as duas flags atuais, mas exigiria parser manual quando os subcomandos das etapas seguintes existirem.
- **Verificação oficial:** <https://docs.rs/clap/latest/clap/> (4.6.10, publicado em 24/11/2026; MSRV declarado 1.74) e <https://github.com/clap-rs/clap>.
- **Impacto:** 21 pacotes externos resolvidos no `Cargo.lock` (23 entradas no total, incluindo os dois crates do workspace); nenhuma dependência adicionada ao `auditor-core`.

## D1.3 — Toolchain Rust mantida
- **Decisão:** continuar com rustc/cargo 1.98.1; não atualizar para 1.100.0 nesta etapa.
- **Motivo:** o workspace e o `clap` 4.6.10 compilam sem exigência de versão superior; a atualização permanece sem autorização (D0.5).
- **Impacto:** nenhuma mudança no ambiente.

## D1.4 — `.gitignore` mínimo
- **Decisão:** criar `.gitignore` com `/target/` e `data/`.
- **Motivo:** resolve a pendência registrada em D0.5 agora que existe build; evita versionar artefatos e dados locais futuros, conforme a estrutura de referência do plano.

## D1.5 — Fronteira núcleo/CLI
- **Decisão:** na Etapa 1, `auditor-cli` declara dependência de caminho para `auditor-core`, mas ainda não a utiliza; o núcleo expõe apenas `CORE_NAME` e um teste.
- **Motivo:** fixar o limite arquitetural desde já sem inventar funções de negócio. O uso efetivo começa com o comando `init` da Etapa 2, que compartilha o modelo de contrato do núcleo.
- **Impacto:** nenhuma regra ou analisador implementado; `--help` e `--version` são fornecidos pelo Clap.

## D1.6 — Limitação ambiental do Cargo no OneDrive
- **Observação:** com o repositório dentro de `OneDrive`, o Cargo emite `note: did not finalize incremental compilation session directory ... Acesso negado (os error 5)` quando a compilação incremental está ativa.
- **Decisão:** registrar como limitação do ambiente, não do código. Para verificação limpa, executar `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`.
- **Impacto:** todos os comandos de verificação terminam com sucesso; o aviso não altera os resultados.

## D2.1 — Schema v1 do contrato
- **Decisão:** `schema_version = 1`; `[project] name`; `[scan] include` e `exclude` obrigatórios; `[architecture] style` aceitando apenas `modular_monolith`; `[[architecture.modules]]` com `name`, `path`, `responsibility` e `allowed_dependencies` opcional (padrão `[]`). Propriedades desconhecidas são rejeitadas por `serde(deny_unknown_fields)`.
- **Motivo:** manter o formato pequeno e inteiramente implementado, sem opções decorativas ou hábitos futuros.
- **Impacto:** um contrato válido precisa declarar explicitamente inclusões e exclusões; não há defaults silenciosos nesta versão.

## D2.2 — Resolução das ambiguidades do item 5 do plano
- **Dependências proibidas:** não existe campo `forbidden_dependencies`. `allowed_dependencies` é uma lista fechada; qualquer dependência fora dela será tratada como proibida pelas regras das etapas seguintes.
- **Arquivos não mapeados e módulos sem arquivos:** não são opções do contrato. Serão achados da categoria `needs_review` quando as regras existirem (Etapas 4 e 5).
- **Diretórios gerados:** são declarados em `scan.exclude`. A aplicação automática de padrões como `.git`, `target` e `node_modules` pertence à descoberta (Etapa 3), não ao schema.
- **Evolução do formato:** `schema_version` barra versões não suportadas com mensagem clara. Apenas a v1 existe; uma mudança futura deve introduzir uma nova versão e migração explícita, sem alterar contratos antigos silenciosamente.
- **Motivo:** o plano exige um schema real, testado e documentado, sem adicionar opções que ainda não tenham implementação.

## D2.3 — Dependências para leitura de TOML
- **Decisão:** `toml = "1.2.3"` e `serde = { version = "1.0.229", features = ["derive"] }`, apenas em `auditor-core`.
- **Justificativa:** a biblioteca padrão não possui parser TOML. O derive do Serde permite mapear o contrato em tipos Rust e, com `deny_unknown_fields`, implementa a exigência de não aceitar propriedades desconhecidas silenciosamente.
- **Verificação oficial:** <https://docs.rs/toml/latest/toml/> (1.2.3+spec-1.1.0, 25/11/2026) e <https://docs.rs/serde/latest/serde/> (1.0.229, 24/11/2026).
- **Impacto:** 12 pacotes adicionados ao `Cargo.lock`. O núcleo continua independente de CLI, Tauri, React, SQLite e IA.

## D2.4 — Validação e erros
- **Validação implementada:** presença de `project.name`; `scan.include` não vazio; padrões relativos, não absolutos, sem `..` e usando `/`; nomes e caminhos de módulos únicos; nomes de módulos não vazios; referências de `allowed_dependencies` existentes, não duplicadas e diferentes de si mesmo; `style` reconhecido; versão suportada.
- **Erros:** `Io` (com caminho), `Parse`, `UnsupportedSchemaVersion` (com versão encontrada) e `Validation` (com mensagem explicativa), todos com `Display` e `source`.
- **Impacto:** contratos inválidos falham antes de qualquer uso, com mensagem que identifica o problema.

## D2.5 — Comando `auditor init`
- **Decisão:** `init [PATH]` grava `auditor.toml` no diretório informado (padrão: atual) a partir de `auditor.example.toml`, usando `create_new` para recusar sobrescrita. A saída e o cabeçalho do arquivo afirmam explicitamente que o modelo exige revisão e confirmação.
- **Motivo:** o plano pede um exemplo gerado sem afirmar que ele está aprovado; a auditoria permanece somente leitura, e a escrita ocorre apenas por ação explícita do usuário.
- **Impacto:** nenhum arquivo existente é modificado; uma segunda execução no mesmo diretório falha com erro de arquivo já existente.

## D2.6 — Fixtures e testes
- **Decisão:** um contrato válido e sete contratos inválidos em `tests/fixtures/contracts/`, com testes no núcleo para carregamento, rejeição de campos desconhecidos, versão não suportada, style não suportado, dependência desconhecida, duplicidade, caminho absoluto, nome vazio e arquivo ausente. A CLI testa o parsing do subcomando `init`.
- **Impacto:** 10 testes unitários no total (7 no núcleo, 3 na CLI), todos reproduzíveis sem rede.

## D3.1 — Varredura sem novas dependências
- **Decisão:** usar apenas a biblioteca padrão (`fs::read_dir`, `fs::symlink_metadata` e `fs::canonicalize`). Nenhum crate de glob ou caminho foi adicionado.
- **Motivo:** o contrato usa um subconjunto pequeno e determinístico de padrões; uma dependência externa não traria benefício proporcional nesta etapa.
- **Impacto:** `Cargo.lock` permanece inalterado em relação à Etapa 2; o núcleo continua independente de CLI, Tauri, React e SQLite.

## D3.2 — Subconjunto de padrões suportado
- **Decisão:** `**` é aceito somente como componente inteiro e corresponde a zero ou mais componentes; `*` é aceito dentro de um componente. `?`, classes `[...]` e chaves `{...}` são tratados como caracteres literais.
- **Motivo:** cobrir `crates/**/*.rs` e os exemplos do plano sem implementar um motor de glob completo.
- **Impacto:** padrões com sintaxe não suportada não são fingidos como curingas: seus caracteres são comparados literalmente e, portanto, não produzem a correspondência esperada. Essa limitação é explícita e será revisitada apenas se uma regra futura exigir mais sintaxe.

## D3.3 — Diretórios ignorados por padrão
- **Decisão:** `.git`, `target`, `node_modules`, `dist`, `build`, `.venv` e `venv` são ignorados em qualquer profundidade, além dos padrões de `scan.exclude`.
- **Motivo:** atender ao plano sem duplicar no schema o que é comportamento da descoberta (D2.2).
- **Impacto:** um diretório podado aparece uma única vez em `skipped`, com o motivo; seus descendentes não são listados individualmente.

## D3.4 — Links simbólicos
- **Decisão:** toda entrada é inspecionada com `symlink_metadata`; qualquer link simbólico vira `SkipReason::Symlink` e nunca é seguido, mesmo quando aponta para dentro da raiz. Não existe opção para seguir links nesta etapa.
- **Motivo:** reforçar a regra de segurança do projeto e evitar loops ou travessia para fora da raiz.
- **Impacto:** atalhos legítimos ficam fora do inventário, com motivo registrado. Não há teste automatizado de link simbólico porque a criação depende de privilégio no Windows.

## D3.5 — Metadados e falhas parciais
- **Decisão:** cada arquivo incluído registra `path` relativo com `/`, `extension` e `size` em bytes. Interpretação de "metadados úteis" foi mantida no mínimo determinístico: sem conteúdo, hash ou timestamp. Falha ao ler uma entrada vira `SkipReason::Inaccessible` e a varredura continua; apenas uma raiz não canonicalizável gera `DiscoveryError::Root`.
- **Motivo:** evitar leitura de conteúdo e atributos voláteis, preservando a auditoria somente leitura.
- **Impacto:** não há teste automatizado de permissão negada, pois depende de ACL do sistema. No Windows, a raiz canonicalizada aparece com o prefixo `\\?\` na saída da CLI.

## D3.6 — Comando `scan` e fixtures
- **Decisão:** `scan <RAIZ>` lê `<RAIZ>/auditor.toml`, chama o núcleo e imprime o inventário. A fixture `tests/fixtures/projects/simple` contém três arquivos incluídos, um arquivo excluído pelo contrato, diretórios ignorados por padrão e arquivos fora do `include`.
- **Impacto:** 16 testes no total (12 no núcleo e 4 na CLI). Nenhuma associação de arquivo a módulo ou regra de dependência é executada; isso pertence à Etapa 4.

## D4.1 — Parser Rust para `mod` e `use`

- **Decisão:** `syn = { version = "3.0.10", default-features = false, features = ["full", "parsing"] }` no workspace, consumida somente por `auditor-core`.
- **Alternativas:** um scanner textual próprio evitaria dependência, mas poderia contar comentários e strings como código; `cargo_metadata` exigiria subprocess do Cargo. A escolha foi `syn` por analisar a sintaxe Rust sem executar o projeto.
- **Verificação oficial:** <https://docs.rs/syn/latest/syn/> (3.0.10, 09/11/2026) e <https://crates.io/api/v1/crates/syn/3.0.10> (não yanked, MIT OR Apache-2.0), consultados em 25/11/2026.
- **Impacto:** nenhum pacote novo entrou no `Cargo.lock` — `proc-macro2`, `quote` e `unicode-ident` já vinham de `serde_derive`; apenas `auditor-core` passou a declarar `syn`. O núcleo continua independente de CLI, Tauri, React, SQLite e IA.

## D4.2 — Metadados do workspace

- **Decisão:** ler `<raiz>/Cargo.toml` e o `[package].name` dos membros de `[workspace].members` diretamente com o `toml` já existente. Não executar `cargo metadata`.
- **Motivo:** manter a análise local, previsível e somente leitura, sem depender de subprocess, rede ou cache do Cargo.
- **Impacto:** os nomes aparecem em `WorkspaceInfo`. Um manifest de membro ausente ou inválido é erro operacional. Padrões de membro com glob não são suportados nesta versão.

## D4.3 — Relação arquivo → módulo

- **Decisão:** reutilizar `matches_pattern` da descoberta. O arquivo é associado ao primeiro módulo cujo `path` corresponde; zero correspondências gera `UnmappedFile` e mais de uma gera `AmbiguousModule`, ambos como revisão necessária.
- **Motivo:** manter uma única semântica de caminho entre `include`/`exclude` e o contrato arquitetural.
- **Impacto:** módulos declarados no contrato sem arquivo correspondente ainda não são avaliados; essa regra pertence à Etapa 5.

## D4.4 — Classificação das arestas

- **Decisão:** `allowed_dependencies` continua sendo lista fechada. Destino presente nela gera aresta `Allowed`; destino declarado no contrato, mas ausente dela, gera `Forbidden`; qualquer outro destino gera `Unresolved` e item de revisão. `use crate::X` é mapeado para `X`; `self`, `super`, o próprio módulo e `std`, `core`, `alloc`, `test` não geram arestas.
- **Motivo:** cumprir a Etapa 4 sem inventar regras de achado, que ficam para a Etapa 5.
- **Impacto:** `Analysis.edges` são dados estruturados para inspeção; a CLI apenas os imprime. Ainda não há identificador de regra, relatório nem código de saída de auditoria.

## D4.5 — Evidência e limitações da análise

- **Decisão:** `syn` fornece a estrutura das declarações; arquivo, linha e texto da evidência são localizados por varredura textual do mesmo arquivo, em ordem de aparição. O conteúdo de `mod` inline é percorrido. Arquivo que não faz parse ou linha que não pode ser localizada vira item em `Analysis.limitations`, sem interromper a auditoria.
- **Motivo:** preservar rastreabilidade verificável sem gravar conteúdo integral de arquivos do usuário.
- **Impacto:** imports dentro de corpos de funções, macros complexos, dependências geradas e nomes resolvidos dinamicamente não são classificados. A limitação é registrada explicitamente quando detectável; nenhuma conclusão é inventada para esses casos.

## D4.6 — Comando `analyze` e fixture

- **Decisão:** `analyze <RAIZ>` imprime módulos, pacotes Cargo, arestas com origem, revisões necessárias, declarações `mod` e limitações. A fixture `tests/fixtures/projects/dependencies` contém `application -> domain` permitida, `application -> infrastructure` proibida, `infrastructure -> application` e `infrastructure -> domain` permitidas, `infrastructure -> unknown_crate` não resolvida, `domain -> application` proibida dentro de `mod` inline, `tools/helper.rs` não mapeado e `broken.rs` com erro de parse.
- **Impacto:** 36 testes no total (30 no núcleo e 6 na CLI), todos passando. Nenhum relatório JSON/Markdown, achado formal ou código de saída por auditoria foi adicionado; isso pertence à Etapa 5.

## D4.7 — Revisão de paradigma, nomes, documentação e testes

- **Paradigma:** mantido o monólito modular determinístico: `auditor-core` depende apenas de `serde`, `toml` e `syn`; a CLI depende do núcleo e de `clap`; não há Tauri, React, SQLite, IA, subprocess do Cargo nem escrita nos projetos auditados.
- **Nomes:** helpers genéricos foram renomeados para deixar a intenção explícita — `walk_tree`, `relative_path`, `is_excluded_path`, `matches_path_parts`, `matches_pattern_component`, `locate_source_line`, `is_use_declaration_line`, `is_mod_declaration_line`, `collect_use_path_leaves`, `read_cargo_workspace` e `dependency_kind_label`. `ModuleDeclaration.module` passou a ser `module_name`, e `VisitContext.module` passou a ser `contract_module`.
- **Documentação:** campos públicos de `Contract`, `Project`, `Scan`, `Architecture`, `Module`, `ContractError`, `SkippedEntry`, `SkipReason`, `DiscoveryError`, `WorkspaceInfo`, `WorkspacePackage`, `DependencyEdge`, `ReviewItem`, `ModuleDeclaration` e `AnalysisError` passaram a ter comentários explícitos. Helpers privados também receberam comentários curtos.
- **Testes:** adicionados testes diretos para `require_non_empty`, `validate_pattern`, `relative_path`, `is_excluded_path`, `matches_path_parts`, `matches_pattern_component`, `locate_source_line`, `is_use_declaration_line`, `is_mod_declaration_line`, `collect_use_path_leaves`, `AnalysisError`, `dependency_kind_label` e `ReviewReason::AmbiguousModule`. Helpers de teste também receberam nomes e comentários descritivos.
- **Impacto:** nenhuma regra ou funcionalidade nova; apenas qualidade interna e rastreabilidade. Dois testes criados nesta revisão falharam inicialmente por expectativa incorreta e foram corrigidos antes da execução final; nenhum comportamento de produção foi alterado por essas correções.
