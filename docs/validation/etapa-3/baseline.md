# Baseline — Etapa 3.0 (reproduzível)

**Data:** 26/11/2026  
**Repositório:** `C:\Users\dratz\OneDrive\Documentos\workspace\VibeCodingAuditor`  
**Branch de trabalho:** `etapa-3/validacao-arquitetural`  
**HEAD ao final do baseline:** `fc2dd04`  
**Objetivo:** fotografar o estado real antes de qualquer alteração funcional da Etapa 3.

> Observação sobre a captura: cada comando abaixo foi executado no PowerShell com uma linha adicional `Write-Output "EXIT_CODE=$LASTEXITCODE"` apenas para registrar o código de saída. O comando funcional é o que aparece na coluna “Comando”.

## 1. Estado inicial do Git e decisão do usuário

Comando executado:

```powershell
git status --short
```

Saída real e código de saída `0`:

```text
 D RELATORIO_ETAPA_1_E_PLANO_ETAPA_2.md
?? PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md
```

Conforme a regra 3.0.1, as alterações não foram descartadas nem sobrescritas. Perguntado ao usuário o que fazer, a resposta foi **“Fazer commit primeiro”**. Nenhuma ferramenta foi instalada, atualizada ou removida.

Comandos executados na sequência:

| Comando | Código | Resultado real resumido |
|---|---:|---|
| `git add -- RELATORIO_ETAPA_1_E_PLANO_ETAPA_2.md PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md` | `0` | Aviso apenas: `LF will be replaced by CRLF` no arquivo do plano. |
| `git commit -m 'docs: registrar plano da Etapa 3 e remover relatorio anterior'` | `0` | `[main fc2dd04] docs: registrar plano da Etapa 3 e remover relatorio anterior`; `2 files changed, 710 insertions(+), 469 deletions(-)`; criação do plano e exclusão do relatório anterior. |
| `git status --short` | `0` | Sem saída: árvore limpa. |
| `git switch main` | `0` | `Already on 'main'`; aviso de que o branch está `ahead of 'origin/main' by 1 commit`. Nenhum `push` foi feito. |
| `git pull --ff-only` | `0` | `Already up to date.` |
| `git rev-parse --short HEAD` | `0` | `fc2dd04` |
| `git switch -c etapa-3/validacao-arquitetural` | `0` | Branch não existia antes (verificado com `git branch --list`, saída vazia, código `0`); criado com sucesso. |
| `git status --short` | `0` | Sem saída. |

A branch local ficou um commit à frente de `origin/main` por causa do commit autorizado. Isso foi registrado, mas não foi publicado.

## 2. Versões do ambiente

Todos os comandos desta seção retornaram código `0`.

| Comando | Versão encontrada | Referência do repositório |
|---|---|---|
| `git --version` | `git version 2.55.0.windows.4` | compatível com a documentação (`2.55.0.windows.4`). |
| `rustc --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` | Rust `1.98.1`, sem diferença. |
| `cargo --version` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` | Rust `1.98.1`, sem diferença. |
| `node --version` | `v24.19.0` | Node `24.19.0`, sem diferença. |
| `npm --version` | `11.17.0` | Versão documentada em `README.md`, sem diferença. |

Nenhuma ferramenta do Windows, Rust, Node ou npm foi instalada, atualizada ou removida.

## 3. Verificações Rust

Executadas na raiz do repositório, em sequência:

| Comando | Código | Resultado real resumido |
|---|---:|---|
| `cargo fmt --check` | `0` | Sem saída; nenhuma divergência de formatação. |
| `cargo check --workspace` | `0` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.88s`. |
| `cargo test --workspace` | `0` | **135 testes aprovados, 0 falhas**: 10 na CLI (`vibe-coding-auditor`), 122 em `auditor-core`, 3 no backend desktop e 0 doc-tests. |
| `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings` | `0` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.80s`; nenhum aviso ou erro do Clippy. A opção `build.incremental=false` foi usada conforme `AGENTS.md` para evitar avisos de sessão incremental no OneDrive. |
| `git diff --check` | `0` | Sem saída. |

Distribuição real dos 135 testes, por binário/crate:

- `vibe_coding_auditor` (CLI): `10 passed; 0 failed`.
- `auditor_core`: `122 passed; 0 failed`.
- `auditor_desktop` (lib): `3 passed; 0 failed`.
- `auditor_desktop` (bin): `0 tests`.
- Doc-tests `auditor_core` e `auditor_desktop`: `0 tests`.

## 4. Verificações do desktop

Executadas com `Push-Location apps/desktop` / `Pop-Location`:

| Comando | Código | Resultado real resumido |
|---|---:|---|
| `npm ci` | `0` | `added 28 packages, and audited 29 packages in 9s`; `found 0 vulnerabilities`. |
| `npm test` | `0` | 2 testes Node puros passaram (`skipSummary`); `pass 2`, `fail 0`. |
| `npm run build` | `0` | `tsc --noEmit && vite build`; `✓ 20 modules transformed`; `dist/index.html`, `dist/assets/index-ci7Slmjv.css` e `dist/assets/index-CcvVKZtm.js` gerados; build concluído em `431ms`. |

`npm ci` escreveu `node_modules/` e o build escreveu `dist/`; ambos permanecem fora do controle de versão. Nenhum fonte foi alterado.

## 5. Verificações da extensão VS Code

Executadas com `Push-Location extensions/vscode` / `Pop-Location`:

| Comando | Código | Resultado real resumido |
|---|---:|---|
| `npm ci` | `0` | `added 121 packages, and audited 122 packages in 12s`; `2 moderate severity vulnerabilities`. Houve aviso de depreciação de `@secretlint/secretlint-rule-preset-recommend@10.2.2` e aviso do `allow-scripts` sobre `@vscode/vsce-sign@2.1.1`; nenhum script pendente foi aprovado ou executado manualmente. |
| `npm test` | `0` | Compilação TypeScript seguida de 4 testes de conversão; `pass 4`, `fail 0`. |

O empacotamento opcional (`npm run package`) **não foi executado** neste baseline. Os 2 alertas moderados já são conhecidos e registrados no `README.md` como restritos ao `@vscode/vsce` de desenvolvimento.

## 6. Falhas preexistentes e avisos

- **Falhas preexistentes observadas:** nenhuma. Todos os comandos planejados para 3.0 terminaram com código `0`.
- **Avisos sem efeito nos resultados:** `LF will be replaced by CRLF` no commit do plano; `main` à frente de `origin/main` por 1 commit; depreciação e 2 alertas moderados no `npm ci` da extensão; aviso de script pendente do `@vscode/vsce-sign`.
- Nenhum desses avisos foi tratado como falha, corrigido ou silenciado nesta subetapa.

## 7. Estado final do baseline

Comandos finais executados na raiz:

| Comando | Código | Resultado |
|---|---:|---|
| `git status --short` | `0` | Sem saída: árvore limpa. |
| `git diff --check` | `0` | Sem saída. |

Além deste arquivo de relatório, nenhuma alteração funcional foi feita na Etapa 3.0. O baseline está salvo em `docs/validation/etapa-3/baseline.md`, conforme o critério de aceite.

## 8. Próximo passo

Aguardar orientação explícita do usuário para iniciar a **Etapa 3.1 — Autoauditoria e dependências internas do workspace**. Não foi iniciada nenhuma investigação ou modificação do motor nesta execução.
