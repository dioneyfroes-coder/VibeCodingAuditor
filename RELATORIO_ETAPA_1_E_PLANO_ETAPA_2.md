# Vibe Coding Auditor — Revisão da primeira fase e plano da próxima etapa

**Base analisada:** `VibeCodingAuditor-main.zip`  
**Plano de referência:** `PLANO_VIBE_CODING_AUDITOR.md`  
**Data da revisão:** 25/11/2026  
**Tipo de revisão:** inspeção estática da arquitetura e do código, conferência documental e verificações possíveis no ambiente de análise.

---

## 1. Parecer executivo

O projeto está significativamente mais avançado do que um primeiro MVP. O código implementa praticamente todo o percurso previsto no plano original: workspace Rust, CLI, contrato arquitetural TOML, descoberta de arquivos, análise de dependências, regras, relatórios, histórico SQLite, desktop Tauri/React e extensão VS Code. Também foram adicionados analisadores de TypeScript/JavaScript e Python, regras complementares e filtragem por `git diff`.

A separação arquitetural principal está correta: **o núcleo Rust concentra a lógica; CLI, Tauri e extensão funcionam como adaptadores**. O projeto permanece local, determinístico e somente leitura durante a auditoria normal. Não há dependência de IA para produzir os achados. Essas decisões estão coerentes com o objetivo original.

Entretanto, **não considero o motor pronto para ser tratado como auditor arquitetural confiável em projetos reais de portes variados**. Há cenários em que ele pode deixar código de fora sem avisar, classificar dependências legítimas como não resolvidas ou produzir violações definitivas a partir de um mapeamento ambíguo. A detecção de ciclos também pode crescer de forma muito cara em grafos densos.

Minha recomendação é não acrescentar funcionalidades de produto agora. A próxima etapa deve ser de **confiabilidade, cobertura explícita, precisão de classificação e desempenho do núcleo**. O objetivo é fazer o auditor saber dizer tanto “encontrei uma violação” quanto “não tenho informação suficiente para concluir”.

### Avaliação geral

| Área | Avaliação | Observação |
|---|---|---|
| Direção arquitetural | Boa | Núcleo separado das interfaces; monólito modular adequado. |
| Contrato TOML | Boa para v1 | Explícito, versionado e validado; ainda intencionalmente enxuto. |
| Descoberta de arquivos | Boa, com lacuna de transparência | Registra motivos internamente, mas o relatório final resume quase tudo em uma contagem. |
| Análise Rust | Parcialmente confiável | `syn` é uma boa escolha, mas dependências externas e caminhos relativos precisam de tratamento melhor. |
| Análise TypeScript/JavaScript | Experimental | Scanner próprio por linha tem falsos positivos e omissões previsíveis. |
| Análise Python | Experimental | Cobertura limitada e um caminho ambíguo pode escolher o primeiro destino. |
| Motor de regras | Boa base, com risco de escala | Algumas regras são heurísticas/proxies; detecção de ciclos enumera caminhos simples. |
| Persistência | Boa base | SQLite com migração e sem armazenar conteúdo integral dos arquivos. |
| CLI | Boa | Permite validar o núcleo sem depender da interface. |
| Desktop | Funcional como protótipo | Histórico visual é limitado; configuração do banco é temporária e relativa ao diretório de execução. |
| Extensão VS Code | Integração inicial útil | Consome JSON da CLI; não há testes executados dentro da host do VS Code. |
| Distribuição | Incompleta | `bundle.active` está desativado; não há instalador Tauri configurado. |
| Verificação contínua | Precisa melhorar | Não há workflow de CI no arquivo recebido; os testes não puderam ser reexecutados neste ambiente. |

---

## 2. O que foi implementado em relação ao plano original

O `docs/ROADMAP.md` marca as etapas 0 a 10 como concluídas. A inspeção do repositório confirma a presença das implementações correspondentes.

