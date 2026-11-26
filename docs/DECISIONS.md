# Decisões técnicas

Este arquivo registra decisões explícitas do projeto. Conforme a regra 11 do plano, alterar uma decisão registrada exige documentar a substituição e sua justificativa.

## D0.1 — Executar somente a Etapa 0 neste trabalho
- **Data:** 2026-11-25
- **Decisão:** preservar `PLANO_VIBE_CODING_AUDITOR.md` sem reescrever o escopo, sem criar código de funcionalidade e sem antecipar etapas posteriores. Preservação substituída por `R8` após o fechamento da primeira fase.
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
- **Impacto:** os nomes aparecem em `WorkspaceInfo`. Um manifest de membro ausente ou inválido é erro operacional. Globs com `*` no último componente passaram a ser expandidos em `E2.4`, respeitando `workspace.exclude`; outras formas viram limitação explícita. A leitura de tabelas de dependências foi acrescentada em `E2.3`, sem alterar a decisão de não executar o Cargo.

## D4.3 — Relação arquivo → módulo

- **Decisão:** reutilizar `matches_pattern` da descoberta. Um arquivo é associado a um módulo somente quando exatamente um `path` corresponde. Zero correspondências gera `UnmappedFile`; mais de uma gera `AmbiguousModule` com os nomes dos candidatos ordenados e nenhuma associação definitiva. A política foi revisada em `E2.2`.
- **Motivo:** manter uma única semântica de caminho entre `include`/`exclude` e o contrato arquitetural e evitar que a ordem do TOML produza uma violação definitiva.
- **Impacto:** arquivos ambíguos não geram arestas nem declarações `mod` atribuídas; apenas revisão necessária. Módulos declarados no contrato sem arquivo correspondente ainda não são avaliados; essa regra pertence à Etapa 5.

## D4.4 — Classificação das arestas

- **Decisão:** `allowed_dependencies` continua sendo lista fechada. Destino presente nela gera aresta `Allowed`; destino declarado no contrato, mas ausente dela, gera `Forbidden`; qualquer outro destino gera `Unresolved` e item de revisão. `use crate::X` é mapeado para `X`; `self`, `super`, o próprio módulo e `std`, `core`, `alloc`, `test` não geram arestas. A política de externas declaradas e de caminhos relativos foi revisada em `E2.3`.
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

## D5.1 — Dependência `serde_json` para exportação

- **Decisão:** declarar `serde_json = "1.0.154"` no workspace e no `auditor-core`, sem adicioná-la à CLI.
- **Motivo:** os tipos de achado e relatório já derivam `Serialize`; serializar com `serde_json` evita gerador manual de JSON e garante escape correto de texto oriundo de código-fonte. Renderizadores de texto e Markdown vivem no núcleo para que CLI e futura interface compartilhem a mesma saída.
- **Alternativa rejeitada:** concatenar JSON manualmente na CLI, que exigiria replicar regras de escape e deixaria a saída dependente do frontend.
- **Verificação oficial:** <https://docs.rs/serde_json/latest/serde_json/> e <https://crates.io/api/v1/crates/serde_json/1.0.154> confirmam 1.0.154, não yanked, licença `MIT OR Apache-2.0`, publicada em 11/10/2026.
- **Impacto:** `Cargo.lock` passou a incluir `serde_json 1.0.154` e os transitivos `itoa 1.0.19`, `memchr 2.8.3` e `zmij 1.0.23`. As versões e checksums foram confirmados em <https://crates.io/api/v1/crates/itoa/1.0.19>, <https://crates.io/api/v1/crates/memchr/2.8.3> e <https://crates.io/api/v1/crates/zmij/1.0.23>; nenhum está yanked. Nenhuma ferramenta do sistema foi instalada ou alterada.

## D5.2 — Regras e severidades da Etapa 5

- **Decisão:** implementar seis regras estáveis: `dep.not_allowed` (`error`), `dep.unresolved` (`needs_review`), `file.unmapped` (`needs_review`), `file.ambiguous_module` (`needs_review`), `dep.cycle` (`warning`) e `module.no_files` (`warning`).
- **Motivo:** separar violação determinística de revisão humana e sinal de atenção, conforme a seção 6 do plano. Dependências não resolvidas e arquivos não mapeados já eram itens de revisão na Etapa 4; transformá-los em erros inventaria uma conclusão que a análise não tem. Ciclos são sinais estruturais, mantidos como aviso para preservar a decisão final do usuário.
- **Rejeição explícita:** a regra de limite de tamanho configurável da Etapa 5 foi adiada porque o schema v1 não possui campo para ele; inventar um limite silencioso violaria a regra de não alterar o contrato sem decisão registrada.
- **Impacto:** cada achado contém os doze campos do modelo, arquivo e linha quando disponíveis, evidência verificável e sugestão nunca aplicada. `confidence` é sempre nulo porque nenhuma regra é heurística.

## D5.3 — Relatório e código de saída

- **Decisão:** `audit <raiz>` aceita `--format text|json|markdown` e `--output <arquivo>`. Sem `--output`, imprime no terminal; com ele, grava o arquivo indicado, sobrescrito se existir. Códigos: `0` sem achados, `1` com pelo menos um achado e `2` para falha operacional ou argumento inválido.
- **Motivo:** o plano exige saída legível, exportação estruturada e códigos distintos para sucesso, achados e falha. A escrita permanece explícita fora da raiz auditada por padrão; nenhum comando modifica fontes ou contratos.
- **Impacto:** os códigos passaram a valer para todos os subcomandos. `init`, `scan` e `analyze` retornam `0` em sucesso e `2` em falha, antes retornavam `1`. A mudança foi registrada no `README.md`.

## D5.4 — Orquestração no núcleo e revisão de qualidade

- **Decisão:** criar `audit::audit_project` para encadear contrato, descoberta, análise, regras e relatório. A CLI apenas despacha o comando, renderiza e escolhe o destino da saída.
- **Motivo:** manter o limite "a interface não implementa regras de auditoria" e permitir que a futura aplicação desktop reutilize o mesmo pipeline.
- **Revisão:** nomes como `run` foram trocados por `dispatch_command`; campos, helpers e tipos públicos foram documentados; testes diretos cobrem regras, formatação de local, escape Markdown, serialização JSON, mapeamento de formatos e códigos de saída. Um teste tautológico sobre `ArchitectureStyle` foi removido e substituído por teste de autoimportação.
- **Limitações conhecidas:** ciclos são enumerados como caminhos simples canônicos; a fixture produz três ciclos, não um. A detecção depende das arestas já classificadas na Etapa 4 e herda suas limitações (imports em corpos de funções, macros complexos, código gerado e nomes dinâmicos). Arquivos que não fazem parse continuam apenas em `limitations`, sem virar achado de erro.
- **Impacto:** 60 testes no total (51 no núcleo e 9 na CLI), todos passando. Nenhuma persistência SQLite, interface desktop, extensão VS Code ou uso de IA foi adicionado. A última frase é substituída por D6.1 a partir da Etapa 6.

## D6.1 — Dependência `rusqlite` com SQLite embarcado

- **Decisão:** declarar `rusqlite = { version = "0.43.0", features = ["bundled"] }` no workspace e no `auditor-core`. Esta decisão substitui o escopo de dependências registrado em D4.7 e a ausência de SQLite registrada em D5.4.
- **Motivo:** o plano exige histórico local em SQLite. A feature `bundled` compila o SQLite junto com o núcleo, sem depender de biblioteca do sistema nem de instalação externa no Windows. `rusqlite` mantém o histórico no processo local; a CLI e a futura interface compartilham o mesmo módulo.
- **Alternativas rejeitadas:** arquivo JSON — contrariaria o plano; SQLite do sistema — dependeria de biblioteca externa não garantida; persistência na interface — violaria o limite de que a interface não implementa regras nem acessa dados diretamente.
- **Verificação oficial:** <https://docs.rs/rusqlite/latest/rusqlite/> e <https://crates.io/api/v1/crates/rusqlite/0.43.0> confirmam 0.43.0, não yanked, licença MIT e MSRV 1.89, compatível com o rustc verificado no projeto. Os 17 pacotes adicionados ao `Cargo.lock` — `bitflags 2.13.2`, `cc 1.11.3`, `fallible-iterator 0.3.0`, `fallible-streaming-iterator 0.1.9`, `find-msvc-tools 0.1.15`, `foldhash 0.2.0`, `hashlink 0.12.2`, `libsqlite3-sys 0.40.0`, `pkg-config 0.3.34`, `rsqlite-vfs 0.2.0`, `rusqlite 0.43.0`, `shlex 2.0.1`, `smallvec 1.16.6`, `sqlite-wasm-rs 0.6.3`, `thiserror 2.0.21`, `thiserror-impl 2.0.21` e `vcpkg 0.2.15` — foram consultados na API oficial do crates.io: nenhum está yanked e todos os checksums conferem com o lockfile. Nenhuma ferramenta do sistema foi instalada, atualizada ou removida.
- **Impacto:** `auditor-core` passou a depender de `rusqlite`; a CLI continua dependendo apenas de `clap` e do núcleo. O banco padrão é `data/auditor.db`, relativo ao diretório de trabalho, e o diretório `data/` já era ignorado pelo Git.

## D6.2 — Dados gravados e privacidade

