# Revalidação do baseline — estado atual após a Etapa 3

Data: 26/11/2026
Repositório: `C:\Users\dratz\OneDrive\Documentos\workspace\VibeCodingAuditor`
Branch: `etapa-3/validacao-arquitetural`
HEAD verificado: `e54d233`
Objetivo: reproduzir no estado final da Etapa 3 os mesmos comandos do baseline original ([baseline.md](baseline.md)), documentando versões, resultados e diferenças. Nenhum código foi alterado nesta revalidação.

## 1. Ambiente

Todos os comandos abaixo retornaram código `0`.

| Comando | Versão encontrada | Baseline original |
|---|---|---|
| `git --version` | `git version 2.55.0.windows.4` | igual |
| `rustc --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` | igual |
| `cargo --version` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` | igual |
| `node --version` | `v24.19.0` | igual |
| `npm --version` | `11.17.0` | igual |
| Sistema operacional | `Microsoft Windows 11 Pro`, versão `10.0.26200`, build `26200` | não registrado no baseline original |

Nenhuma ferramenta foi instalada, atualizada ou removida. `git status -sb` retornou `0` com `## etapa-3/validacao-arquitetural` e sem alterações pendentes; `git rev-parse --short HEAD` retornou `0` com `e54d233`.

## 2. Verificações Rust

Executadas na raiz do repositório, em sequência. `cargo fmt --all --check` foi usado no lugar de `cargo fmt --check` para cobrir explicitamente todos os crates do workspace; ambos equivalem neste repositório.

| Comando | Código | Resultado |
|---|---:|---|
| `cargo fmt --all --check` | `0` | sem saída; nenhuma divergência |
| `cargo check --workspace` | `0` | `Finished \`dev\` profile ...`; sem erros |
| `cargo test --workspace` | `0` | **193 testes aprovados, 0 falhas** |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | `0` | `Finished \`dev\` profile ...`; nenhum aviso |
| `git diff --check` | `0` | sem saída |

Distribuição dos 193 testes:

| Destino | Revalidação | Baseline original |
|---|---:|---:|
| `vibe_coding_auditor` (CLI) | 10 | 10 |
| `auditor_core` | 179 | 122 |
| `auditor_desktop` (lib) | 4 | 3 |
| `auditor_desktop` (bin) | 0 | 0 |
| Doc-tests `auditor_core` e `auditor_desktop` | 0 | 0 |

O aumento de 58 testes no núcleo e 1 no desktop decorre das verificações adicionadas nas Etapas 3.1 a 3.6, em especial a cobertura estruturada no histórico e o contrato v2. Nenhum teste mudou de resultado de aprovado para falho.

## 3. Verificações do desktop

Executadas em `apps/desktop`.

| Comando | Código | Resultado |
|---|---:|---|
| `npm ci` | `0` | `added 28 packages, and audited 29 packages in 4s`; `found 0 vulnerabilities` |
| `npm test` | `0` | `tests 12`, `pass 12`, `fail 0` (antes: 2 testes) |
| `npm run build` | `0` | `tsc --noEmit && vite build`; 22 módulos, CSS de 3,99 kB e JS de 230,99 kB, sem erros |

`node_modules/` e `dist/` continuam fora do controle de versão. Nenhum source foi alterado.

## 4. Verificações da extensão VS Code

Executadas em `extensions/vscode`.

| Comando | Código | Resultado |
|---|---:|---|
| `npm ci` | `0` | `added 121 packages, and audited 122 packages in 15s`; `2 moderate severity vulnerabilities` |
| `npm test` | `0` | compilação TypeScript seguida de `tests 7`, `pass 7`, `fail 0` (antes: 4 testes) |

Avisos repetidos do `npm ci`: depreciação de `@secretlint/secretlint-rule-preset-recommend@10.2.2`, dois alertas moderados e aviso de script pendente de `@vscode/vsce-sign@2.1.1`. Nenhum script pendente foi aprovado ou executado manualmente. `npm run package` não foi executado nesta revalidação; o `.vsix` validado na Etapa 3.7 permanece como evidência daquela etapa.

## 5. Falhas e avisos

- **Falhas:** nenhuma. Todos os comandos planejados terminaram com código `0`.
- **Avisos sem efeito nos resultados:** os avisos conhecidos do `npm ci` da extensão. Diferente do baseline original, não houve aviso LF/CRLF: `git diff --check` terminou sem saída.
- Nenhum aviso foi silenciado ou corrigido nesta revalidação.
- Para confirmar as contagens, `npm test` da extensão, `npm run build` do desktop e o `cargo clippy` foram executados uma segunda vez; todos retornaram `0` com os mesmos resultados (7/7 testes, 22 módulos e nenhum aviso, respectivamente). Os tempos de conclusão variam entre execuções e não foram registrados como critério.

## 6. Estado final

| Comando | Código | Resultado |
|---|---:|---|
| `git status -sb` | `0` | `## etapa-3/validacao-arquitetural`, sem alterações antes da criação deste relatório |
| `git diff --check` | `0` | sem saída |

A revalidação não acrescentou funcionalidade, dependência, contrato nem dado ao histórico. O baseline original permanece inalterado como fotografia do início da Etapa 3; este documento registra o estado final após as subetapas 3.0 a 3.7.