| Etapa do plano | Situação encontrada | Comentário |
|---|---|---|
| 0 — Preparação | Implementada | Há `AGENTS.md`, README, decisões e roadmap. O ZIP não inclui metadados Git, então não foi possível conferir o histórico do repositório. |
| 1 — Workspace e CLI | Implementada | Workspace Rust com `auditor-core`, `auditor-cli` e adaptador Rust do desktop. |
| 2 — Contrato | Implementada | Contrato TOML versionado, validação e comando `init`. |
| 3 — Descoberta | Implementada | `include`/`exclude`, diretórios ignorados e tratamento de links simbólicos/entradas inacessíveis. |
| 4 — Análise Rust | Implementada com limitações | Usa `syn` para `mod` e `use`; não interpreta toda referência existente em corpos de funções, macros ou código gerado. |
| 5 — Regras e relatórios | Implementada | Achados, severidades e saídas texto, JSON e Markdown. |
| 6 — Histórico | Implementada | SQLite, migração versionada e comandos `record`, `list`, `show` e `compare`. |
| 7 — Prova do núcleo | Implementada documentalmente | Há fixtures `clean`, `dependencies` e `partial`, além de outras fixtures especializadas. Os resultados anotados no roadmap não puderam ser reproduzidos neste ambiente. |
| 8 — Desktop | Implementada como protótipo | Tauri 2, React e TypeScript chamando o núcleo. O bundle de distribuição está desativado. |
| 9 — VS Code | Implementada como integração inicial | Diagnósticos, navegação até arquivo/linha, comando manual e estado na barra. Não há testes na host real do VS Code. |
| 10 — Expansões | Implementada além do escopo inicial | TypeScript/JavaScript, Python, concentração, duplicação exata, módulo sem referências e filtro de auditoria por patch Git. |

### Escopo que foi ultrapassado

O plano original tratava suporte a outras linguagens, extensão VS Code, heurísticas extras e Git diff como itens posteriores ao MVP. O repositório já contém esses recursos. Isso é positivo como prova de avanço, mas também significa que o núcleo aumentou de tamanho antes de algumas limitações fundamentais de classificação terem sido resolvidas.

A conclusão da primeira fase deve ser entendida como **“protótipo integrado com amplo conjunto inicial de recursos”**, não como “auditor genérico e completo para qualquer projeto”.

---

## 3. Pontos fortes da implementação

### 3.1 Fronteiras entre componentes

A divisão atual é apropriada:

- `crates/auditor-core`: contrato, descoberta, análise, regras, relatórios, diff e histórico.
- `crates/auditor-cli`: argumentos e apresentação do resultado; não deve duplicar regras.
- `apps/desktop/src-tauri`: comandos Tauri que chamam o núcleo.
- `apps/desktop/src`: interface React que apresenta dados do núcleo.
- `extensions/vscode`: converte o JSON da CLI em diagnósticos do editor.

Isso mantém uma única fonte de verdade para as regras e reduz o risco de CLI, desktop e VS Code discordarem entre si.

### 3.2 Contrato explícito e leitura somente

O `auditor.toml` descreve o projeto analisado. O carregamento valida os campos e rejeita propriedades desconhecidas. O comando `init` usa criação exclusiva do arquivo, evitando sobrescrever um contrato que já existe.

A auditoria não executa código do projeto e não modifica fontes ou contrato. Esse comportamento é importante para uma ferramenta que será usada junto de agentes de programação.

### 3.3 Persistência e privacidade

O histórico usa SQLite com migração identificada. A documentação e o código indicam que o banco armazena resumo, achados e limitações, não o conteúdo integral dos arquivos analisados. A abertura do banco habilita chaves estrangeiras e a migração é transacional.

### 3.4 Testabilidade

As fixtures cobrem contratos válidos/inválidos e projetos com dependências permitidas, proibidas, não resolvidas, ambiguidades, duplicações, concentração e módulos sem referências. Esse formato é melhor do que depender somente de testes contra o próprio código do auditor.

### 3.5 Decisões documentadas

`AGENTS.md` dá limites úteis ao OpenCode: uma etapa por vez, justificar mudanças arquiteturais, não adicionar dependências sem motivo, não esconder limitações e manter o usuário como autoridade final. `docs/DECISIONS.md` também registra decisões e limitações em vez de deixar as escolhas implícitas.