- **Decisão:** gravar em `audit_runs` a data/hora em segundos Unix, a raiz canonicalizada, o nome do projeto, a versão do schema do contrato e o resumo; em `audit_findings`, os doze campos de cada achado; em `audit_limitations`, as limitações da análise. `Severity::as_str` passou a ser a representação estável de severidade no banco.
- **Motivo:** o plano pede comparação reproduzível sem guardar código do usuário. O `id` estável do achado permite comparar execuções mesmo quando a mensagem muda; a linha de evidência já era necessária no relatório da Etapa 5.
- **Alternativa rejeitada:** serializar o `AuditReport` inteiro em uma coluna JSON — perderia consultas por execução/achado e dificultaria a comparação por identificador.
- **Impacto:** nenhum conteúdo integral de arquivo é gravado; a verificação sobre a fixture confirmou que `pub fn forbidden()` não aparece no banco, enquanto a evidência `use infrastructure::Client;` aparece. O prefixo `\\?\` do Windows é removido apenas na raiz exibida; ele não altera a auditoria nem os arquivos auditados.

## D6.3 — Política de migração do banco

- **Decisão:** manter `schema_migrations` com a versão aplicada. Na abertura, versão ausente aplica a migração `1` em transação; versão igual à suportada não altera nada; versão superior gera `HistoryError::UnsupportedSchema` e encerra a operação.
- **Motivo:** o plano pede política explícita de migração. Falhar para frente é mais seguro que downgrade silencioso, que poderia descartar dados escritos por versão posterior.
- **Alternativa rejeitada:** `CREATE TABLE IF NOT EXISTS` sem controle de versão — não detectaria schema mais novo nem permitiria evolução futura controlada.
- **Impacto:** não há deleção, limpeza ou alteração destrutiva automática. Bancos de versões futuras exigem atualização do auditor; o tamanho do histórico cresce com novas execuções e cabe ao usuário gerenciar o arquivo.

## D6.4 — Comandos de histórico e códigos de saída

- **Decisão:** adicionar `history record`, `history list`, `history show` e `history compare`, com `--database` global ao subcomando. A renderização em texto (`render_run_list`, `render_run_detail`, `render_comparison`) fica no núcleo. `record` e `audit` retornam `0`/`1` conforme os achados da execução; `list`, `show` e `compare` retornam `0` em sucesso; qualquer falha retorna `2`.
- **Motivo:** manter a CLI como despacho e saída, sem regras nem formatação duplicada. `record` precisa do mesmo código de `audit` porque executa a auditoria; consultas não são auditorias e não devem sinalizar achados como falha.
- **Impacto:** formulários de data/hora e caminhos são determinísticos; comparações usam `BTreeSet` de identificadores e conservam a ordem gravada dos achados. Nenhum comando modifica o projeto auditado.

## D6.5 — Revisão de paradigma, nomes, documentação e testes

- **Paradigma:** monólito modular mantido. O módulo `history` pertence ao núcleo; `History::open`, `record`, `list`, `get` e `compare` são a API pública, e a CLI apenas encaminha argumentos e imprime o texto devolvido pelo núcleo.
- **Nomes:** helpers receberam nomes descritivos — `database_error`, `display_root`, `format_utc`, `civil_from_days`, `read_optional_usize`, `read_usize`, `read_u32`, `conversion_error`, `parse_severity`, `summary_phrase`, `push_finding_header`, `push_detailed_findings` e `push_compact_findings`. O comando `run` da CLI continuou como `dispatch_command`, e `history_command` concentra as operações de histórico.
- **Documentação:** todos os tipos públicos, campos e helpers do módulo `history` foram documentados; `finding_location` foi tornado público e documentado para reutilização consistente. O `README.md` registra comandos, dados gravados, limitações de privacidade, migração e códigos de saída.
- **Testes:** 11 testes focados do histórico cobrem abertura/reabertura, listagem com filtro e limite, consulta, comparação, execução inexistente, schema mais novo, formatação UTC, severidades, prefixo do Windows e os três renderizadores. A CLI ganhou teste de parsing para `record` e `compare`, incluindo `--database` antes e depois da operação.
- **Correções durante a implementação:** `rusqlite::Type` passou a ser importado de `rusqlite::types`; a união de `source()` com tipos diferentes foi separada; colunas opcionais de linha passaram por `read_optional_usize`; transações usam uma cópia do caminho para evitar conflito de empréstimo; o Clippy apontou dois closures redundantes, corrigidos antes da verificação final.
- **Limitações conhecidas:** não há interface para apagar ou exportar o banco, nem compressão de histórico. A data é gravada em segundos, sem fuso horário configurável; a exibição converte para UTC. Comparações consideram apenas o `id` do achado: mudanças de severidade ou mensagem com o mesmo `id` são tratadas como persistentes, usando os dados da execução posterior.
- **Impacto:** 73 testes no total (63 no núcleo e 10 na CLI), todos passando. Nenhuma interface desktop, extensão VS Code ou uso de IA foi adicionado.

## D7.1 — Projetos fictícios de aceitação do núcleo

- **Decisão:** manter `tests/fixtures/projects/simple` (descoberta) e `tests/fixtures/projects/dependencies` (violações intencionais) e criar `tests/fixtures/projects/clean` (arquitetura correta) e `tests/fixtures/projects/partial` (estrutura parcialmente desconhecida). A Etapa 7 passa a ter três projetos para os três cenários exigidos pelo plano, além da fixture antiga de descoberta.
- **Motivo:** `simple` inclui `crates/root.rs` fora dos módulos declarados e, portanto, produz `file.unmapped`; usá-lo como exemplo de “arquitetura correta” seria um falso positivo documentado como sucesso.
- **Alternativa rejeitada:** reescrever `simple` para eliminar o arquivo fora de módulo — isso apagaria o cenário usado pelos testes da Etapa 3 e esconderia uma limitação real da descoberta.
- **Impacto:** `clean` tem três módulos, três pacotes Cargo e três arestas permitidas, sem achados e sem limitações. `partial` tem dependência não resolvida, path sobreposto, arquivo fora de módulo, módulo sem arquivos e um arquivo com erro de parse. Dois testes novos em `audit.rs` verificam as contagens, os identificadores e o determinismo. Nenhuma dependência foi adicionada.

## D7.2 — Avaliação de falsos positivos, mensagens e evidências

- **Decisão:** não alterar regras, severidades, mensagens nem evidências. Registrar o resultado da revisão: zero achados em `clean`; em `dependencies`, os três ciclos são caminhos fechados reais no grafo; em `partial`, o indecidível vira `needs_review` e o arquivo inválido vira apenas limitação.
- **Motivo:** a auditoria deve separar fatos, heurísticas e sugestões. Transformar um arquivo que não faz parse em erro, ou deduplicar ciclos sobrepostos sem mudar o algoritmo, faria o relatório parecer mais limpo sem novo evidência.
- **Alternativa rejeitada:** silenciar ciclos que se sobrepõem ou inventar uma linha de evidência para achados de arquivo — o primeiro esconderia arestas verificáveis e o segundo criaria informação não observada.
- **Impacto:** a sobreposição de ciclos permanece como limitação conhecida da Etapa 5 (D5.4), sem duplicação aqui. Achados de arquivo continuam sem linha, com evidência descritiva explícita. `confidence` segue nulo porque todas as regras são determinísticas. Nenhuma correção de código foi necessária nesta etapa.

## D7.3 — Pipeline pela CLI e formato do contrato

- **Decisão:** executar `audit` nos três projetos em texto, JSON e Markdown, além de `history record` e `history compare`, mantendo o schema v1 do contrato sem campos novos.
- **Motivo:** o critério da Etapa 7 é provar que o motor é útil sem interface. Os três cenários foram expressos com os campos já implementados; o path sobreposto de `partial` foi detectado como ambiguidade, sem precisar de nova opção de configuração.
- **Alternativa rejeitada:** adicionar campos para marcar áreas “conhecidas” ou “desconhecidas” no contrato — seria antecipação de funcionalidade sem implementação e exigiria nova versão de schema sem necessidade demonstrada.
- **Impacto:** códigos observados: `clean` = `0`, `dependencies` = `1`, `partial` = `1`, `compare` = `0`; a comparação entre `clean` e `partial` listou quatro achados novos e nenhum resolvido. `README.md` registra os cenários e os comandos. 75 testes no total (65 no núcleo e 10 na CLI), todos passando. Nenhuma interface desktop, extensão VS Code ou IA foi adicionada.

## D8.1 — Adaptador Tauri e limites do núcleo

- **Decisão:** criar `apps/desktop/src-tauri` como adaptador fino. Os comandos `audit_project`, `analyze_project`, `read_contract`, `history_list` e `history_record` chamam funções públicas de `auditor-core` e devolvem os modelos já serializados pelo núcleo.
- **Motivo:** o critério da Etapa 8 é oferecer a mesma auditoria da CLI; qualquer regra reimplementada no frontend criaria divergência. A serialização de `Analysis` e dos modelos de histórico foi adicionada ao núcleo apenas para transporte, sem alterar regras nem o schema v1 do contrato.
- **Alternativa rejeitada:** implementar filtros de severidade e classificação de arestas em TypeScript — isso duplicaria a semântica das regras e exigiria testes paralelos sem benefício.
- **Impacto:** `auditor-core` passou a derivar `Serialize` para os tipos de análise/histórico. Nenhuma função de regra foi exportada a mais. Os testes do adaptador comparam seus resultados diretamente com o núcleo.

## D8.2 — Dependências, permissões e ícone

- **Decisão:** usar Tauri `2.12.7`, `tauri-build` `2.7.6`, `tauri-plugin-dialog` `2.8.4`, `@tauri-apps/api` `2.12.5`, `@tauri-apps/cli` `2.12.6`, `@tauri-apps/plugin-dialog` `2.8.4`, React/React DOM `19.3.1`, TypeScript `7.0.2` e Vite `8.3.8`. A janela recebe somente `core:default` e `dialog:allow-open`, com CSP explícito.
- **Motivo:** o plano fixa Tauri 2 + React/TypeScript; o seletor nativo atende “selecionar e abrir um projeto”. O plugin de diálogo é a menor dependência que evita digitar o caminho manualmente. Não foi concedido acesso de filesystem à interface: arquivos e banco são lidos exclusivamente pelos comandos Rust.
- **Alternativa rejeitada:** conceder `core:allow-*` irrestrito ou plugin de filesystem — ampliaria a superfície de ataque sem necessidade. Também não foi instalada ferramenta global: `npm install` e o build ocorreram dentro de `apps/desktop`.
- **Impacto:** `Cargo.lock` e `package-lock.json` passaram a existir/ser atualizados. Um ícone ICO mínimo foi criado localmente com `System.Drawing` porque `tauri-build` exige `icons/icon.ico` no Windows; nenhuma imagem foi baixada ou gerada por serviço externo.

## D8.3 — Telas e modelo de dados da interface

- **Decisão:** implementar seleção de pasta, auditoria com resumo e filtros, grafo SVG, visualizador do contrato, histórico com registro e configuração do banco. O contrato permanece somente leitura e não há execução em segundo plano.
- **Motivo:** são as telas iniciais do plano que podem ser entregues sem nova funcionalidade de escrita. Um editor de contrato exigiria confirmação explícita e cálculo de diffs, fora do critério mínimo; visualizar já satisfaz “editor ou visualizador”.
- **Alternativa rejeitada:** adicionar biblioteca de gráficos ou editor de texto — dependências sem necessidade, além de deslocarem responsabilidade da interface para bibliotecas externas.
- **Impacto:** a UI consome apenas `snake_case` já usado pela CLI. Nós tracejados no grafo indicam destinos fora do contrato. A comparação de histórico ficou apenas na CLI, onde já era testada.

## D8.4 — Verificação e limitações permanentes

- **Decisão:** encerrar a Etapa 8 com build de release e smoke de inicialização, sem instalar ferramentas e sem antecipar a Etapa 9.
- **Motivo:** a etapa deve provar que a aplicação abre e usa o mesmo motor, não que gera um instalador de distribuição.
- **Alternativa rejeitada:** marcar a etapa como concluída apenas com `cargo check` — não provaria que o frontend e o contexto Tauri funcionam juntos.
- **Impacto:** `cargo check`, `cargo test` (78 testes: 65 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm install`, `npm run build` e `npm run tauri build` passaram. `target/release/auditor-desktop.exe` foi gerado e iniciou. Limitações registradas no `README.md`: banco relativo ao cwd, bundle desativado, ícone técnico e ausência de comparação na interface. Extensão VS Code não foi iniciada.

## D9.1 — CLI como interface pública da extensão

- **Decisão:** a extensão executa `vibe-coding-auditor audit <pasta> --format json` e converte o `AuditReport` recebido. O código de saída `1` é tratado como “achados presentes”, não como falha; somente `>1` gera mensagem de erro.
- **Motivo:** o plano exige consumir uma interface pública e estável sem criar daemon. A CLI já é testada, determinística e produz exatamente o modelo usado pela desktop.
- **Alternativa rejeitada:** FFI/N-API com o núcleo ou reimplementação das regras em TypeScript — acrescentariam build nativo, acoplamento de versão ou risco de divergência dos achados.
- **Impacto:** nenhuma mudança no núcleo, na CLI ou na desktop. O setting `vibeCodingAuditor.cliPath` permite apontar o executável; o padrão procura no `PATH` e em `target/release`/`target/debug` da pasta aberta.

## D9.2 — Conversão de achados em diagnósticos

- **Decisão:** mover a conversão para `src/model.ts`, sem importar a API do VS Code, e converter os descritores em `vscode.Diagnostic` somente em `extension.ts`. Severidades: `error` → `Error`; `warning` e `needs_review` → `Warning`; `info` → `Information`.
- **Motivo:** `needs_review` exige decisão humana, mas não é violação comprovada; classificá-lo como `Error` inventaria certeza. Isolar a conversão permite testá-la com `node --test`, sem baixar a host do VS Code.
- **Alternativa rejeitada:** criar biblioteca de teste do VS Code ou gerar diagnósticos diretamente na extensão — dependência pesada e lógica sem teste automatizado simples.
- **Impacto:** 4 testes novos cobrem mapeamento de severidade, conversão de linhas 1-based, mensagem rastreável e preservação de arquivo/regra. O `rule_id` vira `Diagnostic.code`; a mensagem contém `id`, evidência, expectativa e sugestão.

## D9.3 — Estado da última auditoria e navegação

- **Decisão:** detectar `auditor.toml` na ativação, publicar o context key `vibeCodingAuditor.projectConfigured`, mostrar state na barra de status e restaurar o último relatório do `workspaceState`. O comando “Abrir achado...” usa o seletor rápido e posiciona o cursor na linha do achado.
- **Motivo:** os primeiros funções do plano pedem detecção do projeto configurado, estado da última auditoria e abertura do arquivo/linha. Guardar apenas o relatório evita reler ou copiar arquivos do projeto.
- **Alternativa rejeitada:** varredura em segundo plano ou cache do conteúdo integral dos arquivos — ambos contrariam os limites de somente leitura e privacidade.
- **Impacto:** o estado local contém metadados, contagens, achados e linhas de evidência já presentes no relatório. “Ocultar diagnósticos” limpa apenas o painel; a auditoria permanece registrada no estado. Nenhum arquivo do projeto é modificado.

## D9.4 — Verificação, empacotamento e limitações

- **Decisão:** encerrar a Etapa 9 com testes de conversão, geração do `.vsix` e smoke da sequência CLI → JSON → diagnósticos nas três fixtures, sem instalar extensões na instância do usuário.
- **Motivo:** o critério é que os achados no VS Code correspondam aos relatórios do núcleo. A verificação direta do contrato de dados prova isso sem automatizar a host gráfica.
- **Alternativa rejeitada:** instalar a extensão com `code --install-extension` apenas para o teste — mudaria o ambiente do usuário sem necessidade. Também foi rejeitado o downgrade de `@vscode/vsce` para `3.9.1`: aumentou os alertas de `2` para `3` e trouxe um módulo nativo.
- **Impacto:** `npm test` passou com 4 testes; `npm run package` gerou `vibe-coding-auditor-0.1.0.vsix` com 7 arquivos. Smoke: `clean` = 0 achados/0 diagnósticos/saída `0`; `dependencies` = 7/7/saída `1`; `partial` = 4/4/saída `1`. `npm audit --omit=dev` retornou `0`; `npm audit` mantém 2 alertas moderados apenas no `@vscode/vsce` de desenvolvimento. Não há testes automatizados dentro da host do VS Code; extensão não foi instalada; Etapa 10 não foi iniciada.

