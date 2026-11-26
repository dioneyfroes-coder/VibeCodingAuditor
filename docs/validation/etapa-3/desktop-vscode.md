# Etapa 3.7 — Validação ponta a ponta do desktop e da extensão

Data: 26/11/2026
Branch: `etapa-3/validacao-arquitetural`
Status: **paridade automatizada e feedback da interface concluídos; checklist manual ainda pendente de execução humana**

## Método

Foi usado um único relatório real, produzido pela CLI sobre `tests/fixtures/projects/layer_boundaries` com o contrato v2:

```powershell
cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format json --output docs/validation/etapa-3/cli-layer-boundaries.json
```

O código de saída `1` é esperado porque há achados. O arquivo gerado é UTF-8, sem BOM, e serve de entrada para os dois adaptadores. A primeira tentativa de gerá-lo com redirecionamento `>` do PowerShell produziu UTF-16 e o `JSON.parse` falhou; o comando foi repetido com `--output` e passou.

O desktop passou a apresentar o relatório por helpers puros em `apps/desktop/src/reportView.ts`, testados com `node --test`. A extensão já convertia o relatório em `extensions/vscode/src/model.ts`; o teste novo consome o mesmo JSON da CLI. Os comandos Tauri continuam sendo comparados diretamente com o núcleo nos testes Rust, sem regra reimplementada no adaptador.

## Comparação automatizada

| Item | CLI JSON | Desktop (helpers) | Extensão (diagnósticos) |
|---|---|---|---|
| Total de achados | 3 | 3 | 3 |
| Erros | 2 | `3 achados (2 erros, 0 avisos, 0 informações, 1 revisões)` | 2 com severidade `error` |
| Revisões | 1 | mesma frase acima | 1 com severidade `warning` |
| Identificadores | 3 IDs distintos | preservados nos filtros | incluídos no início da mensagem |
| Regras | `dep.layer_not_allowed`, `dep.not_allowed`, `dep.unresolved` | mesmas opções do filtro | mesmos valores em `code` |
| Arquivos e linhas | caminho e `line_start` de cada achado | exibidos na lista | mesmos caminhos; linha convertida para 0-based |
| Cobertura | 5 de 5 arquivos analisados | mesma frase | não apresentada pela extensão |
| Exclusões | 1 `not_included` | mesmo resumo | não apresentadas pela extensão |

Diferenças de apresentação são esperadas: a extensão agrupa `needs_review` como aviso e não mostra cobertura nem motivos de exclusão. Nenhuma regra foi duplicada nos adaptadores.

### Comandos executados e resultados

| Comando | Código | Resultado |
|---|---:|---|
| `cargo run -q -p auditor-cli -- audit tests/fixtures/projects/layer_boundaries --format json --output docs/validation/etapa-3/cli-layer-boundaries.json` | `1` | JSON real com 3 achados, conforme esperado |
| `node --test tests/reportView.test.mjs` em `apps/desktop`, antes da implementação | `1` | falhou com `ERR_MODULE_NOT_FOUND` para `out/reportView.js` |
| `npm test` em `apps/desktop` | `0` | 7 testes passaram (6 de apresentação/resumo/filtros/cobertura/arestas e 1 de exclusões) |
| `npm test` em `extensions/vscode` | `0` | 7 testes passaram |
| `npm run build` em `apps/desktop`, primeira execução | `1` | `TS7053`: `availableSeverities` devolvia `string[]` e não podia indexar `Record<Severity, string>` |
| `npm run build` em `apps/desktop`, após tipar o retorno como `Severity[]` | `0` | frontend compilado |
| `npm run package` em `extensions/vscode` | `0` | pacote `.vsix` gerado com 7 arquivos |
| `cargo test -p auditor-desktop --lib` | `0` | comandos Tauri igualam ao núcleo, incluindo a regra de camada |