---

## 4. Problemas que precisam ser corrigidos

Os níveis abaixo indicam prioridade para a próxima etapa, não severidade de segurança.

### P1 — Arquivos de linguagens não suportadas desaparecem da análise sem aviso

**Local:** `crates/auditor-core/src/analysis/mod.rs`, aproximadamente linhas 77–82 e 301–305.

`is_source_file()` reconhece apenas `.rs`, `.ts`, `.tsx`, `.js`, `.jsx` e `.py`. O laço principal ignora qualquer outra extensão com `continue`, sem adicionar uma limitação ao relatório.

Exemplo: um projeto pode incluir `.go`, `.java`, `.cs`, `.cpp` ou `.rb` no inventário. Esses arquivos contam como incluídos, mas não entram na análise de dependências e não geram aviso de linguagem não suportada. O relatório pode parecer limpo mesmo com parte importante do projeto fora da cobertura.

**Correção necessária:** separar “arquivo incluído no inventário” de “arquivo efetivamente analisado”. Registrar cobertura por extensão/linguagem e adicionar uma limitação explícita sempre que houver arquivo de código incluído sem analisador. O relatório deve deixar claro quando a auditoria é parcial.

**Critério de aceite:** uma fixture com pelo menos uma extensão não suportada precisa gerar uma limitação visível em texto, Markdown, JSON e desktop. Não deve ser apresentada como auditoria integral.

### P1 — Arquivo associado a mais de um módulo ainda é analisado pelo primeiro módulo

**Local:** `crates/auditor-core/src/analysis/mod.rs`, aproximadamente linhas 479–510.

`module_for_file()` registra `AmbiguousModule`, mas retorna `matching_modules.first()`. O restante da análise continua como se essa associação fosse verdadeira. Assim, uma dependência pode ser classificada como proibida com base em uma escolha arbitrária da ordem do contrato.

Uma indicação de ambiguidade não pode coexistir com uma conclusão definitiva baseada no primeiro resultado.

**Correção necessária:** quando houver mais de um módulo compatível, não escolher um módulo como origem. Interromper a classificação arquitetural daquele arquivo ou produzir apenas revisão necessária, com os módulos candidatos listados.

**Critério de aceite:** reorganizar a ordem dos módulos no TOML não pode alterar achados definitivos de um arquivo ambíguo. A fixture precisa verificar que nenhuma violação `error` é emitida a partir dessa associação incerta.

### P1 — Dependências externas Rust são confundidas com módulos não resolvidos

**Local:** `crates/auditor-core/src/analysis/mod.rs`, aproximadamente linhas 551–598.

O classificador de `use` trata raízes conhecidas como `std`, `core`, `alloc` e `test` como padrões ignorados. Se a raiz não for um módulo do contrato nem um pacote reconhecido do workspace, ela vira `Unresolved`.

Por isso, imports comuns como `use serde::Serialize;`, `use syn::Item;` ou `use clap::Parser;` podem aparecer como dependências não resolvidas, embora sejam crates externas válidas. O código atual lê nomes de pacotes do workspace, mas não usa as tabelas de dependências do Cargo para classificar crates externas.

**Correção necessária:** distinguir pelo menos três categorias: módulo arquitetural do contrato, pacote externo declarado no Cargo e destino realmente não identificado. Considerar dependências normais, de desenvolvimento, de build, dependências específicas por plataforma, `[workspace.dependencies]` e aliases/nomes renomeados. Não transformar automaticamente qualquer raiz desconhecida em dependência externa: ela precisa ser justificada pelo manifesto ou continuar como revisão.

**Critério de aceite:** imports de uma dependência declarada, inclusive com alias, não devem criar `dep.unresolved`; um nome ausente tanto do contrato quanto dos manifests continua sendo revisão.

### P1 — O custo da detecção de ciclos pode crescer exponencialmente

**Local:** `crates/auditor-core/src/rules/mod.rs`, aproximadamente linhas 457–499.