## D10.1 — Primeira expansão da Etapa 10: TypeScript e JavaScript

- **Decisão:** adicionar ao núcleo um analisador determinístico de `.ts`, `.tsx`, `.js` e `.jsx` para `import`, `export ... from`, `require(...)` e `import(...)`. Imports relativos são resolvidos lexicalmente contra os arquivos do inventário; quando o arquivo resolvido pertence a um módulo do contrato, a aresta segue as mesmas regras do Rust. Especificadores sem caminho relativo que coincidem com nome de módulo viram `needs_review`; os demais são tratados como pacote externo, fora do escopo arquitetural.
- **Motivo:** é o primeiro item da Etapa 10 com caso de uso real e imediato: o projeto já tem extensão VS Code e frontend desktop em TypeScript, mas a auditoria só via arquivos `.rs`. A descoberta já guarda extensões e as regras consomem arestas genéricas, então o risco de mudança é concentrado na análise.
- **Alternativas rejeitadas:** (a) `tree-sitter`/`swc` — nova dependência nativa ou parser complexo sem necessidade para reconhecer declarações de import; (b) reimplementar regras na extensão ou na UI — violaria D8.1 e dividiria a verdade; (c) Python, IA, Git diff, agendamento, duplicação e regras personalizadas — itens posteriores do plano, sem caso de uso validado nesta etapa.
- **Impacto:** nenhum novo crate; nenhuma mudança no contrato (`auditor.toml` continua o mesmo, com globs explícitos por extensão); CLI, desktop e extensão continuam consumindo o mesmo `AuditReport`. Novos rótulos deixam de dizer “Rust” onde passam a valer também para TS/JS. Limitações documentadas: sem `tsconfig.paths`, workspaces npm ou resolução de pacotes; `import type` é tratado como acoplamento; templates e código dinâmico são ignorados com limitation; `packages` continua contando apenas pacotes Cargo. A política do scanner foi refinada em `E2.5`.

## D10.2 — Escopo encerrado da Etapa 10 e verificação

- **Decisão:** encerrar esta rodada da Etapa 10 somente com TypeScript/JavaScript. Python, complexidade, duplicação, código sem referências, regras personalizadas, sugestões com IA, integração Git e auditoria agendada ficam diferidos até existir caso de uso real e critérios de desempenho medidos.
- **Motivo:** o plano exige caso de uso, testes e impacto conhecidos por recurso. O projeto já tem frontend em TypeScript; os outros itens não têm necessidade demonstrada e alguns (IA no núcleo) contrariam regra explícita do plano.
- **Alternativa rejeitada:** implementar vários recursos da Etapa 10 de uma vez — violaria “uma etapa por solicitação” e impediria avaliar regressões de cada mudança.
- **Impacto:** a fixture `typescript` produziu 8 arestas, 4 achados (2 `dep.not_allowed`, 1 `dep.unresolved`, 1 `file.unmapped`) e 1 limitação, com saída `1`. A conversão da extensão gerou 4 diagnósticos (2 `error`, 2 `warning`). `cargo check`, `cargo test` (84 testes: 71 no núcleo, 10 na CLI e 3 no desktop), `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão e `npm run build` da desktop passaram. As contagens das fixtures `clean`, `dependencies` e `partial` permaneceram inalteradas.

## R1 — Revisão final do plano

- **Decisão:** antes de qualquer recurso adicional, conferir o estado final das Etapas 0–10 sem alterar código. O usuário autorizou explicitamente cinco trabalhos seguintes, um por vez: revisão, Python, complexidade/concentração, duplicação/referências e Git diff.
- **Motivo:** o plano termina na Etapa 10; cada item seguinte precisa de caso de uso, impacto conhecido e verificação própria. A revisão estabelece a linha de base e evita atribuir regressões a mudanças futuras.
- **Alternativa rejeitada:** iniciar vários recursos em paralelo — dificultaria atribuir falhas e contrariaria a regra de uma mudança por vez.
- **Impacto:** `--help` e `--version` retornaram `0`; `init` criou o modelo e recusou sobrescrever com `2`; `scan`, `analyze` e `audit clean` retornaram `0`; `audit dependencies` e `audit partial` retornaram `1` pela presença de achados; `history record/list/show/compare` retornaram `0` com banco temporário. Nenhuma ferramenta global foi instalada e nenhum arquivo do repositório foi alterado nesta revisão.

## R2 — Suporte determinístico a Python

- **Decisão:** estender o núcleo com analisador próprio para `.py`, reaproveitando `record_dependency`, o contrato e as regras existentes. O usuário autorizou este item depois da revisão final.
- **Motivo:** Python é uma linguagem comum em projetos de pequeno e médio porte e permite validar que o modelo de análise não está preso a Rust/TypeScript. O caso de uso é auditar a arquitetura de um projeto Python sem executar seu código.
- **Alternativas rejeitadas:** parser completo (nova dependência e complexidade desnecessária); regras em CLI/UI/extensão (duplicação); IA (vedada no núcleo determinístico).
- **Impacto:** nenhuma dependência nova, nenhum campo novo em `auditor.toml` e nenhuma alteração no contrato JSON consumido por desktop/extensão. Foram adicionados o módulo `analysis/python.rs` e a fixture `tests/fixtures/projects/python`. A fixture produziu 4 arestas, 3 achados (2 `dep.not_allowed` e 1 `file.unmapped`) e 1 limitação, com saída `1`. Limitações explícitas: somente `import` e `from ... import` em uma linha; comentários e strings triplas são mascarados; `import` absoluto sem sufixo de arquivo, imports relativos fora da raiz, `import *`, pacotes pip e resolução dinâmica não geram arestas. `packages` continua contando apenas pacotes Cargo. Verificação complementar: `cargo check` retornou `0`; `cargo test` passou com 89 testes (76 no núcleo, 10 na CLI e 3 no desktop); `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; smoke CLI→JSON→extensão converteu 3 achados em 3 diagnósticos (2 `error` e 1 `warning`); `npm test` da extensão e `npm run build` da desktop passaram.

## R3 — Concentração de dependências

- **Decisão:** adicionar a regra determinística `module.concentration` no núcleo. Um módulo vira `needs_review` quando possui 3 ou mais módulos de origem ou 3 ou mais módulos de destino distintos entre as arestas resolvidas.
- **Motivo:** um módulo pode respeitar todas as dependências permitidas e ainda concentrar acoplamento. O caso de uso é revisar pontos de concentração antes de aceitar uma arquitetura; a regra usa somente o grafo já calculado, sem ler arquivos novamente.
- **Alternativas rejeitadas:** limiar baseado em maioria das arestas (matematicamente não distingue um hubs quando todas as arestas incidem sobre ele); limite configurável no contrato (mudaria o schema sem necessidade); IA e regras duplicadas na interface.
- **Impacto:** custo `O(M + E)` por auditoria, nenhuma nova dependência e nenhuma mudança no contrato JSON. Fixture `tests/fixtures/projects/concentration`: 3 arestas permitidas, 1 achado `module.concentration:application` com severidade `needs_review`, 0 erros, 0 avisos e 0 limitações. As fixtures `clean`, `dependencies`, `partial`, `typescript` e `python` mantiveram as contagens anteriores porque nenhum módulo atinge 3 origens ou destinos distintos. Verificação complementar: `cargo check` retornou `0`; `cargo test` passou com 90 testes (77 no núcleo, 10 na CLI e 3 no desktop); `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; smoke CLI→JSON→extensão converteu 1 achado em 1 diagnóstico `warning`; `npm test` da extensão passou.

## R4 — Duplicação exata de linhas

- **Decisão:** adicionar `code.duplicated` como revisão no núcleo. A análise coleta, durante a leitura já existente, linhas com 40 ou mais caracteres repetidas em mais de um arquivo incluído, ignorando comentários e linhas de import/export/use.
- **Motivo:** o caso de uso é sinalizar copy-paste que as regras de dependência não enxergam. A comparação exata evita inventar semântica e mantém a auditoria somente leitura.
- **Alternativas rejeitadas:** IA ou comparação semântica (vedadas no núcleo determinístico); detecção de blocos ou normalização de nomes (mais falsos positivos e custo maior); parâmetro configurável no contrato (mudaria o schema sem necessidade).
- **Impacto:** nenhuma dependência nova e nenhuma alteração no contrato JSON. O campo interno `Analysis.duplicate_lines` não é exposto no relatório; cada achado traz apenas uma linha como evidência, não o conteúdo integral dos arquivos. Identificadores usam hash FNV-1a implementado no próprio núcleo. Fixture `tests/fixtures/projects/duplication`: 1 aresta, 1 revisão, 0 erros, 0 avisos e 0 limitações. Limitações: apenas linhas idênticas; blocos, renomeações e trechos formatados de forma diferente não são detectados. As fixtures anteriores mantiveram as contagens após excluir imports repetidos da comparação. Verificação complementar: `cargo check` retornou `0`; `cargo test` passou com 92 testes (79 no núcleo, 10 na CLI e 3 no desktop); `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; smoke CLI→JSON→extensão converteu 1 achado em 1 diagnóstico `warning`.

## R5 — Código sem referências

- **Decisão:** adicionar `module.unreferenced` como revisão para módulos declarados que possuem arquivos incluídos, mas nenhuma aresta resolvida incidente.
- **Motivo:** o caso de uso é descobrir código isolado que ainda não participa da arquitetura. O grafo já contém origem e destino por módulo, então a regra não lê arquivos nem introduz heurística de texto.
- **Alternativas rejeitadas:** detecção por arquivo (o modelo não registra arquivo de destino); remoção automática (violaria somente leitura e decisão humana); IA (vedada no núcleo).
- **Impacto:** nenhuma dependência nova e nenhum campo novo no contrato. Fixture `tests/fixtures/projects/unreferenced`: 1 aresta, 1 revisão `module.unreferenced:orphan`, 0 erros, 0 avisos e 0 limitações. A fixture `partial` passou de 4 para 5 achados porque `shared` tem arquivos e nenhuma aresta; as demais fixtures mantiveram as contagens. Limitações: módulo é um proxy para código isolado, não prova de código morto; módulos sem arquivos continuam apenas com `module.no_files`. Custo `O(M + E)`. Verificação complementar: `cargo check` retornou `0`; `cargo test` passou com 93 testes (80 no núcleo, 10 na CLI e 3 no desktop); `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; smoke CLI→JSON→extensão converteu 1 achado em 1 diagnóstico `warning`; `npm test` da extensão e `npm run build` da desktop passaram.

## R6 — Integração com Git diff

- **Decisão:** aceitar um patch unificado salvo de `git diff` em `audit <raiz> --diff <arquivo>`. O núcleo interpreta o patch e recorta o relatório para achados em arquivos ou linhas alteradas.
- **Motivo:** revisar apenas o que mudou reduz ruído em auditorias incrementais sem criar uma segunda implementação das regras. O filtro é uma visão parcial do relatório completo, não uma nova classificação de severidade.
- **Alternativas rejeitadas:** executar `git` diretamente na CLI (binário externo e saída dependente de configuração local); filtrar na extensão ou no desktop (duplicação de escopo fora do núcleo); IA (vedada no núcleo determinístico).
- **Impacto:** nenhuma dependência nova, nenhum campo novo em `auditor.toml` e nenhuma mudança no esquema JSON. Achados com linha conhecida entram quando a linha foi adicionada; achados sem linha ou com caminho de módulo entram quando o arquivo alterado corresponde ao caminho. O resumo é recalculado e uma limitação avisa que achados fora do diff foram omitidos. Custo linear no tamanho do patch e no número de achados. Limitações: diffs combinados e caminhos citados com escapes não são interpretados; renomeações puras usam o destino; linhas removidas não têm linha atual correspondente.
- **Verificação:** smoke com patch real de `git diff` sobre uma cópia temporária de `dependencies` reduziu 7 achados para 3, todos em `crates/application/src/forbidden.rs`; a conversão CLI→JSON→extensão gerou 3 diagnósticos (1 `error` e 2 `warning`). Testes unitários cobrem intervalos adicionados, arquivo excluído, renomeação, hunk malformado, recorte por linha, por ausência de linha e por padrão de módulo. Após revisão, dois problemas foram corrigidos com testes de regressão: linhas de código começando com `--- ` ou `+++ ` dentro de um hunk eram confundidas com cabeçalhos de arquivo, e arquivos sem linhas adicionadas incluíam todos os achados com linha. Verificação complementar: `cargo check` retornou `0`; `cargo test` passou com 101 testes (88 no núcleo, 10 na CLI e 3 no desktop); `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; `cargo run -p auditor-cli -- audit --help` mostrou `--diff <ARQUIVO>`; `npm test` da extensão e `npm run build` da desktop passaram.

