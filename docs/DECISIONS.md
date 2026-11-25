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
- **Impacto:** nenhum novo crate; nenhuma mudança no contrato (`auditor.toml` continua o mesmo, com globs explícitos por extensão); CLI, desktop e extensão continuam consumindo o mesmo `AuditReport`. Novos rótulos deixam de dizer “Rust” onde passam a valer também para TS/JS. Limitações documentadas: sem `tsconfig.paths`, workspaces npm ou resolução de pacotes; `import type` é tratado como acoplamento; templates e código dinâmico são ignorados com limitation; `packages` continua contando apenas pacotes Cargo.

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
- **Impacto:** nenhuma dependência nova e nenhum campo novo no contrato. Fixture `tests/fixtures/projects/unreferenced`: 1 aresta, 1 revisão `module.unreferenced:orphan`, 0 erros, 0 avisos e 0 limitações. A fixture `partial` passou de 4 para 5 achados porque `shared` tem arquivos e nenhuma aresta; as demais fixtures mantiveram as contagens. Limitações: módulo é um proxy para código isolado, não prova de código morto; módulos sem arquivos continuam apenas com `module.no_files`. Custo `O(M + E)`. Verificação complementar: `cargo check` retornou `0`; `cargo test` passou com 93 testes (80 no núcleo, 10 na CLI e 3 no desktop); `cargo fmt --check`, `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` e `git diff --check` retornaram `0`; smoke CLI→JSON→extensão converteu 1 achado em 1 diagnóstico `warning`; `npm test` da extensão e `npm run build` da desktop passaram."""