`find_cycles()` inicia uma busca a partir de cada nó e `visit_cycles()` enumera caminhos simples até encontrar fechamentos de ciclos. Em grafos densos, a quantidade de caminhos/ciclos simples pode crescer exponencialmente. A documentação já reconhece que os ciclos podem se sobrepor e produzir mais de um caminho para a mesma região cíclica.

**Correção necessária:** usar componentes fortemente conectados (SCC), por Tarjan ou Kosaraju, para detectar regiões cíclicas em tempo linear em relação a nós e arestas (`O(V + E)`). O relatório pode emitir um achado por componente cíclico ou um conjunto controlado de evidências, sem enumerar todas as permutações de caminhos.

**Critério de aceite:** teste de grafo grande e denso com limite de tempo razoável e resultado determinístico. O número de achados não deve explodir com a quantidade de ciclos simples sobrepostos.

### P2 — Caminhos Rust `self::` e `super::` são ignorados incondicionalmente

**Local:** `crates/auditor-core/src/analysis/mod.rs`, aproximadamente linhas 565–575.

O classificador retorna imediatamente para `self` e `super`, sob a suposição de que são dependências internas ao mesmo módulo arquitetural. Isso nem sempre é verdade: um arquivo pode estar em um módulo do contrato e referenciar um irmão ou ancestral que pertence a outro módulo do contrato.

**Correção necessária:** resolver caminhos relativos a partir do arquivo/módulo Rust de origem e relacionar o destino ao contrato. Se a resolução não for segura, registrar `needs_review`/limitação explícita. Não tratar todo `self`/`super` como irrelevante.

**Critério de aceite:** fixtures com `super::` e `self::` dentro e entre fronteiras declaradas, verificando dependência permitida, proibida ou revisão conforme o caso.

### P2 — Padrões glob em `[workspace].members` não são suportados

**Local:** `crates/auditor-core/src/analysis/mod.rs`, aproximadamente linhas 708–724. Limitação reconhecida em `docs/DECISIONS.md`, seção D4.2.

O código concatena cada entrada de `workspace.members` diretamente com `Cargo.toml`. Um membro declarado como `crates/*` é tratado como caminho literal e causa falha ao tentar ler `crates/*/Cargo.toml`.

Esse padrão é comum em workspaces Rust com vários crates.

**Correção necessária:** expandir os padrões de forma segura e determinística, respeitando também `workspace.exclude`; ou, se a expansão não for suportada, detectar o padrão antes da leitura e registrar uma limitação clara em vez de uma falha de arquivo inexistente. A decisão sobre uma dependência nova deve ser justificada antes de adicioná-la.

**Critério de aceite:** fixture de workspace com `members = ["crates/*"]` e exclusões. A análise deve descobrir os membros reais, sem executar o Cargo, ou relatar a limitação explicitamente.

### P2 — Scanner TypeScript/JavaScript pode perder imports e produzir falsos positivos

**Local:** `crates/auditor-core/src/analysis/typescript.rs`, aproximadamente linhas 111–160 e 336–351.

O scanner analisa linha a linha. Isso falha com imports estáticos multilinha, por exemplo:

```ts
import {
  User,
  Account
} from "./domain";
```

A linha que contém `import` não contém o `from` nem o literal; a linha final não contém a palavra `import`. A dependência pode não ser detectada.

Também há risco de falso positivo: o parser procura a palavra `import` e, quando não encontra `from`, procura um literal posterior na mesma linha. Uma linha contendo `import.meta` e uma string posterior não relacionada pode ser interpretada como importação.

**Correção necessária:** substituir a leitura ingênua por linha por um scanner de declarações que reconheça limites de instrução e contexto lexical, ou usar parser apropriado se o custo for justificado. Diferenciar `import.meta` de `import`, preservar evidência/linha e converter construções não suportadas em limitações explícitas.

**Critério de aceite:** testes para import multilinha, `import.meta`, `import()`, `require()`, `export ... from`, comentários, strings, template literals e regex. O scanner não pode classificar uma string sem relação como destino de import.

### P2 — O analisador Python escolhe o primeiro módulo de destino em caso de ambiguidade