## R7 — Fechamento da primeira fase

- **Decisão:** encerrar a primeira fase do projeto sem alterar o schema do contrato, o banco ou o esquema JSON. Os pontos em aberto da seção 13 do plano e os recursos diferidos da Etapa 10 são resolvidos por decisão registrada, não por implementação.
- **Motivo:** o usuário confirmou explicitamente as três escolhas de escopo. O núcleo, a CLI, a aplicação desktop e a extensão já passam na verificação completa; abrir novas frentes agora aumentaria risco sem caso de uso validado.
- **Alternativas rejeitadas:** incluir `decisões aprovadas` no SQLite com migração v2 e confirmação explícita; implementar limite de tamanho de arquivo com campo novo no contrato; implementar regras personalizadas, sugestões com IA ou auditoria agendada nesta fase. IA no núcleo permanece vedada pelo princípio 4 do plano.
- **Impacto:** `decisões aprovadas` ficam fora do escopo da ferramenta: o auditor permanece somente leitura e a aprovação de uma mudança arquitetural é ação humana externa, registrada no projeto auditado. A regra de limite de tamanho permanece fora do MVP, conforme D5.2, sem limite inventado. Regras personalizadas, IA e agendamento seguem diferidos. Nenhuma dependência, campo de contrato, tabela, regra ou mensagem de relatório mudou. Documentação atualizada: `README.md` (status e nomenclatura do `init`), árvore da seção 4 do plano com `history/` e este registro.
- **Verificação:** após os ajustes documentais, `cargo check`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; `cargo test` passou com 101 testes (88 no núcleo, 10 na CLI e 3 no desktop); `npm test` da extensão passou com 4 testes; `npm run build` da desktop retornou `0`. `git diff --check` emitiu apenas os avisos LF/CRLF esperados do OneDrive.

## R8 — Remoção do plano inicial e limpeza documental

- **Decisão:** remover `PLANO_VIBE_CODING_AUDITOR.md` e as referências a ele, mantendo `docs/ROADMAP.md`, `docs/DECISIONS.md`, `README.md` e `AGENTS.md` como registro corrente. Esta decisão substitui a preservação registrada em `D0.1`.
- **Motivo:** o usuário pediu a remoção depois de a primeira fase ter sido declarada concluída. O plano era documento de partida; critérios concluídos, decisões tomadas e escopo remanescente já estão consolidados nos documentos corrente.
- **Alternativas rejeitadas:** manter o plano como arquivo histórico; reescrever seu conteúdo para refletir o estado final. A primeira manteria referência morta e a segunda duplicaria o roadmap.
- **Impacto:** nenhum código, contrato, schema ou teste mudou. Referências atualizadas em `AGENTS.md`, `README.md` e `docs/ROADMAP.md`; `D0.1` passou a indicar a substituição. Não havia arquivos vazios nem pastas vazias fora de diretórios de build; `*.vsix` e `target/` foram mantidos por serem artefatos de build, não arquivos inúteis do projeto.
- **Verificação:** após a remoção, `cargo check`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; `cargo test` passou com 101 testes (88 no núcleo, 10 na CLI e 3 no desktop); `npm test` da extensão passou com 4 testes; `npm run build` da desktop retornou `0`. Busca por `PLANO_VIBE_CODING_AUDITOR` encontrou apenas as duas referências históricas em `docs/DECISIONS.md` (`D0.1` e `R8`). `git diff --check` emitiu apenas os avisos LF/CRLF esperados do OneDrive.

## E2.1 — Cobertura honesta dos arquivos

