# Etapa 3.7 — Validação ponta a ponta do desktop e da extensão

Data: 26/11/2026
Branch: `etapa-3/validacao-arquitetural`
Status: **concluída em 26/11/2026; paridade automatizada e checklists manuais do desktop e da extensão VS Code confirmados pelo usuário**

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

## Checklist manual

Os itens do desktop exigiram observação da janela Tauri e foram confirmados pelo usuário em 26/11/2026, que respondeu: “agora todos botões tem função executada, visual ficou mais agradável”. Ao ser perguntado explicitamente, confirmou também filtros, grafo, contrato somente leitura, registro/consulta de histórico e erro operacional. Os itens da extensão foram executados na host do VS Code e confirmados pelo usuário em 26/11/2026, depois da instalação do `.vsix` e da configuração do caminho da CLI. Não houve automação gráfica neste ambiente nem capturas de tela; as confirmações vieram do usuário e do estado persistido pela extensão.

### Desktop — concluído em 26/11/2026

| Item verificado | Estado |
|---|---|
| Aplicação inicia sem erro (`npm run tauri dev` em `apps/desktop`) | Concluído pelo usuário |
| Selecionar pasta pelo diálogo nativo | Concluído pelo usuário |
| Iniciar auditoria manualmente sobre `tests/fixtures/projects/layer_boundaries` | Concluído pelo usuário (alvo sintético também auditado) |
| Resumo, cobertura, exclusões e achados na tela conferem com o JSON da CLI | Concluído pelo usuário; dados também conferidos por teste |
| Filtros de severidade e regra funcionam | Concluído pelo usuário; lógica também conferida por teste |
| Grafo mostra as mesmas arestas do núcleo | Concluído pelo usuário; dados também conferidos por teste Rust |
| Contrato exibido somente para leitura | Concluído pelo usuário |
| Registrar e consultar histórico por ação explícita | Concluído pelo usuário |
| Erro operacional aparece sem travar a janela | Concluído pelo usuário |
| Caminho do banco pode ser configurado | Concluído pelo usuário |
| Área de status mostra o resultado de cada ação | Concluído pelo usuário; helpers cobertos por 5 testes |
| Contrato esperado é informado após selecionar a pasta | Concluído pelo usuário |
| Histórico vazio mostra mensagem explícita | Concluído pelo usuário; helper coberto por teste |
| A tela rola até o painel gerado pela ação | Concluído pelo usuário |

O executável release foi recompilado depois das correções (`npm run tauri build` = `0`) e a auditoria do alvo sintético foi repetida: `audit` retornou `1` com 7 achados e `analyze` retornou `0` com 6 arestas. Nenhum arquivo do alvo foi alterado.

### Extensão VS Code — concluída em 26/11/2026

Antes do teste, a CLI release foi compilada e o `.vsix` foi instalado na instância do VS Code do usuário. O comando `vibeCodingAuditor.cliPath` foi configurado no `settings.json` de usuário, fora do repositório, porque a pasta auditada não contém `target/release`. O usuário inicialmente procurou por um comando chamado “diagnóstico” e queixou-se de ausência de menu de contexto; esclarecido que o painel se chama **Problems** e que o menu aparece ao clicar com o botão direito em uma **pasta**, todos os itens foram conferidos.

| Item verificado | Estado |
|---|---|
| Detecção de `auditor.toml` na pasta aberta | Concluído pelo usuário; a auditoria foi executada na raiz do alvo |
| Comando de auditoria pela paleta e pelo menu de contexto | Concluído pelo usuário: `Vibe` na paleta e botão direito na pasta `src` |
| Diagnósticos aparecem no painel Problems | Concluído pelo usuário: 7 itens na aba Problemas junto do terminal |
| Severidade e `rule_id` conferem com a CLI | Concluído pelo usuário: 3 `dep.not_allowed`, 1 `dep.cycle`, 1 `code.duplicated` e 2 `file.unmapped` |
| Clique no achado abre arquivo e linha corretos | Concluído pelo usuário |
| Estado da última auditoria aparece na barra de status | Concluído pelo usuário: `VCA: 7 achados` |
| Códigos de saída `0` e `1` são aceitos | Concluído na host: a CLI retornou `1` e a extensão concluiu sem erro operacional |

Evidências automaticamente verificáveis:

| Comando/arquivo | Resultado |
|---|---|
| `cargo build --release -p auditor-cli` | `0`; gerou `target/release/vibe-coding-auditor.exe` (5.760.512 bytes) |
| `code --install-extension extensions\vscode\vibe-coding-auditor-0.1.0.vsix --force` | `0`; `code --list-extensions --show-versions` mostrou `dioneyfroes-coder.vibe-coding-auditor@0.1.0` |
| `vibe-coding-auditor.exe audit validation/targets/fake-order-service --format json` | `1`; 7 achados (3 erros, 1 aviso, 3 revisões) |
| `exthost.log` da janela do alvo | extensão ativada em `15:30:54` sem erro |
| `workspaceStorage\27e98a66...\state.vscdb` | chave `dioneyfroes-coder.vibe-coding-auditor` com `lastAudit`: mesmo resumo da CLI |

## Limitações desta validação

- Não houve automação gráfica nesta sessão; a janela Tauri e a host do VS Code foram conferidas pelo usuário, sem capturas de tela.
- A extensão passou a ser instalada e testada na instância do VS Code do usuário; a configuração do caminho da CLI ficou no `settings.json` de usuário, fora do repositório.
- Nenhuma captura de tela foi gerada, para não registrar dados locais do projeto.
- Os testes provam paridade de dados; os eventos de clique, foco, diálogo nativo e renderização efetiva foram confirmados pelo usuário, não por automação.

## Critério para concluir a Etapa 3.7

Os dois checklists foram preenchidos e confirmados pelo usuário em 26/11/2026, com a divergência inicial registrada (termo “diagnóstico” e menu de contexto confundido com arquivo). A Etapa 3.7 está concluída. A Etapa 3.6 foi concluída posteriormente, com aprovação registrada em `E3.12`; a Etapa 3 foi declarada concluída em 26/11/2026.