**Local:** `crates/auditor-core/src/analysis/python.rs`, aproximadamente linhas 356–376.

Depois de resolver um arquivo de destino, o código coleta os módulos do contrato que correspondem ao caminho e usa `targets.first()`. Diferentemente do tratamento de nomes de import ambíguos, esse trecho não verifica se mais de um módulo corresponde ao arquivo.

**Correção necessária:** aplicar a mesma política de ambiguidade definida para Rust e TypeScript: não escolher o primeiro destino; registrar revisão com todos os candidatos.

**Critério de aceite:** fixture em que um caminho Python corresponda a dois módulos; nenhum `dep.not_allowed` pode depender da ordem do contrato.

### P2 — Os motivos de arquivos ignorados não chegam ao relatório final

**Local:** `crates/auditor-core/src/discovery/mod.rs`, `crates/auditor-core/src/report/mod.rs` e interface desktop.

A descoberta armazena cada entrada ignorada com um motivo (`DefaultIgnored`, `Excluded`, `NotIncluded`, `Symlink`, `Inaccessible`, `Unsupported`). Porém, o relatório final expõe principalmente `files_skipped` como contagem. A informação detalhada não acompanha o relatório de auditoria.

Isso reduz a capacidade de explicar por que um arquivo não foi considerado: um diretório gerado ignorado de propósito e um arquivo inacessível não deveriam parecer a mesma coisa.

**Correção necessária:** transportar um resumo estruturado dos motivos para o relatório. Mostrar contagens por motivo e destacar situações que reduzem a cobertura, como entradas inacessíveis ou tipos não tratados. A interface desktop deve apresentar esse resumo.

**Critério de aceite:** fixture com pelo menos três motivos distintos; CLI e desktop exibem a diferença sem despejar listas enormes por padrão.

---

## 5. Limitações conhecidas que são aceitáveis, desde que permaneçam explícitas

Nem toda limitação precisa ser removida na próxima etapa. Algumas são escolhas razoáveis para um primeiro motor determinístico.

- **Duplicação:** a regra compara linhas idênticas com pelo menos 40 caracteres. Não identifica blocos equivalentes, renomeações ou duplicação semântica. Deve continuar sendo apresentada como sinal, não prova de código duplicado em sentido amplo.
- **Módulo sem referências:** zero arestas incidentes não prova que o código esteja morto. Entradas dinâmicas, reflexão, macros, registradores, plugins e pontos de entrada podem não aparecer no grafo.
- **Concentração:** o limite fixo de três origens/destinos é uma heurística de revisão. Não é uma violação arquitetural universal.
- **Rust:** a análise se concentra em declarações `mod` e `use`. Não é uma análise completa de todos os usos de símbolos em corpos, macros, código gerado ou resolução de nomes do compilador.
- **TypeScript/JavaScript:** não há resolução de `tsconfig.paths`, workspaces npm e pacotes npm. Imports dinâmicos construídos em runtime não são resolvidos.
- **Python:** imports dinâmicos, `import *`, alguns imports relativos e resolução de ambientes/pacotes ficam fora da cobertura.
- **Git diff:** a filtragem por patch tem limitações documentadas para diffs combinados, escapes, renomeações puras e linhas removidas.
- **VS Code:** o roadmap declara que não existem testes executados na host real do VS Code. Os testes puros da conversão de diagnósticos não substituem um teste de integração do editor.

A regra geral deve ser: **limitação conhecida e visível é aceitável; omissão silenciosa que aparenta cobertura completa não é.**

---

## 6. Interface desktop e distribuição

### 6.1 Histórico visual incompleto

O núcleo e a CLI oferecem `history list`, `show` e `compare`. A interface desktop mostra a lista resumida de execuções, mas não oferece uma navegação completa para abrir os detalhes de uma execução ou comparar duas delas.

Isso não invalida o histórico do núcleo, mas significa que a integração desktop está parcial em relação ao valor disponível na camada central. Recomendo não priorizar essa expansão antes de corrigir a confiabilidade da análise.

### 6.2 Caminho do banco de dados