- **Data:** 25/11/2026
- **Decisão:** adicionar `AnalysisCoverage` ao `Analysis` e `AuditReport`, com contagens de arquivos de código incluídos, analisados e não analisados, contagem por extensão sem analisador e o sinalizador `partial`. Extensões de código conhecidas sem analisador passam a gerar limitação explícita. Falhas de leitura ou parse também contam como código não analisado e mantêm suas limitações específicas.
- **Motivo:** o relatório anterior mostrava apenas o total de arquivos incluídos. Arquivos `.go`, `.rb` e outras linguagens sem analisador entravam no inventário e desapareciam sem aviso, podendo aparentar auditoria completa.
- **Alternativas rejeitadas:** (a) apenas acrescentar texto de limitação, sem dado estruturado — CLI e desktop teriam que interpretar mensagens; (b) ampliar `ReportSummary` — obrigaria alterar o esquema SQLite nesta etapa; (c) considerar código toda extensão desconhecida — geraria falsos positivos em documentos, dados e imagens; (d) eliminar silenciosamente extensões sem analisador do inventário — esconderia a lacuna de cobertura.
- **Impacto:** nenhum novo crate, nenhum campo em `auditor.toml`, nenhuma mudança nas regras ou no esquema JSON existente; `coverage` é aditivo e a extensão VS Code continua lendo `findings`. `analyze`, texto, Markdown, JSON e desktop mostram a mesma cobertura vinda do núcleo. O histórico continua gravando a limitação em texto, sem coluna para `coverage`; isso evita migração fora do item 2.7. Limitação restante: extensões fora da lista conhecida de código não são classificadas como código e, portanto, não geram aviso. Essa lista é explícita no núcleo e documentada no `README.md`.
- **Verificação:** testes escritos antes da implementação falharam por ausência de `coverage` e de `is_unsupported_code_extension`; após a implementação, `cargo fmt --check`, `cargo test --workspace` (105 testes: 92 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke na fixture `partial_coverage`: `audit` em texto, JSON e Markdown e `analyze` mostraram 5 arquivos incluídos, 4 de código, 2 analisados, `.go (1)`, `.rb (1)` sem analisador, 1 limitação, 0 achados e nenhuma violação inventada a partir dos arquivos não analisados. Nenhum item além do 2.1 foi iniciado.

## E2.2 — Mapeamentos ambíguos sem conclusão definitiva

- **Data:** 25/11/2026
- **Decisão:** `module_for_file()` só devolve um módulo quando exatamente um `path` do contrato corresponde ao arquivo. Com múltiplas correspondências, registra `AmbiguousModule` com os candidatos ordenados e não atribui módulo: não há arestas nem declarações `mod` originadas daquele arquivo. No analisador Python, um arquivo de destino pertencente a mais de um módulo gera aresta `Unresolved` com o especificador como destino e os candidatos ordenados na explicação, nunca aresta `Forbidden`. A mensagem de `dep.unresolved` passa a dizer que o destino não pôde ser associado com segurança a um único módulo.
- **Motivo:** antes desta mudança, a ordem das entradas em `auditor.toml` decidia qual módulo vencia. Um mesmo projeto podia produzir ou não produzir `dep.not_allowed`, apenas por reordenar o contrato. Uma indicação de ambiguidade não pode coexistir com uma conclusão arquitetural definitiva.
- **Alternativas rejeitadas:** (a) manter o primeiro módulo e apenas acrescentar revisão — contradiz o critério de independência da ordem; (b) criar novo `DependencyKind::Ambiguous` e novo `rule_id` — mudaria JSON, desktop e extensão sem necessidade; (c) ordenar os módulos antes de escolher o primeiro — ainda seria uma escolha arbitrária com severidade `error`; (d) descartar a ambiguidade — esconderia a limitação; (e) adicionar dependência externa para resolver paths — desnecessário para `matches_pattern`.
- **Impacto:** nenhum campo em `auditor.toml`, nenhuma nova dependência e nenhum novo `rule_id`. `D4.3` foi revisada para substituir a política antiga do primeiro módulo. As fixtures `ambiguous_mapping_a` e `ambiguous_mapping_b` repetem o mesmo projeto com ordem invertida e produzem relatórios idênticos: 0 erros, nenhum `dep.not_allowed` para os arquivos ambíguos, `file.ambiguous_module` com candidatos `domain, overlap` e `domain, entities`, e `dep.unresolved` para `domain.entity` com candidatos `domain, entities`. A redação de `expected` e `suggestion` de `dep.unresolved` também foi ajustada para descrever associação inequívoca, sem alterar o esquema do achado. Limitação restante: a regra continua exibindo módulos sem arestas incidentes como `module.unreferenced`; isso é um proxy, não prova de código morto, e permanece como revisão.
- **Verificação:** testes escritos antes da implementação falharam com `assertion left == right failed` em `module_for_file`, `classify_specifier` e na comparação dos relatórios das duas ordens. Após a implementação, `cargo fmt --check`, `cargo test --workspace` (107 testes: 94 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke real: `audit` nas duas fixtures em JSON produziu texto idêntico e nenhum erro definitivo. Nenhum item além do 2.2 foi iniciado nesta entrega.

## E2.3 — Dependências externas Rust e caminhos relativos

- **Data:** 25/11/2026
- **Decisão:** `read_cargo_workspace()` passa a ler `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[target.*.dependencies]` (e equivalentes de dev/build por plataforma) e `[workspace.dependencies]`, resolvendo `workspace = true` e aliases `package = "..."`. Os nomes são normalizados (`-` → `_`) e publicados em `WorkspaceInfo.declared_dependencies`, também impressos pelo `analyze`. Um `use` cuja raiz é uma crate declarada não gera aresta nem revisão; uma raiz ausente do contrato e dos manifests continua `Unresolved`. `self::`/`super::` deixam de ser descartados: se o primeiro segmento identifica outro módulo do contrato, a aresta vira `Allowed` ou `Forbidden`; se identifica a própria crate externa declarada ou um símbolo do mesmo módulo, não há aresta; sem arquivo de destino no inventário nem módulo ancestral compatível, há revisão `UnresolvedDependency`. `crate::self`/`crate::super` seguem a mesma regra relativa.
- **Motivo:** `use serde::Serialize;` e imports com alias apareciam como `dep.unresolved` embora a crate estivesse declarada no Cargo. Além disso, `super::` entre fronteiras do contrato era ignorado sem exame, permitindo acoplamento não registrado.
- **Alternativas rejeitadas:** (a) executar `cargo metadata` — dependeria de subprocess, rede e cache, contrariando a análise local e previsível de `D4.2`; (b) criar `DependencyKind::External` e novo `rule_id` — mudaria JSON, CLI, desktop, extensão e histórico sem necessidade; (c) tratar toda raiz desconhecida como externa — esconderia símbolos realmente desconhecidos; (d) continuar descartando todo `self`/`super` — deixaria travessias de fronteira sem registro; (e) adicionar crate para expandir manifests ou globs — desnecessário neste item.
- **Impacto:** nenhum campo em `auditor.toml`, nenhum `rule_id` novo, nenhuma nova dependência. `WorkspaceInfo` ganha campo aditivo; `ReportSummary.packages` continua contando somente pacotes do workspace. A resolução relativa usa diretórios e arquivos do inventário, não a árvore completa de módulos Rust, e nome que coincide com módulo do contrato é classificado pelo contrato; quando não há evidência, a revisão é explícita. `crate::` continua resolvendo pelo primeiro segmento após a raiz. Globs em `workspace.members` foram tratados em `E2.4`.
- **Verificação:** os testes de regressão foram escritos antes da implementação; o primeiro comando falhou em `declared_cargo_dependencies_are_read_from_all_tables` porque `declared_dependencies` ainda não existia. Após a implementação, `cargo fmt --check`, `cargo test --workspace` (115 testes: 102 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke na fixture `rust_dependencies`: `analyze` mostrou 8 crates declaradas e 5 arestas sem `serde`, `json_alias`, `alias_api`, `dev_helper`, `build_tool` ou `unix_only`; `audit` em texto mostrou 1 erro (`dep.not_allowed` de `domain` para `application` via `super::`), 1 aviso de ciclo, 3 revisões e nenhum achado para crates declaradas. Nenhum item além do 2.3 foi iniciado nesta entrega.

## E2.4 — Globs seguros em `workspace.members`

- **Data:** 25/11/2026
- **Decisão:** `read_cargo_workspace()` recebe o vetor de limitações e expande apenas um `*` no último componente de cada entrada de `[workspace].members`. São aceitos somente diretórios reais, não simbólicos, com `Cargo.toml`; os caminhos são ordenados, deduplicados e filtrados por `workspace.exclude` antes da leitura. Membros literais mantêm o comportamento anterior. `**`, `?`, classes de caracteres, `*` em componente intermediário, componentes `.`/`..`, diretório ausente e glob sem correspondência viram mensagem específica em `analysis.limitations`, visível em texto, JSON, Markdown e desktop; não viram erro operacional.
- **Motivo:** `members = ["crates/*"]` era concatenado como caminho literal e a análise falhava ao tentar ler `crates/*/Cargo.toml`, impedindo até mesmo um relatório parcial. O padrão é comum em workspaces reais e a expansão precisa permanecer local e previsível.
- **Alternativas rejeitadas:** (a) executar `cargo metadata` — subprocess, rede e cache, contrariando `D4.2`; (b) adicionar crate de glob — dependência nova para um subconjunto restrito que `matches_pattern` já resolve; (c) aceitar `**`, `?` e glob em componente intermediário neste item — exigiria varredura recursiva e nova semântica fora do escopo; (d) transformar glob não suportado em erro — perderia o relatório; (e) silenciar o padrão — esconderia a limitação; (f) seguir links simbólicos ou caminhos `..` — violaria a regra de não sair da raiz sem autorização.
- **Impacto:** nenhum campo em `auditor.toml`, nenhum `rule_id` novo, nenhuma dependência nova e nenhuma mudança no esquema JSON. `read_cargo_workspace()` ganha parâmetro interno para limitações; `WorkspaceInfo` permanece igual. A resolução só olha o diretório imediato do prefixo, sem recursão; `workspace.exclude` filtra somente os membros vindos de glob, preservando o comportamento dos membros literais. As contagens anteriores das fixtures e o consumo de `findings` pela extensão foram preservados.
- **Verificação:** os testes de regressão foram escritos antes da implementação; a primeira execução falhou com `InvalidFilename` ao tentar abrir `crates/**/Cargo.toml`, confirmando a falha relatada. Após a implementação, `cargo fmt --check`, `cargo test --workspace` (120 testes: 107 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke na fixture `workspace_glob`: `analyze` mostrou `alpha, beta, legacy`, `serde`, 1 aresta `application -> domain`, 2 de 2 arquivos analisados e 0 limitações; `audit` terminou com 0 achados. Smoke na fixture `workspace_glob_unsupported`: `audit` terminou com 0 achados, 0 pacotes e 4 limitações — `crates/**`, `crates/?lpha` e `../outside/*` como não suportados, e `empty/*` sem `Cargo.toml` — sem erro de leitura. Nenhum item além do 2.4 foi iniciado nesta entrega.

## E2.5 — Scanner TypeScript/JavaScript por declarações

- **Data:** 25/11/2026
- **Decisão:** o analisador passa a mascarar cada linha em um documento contínuo e a delimitar declarações pela profundidade de `()`, `[]` e `{}`, com limite em `;` ou quebra de linha quando a profundidade é zero. Comentários, strings, template literals e regex literals são mascarados antes da busca por palavras; `import.meta` é ignorado explicitamente. `import()` e `require()` com literal de aspas simples ou duplas continuam gerando aresta, como na fixture original; template literal como especificador e chamada sem literal geram limitação acionável. A evidência da aresta é a declaração reconstruída em uma linha, com a linha de início.
- **Motivo:** a leitura linha a linha perdia `import { ... } from "./domain"` multilinha e permitia que `import.meta.url`, seguido de uma string qualquer, virasse dependência. Regex com texto parecido com `import` também podia gerar aresta falsa.
- **Alternativas rejeitadas:** (a) adicionar parser TypeScript/`tree-sitter`/`swc` — dependência nova e escopo maior que o necessário para declarações de import; (b) continuar varredura linha a linha — não resolve os falsos positivos e negativos conhecidos; (c) descartar todo `import()`/`require()` — mudaria a fixture `typescript` e removeria informação já útil; (d) transformar regex ou template literal em erro — perderia o relatório; (e) criar novo `rule_id` ou campo no contrato — desnecessário para uma limitação de scanner.
- **Impacto:** nenhum novo crate, nenhum campo em `auditor.toml`, nenhum `rule_id` novo e nenhuma mudança no JSON consumido pela extensão ou no SQLite. A fixture `typescript` mantém 8 arestas, 4 achados (2 erros, 2 revisões) e 1 limitação. A fixture `typescript_scanner` produz 1 aresta permitida de `application` para `domain`, 0 achados e 3 limitações: pacotes npm, `import()` sem literal e especificador de template literal. A detecção de regex é heurística e depende de contexto como `=`, `(`, `return` e `case`; uma barra em contexto ambíguo pode ser tratada como divisão, então a limitação permanece documentada. `tsconfig.paths`, workspaces npm e resolução de pacotes seguem fora do escopo.
- **Verificação:** os testes de regressão foram escritos antes da implementação. A fixture nova falhou com `left: 2, right: 1` em `summary.edges`; o teste de `import.meta` devolveu `["domain", "domain"]` e o de regex devolveu um especificador em vez de nenhum. Após a implementação, `cargo fmt --check`, `cargo test --workspace` (126 testes: 113 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke: `audit` em `typescript` repetiu 8 arestas, 4 achados e 1 limitação; `analyze` em `typescript_scanner` mostrou `application -> domain [permitida]` em `src/application/multiline.ts:1` com evidência `import { Entity, Repository } from "../domain/entity";`, 0 revisões e 3 limitações; `audit` na mesma fixture terminou com 0 achados. Nenhum item além do 2.5 foi iniciado nesta entrega.

## E2.6 — Detecção de ciclos por componentes fortemente conectados

- **Data:** 25/11/2026
- **Decisão:** `find_cycles()` foi substituído por Tarjan iterativo em `find_cyclic_components()`, com pilha explícita e índices/lowlinks ordenados por `BTreeMap`/`BTreeSet`. Somente componentes com mais de um módulo, ou com autoaresta, viram achado. As arestas são indexadas por componente em uma única passagem; cada componente gera um único `dep.cycle`, o `id` usa os membros ordenados (`dep.cycle:application->domain->infrastructure`), a origem é a primeira aresta interna ordenada, e a evidência lista no máximo 8 arestas internas seguidas da contagem das restantes.
- **Motivo:** a enumeração de caminhos simples começava uma DFS em cada nó e podia crescer exponencialmente em grafos densos, além de produzir três avisos sobrepostos para o único componente de três módulos da fixture `dependencies`.
- **Alternativas rejeitadas:** (a) manter a enumeração e deduplicar depois — ainda pagaria o custo exponencial; (b) Kosaraju recursivo — duas passagens e profundidade de pilha proporcional ao número de módulos; (c) biblioteca externa de grafos — dependência nova para um algoritmo curto e determinístico; (d) novo `rule_id` ou campo no contrato — a regra existente continua suficiente; (e) limite de tempo estreito no teste — resultado flaky em CI.
- **Impacto:** nenhuma dependência nova, nenhum campo em `auditor.toml`, nenhum `rule_id` novo e nenhuma mudança no JSON consumido pela extensão ou no esquema SQLite. Tarjan e a indexação de arestas são `O(V + E)`; sem a indexação, a coleta de evidências poderia refazer `O(E)` por componente. O `id` dos componentes com 3+ módulos mudou; ciclos de dois módulos e autoarestas isoladas mantêm o formato anterior. Uma autoaresta dentro de um componente maior passa a ser coberta pelo achado do componente, sem `id` separado. A fixture `dependencies` passou de 7 para 5 achados (2 erros, 1 aviso, 2 revisões); `rust_dependencies` manteve 5 achados e 1 aviso de ciclo. As contagens e textos de histórico foram ajustados para o novo resumo.
- **Verificação:** os testes de regressão foram escritos antes da implementação; a fase vermelha mostrou `left: 3, right: 1` em `fixture_findings_are_classified`, `left: "dep.cycle:application->domain", right: "dep.cycle:application->domain->infrastructure"` em `cycle_evidence_traces_component_edges` e três IDs antigos em `one_finding_per_cyclic_component`. Após a implementação, `cargo fmt --check`, `cargo test --workspace` (129 testes: 116 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. O teste denso com 80 módulos e 6400 arestas terminou em menos de 5 s com exatamente 1 achado e evidência limitada a 8 arestas mais `+6392 aresta(s) adicionais no componente`; o teste com 600 componentes desconectados terminou com 600 achados e IDs ordenados. Smoke: `audit` em `dependencies` terminou com 5 achados, 1 aviso `dep.cycle:application->domain->infrastructure` e 5 arestas na evidência; `audit` em `rust_dependencies` manteve 5 achados e 1 aviso `dep.cycle:application->domain`. Nenhum item além do 2.6 foi iniciado nesta entrega.

## E2.7 — Motivos de exclusão no relatório

- **Data:** 25/11/2026
- **Decisão:** `AuditReport` ganhou o campo aditivo `skip_summary`, calculado no núcleo a partir de `Inventory.skipped`. O resumo traz `total`, `intentional`, `coverage_gaps` e uma lista `reasons` ordenada; cada reason contém `reason`, `label`, `count`, até 3 `examples` e `additional`. Diretórios ignorados por padrão, exclusões do contrato e arquivos fora de `include` são classificados como intencionais. Link simbólico, entrada inacessível e tipo não tratado são marcados como `reduces_coverage`, porque impedem a varredura de considerar uma entrada relevante. Texto, Markdown e desktop mostram as mesmas contagens e exemplos limitados; o JSON contém o modelo estruturado completo.
- **Motivo:** a descoberta já registrava seis motivos, mas o relatório final resumia tudo em `files_skipped`. Um diretório gerado ignorado, um arquivo fora do `include` e uma entrada inacessível pareciam equivalentes para quem lia o relatório.
- **Alternativas rejeitadas:** (a) manter apenas `files_skipped` — não resolve a transparência pedida; (b) listar todos os caminhos ignorados — pode produzir saídas enormes e foi explicitamente descartado; (c) calcular rótulos e categorias no desktop — duplicaria regras de apresentação fora do núcleo; (d) gravar `coverage` e `skip_summary` em colunas SQLite — ampliaria o item para migração e histórico, sem pedido nesta etapa; (e) criar `rule_id` novo — exclusão de escopo não é violação arquitetural.
- **Impacto:** nenhuma dependência nova, nenhum campo em `auditor.toml`, nenhum `rule_id` novo e nenhuma mudança no esquema SQLite. `SkipReason` passou a ser serializável e o JSON ganhou um campo aditivo; a extensão VS Code continua lendo `findings` como antes. As contagens das fixtures anteriores permaneceram: `dependencies` com 5 achados e `typescript` com 8 arestas, 4 achados e 1 limitação. A fixture `simple` passou a documentar 14 entradas fora do inventário: 10 ignoradas por padrão, 1 excluída e 3 fora do `include`, todas intencionais. O histórico continua sem colunas para cobertura estruturada ou motivos de exclusão; isso foi reavaliado e mantido fora do escopo.
- **Verificação:** o teste de regressão foi escrito antes da implementação e falhou com `assertion failed: text.contains("Motivos dos não incluídos: 14")`. Durante a implementação, o mesmo teste revelou que `auditor.toml` também é `NotIncluded` na fixture `simple`, então a expectativa de `fora do include` foi corrigida de 2 para 3 antes de prosseguir. Após a implementação, `cargo fmt --check`, `cargo test --workspace` (132 testes: 119 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke: `audit` em `simple` mostrou 14 entradas fora do inventário, com `padrão ignorado: 10`, `excluído: 1`, `fora do include: 3` e `+7 outros`; `--format json` devolveu `total: 14`, `intentional: 14`, `coverage_gaps: 0` e três motivos com séries limitadas a 3 exemplos; `--format markdown` repetiu o resumo. Testes unitários cobriram `link simbólico` e `inacessível` como `reduz cobertura`, além de 20 entradas com apenas 3 exemplos e `additional: 17`.
- **Revisão complementar (25/11/2026):** a verificação anterior cobria três motivos na fixture `simple` e os outros três apenas em testes sintéticos do renderizador. Foi acrescentado um teste em `audit.rs` que monta um inventário com os seis motivos e passa pelo `AuditReport::from_pipeline` real, conferindo as três saídas, a separação `6 intencionais` / `3 reduzem cobertura`, os rótulos estáveis e o limite de exemplos sem vazar o quarto caminho ignorado. No desktop, os textos do resumo saíram de `App.tsx` para `src/skipSummary.ts`; o teste Node usa apenas `node:test` e `node:assert`, sem dependência nova, e falhou primeiro com `error TS18003: No inputs were found in config file 'tsconfig.test.json'`. Alternativas rejeitadas na revisão: criar fixture com symlink/FIFO e permissão negada — não reproduzível de forma igual no Windows; abrir a janela Tauri para inspeção visual — não automatizável neste ambiente; duplicar a formatação em `App.tsx` — continuaria sem teste isolado. Impacto: nenhuma regra de auditoria foi movida para o frontend; o helper apenas formata dados já classificados pelo núcleo. Verificação final: `cargo fmt --check`, `cargo check`, `cargo test --workspace` (133 testes: 120 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm test` da desktop (2 testes), `npm test` da extensão (4 testes) e `npm run build` da desktop passaram. Smoke repetido em `simple` (14 não incluídos) e `dependencies` (5 achados) sem mudança de contagem. Limitações mantidas: nenhuma execução visual da janela Tauri e nenhum teste com symlink/FIFO reais. Nenhum item além do 2.7 foi iniciado nesta entrega.

## E2.8 — Reforçar a verificação automatizada

- **Data:** 26/11/2026
- **Decisão:** criar `.github/workflows/ci.yml` para `windows-latest`, disparado em push e pull request para `main`, com Rust `1.98.1` e Node `24.19.0`. O workflow instala as duas pastas Node com `npm ci`, executa `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm test` e `npm run build` da desktop e `npm test` da extensão. O workspace passa a declarar `rust-version = 1.98.1`, e os três crates herdam o valor. O teste com 600 componentes SCC ganhou limite de 10 s, ao lado do caso denso de 80 módulos/6400 arestas com limite de 5 s. Novos testes de regressão cobrem, em uma única varredura, ruído conhecido dos scanners TS/JS e Python sem criar arestas falsas.
- **Motivo:** a Etapa 2 havia sido validada manualmente no Windows, mas uma mudança futura ainda podia ser mesclada sem executar os verificadores. Além disso, os testes de ciclo e dos scanners estavam espalhados e não protegiam combinações de casos já conhecidas.
- **Alternativas rejeitadas:** (a) CI em Linux — não é a plataforma-alvo inicial; (b) `rust-toolchain.toml` — fixaria a toolchain para todo desenvolvedor e exigiria download local; preferi declarar `rust-version` e instalar `1.98.1` apenas no runner; (c) limite de tempo rígido e baixo para SCC — tornaria o teste instável em runners lentos; os limites são generosos e as asserções principais são determinísticas; (d) executar `npm run tauri build` no CI — `bundle.active` é `false` e instaladores permanecem fora do escopo; (e) adicionar cache de Cargo por ação de terceiros — sem necessidade para o volume atual de verificações.
- **Impacto:** nenhuma dependência nova em Cargo ou npm, nenhum campo em `auditor.toml`, nenhum `rule_id` novo e nenhuma mudança no esquema SQLite. O workflow não altera código de auditoria e não habilita empacotamento. As contagens das fixtures anteriores permaneceram: `simple` com 14 não incluídos, `dependencies` com 5 achados e `typescript` com 8 arestas, 4 achados e 1 limitação.
- **Verificação:** os testes de regressão e o limite de tempo foram escritos antes da implementação do CI e passaram em `cargo test -p auditor-core scanner_regression -- --nocapture` (2 testes) e `cargo test -p auditor-core many_cyclic_components -- --nocapture` (1 teste). Após a implementação, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (135 testes: 122 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm ci` nas duas pastas Node, `npm test` da desktop (2 testes), `npm run build` da desktop e `npm test` da extensão (4 testes) passaram localmente no Windows. O workflow foi revisado e sua primeira execução remota foi observada em 26/11/2026: `CI Windows` #1, commit `7a07286`, `status: completed`, `conclusion: success` em `windows-latest`, com todas as etapas do job `verify` aprovadas. A consulta foi feita pela API pública porque `gh` não está autenticado neste ambiente. Também não houve inspeção visual da janela Tauri nem teste com symlink/FIFO reais. Nenhum item posterior à Etapa 2 foi iniciado nesta entrega.

## E2.8 — Ajuste de CI após verificação remota

- **Data:** 26/11/2026
- **Decisão:** atualizar `actions/checkout` e `actions/setup-node` de `v4` para `v5` no workflow. As versões `v5` usam Node 24 e mantêm as entradas `node-version`, `cache` e `cache-dependency-path` já empregadas.
- **Motivo:** a execução remota #2 passou, mas anotou que as actions `v4` ainda target Node.js 20 e estavam sendo forçadas para Node.js 24 pelo runner. O aviso não falhava o job, mas deixaria o CI desatualizado em relação ao runtime do runner.
- **Alternativas rejeitadas:** manter `v4` e aceitar o aviso — o workflow funciona, mas a configuração fica defasada; saltar direto para versões majors mais novas do `setup-node` — sem necessidade, pois `v5` já resolve o aviso e preserva as entradas usadas.
- **Impacto:** alteração restrita a `.github/workflows/ci.yml`; nenhuma regra, contrato, JSON, SQLite ou dependência de projeto foi mudada. A execução #3 confirmou `Status: Success` e não exibiu mais a anotação de Node.js 20.
- **Verificação:** execuções [#1](https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/45811082223), [#2](https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/45811839648) e [#3](https://github.com/dioneyfroes-coder/VibeCodingAuditor/actions/runs/45813011665) observadas em 26/11/2026. `gh` não está autenticado neste ambiente e a API pública atingiu o limite de requisições; por isso o estado final foi lido pela página HTML do GitHub. O commit documental usa `[skip ci]` porque não há mudança de código a verificar. Nenhum item posterior à Etapa 2 foi iniciado.

## E3.1 — Dependências Cargo `path` do workspace

- **Data:** 26/11/2026
- **Decisão:** distinguir dependências externas declaradas de dependências `path` que apontam para pacotes do mesmo workspace. O caminho é resolvido lexicalmente dentro da raiz; o `path` herdado de `[workspace.dependencies]` usa a raiz do workspace como base, e o `path` direto de um manifesto de membro usa o diretório daquele manifesto. O diretório resolvido precisa corresponder a um membro conhecido do workspace e a um único módulo do contrato por prefixo literal do `path`; aliases `package = "..."` e `workspace = true` são considerados. O destino sem associação segura permanece visível como `Unresolved`, que é o mecanismo existente para produzir `dep.unresolved`, sem virar aresta `Allowed`/`Forbidden`.
- **Motivo:** a CLI e o backend desktop declaram `auditor-core` com `path`. Como toda crate declarada era descartada por `classify_use`, a relação real entre os crates do produto não aparecia no grafo. Antes de autoauditar o repositório, era necessário provar que o analisador não confundia essa dependência local com pacote externo nem a escondia.
- **Alternativas rejeitadas:** (a) continuar descartando toda crate declarada — repetiria o falso negativo; (b) transformar toda crate declarada em aresta — reintroduziria os falsos positivos corrigidos em `E2.3`; (c) executar `cargo metadata` — subprocess, rede e cache, contrariando `D4.2`; (d) seguir links simbólicos ou permitir `..` fora da raiz — violaria as regras de segurança; (e) escolher o primeiro módulo cujo padrão corresponda — reintroduziria dependência da ordem do contrato, proibida em `E2.2`; (f) criar novo `rule_id`, campo no contrato ou mudança no SQLite — desnecessário para o caso.
- **Impacto:** nenhuma dependência nova, nenhum campo em `auditor.toml`, nenhum `rule_id` novo e nenhuma mudança no esquema JSON de `findings`. `declared_dependencies` continua listando também externas e aliases, como antes. Limitações explícitas: módulos com curinga em componente intermediário não são associados a um pacote local; pacote local sem módulo no contrato, com múltiplos módulos ou com alias conflitante gera revisão; caminhos absolutos ou que tentam sair da raiz não são resolvidos; imports ainda dependem das regras já documentadas para `syn` e para os scanners TS/JS e Python.
- **Verificação:** a fixture `tests/fixtures/projects/workspace_local_dependencies` e os testes foram escritos antes da implementação; a fase vermelha teve dois testes falhando pela ausência da aresta local e da revisão, e um por expectativa incorreta sobre a ordem de `Analysis.modules`, ajustado para comparar apenas a classificação. Após a implementação, o smoke mostrou `cli -> core [permitida]` e `cli -> tools [não resolvida]`, com `serde` ignorado. O smoke também revelou que o `path` herdado estava sendo resolvido a partir do membro, gerando revisão de alias conflitante; a base foi corrigida e o teste `inherited_workspace_path_is_resolved_from_the_workspace_root` passou a proteger o caso. Verificação final: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (144 testes: 131 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; `analyze` na fixture retornou `0` e `audit` retornou `1` por duas revisões, sem erros ou avisos.

## E3.2 — Contrato de autoauditoria Rust-only

- **Data:** 26/11/2026
- **Decisão:** criar `auditor.toml` na raiz com três módulos: `core` sem dependências permitidas, `cli` e `desktop-backend` permitindo somente `core`. `scan.include` limita a autoauditoria aos diretórios `src` dos três crates Rust de produção; fixtures, UI React, extensão, `target` e `node_modules` ficam fora por exclusão intencional.
- **Motivo:** o plano pede um contrato desenhado a partir da árvore real, não copiado do exemplo. A fronteira TypeScript ↔ Tauri não é observável pelo grafo atual; incluí-la sem evidência de comunicação produziria um contrato que a ferramenta não consegue verificar.
- **Alternativas rejeitadas:** incluir UI e extensão como módulos — ampliaria o contrato sem arestas verificáveis; usar módulos internos do núcleo — perderia o mapeamento simples de `auditor-core` para `core` e exigiria paths sobrepostos; copiar `auditor.example.toml` — não reflete a arquitetura real.
- **Impacto:** nenhum campo novo em `auditor.toml` v1, nenhuma regra de auditoria alterada e nenhum arquivo de código modificado. `responsibility` continua sendo descrição humana; a decisão final sobre a fronteira frontend permanece com o usuário.
- **Verificação:** contrato submetido como `0a1eac4`; `scan .` inventariou 15 arquivos Rust e 47 entradas não incluídas, todas classificadas como intencionais. `analyze .` mostrou os três módulos e os três pacotes Cargo esperados.

## E3.3 — Resolução de `crate::` e autoauditoria

- **Data:** 26/11/2026
- **Decisão:** resolver caminhos `crate::` contra o inventário e o contrato antes de aplicar a regra do primeiro segmento. A raiz de fontes é inferida do componente `src` do arquivo auditado; os prefixos mais longos são testados primeiro. Um arquivo que pertence ao mesmo módulo não gera aresta; um arquivo de outro módulo gera `Allowed`/`Forbidden`; ambiguidade vira revisão; sem arquivo correspondente, prevalece o comportamento anterior. O contrato de autoauditoria permanece Rust-only.
- **Motivo:** a primeira autoauditoria produziu 54 achados `dep.unresolved`, dos quais 46 eram imports internos como `use crate::analysis` dentro de `core`. Isso impediria declarar a Etapa 3.1 concluída com confiança, porque falsos positivos dominavam o relatório.
- **Alternativas rejeitadas:** manter o ruído e registrar apenas como limitação — a autoauditoria não provaria as relações principais; transformar `core` em vários módulos internos — paths sobrepostos e perda do mapeamento de dependência `path`; excluir módulos de teste — não resolveria os imports de produção; usar `syn` com localizações de span — exigiria nova feature/dependência e não foi necessário para a associação por arquivo.
- **Impacto:** nenhuma dependência nova, nenhum `rule_id`, nenhum campo em `auditor.toml` v1 e nenhuma mudança no SQLite ou no JSON de `findings`. Limitações explícitas mantidas: `super::` que aponta para símbolo do próprio módulo continua sem associação e vira revisão; a evidência de alguns `use` agrupados aparece deslocada em uma linha; `code.duplicated` detecta repetição exata de linha, não semântica. Fronteiras React ↔ Tauri e CLI ↔ extensão continuam fora do grafo e não são declaradas como verificadas.
- **Verificação:** a fixture `tests/fixtures/projects/crate_module_resolution` e os testes foram escritos antes da implementação. O teste interno falhou com `exit code 101`; os testes de fallback (`crate::unknown`), `super::`, `crate::self` e dependência `path` passaram. Após a correção, `analyze .` retornou `0` com 17 arestas (9 `permitidas`, 0 `proibidas`, 8 `não resolvidas`) e nenhum `use crate::*` não resolvido; `audit .` e as exportações retornaram `1` por 62 revisões, com 0 erros, 0 avisos e 0 limitações. Verificação final: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (147 testes: 134 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check`, `npm ci`/`npm test`/`npm run build` da desktop e `npm ci`/`npm test` da extensão retornaram `0`; o `git diff --check` emitiu apenas o aviso LF/CRLF esperado do OneDrive. Relatório integral em `docs/validation/etapa-3/self-audit.md`.

## E3.4 — Identidade dos achados fora da linha

- **Data:** 26/11/2026
- **Decisão:** `dep.not_allowed` e `dep.unresolved` deixam de incluir a linha no `Finding.id`. A identidade passa a ser o vínculo arquitetural: regra, origem, destino e arquivo, com o destino dispensável quando a própria regra já o define. Linha e evidência permanecem nos campos de localização e justificativa. Achados com o mesmo vínculo no mesmo arquivo são deduplicados de forma determinística, preservando a primeira ocorrência após ordenação por `id`, caminho, linha e evidência. IDs legados com sufixo `:<linha>` são normalizados somente em `history compare`, sem reescrever o banco.
- **Motivo:** inserir comentários antes de uma dependência proibida mudava o ID e fazia o histórico reportar um achado resolvido e outro novo sem mudança arquitetural. A auditoria do código confirmou que apenas essas duas regras usavam a linha como parte da identidade; as demais já eram estáveis por arquivo, módulo ou componente.
- **Alternativas rejeitadas:** manter a linha e aceitar o falso `resolved`+`new` — contrariaria o objetivo da etapa; usar a evidência textual crua como discriminador — qualquer reforço de estilo criaria nova identidade; usar índice de ocorrência — inserir uma importação anterior deslocaria todos os índices; criar coluna SQLite para chave de identidade — mudaria o modelo persistido sem necessidade, pois a normalização na comparação basta; promover cada importação repetida a dois achados — duas importações do mesmo destino no mesmo arquivo são a mesma violação lógica.
- **Impacto:** o formato do `id` mudou para duas regras, mas os doze campos do JSON, os `rule_id`, o contrato v1 e o schema SQLite permaneceram. O `UNIQUE (run_id, finding_id)` continua válido porque a deduplicação acontece antes da persistência. `history show` devolve o ID exatamente como gravado; bancos antigos continuam legíveis e comparáveis, e a normalização exige a contagem exata de componentes do formato legado para não confundir um diretório terminado em dígito com sufixo de linha. A extensão continua lendo `rule_id` como código e `line_start`/`evidence` para localização e mensagem, sem regras próprias. Limitação explícita: não há promessa de identidade através de renomeação de arquivo.
- **Verificação:** a fixture `tests/fixtures/projects/finding_identity` e os testes foram escritos antes da correção. A fase vermelha teve `inserted_comment_lines_do_not_create_a_new_finding` e `repeated_forbidden_binding_is_reported_once` com `exit code 101`; `changed_evidence_without_new_binding_keeps_identity` já passava e ficou protegido. O teste de ID legado falhou primeiro por expectativa incorreta no gerador de relatório antigo — ele acrescentava linha a regras que nunca a tiveram — e passou após o gerador ser limitado às duas regras de dependência. Smoke na CLI gravou `before` e `after` com o mesmo `id`, local `src/lib.rs:1` e `src/lib.rs:4`, e `history compare 1 2` retornou 0 novos, 0 resolvidos e 2 persistentes. Verificação final: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (154 testes: 141 no núcleo, 10 na CLI e 3 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `git diff --check` e `npm --prefix extensions/vscode test` retornaram `0`; o `git diff --check` emitiu apenas o aviso LF/CRLF esperado do OneDrive.

## E3.5 — Validação em alvo sintético fora do auditor

- **Data:** 26/11/2026
- **Decisão:** criar, sob pedido explícito do usuário, o projeto falso `validation/targets/fake-order-service` para a Etapa 3.3, com contrato v1 próprio e violações plantadas. Registrar a execução como validação sintética, não como prova em projeto real de terceiros. Não alterar o motor: a evidência prioriza a proposta de contrato v2 para expressar fronteiras de `shared` e frontend, antes de parsers adicionais.
- **Motivo:** o plano pedia um segundo projeto indicado pelo usuário; ele pediu um alvo falso com erros de propósito. O alvo permite testar o contrato e as regras fora da autoauditoria sem tocar em pastas pessoais, executar scripts ou enviar código para serviços externos.
- **Alternativas rejeitadas:** varrer pastas pessoais sem autorização — violaria privacidade e o limite de escolha do usuário; reaproveitar apenas as fixtures de teste — repetiria a arquitetura já conhecida; declarar a etapa bloqueada apesar do pedido — ignoraria a instrução recebida; adicionar analisador Go ou inventar aresta para `import()` dinâmico — o alvo não tem código Go não analisado relevante e o destino dinâmico não é determinístico.
- **Impacto:** nenhum `rule_id`, campo do contrato v1, schema SQLite, JSON ou código do núcleo mudou. O diretório `validation/targets/fake-order-service` passa a fazer parte do repositório como alvo de validação; seus 20 arquivos foram comparados por SHA-256 antes e depois da auditoria e permaneceram idênticos. O contrato foi criado em diretório novo, sem sobrescrever contrato existente.
- **Verificação:** `scan` retornou `0` (12 incluídos, 8 intencionais); `analyze` retornou `0` (6 arestas, 3 proibidas, 2 revisões); `audit` retornou `1` (7 achados: 3 erros, 1 aviso, 3 revisões). Censo manual: 4 verdadeiros positivos, 3 revisões corretas, 0 falsos positivos e 0 violações plantadas não detectadas. Cobertura: 11/12 arquivos de código; `tools/check.go` sem analisador. Tempos em execução única: 1063 ms, 875 ms, 863 ms e 873 ms. Relatório integral em `docs/validation/etapa-3/second-project-synthetic.md`.

## E3.6 — Proposta de contrato v2 aguardando aprovação

- **Data:** 26/11/2026
- **Decisão:** propor leitura de v1 e v2 no mesmo núcleo com normalização interna, sem reescrita automática; incluir `architecture.layers[]` com `name`, `modules` e `allowed_dependencies` e a regra `dep.layer_not_allowed`. Domínios, exceções locais, limiares de métrica, linguagem genérica de regras, novos parsers e dependências ficam de fora até haver evidência e testes.
- **Motivo:** os dois pilotos mostraram limitação de expressividade, não falha das regras atuais. A autoauditoria não consegue declarar a direção de camadas como campo; o alvo sintético não consegue dizer que `web` é apresentação nem que `shared` é uma fronteira, e a única forma atual de permitir uma exceção seria ampliar a lista do módulo inteiro.
- **Alternativas rejeitadas:** interpretar `responsibility` como regra — exigiria processamento de linguagem natural e contrariaria o princípio determinístico;apenas `rank` numérico — a direção seria implícita e sujeita a erro de leitura; domínios nesta versão — segundo eixo de agrupamento sem evidência nos pilotos; exceção global — silenciaria erros; migração automática — mudaria contrato do usuário sem inspeção; novo parser Go ou resolução de `import()` dinâmico — não há evidência de violação perdida e o destino não é determinístico.
- **Impacto:** nenhum código foi alterado nesta decisão. A Etapa 3.4 permanece aberta até aprovação do documento `docs/architecture/CONTRACT_V2_PROPOSAL.md`; a Etapa 3.5 só começa depois disso. Quando implementado, o novo `rule_id` será armazenado como texto no SQLite existente, sem migração, e CLI, desktop e extensão continuarão consumindo `findings` do núcleo.
- **Verificação:** proposta escrita antes de qualquer implementação, com problemas concretos citados dos dois relatórios de validação, exemplos válidos e inválidos, definição de sobreposição de paths, precedência entre regras, política para não resolvidos e testes de aceitação por campo. Nenhum teste de código foi executado porque não houve mudança de código; a checagem de whitespace do diff é registrada no relatório da sessão. A pendência de aprovação foi resolvida em `E3.7`.

## E3.7 — Contrato v2 aprovado e implementado

- **Data:** 26/11/2026
- **Decisão:** aprovar e implementar a proposta `E3.6` com escopo restrito a `architecture.layers[]` (`name`, `modules`, `allowed_dependencies`) e à regra `dep.layer_not_allowed`. Leitura de v1 e v2 no mesmo núcleo com normalização interna (`Contract::normalized_layers()`), sem reescrever contratos e sem alterar schema SQLite, `rule_id` existentes ou o formato dos doze campos de `findings`. `dep.not_allowed` prevalece na mesma aresta; destino não resolvido continua em `dep.unresolved`; mapeamento ausente em tempo de execução vira limitação.
- **Motivo:** o usuário aprovou explicitamente a proposta com “Aprovar e iniciar 3.5 (Recommended)”. Os pilotos mostraram limitação de expressividade para camadas e fronteiras, não falha das regras atuais.
- **Alternativas rejeitadas:** migração automática do contrato — mudaria arquivo do usuário sem inspeção; interpretar `responsibility` como regra — exigiria linguagem natural/IA, vedada no núcleo; domínios, exceções locais e limiares de métrica — sem evidência nos pilotos; novo parser, dependência, daemon ou agendamento — fora do escopo aprovado; duplicar a regra na CLI, no desktop ou na extensão — dividiria a verdade do núcleo.
- **Impacto:** `auditor init` continua gerando v1. Novos erros de validação nomeiam o campo, por exemplo `architecture.layers[1].allowed_dependencies` e `architecture.modules[0].name`. Nova identidade determinística `dep.layer_not_allowed:{origem}:{destino}:{path}`, sem linha e evidência fora da identidade. O `rule_id` novo é gravado como texto no banco existente; desktop e extensão consomem o achado sem lógica própria. Limitações mantidas: checagem de sobreposição é conservadora; `responsibility` permanece prosa para revisão humana; domínios e exceções ficam de fora.
- **Verificação:** testes escritos antes da implementação falharam na fase vermelha com `exit code 101`; depois da implementação, `cargo fmt --all`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (185 testes: 171 no núcleo, 10 na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão (5 testes) e `git diff --check` retornaram `0`. Smoke na fixture `layer_boundaries`: `audit` retornou `1` com 3 achados e os mesmos IDs em texto, JSON e Markdown; `history record`/`show`/`compare` mantiveram o novo `rule_id` como persistente sem migração. Contratos v2 equivalentes reproduziram exatamente os resultados v1 na autoauditoria e no alvo sintético. Relatório completo em `docs/validation/etapa-3/contract-v2.md`. A subetapa 3.6 continuava pendente e a 3.7 tinha apenas a paridade automatizada concluída; a Etapa 3 não foi declarada concluída nesta decisão. Estado posterior em `E3.12`.

## E3.8 — Paridade automatizada desktop/extensão na Etapa 3.7

- **Data:** 26/11/2026
- **Decisão:** validar os adaptadores com um único JSON real da CLI e mover a apresentação do desktop para `apps/desktop/src/reportView.ts`, com helpers puros para resumo, filtros, opções, cobertura e lista de arestas. A extensão continuou usando `src/model.ts`; foi acrescentado um teste de paridade sobre o mesmo JSON.
- **Motivo:** o checklist da Etapa 3.7 exige comparar CLI, desktop e extensão para o mesmo projeto. Sem extrair a apresentação, os filtros e o texto do resumo só poderiam ser conferidos olhando a janela, sem teste reproduzível.
- **Alternativas rejeitadas:** executar `cargo run` dentro dos testes Node (dependência do ambiente e lentidão); duplicar as regras no frontend (violaria `D8.1` e dividiria a verdade do núcleo); instalar a extensão ou automatizar a janela sem ferramenta disponível (mudaria o ambiente sem evidência verificável); gerar o JSON com redirecionamento do PowerShell (gravou UTF-16 e quebrou `JSON.parse`).
- **Impacto:** nenhuma regra de auditoria, contrato, schema SQLite ou formato de `findings` mudou. O JSON de evidência `docs/validation/etapa-3/cli-layer-boundaries.json` é somente leitura e foi gerado pela opção `--output`. Durante a verificação, o build encontrou `TS7053` porque `availableSeverities` devolvia `string[]`; o retorno passou a ser `Severity[]`, sem alterar comportamento. Limitações: não houve inspeção visual da janela Tauri nem da host do VS Code, nem captura de tela; os checklists manuais permanecem pendentes.
- **Verificação:** fase vermelha do desktop falhou com `ERR_MODULE_NOT_FOUND`; depois da implementação, `npm test` do desktop passou com 7 testes e `npm test` da extensão com 7. `npm run build` do desktop e `npm run package` da extensão retornaram `0`; o pacote ficou com 7 arquivos após ajustar `.vscodeignore`. `cargo test -p auditor-desktop --lib` confirmou que os comandos Tauri devolvem exatamente o relatório do núcleo, incluindo `dep.layer_not_allowed`. A verificação Rust completa também passou (185 testes). Relatório em `docs/validation/etapa-3/desktop-vscode.md`. A Etapa 3.7 não foi marcada como concluída.

## E3.9 — Feedback e apresentação da interface desktop

- **Data:** 26/11/2026
- **Decisão:** aprovar a sugestão do usuário de melhorar a interface desktop após a tentativa manual. A área de status passou a registrar as seis ações (selecionar, auditar, grafo, contrato, registrar e atualizar histórico), incluindo falhas operacionais; o histórico vazio ganhou mensagem explícita; a tela informa o caminho esperado do `auditor.toml` e rola até o painel gerado. Novos helpers puros em `apps/desktop/src/status.ts` e CSS com botões estilizados, badges de severidade, legenda do grafo e estados de foco. O usuário escolheu explicitamente o escopo “Feedback + visual, completo”.
- **Motivo:** o usuário relatou que, ao abrir o executável, o erro de contrato ausente aparecia no topo, que `Ver grafo` e `Registrar execução` “pareciam não executar” e que `Ver contrato` só aparecia no fim da janela. Os comandos de fato rodavam; faltava evidente feedback perto dos botões. A queixa de interface “simples demais, sem cor” também foi registrada.
- **Alternativas rejeitadas:** criar ou copiar `auditor.toml` automaticamente — viola a regra de nunca alterar contrato silenciosamente; procurar contrato em diretórios pais — mudaria a semântica da raiz auditada e poderia escolher projeto errado; gerar `auditor.toml` apenas para a UI — esconderia a causa do erro; adicionar biblioteca de componentes ou tema — dependência nova sem necessidade; mover regras para o frontend — dividiria a verdade do núcleo; mostrar apenas um status único — perderia o histórico das ações feitas.
- **Impacto:** nenhuma mudança no núcleo, no contrato, no SQLite, no formato de `findings` ou nos comandos Tauri. O erro de contrato ausente continua vindo do núcleo (`Falha operacional: contrato: não foi possível ler ...\auditor.toml: O sistema não pode encontrar o arquivo especificado`), agora exibido na área de status com o rótulo da ação. Nenhum dado novo é inventado: os textos vêm das contagens do relatório, da análise, do tamanho do contrato lido e do ID devolvido pelo banco. O painel de status guarda no máximo cinco entradas. Limitações mantidas: naquele momento a janela ainda não havia sido inspecionada visualmente, o que foi resolvido pela confirmação registrada em `E3.10`; o caminho do banco continua relativo ao diretório de execução do aplicativo.
- **Verificação:** teste de status escrito antes da implementação falhou com `ERR_MODULE_NOT_FOUND` para `out/status.js`; durante a implementação, a importação de `reportView` em `status.ts` também falhou em Node ESM e foi removida, passando a frase do resumo a ser argumento. Depois disso, `npm test` do desktop passou com 12 testes e `npm run build` retornou `0` (CSS de 3,99 kB e JS de 230,99 kB). `cargo fmt --all`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (185 testes), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` da extensão e `git diff --check` foram repetidos na verificação final. A Etapa 3.7 segue em andamento até que o usuário preencha os checklists visuais.

## E3.10 — Confirmação manual do desktop e auditoria do alvo sintético

- **Data:** 26/11/2026
- **Decisão:** registrar a confirmação do usuário de que todos os botões do desktop executam e o visual ficou mais agradável, marcando como concluído apenas o checklist da janela Tauri. O checklist da extensão VS Code continua pendente porque o usuário respondeu que ainda não instalou nem testou o `.vsix`. A Etapa 3.7 permanece em andamento.
- **Motivo:** a Etapa 3.7 exige as duas validações manuais; afirmar que foi concluída sem testar a extensão seria exagerar a evidência.
- **Alternativas rejeitadas:** marcar a etapa inteira como concluída com confirmação parcial; instalar a extensão no ambiente do usuário sem autorização; gerar capturas de tela com dados locais do projeto.
- **Impacto:** nenhuma mudança de código. O executável release foi recompilado com `npm run tauri build` em `apps/desktop` (`0`), gerando `target/release/auditor-desktop.exe` com 14.235.136 bytes. A auditoria do alvo sintético `validation/targets/fake-order-service` foi repetida sem alterar arquivos: `audit` retornou `1` com 7 achados (3 erros, 1 aviso, 3 revisões) e `analyze` retornou `0` com 6 arestas, 3 permitidas e 3 proibidas. A árvore Git permaneceu limpa.
- **Verificação:** `npm run tauri build` retornou `0`; `git status --short` retornou `0` sem saída. Nenhum teste foi executado nesta entrega porque não houve alteração de código; a suíte completa já havia passado na entrega anterior.

## E3.11 — Conclusão da Etapa 3.7 com a validação da extensão VS Code

- **Data:** 26/11/2026
- **Decisão:** marcar a Etapa 3.7 como concluída após a confirmação manual da host do VS Code. Esta decisão substitui o estado provisório registrado em `E3.10`, onde apenas o desktop estava validado.
- **Motivo:** o usuário executou o comando pela paleta e pelo menu de contexto, conferiu os 7 diagnósticos com severidades e códigos, a abertura do arquivo/linha, a barra de status `VCA: 7 achados` e a aceitação do código `1` da CLI. O estado persistido pela extensão contém o mesmo relatório da CLI, o que confirma a execução sem depender apenas da interpretação do popup.
- **Alternativas rejeitadas:** declarar a etapa concluída com as respostas iniciais contraditórias; automatizar a interface com nova dependência; instalar a extensão no repositório; alterar o contrato do alvo ou o formato de `findings` para facilitar o teste; criar um comando chamado “diagnóstico”.
- **Impacto:** nenhuma mudança de código, schema, contrato, SQLite ou comandos. Fora do repositório, a CLI release foi compilada, o `.vsix` foi instalado e `vibeCodingAuditor.cliPath` foi configurado no `settings.json` de usuário do VS Code. O alvo sintético não foi modificado. O código de saída `1` continua significando “há achados”, não falha operacional.
- **Verificação:** `cargo build --release -p auditor-cli` = `0`; `code --install-extension extensions\vscode\vibe-coding-auditor-0.1.0.vsix --force` = `0`; `code --list-extensions --show-versions` = `0` com `dioneyfroes-coder.vibe-coding-auditor@0.1.0`; auditoria direta da CLI = `1` com 7 achados; `exthost.log` mostrou ativação sem erro; `state.vscdb` da janela do alvo guardou `lastAudit` com o mesmo resumo. Nenhum teste foi executado nesta entrega porque a mudança foi apenas documental; a suíte completa já havia passado no commit `c035531`. Relatório atualizado em `docs/validation/etapa-3/desktop-vscode.md`.

## E3.12 — Cobertura estruturada no histórico e fechamento da Etapa 3

- **Data:** 26/11/2026
- **Decisão:** implementar a Etapa 3.6 com migração SQLite versionada para `SCHEMA_VERSION = 2`. Novas tabelas agregadas por execução guardam contagens de cobertura, extensões sem analisador e motivos de exclusão com rótulo, posição estável, contagem e impacto na cobertura. `history show` e `history compare` expõem esses dados; execuções antigas aparecem como `sem dados`. O identificador `E3.6` permanece reservado à proposta de contrato v2, então esta decisão usa `E3.12`.
- **Motivo:** o usuário confirmou que comparar cobertura entre execuções históricas é requisito real. O banco v1 guardava apenas `files_included`/`files_skipped` e a limitação em texto, sem distinção entre código analisado e não analisado, extensões sem analisador ou motivos de exclusão.
- **Alternativas rejeitadas:** interpretar a mensagem de limitação — texto livre não é contrato de dados e não cobre `skip_summary`; gravar o `AuditReport` inteiro em JSON — rejeitado em `D6.2` pelo volume e pela proximidade com conteúdo dos arquivos; comparar só `files_included`/`files_skipped` sem migração — não atende ao requisito e esconderia perda real de cobertura; gravar exemplos de caminho — aumentaria o banco e aproximaria o histórico do conteúdo auditado; adicionar comparação no desktop ou na extensão — fora do escopo aprovado, sem comando Tauri ou painel pedido.
- **Impacto:** `migrate()` passa a aplicar versões pendentes em ordem, registrando o número real de cada uma. Bancos v1 são migrados em uma transação, bancos v2 reabertos sem alteração e versões futuras continuam rejeitadas; uma falha de DDL reverte a versão registrada. Nenhuma mudança em `auditor.toml`, `rule_id` existente, formato dos doze campos de `findings`, comandos ou códigos de saída da CLI, desktop e extensão. Nenhuma dependência nova. A comparação não valida se as execuções pertencem ao mesmo projeto e permanece apenas na CLI. Com esta decisão, a Etapa 3 é declarada concluída.
- **Verificação:** proposta em `docs/architecture/HISTORY_COVERAGE_V2_PROPOSAL.md` aprovada antes da implementação. Testes escritos antes do código falharam na fase vermelha com `exit code 101`; a suíte focada de histórico passou com 22 testes, incluindo 2 testes diretos dos helpers de renderização. `cargo fmt --all`, `cargo fmt --check`, `cargo test --workspace` (193 testes: 179 no núcleo, 10 na CLI e 4 no desktop) e `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` retornaram `0`. Smoke real com banco temporário: `history --database ... record` nas fixtures `partial_coverage` e `clean`, `show 1` e `compare 1 2` retornaram `0`, com cobertura 2/4 parcial → 3/3 completa, deltas `+1` analisados e `-2` não analisados e motivos de exclusão `1 -> 5`. Depois do fechamento da etapa, os binários release foram recompilados (`cargo build --release -p auditor-cli` = `0`, 5.786.112 bytes; `npm run tauri build` = `0`, 14.241.792 bytes) e o executável release repetiu o smoke e a migração de um banco v1 real, com `show`, `list` e `compare` retornando `0` e `schema_migrations` passando a conter as versões `1` e `2`. Relatório completo em `docs/validation/etapa-3/history-coverage.md`.

## E3.13 — Correção da confiabilidade da autoauditoria

- **Data:** 26/11/2026
- **Decisão:** substituir a limitação de evidência deslocada registrada em `E3.3` por localização real via spans e ampliar o escopo de produção do analisador Rust em três incrementos aprovados: (A) `use` e `mod` passam a usar `proc_macro2::Span` com a feature `span-locations`, em vez do cursor textual; (B) módulos inline `#[cfg(test)]` deixam de gerar arestas, declarações `mod` ou candidatos a duplicação; (C) caminhos qualificados em expressões e tipos passam a gerar arestas quando a raiz é `crate`, `self`, `super`, módulo do contrato ou dependência local declarada. A visita usa a feature `visit` do `syn`, pacote já presente no workspace; nenhum pacote novo foi instalado.
- **Motivo:** `audit .` devolvia 71 achados, todos `needs_review`. Sete `dep.unresolved` tinham linha e evidência erradas porque o cursor textual contava `use` dentro de função que o visitante de itens não percorre; os oito restantes vinham de `#[cfg(test)]`, contrariando o escopo “somente código Rust de produção” de `auditor.toml`. Depois de excluir testes, apareceu falso negativo maior: o CLI usa `auditor_core::...` qualificado em produção e não tinha aresta `cli -> core`, gerando `module.unreferenced:cli` falso.
- **Alternativas rejeitadas:** manter o cursor e aceitar a limitação — deixaria evidência errada no relatório; scanner textual com profundidade de chaves — frágil com strings e macros; campo novo no contrato para excluir testes — mudaria `auditor.toml` sem demanda; inferir arestas só do `Cargo.toml` — superinformaria dependências de dev/build e perderia a localização; classificar qualquer caminho de dois segmentos — teste intermediário gerou 444 revisões falsas (`Vec::new`, `fmt::Result`, `fs::read`); silenciar `code.duplicated` — removeria heurítica sem evidência de que as repetições restantes são falsos positivos; alterar o código do CLI para agradar ao auditor — esconderia o defeito do analisador.
- **Impacto:** nenhum `rule_id` novo, nenhum campo novo em `findings` ou `auditor.toml`, nenhuma mudança em comandos/códigos da CLI, desktop, extensão ou SQLite. `crates/auditor-core/Cargo.toml` passa a declarar `proc-macro2` como dependência direta com `span-locations`; `Cargo.toml` passa a declarar `syn` com a feature `visit`. Nenhuma versão nova de pacote entrou em `Cargo.lock`. Limitações explícitas: a feature `visit` não interpreta tokens dentro de atributos, então `#[arg(default_value = auditor_core::...)]` continua fora do grafo; caminhos qualificados cuja raiz é um apelido local ou pacote externo não declarado como dependência local não são classificados; `code.duplicated` continua sendo repetição exata de linha, não semântica. A autoauditoria agora mostra 22 arestas e 32 achados `code.duplicated`; os achados que existiam apenas em código de teste aparecerão como resolvidos na próxima comparação de histórico.
- **Verificação:** os três testes de regressão foram escritos antes da implementação e falharam na fase vermelha com `exit code 101` (linha `2` em vez de `6`; aresta de `#[cfg(test)]` presente; aresta de caminho qualificado ausente). Durante o incremento C, a primeira versão sem filtro de raiz gerou 444 revisões; o teste `unresolved_dependency_becomes_review` falhou com `crates/application/src/lib.rs` no lugar de `infra/src/unresolved.rs`, levando ao filtro de raiz. Verificação final: `cargo fmt --all` = `0`, `cargo fmt --all --check` = `0`, `cargo check --workspace` = `0`, `cargo test --workspace` = `0` (194 testes: 180 no núcleo, 10 na CLI e 4 no desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` = `0`, `git diff --check` = `0` (apenas avisos LF/CRLF do OneDrive), `npm --prefix apps/desktop test` = `0` (12), `npm --prefix apps/desktop run build` = `0` e `npm --prefix extensions/vscode test` = `0` (7). Smoke: `analyze .` = `0` com 22 arestas, 0 revisões, 15/15 arquivos analisados e 0 limitações; `audit . --format json --output <temporário>` = `1` com 32 achados, todos `code.duplicated`. Relatório em `docs/validation/etapa-3/self-audit-correcao.md`.
