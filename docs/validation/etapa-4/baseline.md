# Baseline da Etapa 4.0

Data: 26/11/2026
Branch: `main`
HEAD: `84192bf docs: registrar correcao da autoauditoria`

Este documento congela o estado observado antes de qualquer mudança da Etapa 4. Nenhuma funcionalidade foi implementada nesta subetapa. O objetivo é servir de referência para comparar compatibilidade depois que a descoberta sem contrato e o Architecture Builder forem introduzidos.

## Estado do Git

`git status --short` retornou `0` com duas mudanças locais do usuário:

- `D PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md` — arquivo versionado apagado.
- `?? PLANO_ETAPA_4_DESCOBERTA_E_ARCHITECTURE_BUILDER.md` — plano novo, sem tracking.

Ambas foram preservadas: não houve restore, `git add`, commit, rebase ou sobrescrita. `git branch --show-current` retornou `main` e `git log -5 --oneline` confirmou `84192bf`, `04c976a`, `14ed6e8`, `1a42a97` e `2b56ed2`.

## Versões observadas

| Ferramenta | Versão |
|---|---|
| Git | `2.55.0.windows.4` |
| cargo | `1.98.1 (797e8a9bc 2026-08-05)` |
| rustc | `1.98.1 (48a229cea 2026-09-01)` |
| Node.js | `v24.19.0` |
| npm | `11.17.0` |

Nenhuma ferramenta do sistema foi instalada, atualizada ou removida. `npm ci` instalou apenas dependências já travadas nos `package-lock.json` dos projetos desktop e extensão.

## Comandos executados

Executados na raiz do repositório, no PowerShell, antes de alterar arquivos de código:

| Comando | Resultado | Código |
|---|---|---:|
| `cargo fmt --all --check` | sem diferenças | `0` |
| `cargo check --workspace` | sem erros | `0` |
| `cargo test --workspace` | 194 testes: 10 CLI, 180 núcleo, 4 desktop; 0 falhas | `0` |
| `cargo clippy --workspace --all-targets -- -D warnings` | sem avisos de código; avisos de finalização incremental do OneDrive | `0` |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | sem avisos | `0` |
| `npm --prefix apps/desktop ci` | 28 pacotes adicionados; 0 vulnerabilidades | `0` |
| `npm --prefix apps/desktop test` | 12 testes passaram | `0` |
| `npm --prefix apps/desktop run build` | CSS 3,99 kB; JS 230,99 kB | `0` |
| `npm --prefix extensions/vscode ci` | 121 pacotes adicionados; 2 vulnerabilidades moderadas | `0` |
| `npm --prefix extensions/vscode test` | 7 testes passaram | `0` |
| `npm --prefix extensions/vscode run package` | `.vsix` gerado com 7 arquivos, 8,98 kB | `0` |
| `git diff --check` | sem problemas de whitespace nas mudanças locais | `0` |
| `git status --short` | apenas as duas mudanças do usuário | `0` |

## Avisos que não falharam

- O Clippy sem `--config 'build.incremental=false'` emitiu vários avisos `Acesso negado (os error 5)` ao finalizar diretórios de compilação incremental sob o OneDrive. A segunda execução, com incremental desativado, terminou sem avisos. Ambos retornaram `0`.
- `npm --prefix extensions/vscode ci` informou um pacote depreciado, duas vulnerabilidades moderadas e um script de pós-instalação ainda não aprovado pelo npm. Nenhuma correção foi aplicada: isso exigiria alterar dependências, fora do escopo da Etapa 4.0.
- `npm --prefix extensions/vscode run package` avisou que não há arquivo `LICENSE`. O `.vsix` foi gerado normalmente.

## Leitura prévia

Foram lidos, sem alteração: `AGENTS.md`, `README.md`, `docs/ROADMAP.md`, `docs/DECISIONS.md`, o próprio `PLANO_ETAPA_4_DESCOBERTA_E_ARCHITECTURE_BUILDER.md`, os scripts de `apps/desktop/package.json`, `extensions/vscode/package.json`, `.github/workflows/ci.yml` e os pontos públicos de `auditor-core`, CLI, backend Tauri, desktop e extensão. Um mapeamento detalhado e somente leitura revisitou todo o código relevante e confirmou que não existe API pública de descoberta sem contrato; os números de linha e os consumidores afetados estão consolidados em `docs/architecture/DISCOVERY_MODE_PROPOSAL.md`.

O workflow atual usa Rust `1.98.1`, Node `24.19.0`, `npm ci`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, testes e build da desktop, e testes da extensão. Ele não executa empacotamento Tauri.

## Conclusão do baseline

O estado inicial está verde e pode ser usado como referência de regressão. A Etapa 4.0 não alterou código-fonte, contratos, fixtures versionadas, schema SQLite, comandos ou documentação de comportamento implementado. O mapeamento de dependências de `auditor.toml` e a proposta técnica estão registrados em `docs/architecture/DISCOVERY_MODE_PROPOSAL.md`.