Em `apps/desktop/src/App.tsx`, o caminho começa em `data/auditor.db` e fica em estado React. A própria interface informa que o caminho é relativo ao diretório de execução. O valor personalizado não é persistido entre sessões.

Para uma versão distribuível, o padrão mais robusto será usar o diretório de dados da aplicação fornecido pelo Tauri e persistir a preferência do usuário. Isso deve ser feito numa etapa posterior, sem misturar essa tarefa às correções do motor.

### 6.3 Empacotamento

`apps/desktop/src-tauri/tauri.conf.json` contém `"bundle": { "active": false }`. Isso permite manter o desenvolvimento/compilação do aplicativo, mas desativa a geração dos instaladores/artefatos de bundle do Tauri. O projeto ainda não deve ser tratado como instalador pronto para distribuição.

---

## 7. Testes e verificações desta revisão

### 7.1 O que foi conferido

- Foram encontrados **101 testes Rust** no código: 88 no `auditor-core`, 10 na CLI e 3 no backend Tauri.
- Foram encontrados **4 testes** na extensão VS Code.
- Todos os arquivos `.toml` e `.json` do repositório passaram por uma leitura sintática nesta revisão.
- A presença dos testes, fixtures, comandos e integrações foi conferida estaticamente.
- O `docs/ROADMAP.md` registra que as suítes e comandos de build passaram no ambiente de desenvolvimento do projeto em 25/11/2026.

### 7.2 O que não foi possível reproduzir aqui

- O ambiente desta revisão não tem `cargo` nem `rustc` instalados; portanto não foi possível executar `cargo test`, `cargo clippy`, `cargo fmt` ou compilar o backend Rust.
- A instalação das dependências Node da extensão não foi concluída dentro do tempo disponível. A tentativa de executar os testes sem as dependências completas falhou por ausência de tipos de `node` e `vscode`. Não considero isso uma falha comprovada do código-fonte; é uma verificação que permanece pendente.
- Não foi possível executar a aplicação Tauri nem abrir uma instância real do VS Code.
- O ZIP não inclui a pasta `.git`, então não foi possível repetir `git diff --check` nem inspecionar commits.

**Interpretação correta:** os números de testes e os resultados de build do roadmap são resultados documentados pelo projeto, mas não foram confirmados independentemente nesta execução. A análise de código e a validação sintática de TOML/JSON foram feitas; a validação de compilação precisa ser repetida no Windows 11 de desenvolvimento.

### 7.3 Verificação necessária no ambiente Windows do projeto

Antes de iniciar alterações, executar e guardar a saída de:

```powershell
cargo --version
rustc --version
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

cd extensions/vscode
npm ci
npm test

cd ../../apps/desktop
npm ci
npm run build
```

O build empacotado do Tauri deve continuar fora do escopo imediato enquanto `bundle.active` permanecer desativado.

---

## 8. Plano da Etapa 2 — Confiabilidade do motor

### Objetivo

Tornar as conclusões da auditoria mais seguras e tornar explícita a cobertura real. Não ampliar a lista de funcionalidades de produto até que o núcleo seja confiável nos cenários já suportados.

### Regras de escopo

1. Não implementar IA, agentes autônomos, sugestões automáticas ou edição de arquivos auditados.
2. Não adicionar novas linguagens nesta etapa.
3. Não criar novos serviços, daemon ou microserviços.
4. Não duplicar regras em CLI, Tauri, React ou VS Code.
5. Preservar a compatibilidade do JSON consumido pela extensão sempre que possível. Se uma mudança incompatível for indispensável, documentar e versionar explicitamente.
6. Não adicionar dependências sem apresentar justificativa, alternativas e impacto.
7. Corrigir um item por vez: teste de regressão primeiro, implementação depois, revisão final por último.
8. Não marcar como concluído um item só porque compila; todos os critérios de aceite daquele item precisam estar cobertos por testes.

### Ordem de execução

#### 2.1 — Cobertura honesta dos arquivos

