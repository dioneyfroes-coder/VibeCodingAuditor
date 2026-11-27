# Vibe Coding Auditor para VS Code

Extensão que mostra no painel **Problems** os achados determinísticos produzidos pelo núcleo do Vibe Coding Auditor. Ela não avalia regras: executa o CLI `vibe-coding-auditor-cli audit <pasta> --format json` e converte o relatório recebido. Em conformidade, os achados viram diagnósticos; em descoberta, as observações e limitações do núcleo são registradas no canal de saída **Vibe Coding Auditor** e a extensão exibe um único aviso informativo.

## Uso

1. Abra no VS Code a pasta do projeto; `auditor.toml` é opcional.
2. Execute **Vibe Coding Auditor: Auditar ou descobrir projeto** na paleta de comandos ou no menu de contexto da pasta. O `auditor.toml` é opcional: sem contrato, a extensão registra observações de descoberta no canal de saída; com contrato, mostra achados de conformidade.
3. Clique em um diagnóstico para abrir o arquivo e a linha do achado.
4. Use **Vibe Coding Auditor: Abrir achado...** para escolher um achado pela Paleta rápida.
5. Use **Vibe Coding Auditor: Ocultar diagnósticos** para limpar o painel sem apagar o estado da última auditoria.

A barra de status mostra `VCA: descoberta disponível`, `VCA: sem auditoria` ou o resumo da última execução. Esse estado é restaurado ao reabrir o workspace.

## Configuração

`vibeCodingAuditor.cliPath` define o executável. O padrão `vibe-coding-auditor-cli` procura no `PATH`, depois em `target/release/` e `target/debug/` da pasta aberta e, por fim, nos mesmos caminhos do repositório que contém a extensão. Para instalações fora desses locais, configure um caminho absoluto.

A CLI retorna `0` sem achados e `1` com achados. A extensão aceita ambos; somente códigos maiores que `1` são tratados como falha operacional.

## Mapeamento de severidade

| Severidade do núcleo | Diagnóstico no VS Code |
|---|---|
| `error` | `Error` |
| `warning` | `Warning` |
| `needs_review` | `Warning` (exige decisão humana) |
| `info` | `Information` |

O `rule_id` aparece como código do diagnóstico e a mensagem traz `id`, título, evidência, expectativa e sugestão. Achados sem linha são ancorados na primeira linha do arquivo.

## Privacidade e limites

- Não há daemon, varredura automática nem envio de código para serviços externos.
- A extensão salva no estado local do workspace apenas metadados do relatório: nome do projeto, contagens, achados e linhas de evidência já presentes no relatório. Conteúdo integral de arquivos não é copiado.
- O contrato nunca é alterado pela extensão.
- Não há integração com o histórico SQLite na extensão; consulte-o pela CLI ou pelo aplicativo desktop.
- A compatibilidade declarada é VS Code `^1.146.0`.
- `npm audit --omit=dev` reporta `0` vulnerabilidades; `npm audit` reporta 2 alertas moderados em dependência de desenvolvimento do `@vscode/vsce`. Eles não entram na extensão empacotada (`--no-dependencies`).

## Desenvolvimento

```sh
npm install
npm test
npm run package
```

`npm test` compila o TypeScript e executa os testes puros de conversão com `node --test`. `npm run package` gera `vibe-coding-auditor-0.1.0.vsix`.