A verificação completa foi repetida depois das edições finais: `cargo fmt --all`, `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (185 testes: 171 núcleo, 10 CLI, 4 desktop), `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`, `npm test` e `npm run build` do desktop (7 testes), `npm test` da extensão (7 testes) e `git diff --check` retornaram `0`. O `.vscodeignore` foi ajustado para manter as exclusões originais e acrescentar `tests/**`, evitando incluir fontes de teste no pacote.

## Diagnóstico do relato do usuário e correções

Após abrir o executável diretamente e selecionar um projeto, o usuário relatou três problemas. Eles foram investigados no código e nos comandos reais:

1. **“Não encontra o arquivo `auditor.toml`”** — comportamento esperado do núcleo, não falha do executável. O contrato precisa estar na raiz da pasta selecionada; `apps/desktop` não contém `auditor.toml`. Reprodução:

   ```powershell
   cargo run -q -p auditor-cli -- audit apps/desktop
   ```

   Saída `2`: `Falha operacional: contrato: não foi possível ler apps\desktop\auditor.toml: O sistema não pode encontrar o arquivo especificado. (os error 2)`. O mesmo comando na raiz do repositório retornou resumo com 15 arquivos incluídos, 3 módulos e 17 arestas. A interface agora mostra o caminho esperado do contrato ao selecionar uma pasta; nenhum arquivo é criado ou alterado.

2. **“Só o botão auditar faz algo”** — os cinco comandos Tauri eram chamados corretamente, mas o feedback ficava longe dos botões: erros apenas no topo, grafo e contrato no fim da janela, e histórico vazio não renderizava nada. Foi acrescentada uma área de status com as seis ações e o resultado de cada uma, além de rolagem até o painel gerado.

3. **“UI simples, sem cor”** — o CSS original não estilizava botões nem severidades. Foram adicionados estados de botão, badges coloridos por severidade, legenda do grafo e hierarquia visual. Nenhuma dependência nova foi introduzida.

Os textos de status vivem em `apps/desktop/src/status.ts`, sem regras de auditoria. A fase vermelha falhou com `ERR_MODULE_NOT_FOUND`; a importação de `reportView` dentro de `status.ts` também quebrou em Node ESM e foi substituída por um argumento com a frase já pronta do resumo.

| Comando | Código | Resultado |
|---|---:|---|
| `cargo run -q -p auditor-cli -- audit apps/desktop` | `2` | reproduziu o erro de contrato ausente |
| `node --test tests/status.test.mjs`, antes da implementação | `1` | `ERR_MODULE_NOT_FOUND` para `out/status.js` |
| `npm test` em `apps/desktop` | `0` | 12 testes passaram |
| `npm run build` em `apps/desktop` | `0` | frontend compilado com o novo CSS |

## Checklist manual — pendente

Os itens abaixo exigem observação da janela Tauri ou da host do VS Code. Não houve automação gráfica neste ambiente; portanto **não foram marcados como concluídos**.

### Desktop

| Item verificado | Estado |
|---|---|
| Aplicação inicia sem erro (`npm run tauri dev` em `apps/desktop`) | Pendente |
| Selecionar pasta pelo diálogo nativo | Pendente |
| Iniciar auditoria manualmente sobre `tests/fixtures/projects/layer_boundaries` | Pendente |
| Resumo, cobertura, exclusões e achados na tela conferem com o JSON da CLI | Pendente na tela (dados já conferidos por teste) |
| Filtros de severidade e regra funcionam | Pendente na tela (lógica já conferida por teste) |
| Grafo mostra as mesmas arestas do núcleo | Pendente na tela (dados já conferidos por teste Rust) |
| Contrato exibido somente para leitura | Pendente |
| Registrar e consultar histórico por ação explícita | Pendente |
| Erro operacional aparece sem travar a janela | Pendente na tela (agora exibido na área de status com o rótulo da ação) |
| Caminho do banco pode ser configurado | Pendente |
| Área de status mostra o resultado de cada ação | Pendente na tela (5 testes dos helpers passaram) |
| Contrato esperado é informado após selecionar a pasta | Pendente na tela (caminho montado sem criar arquivo) |
| Histórico vazio mostra mensagem explícita | Pendente na tela (teste do helper passou) |
| A tela rola até o painel gerado pela ação | Pendente na tela |

### Extensão VS Code

| Item verificado | Estado |
|---|---|
| Detecção de `auditor.toml` na pasta aberta | Pendente |
| Comando de auditoria pela paleta e pelo menu de contexto | Pendente |
| Diagnósticos aparecem no painel Problems | Pendente |
| Severidade e `rule_id` conferem com a CLI | Pendente na host (conversão já conferida por teste) |
| Clique no achado abre arquivo e linha corretos | Pendente |
| Estado da última auditoria aparece na barra de status | Pendente |
| Códigos de saída `0` e `1` são aceitos | Pendente na host (códigos já testados na CLI) |

## Limitações desta validação

- Não há ferramenta de automação gráfica nesta sessão; a janela Tauri não foi inspecionada visualmente.
- A extensão não foi instalada na instância do VS Code do usuário; o pacote `.vsix` foi gerado mas não instalado.
- Nenhuma captura de tela foi gerada, para não registrar dados locais do projeto.
- Os testes provam paridade de dados, não eventos de clique, foco, diálogo nativo nem renderização efetiva no editor.

## Critério para concluir a Etapa 3.7

A etapa só será marcada como concluída depois que os dois checklists forem preenchidos pelo usuário, com as divergências registradas. Enquanto isso, a Etapa 3 permanece em andamento e a Etapa 3.6 continua pendente.