- Diferenciar arquivos inventariados de arquivos analisados.
- Detectar extensões de código incluídas que ainda não possuem analisador.
- Acrescentar limitações explícitas ao relatório e à interface.
- Definir uma indicação estruturada de cobertura parcial, sem inventar uma nota geral de qualidade.

**Pronto quando:** uma fixture com código em extensão não suportada deixa claro que a auditoria não cobriu aquele arquivo.

#### 2.2 — Eliminar conclusões definitivas baseadas em mapeamentos ambíguos

- Corrigir `module_for_file()` para não retornar o primeiro módulo quando houver mais de um candidato.
- Corrigir a seleção de destino ambígua no Python.
- Garantir que ambiguidade gere apenas revisão necessária, não erro arquitetural definitivo.
- Verificar se o resultado independe da ordem dos módulos em `auditor.toml`.

**Pronto quando:** testes comprovam que a ordem do contrato não muda a classificação de arquivos ambíguos.

#### 2.3 — Classificar dependências externas Rust corretamente

- Ler as tabelas relevantes dos manifests Cargo sem executar o projeto.
- Reconhecer dependências normais, de desenvolvimento e de build.
- Considerar dependências declaradas no workspace e aliases/nomes renomeados.
- Separar crate externa declarada de módulo arquitetural e de símbolo desconhecido.
- Resolver `self::` e `super::` quando houver evidência suficiente; caso contrário, registrar revisão explícita.

**Pronto quando:** crates externas válidas não aparecem como `dep.unresolved`; caminhos relativos entre fronteiras do contrato não são ignorados silenciosamente.

#### 2.4 — Tratar `workspace.members` com glob

- Implementar expansão segura de padrões como `crates/*`, respeitando exclusões do workspace.
- Evitar executar `cargo metadata` ou comandos do projeto durante a auditoria normal.
- Se uma forma de glob não for suportada, apresentar uma limitação específica e acionável.

**Pronto quando:** fixtures com membros explícitos, glob e `workspace.exclude` têm resultado determinístico.

#### 2.5 — Corrigir a análise TypeScript/JavaScript

- Reconhecer imports estáticos multilinha.
- Não confundir `import.meta` com declaração de import.
- Testar comentários, strings, template literals, regex e imports dinâmicos.
- Garantir que um literal não relacionado não seja escolhido como destino de import.
- Registrar construções não suportadas como limitações.

**Pronto quando:** os testes de regressão cobrem falsos positivos e falsos negativos conhecidos e a fixture anterior mantém seu resultado esperado.

#### 2.6 — Substituir enumeração de ciclos por componentes fortemente conectados

- Implementar Tarjan ou Kosaraju.
- Definir saída determinística para cada componente cíclico.
- Preservar evidências úteis sem enumerar todos os caminhos simples.
- Adicionar testes de grafo grande, denso, desconectado e sem ciclo.

**Pronto quando:** o algoritmo opera em `O(V + E)` e o número de achados é limitado por componente, não pelo número de caminhos simples.

#### 2.7 — Expor os motivos de exclusão e a cobertura no relatório

- Transportar motivos da descoberta até `AuditReport`.
- Expor contagens por motivo.
- Distinguir exclusão intencional de falha de acesso e tipo não suportado.
- Apresentar as informações no terminal, JSON, Markdown e desktop.

**Pronto quando:** o usuário consegue explicar por que os arquivos ficaram fora sem precisar inspecionar logs internos.

#### 2.8 — Reforçar a verificação automatizada

- Criar workflow de CI para Windows, que é a plataforma-alvo inicial.
- Executar fmt, testes, clippy, build da desktop e testes da extensão.
- Adicionar testes de desempenho para SCC e testes de regressão para os scanners.
- Manter a versão mínima do Rust e pré-requisitos documentados de forma reproduzível.
- Não declarar o bundle Tauri pronto enquanto o empacotamento continuar desativado.

**Pronto quando:** uma mudança futura no núcleo não consegue ser mesclada sem executar os verificadores principais em ambiente Windows automatizado.

---

## 9. Critérios para fechar a Etapa 2

A Etapa 2 só estará concluída quando:

