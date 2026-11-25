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
| Inicializar o repositório Git nesta pasta (`git init`) | Concluído em 25/11/2026, após autorização; branch inicial `main` | Repositório local criado; nenhum commit realizado. |
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
- **Impacto:** 21 pacotes resolvidos no `Cargo.lock`; nenhuma dependência adicionada ao `auditor-core`.

## D1.3 — Toolchain Rust mantida
- **Decisão:** continuar com rustc/cargo 1.98.1; não atualizar para 1.100.0 nesta etapa.
- **Motivo:** o workspace e o `clap` 4.6.10 compilam sem exigência de versão superior; a atualização permanece sem autorização (D0.5).
- **Impacto:** nenhuma mudança no ambiente.

## D1.4 — `.gitignore` mínimo
- **Decisão:** criar `.gitignore` com `/target/` e `data/`.
- **Motivo:** resolve a pendência registrada em D0.5 agora que existe build; evita versionar artefatos e dados locais futuros, conforme a estrutura de referência do plano.

## D1.5 — Fronteira núcleo/CLI
- **Decisão:** `auditor-cli` declara dependência de caminho para `auditor-core`, mas ainda não a utiliza; o núcleo expõe apenas `CORE_NAME` e um teste.
- **Motivo:** fixar o limite arquitetural desde já sem inventar funções de negócio. O uso efetivo começa quando houver comandos de auditoria (Etapa 2).
- **Impacto:** nenhuma regra ou analisador implementado; `--help` e `--version` são fornecidos pelo Clap.

## D1.6 — Limitação ambiental do Cargo no OneDrive
- **Observação:** com o repositório dentro de `OneDrive`, o Cargo emite `note: did not finalize incremental compilation session directory ... Acesso negado (os error 5)` quando a compilação incremental está ativa.
- **Decisão:** registrar como limitação do ambiente, não do código. Para verificação limpa, executar `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`.
- **Impacto:** todos os comandos de verificação terminam com sucesso; o aviso não altera os resultados.