- [ ] Arquivos incluídos sem analisador geram aviso de cobertura parcial.
- [ ] Mapeamentos ambíguos não originam violações definitivas.
- [ ] Dependências externas Rust declaradas são classificadas corretamente.
- [ ] `self::` e `super::` não são descartados sem resolução ou revisão explícita.
- [ ] Workspaces com glob são analisados ou recebem limitação clara e testada.
- [ ] Imports TypeScript multilinha e `import.meta` têm comportamento coberto por testes.
- [ ] Detecção de ciclos usa algoritmo de componentes fortemente conectados.
- [ ] Motivos de arquivos ignorados aparecem no relatório.
- [ ] CLI, desktop e extensão continuam consumindo o mesmo modelo de achados.
- [ ] Suíte Rust completa, fmt e clippy passam no Windows.
- [ ] Build TypeScript/React e testes da extensão passam.
- [ ] README, roadmap e decisões refletem somente o que foi efetivamente validado.

---

## 10. Primeira instrução para o OpenCode

**Não peça ao OpenCode para executar a Etapa 2 inteira em uma única solicitação.** Comece pelo item 2.1 e faça uma entrega pequena, revisável e testada.

Copie este prompt para o OpenCode com a pasta do projeto aberta:

```text
Leia integralmente AGENTS.md, docs/ROADMAP.md, docs/DECISIONS.md e
RELATORIO_ETAPA_1_E_PLANO_ETAPA_2.md.

O projeto concluiu a primeira fase, mas a revisão identificou lacunas de
confiabilidade. Vamos iniciar a Etapa 2, item 2.1 — Cobertura honesta dos
arquivos.

Escopo exclusivo desta tarefa:
1. Inspecionar o fluxo de inventário, análise, relatório JSON/Markdown/texto,
   CLI e apresentação desktop.
2. Reproduzir com uma fixture o caso de arquivo de código incluído no inventário
   cuja extensão ainda não possui analisador.
3. Criar primeiro os testes que demonstram o comportamento esperado.
4. Fazer a menor alteração arquitetural necessária para indicar explicitamente
   que a auditoria é parcial quando houver arquivo de código não analisado.
5. Preservar o contrato TOML v1 e a compatibilidade do JSON consumido pela
   extensão, sempre que possível. Se isso não for possível, pare e explique
   a mudança de contrato proposta antes de implementá-la.
6. Atualizar README, ROADMAP e DECISIONS somente com fatos verificados.

Restrições:
- Não implementar nenhum outro item da Etapa 2.
- Não adicionar linguagens, IA, daemon, agente, refatoração automática ou
  funcionalidades novas.
- Não modificar arquivos do projeto auditado durante a auditoria.
- Não adicionar dependência sem justificar necessidade e alternativas.
- Não escolher silenciosamente um comportamento quando houver ambiguidade.
- Não enfraquecer, remover ou alterar testes existentes apenas para fazê-los
  passar.
- Antes de editar, inspecionar o estado atual do repositório e preservar toda
  alteração preexistente do usuário.

Ao concluir, informe:
- arquivos alterados;
- decisão técnica tomada e alternativas rejeitadas;
- testes novos e o que cada um protege;
- comandos executados e resultados reais;
- limitações que continuam abertas;
- confirmação explícita de que nenhum item além do 2.1 foi iniciado.

Não marque o item como concluído se qualquer verificação relevante falhar.
```

---

## 11. Conclusão

A primeira fase entregou uma base real, não apenas um esqueleto. O desenho Rust + CLI + Tauri + React + SQLite + VS Code está coerente, e o núcleo já oferece um conjunto considerável de recursos.

O principal risco atual não é falta de funcionalidades. É o auditor transmitir mais certeza do que a análise realmente possui. A próxima etapa deve reduzir falsos positivos, evitar conclusões baseadas em ambiguidades, expor lacunas de cobertura e manter o custo dos algoritmos previsível.

**Direção recomendada:** congelar novas funcionalidades e tornar o motor confiável antes de começar o projeto Artificial Life Lab usando o auditor como ferramenta de fiscalização.
