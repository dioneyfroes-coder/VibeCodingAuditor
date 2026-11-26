# Vibe Coding Auditor — Plano de modificações e execuções da Etapa 3

**Data de referência:** 26/11/2026  
**Repositório:** https://github.com/dioneyfroes-coder/VibeCodingAuditor  
**Branch de referência analisada:** `main`  
**Objetivo:** validar o auditor em projetos reais, corrigir pontos que possam comprometer a confiança dos resultados e evoluir o contrato arquitetural sem quebrar projetos que ainda usam o schema v1.

---

## 1. Instruções para o OpenCode

Este documento é o plano de trabalho da próxima etapa. Leia também, antes de alterar qualquer arquivo:

1. `AGENTS.md`
2. `README.md`
3. `docs/ROADMAP.md`
4. `docs/DECISIONS.md`
5. `auditor.example.toml`

### Regras obrigatórias

- Trabalhe em uma tarefa por vez e respeite a ordem deste documento.
- Antes de editar, inspecione a implementação atual e confirme se o problema descrito ainda existe.
- Não marque uma tarefa como concluída apenas porque o código compila. Execute os testes e os cenários de aceitação descritos.
- Crie ou atualize testes de regressão antes de corrigir comportamentos existentes.
- Não faça refatorações amplas sem necessidade demonstrada.
- Não adicione dependências sem justificar o benefício, custo e alternativa sem dependência.
- Não altere silenciosamente o schema do contrato, o schema SQLite, os `rule_id` ou o formato JSON consumido pela CLI, desktop e extensão.
- Não remova nem recrie o banco de histórico existente para facilitar uma migração.
- Não envie código-fonte para serviços externos. A análise deve permanecer local.
- Não execute scripts, builds ou comandos do projeto que está sendo auditado. O Vibe Coding Auditor deve continuar sendo somente leitura em relação ao projeto-alvo, exceto quando o usuário autorizar explicitamente uma operação.
- Não instale, atualize ou remova ferramentas do Windows sem autorização explícita.
- Não invente resultados. Registre os comandos executados, código de saída e resumo real da saída.
- Não implemente IA, regras em linguagem natural, auditoria agendada, daemon ou microserviços nesta etapa.
- Não avance para o Artificial Life Lab. O objetivo desta etapa é provar a confiabilidade do auditor.

### Processo de cada tarefa

1. Ler os arquivos envolvidos e explicar brevemente o comportamento atual.
2. Criar um teste que reproduza o problema ou comprove a nova capacidade.
3. Implementar a menor mudança necessária.
4. Rodar os testes focados.
5. Rodar as verificações gerais aplicáveis.
6. Revisar nomes, responsabilidades, mensagens, documentação e possíveis efeitos colaterais.
7. Atualizar `docs/ROADMAP.md` e `docs/DECISIONS.md` somente com fatos comprovados.
8. Apresentar um resumo do que mudou e dos comandos que passaram/falharam.

Se uma tarefa depender de decisão de produto ou de uma definição arquitetural que ainda não existe, pare nessa tarefa e apresente alternativas. Não escolha silenciosamente pelo usuário.

---

## 2. Estado de partida conhecido

O `main` já contém:

- Workspace Rust com `auditor-core`, `auditor-cli` e backend Tauri.
- CLI com `init`, `scan`, `analyze`, `audit` e comandos de histórico.
- Contrato arquitetural `auditor.toml`, atualmente `schema_version = 1`.
- Análise de Rust, TypeScript/JavaScript e Python, com limitações documentadas.
- Regras determinísticas, relatórios em texto, JSON e Markdown e histórico SQLite.
- Aplicação desktop Tauri 2 + React + TypeScript.
- Extensão VS Code que consome o JSON da CLI.
- Etapas 0–10, trabalhos R1–R6 e Etapa 2 (itens 2.1–2.8) registradas como concluídas no roadmap.
- CI Windows com `cargo fmt`, `check`, `test`, `clippy`, testes/build do desktop e teste da extensão.

A Etapa 3 proposta aqui ainda **não está implementada**. Este arquivo é um plano, não um relatório de conclusão.

### Observação crítica para validar primeiro

O CLI e o backend Tauri declaram dependência local de `auditor-core` por `path`:

- `crates/auditor-cli/Cargo.toml` → `auditor-core = { path = "../auditor-core" }`
- `apps/desktop/src-tauri/Cargo.toml` → `auditor-core = { path = "../../../crates/auditor-core" }`

A documentação atual do analisador informa que crates declaradas no Cargo não geram arestas arquiteturais. Portanto, é necessário provar com um teste se dependências locais do próprio workspace estão sendo ignoradas junto com crates externas. Não assumir que o grafo atual representa corretamente a arquitetura entre os crates do produto.

---

# 3. Etapa 3.0 — Baseline reproduzível

**Objetivo:** estabelecer uma fotografia verificável do estado atual antes de qualquer alteração.

### 3.0.1. Sincronização e segurança do Git

Na raiz do repositório, executar no PowerShell:

```powershell
git status --short
```

- Se houver alterações locais, não descartá-las nem sobrescrevê-las. Identificar com o usuário se devem ser preservadas, commitadas ou deixadas fora do trabalho.
- Somente com a árvore de trabalho em estado seguro, atualizar a branch de referência:

```powershell
git switch main
git pull --ff-only
git rev-parse --short HEAD
```

- Criar uma branch de trabalho para a etapa, depois de confirmar que ela ainda não existe:

```powershell
git switch -c etapa-3/validacao-arquitetural
```

Se a branch já existir, verificar seu conteúdo antes de reutilizá-la. Não apagar branch existente.

### 3.0.2. Registrar versões do ambiente

```powershell
git --version
rustc --version
cargo --version
node --version
npm --version
```

A referência documentada no repositório é Rust `1.98.1` e Node `24.19.0`. Se o ambiente estiver diferente, registrar a diferença; não atualizar ferramentas automaticamente.

### 3.0.3. Verificações Rust

Executar na raiz:

```powershell
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings
git diff --check
```

O uso de `build.incremental=false` é o caminho documentado para evitar avisos de sessão incremental em ambiente Windows/OneDrive. Se `clippy` falhar, registrar a saída integral relevante e distinguir erro do código de problema ambiental.

### 3.0.4. Verificações do desktop

```powershell
Push-Location apps/desktop
npm ci
npm test
npm run build
Pop-Location
```

### 3.0.5. Verificações da extensão VS Code

```powershell
Push-Location extensions/vscode
npm ci
npm test
Pop-Location
```

Empacotamento opcional da extensão, caso seja necessário validar o `.vsix`:

```powershell
npm run package
```

Não é requisito gerar instalador Tauri nesta etapa. O bundle Tauri está deliberadamente desativado no estado atual.

### Critérios de aceite da Etapa 3.0

- Todos os comandos executados aparecem no relatório de baseline.
- Falhas preexistentes são registradas antes de qualquer correção.
- Versões reais do ambiente são registradas.
- Nenhuma alteração funcional é feita nesta subetapa.
- O baseline é salvo em `docs/validation/etapa-3/baseline.md` ou em caminho equivalente já adotado pelo projeto.

---

# 4. Etapa 3.1 — Autoauditoria e dependências internas do workspace

**Prioridade máxima.** Antes de confiar no auditor em projetos de terceiros, ele precisa representar corretamente a própria arquitetura.

## 4.1.1. Investigar a resolução de dependências Cargo

Inspecionar o fluxo que:

- lê os manifests `Cargo.toml`;
- identifica dependências declaradas;
- distingue dependências externas de crates locais do workspace;
- transforma importações `use` em arestas do grafo;
- associa arestas às regras arquiteturais.

Verificar especificamente `auditor-core` declarado como dependência `path` pela CLI e pelo backend Tauri.

### Teste de regressão obrigatório

Criar uma fixture pequena com dois crates locais no mesmo workspace:

- `core` fornece uma API;
- `cli` depende de `core` com `path = "../core"` e importa a API;
- um terceiro crate externo declarado no manifest também é importado.

Esperado:

- dependência local do workspace deve ser reconhecida como relação interna de arquitetura quando ambos os lados estiverem mapeados no contrato;
- dependência externa declarada deve continuar sendo tratada como pacote externo, sem gerar `dep.unresolved`;
- dependência local fora do contrato ou sem mapeamento inequívoco deve gerar revisão explícita, não ser descartada silenciosamente;
- a ordem dos membros e módulos no TOML não deve alterar o resultado.

### Implementação, somente se o teste confirmar a lacuna

- Distinguir dependências Cargo externas de dependências `path` que apontam para membros locais do workspace.
- Resolver o caminho do pacote local de maneira lexical e segura, sem executar Cargo ou código do projeto auditado.
- Associar o pacote local a um módulo do contrato apenas quando a associação for inequívoca.
- Gerar limitação/revisão explícita nos casos sem evidência suficiente.
- Preservar o comportamento atual de crates externas.
- Não criar arestas para dependências locais que não possam ser associadas com segurança.

## 4.1.2. Criar o contrato de autoauditoria

Depois de resolver ou documentar a limitação acima, criar um `auditor.toml` real para o VibeCodingAuditor.

A configuração deve ser desenhada após ler a árvore real, não copiada cegamente de `auditor.example.toml`.

Começar com módulos Rust bem definidos, por exemplo:

- `core`: `crates/auditor-core/src/**`;
- `cli`: `crates/auditor-cli/src/**`;
- `desktop-backend`: `apps/desktop/src-tauri/src/**`.

As dependências permitidas devem refletir os manifests e a intenção registrada em `AGENTS.md`: CLI e backend desktop podem depender do núcleo; o núcleo não pode depender da CLI, Tauri ou da interface.

A inclusão da interface React e da extensão TypeScript pode ser feita como módulos separados, mas não fingir que o analisador consegue inferir automaticamente a comunicação entre frontend TypeScript e backend Rust. Registrar as fronteiras que não são observáveis pelo grafo atual.

Requisitos do contrato:

- caminhos relativos à raiz e compatíveis com o matcher atual;
- nenhuma sobreposição de paths sem justificativa;
- `allowed_dependencies` explícito;
- `include` limitado aos arquivos relevantes para o escopo da primeira autoauditoria;
- exclusões intencionais documentadas;
- `responsibility` tratada como descrição humana, não como regra semanticamente verificada.

## 4.1.3. Executar autoauditoria

Depois de criar e revisar o contrato:

```powershell
cargo run -p auditor-cli -- scan .
cargo run -p auditor-cli -- analyze .
cargo run -p auditor-cli -- audit .
cargo run -p auditor-cli -- audit . --format json --output "$env:TEMP\vca-self-audit.json"
cargo run -p auditor-cli -- audit . --format markdown --output "$env:TEMP\vca-self-audit.md"
```

Registrar:

- arquivos inventariados;
- arquivos de código analisados e não analisados;
- dependências internas reconhecidas;
- dependências externas ignoradas corretamente;
- arestas permitidas, proibidas e não resolvidas;
- achados por severidade;
- limitações e exclusões;
- módulos sem arquivos ou sem referências, com avaliação humana para evitar interpretar o proxy como prova de código morto.

### Critérios de aceite da Etapa 3.1

- A relação CLI → core aparece no grafo quando o contrato a define.
- A relação desktop-backend → core aparece no grafo quando o contrato a define.
- Nenhuma dependência externa comum é falsamente classificada como módulo interno.
- O núcleo não recebe dependência proibida da CLI ou do desktop.
- A cobertura parcial e as limitações ficam visíveis.
- O contrato de autoauditoria tem documentação curta explicando cada módulo e dependência permitida.
- Se alguma fronteira não puder ser analisada, ela aparece como limitação conhecida; não é declarada como verificada.

---

# 5. Etapa 3.2 — Identidade dos achados e confiabilidade do histórico

**Objetivo:** garantir que o histórico acompanhe problemas reais, em vez de confundir mudança de posição no arquivo com desaparecimento e reaparecimento de uma violação.

## 5.2.1. Auditoria do identificador atual

Inspecionar:

- estrutura `Finding`;
- função que gera `Finding.id`;
- locais onde o identificador é usado para deduplicação;
- persistência SQLite;
- implementação de `history compare`;
- conversão de achados para JSON e diagnósticos da extensão.

Não presumir que a crítica anterior continua válida: confirmar no código atual se linha/coluna ou conteúdo volátil participa indevidamente da identidade.

## 5.2.2. Testes obrigatórios

Criar testes que comprovem:

1. Inserir linhas em branco/comentários antes de uma dependência proibida não deve transformar a mesma violação lógica em um achado novo.
2. Duas violações distintas da mesma regra no mesmo arquivo continuam tendo IDs diferentes.
3. Alterar a evidência textual sem alterar o vínculo arquitetural deve ter comportamento definido e testado.
4. Alterar módulo de origem, destino ou regra deve produzir identidade diferente.
5. Resultado deve ser determinístico entre execuções.
6. O histórico deve classificar corretamente achados novos, resolvidos e persistentes.

## 5.2.3. Política de identidade

Separar explicitamente:

- **identidade estável:** identifica a ocorrência lógica;
- **localização:** caminho, linha e coluna atuais;
- **evidência:** trecho que justifica o achado;
- **regra:** `rule_id` estável.

A linha não deve ser a única base da identidade. Quando necessário para distinguir ocorrências semelhantes, usar um discriminador estável derivado de contexto estrutural disponível. Não prometer preservar identidade através de renomeações de arquivo se isso não puder ser garantido.

## 5.2.4. Compatibilidade do SQLite

Se a correção exigir alterar o modelo persistido:

- criar migração versionada e transacional;
- preservar os registros já existentes;
- não apagar banco nem recriar histórico;
- definir como comparar registros antigos com IDs legados e execuções novas;
- testar migração de um banco realista da versão anterior;
- testar abertura de banco já migrado;
- rejeitar schema futuro com mensagem clara, conforme política já existente.

### Critérios de aceite da Etapa 3.2

- Testes de deslocamento de linha e duplicidade de achados passam.
- `history compare` não reporta falso `resolved` + `new` apenas por deslocamento de linha.
- Histórico antigo continua legível.
- Migração, se houver, preserva execuções e achados anteriores.
- JSON e extensão continuam recebendo localização e evidência corretas.

---

# 6. Etapa 3.3 — Validação em um segundo projeto real

**Objetivo:** descobrir se o contrato e as regras funcionam fora da arquitetura do próprio auditor.

Esta tarefa depende de o usuário indicar um projeto local apropriado. O OpenCode não deve escolher nem varrer pastas pessoais arbitrariamente.

## 6.3.1. Escolha do projeto

Preferir um projeto de porte médio que tenha:

- pelo menos três módulos ou áreas com responsabilidades distintas;
- dependências internas entre módulos;
- testes e configuração próprios;
- pelo menos uma regra arquitetural que deveria ser proibida;
- documentação suficiente para entender a intenção.

Não enviar o código para nuvem. Não executar scripts do projeto-alvo. Não modificar seus arquivos.

## 6.3.2. Criar contrato temporário

- Criar o contrato em uma cópia de trabalho ou diretório temporário de validação.
- Não substituir `auditor.toml` já existente.
- Registrar as decisões de classificação de módulos.
- Identificar quais módulos não podem ser analisados pela cobertura atual.
- Separar violações confirmadas de itens `needs_review`.

## 6.3.3. Avaliação manual dos resultados

Para cada achado, classificar:

- verdadeiro positivo;
- falso positivo;
- falso negativo conhecido;
- revisão correta por falta de evidência;
- limitação de linguagem/resolução;
- problema de configuração do contrato.

Medir pelo menos:

- proporção de achados confirmados manualmente;
- violações arquiteturais conhecidas que foram detectadas;
- violações conhecidas que não foram detectadas;
- arquivos de código não analisados;
- tempo total da auditoria;
- ruído gerado por `needs_review`.

Não criar uma nota única de qualidade sem metodologia e amostra suficientes.

### Critérios de aceite da Etapa 3.3

- Existe um relatório de validação do segundo projeto.
- O projeto-alvo não foi modificado.
- Cada achado amostrado tem classificação manual.
- Limitações e falsos negativos conhecidos estão documentados.
- As alterações no motor são priorizadas pela evidência obtida, não por preferência abstrata.

Se ainda não houver um segundo projeto disponível, registrar a tarefa como bloqueada por falta de projeto-alvo. Não substituir silenciosamente por uma fixture e declarar validação real concluída.

---

# 7. Etapa 3.4 — Especificação do contrato arquitetural v2

**Objetivo:** permitir que o contrato descreva melhor a arquitetura pretendida, sem tentar interpretar prosa livre como regra de máquina.

O contrato v1 atualmente descreve `modular_monolith`, módulos, paths, responsabilidades e dependências permitidas. Essa representação é útil, mas ainda não expressa diretamente camadas e agrupamentos de domínio.

## 7.4.1. Escrever uma proposta antes de codificar

Criar `docs/architecture/CONTRACT_V2_PROPOSAL.md` contendo:

- problemas concretos observados nos dois projetos-piloto;
- campos propostos e seus tipos;
- significado preciso de cada campo;
- exemplos válidos e inválidos;
- validação de paths e sobreposições;
- semântica da direção das dependências;
- política para elementos não resolvidos;
- compatibilidade com schema v1;
- impacto no motor, CLI, desktop, extensão e SQLite;
- critérios para aceitar ou rejeitar a proposta.

## 7.4.2. Capacidades candidatas para v2

Avaliar, sem assumir que todas devem entrar:

### Camadas

Permitir agrupar módulos em camadas explícitas, com direção de dependência verificável. Exemplos: `presentation`, `application`, `domain` e `infrastructure`.

### Domínios

Permitir agrupar módulos por domínio/subdomínio, com limites de dependência verificáveis entre grupos.

### Restrições estruturais

Expressar proibições e permissões de forma declarativa e determinística. Evitar uma linguagem genérica de regras se um modelo simples resolver os casos reais.

### Exceções justificadas

Se houver exceção arquitetural, ela deve ser explícita, localizada, explicável e passível de revisão. Não criar mecanismo que permita silenciar erros globalmente.

### Limites mensuráveis

Avaliar parâmetros de métricas somente quando houver métrica real implementada. Não chamar `module.concentration` de complexidade ciclomática/cognitiva: ela mede concentração no grafo de dependências.

## 7.4.3. Semântica que precisa ficar clara

- `responsibility` continua sendo descrição para humanos, salvo se houver um mecanismo determinístico separado.
- O auditor não deve alegar que entende semanticamente o conteúdo de uma responsabilidade escrita em português.
- Uma camada ou domínio só pode ser validado se o mapeamento dos módulos for inequívoco.
- `needs_review` deve ser usado quando falta evidência; `error` só para violação demonstrada.
- Não adicionar campo que não tenha efeito implementado e testado.

## 7.4.4. Compatibilidade

A proposta deve escolher explicitamente uma política:

1. leitor suporta v1 e v2, normalizando ambos para um modelo interno comum; ou
2. ferramenta fornece migração explícita e reversível/inspecionável, mantendo o original intacto.

Preferência: leitura de v1 e v2 no mesmo núcleo, com normalização interna e sem reescrever arquivos automaticamente.

### Critérios de aceite da Etapa 3.4

- Proposta revisada e aprovada antes da implementação.
- Cada campo tem semântica e teste de aceitação definidos.
- Compatibilidade v1 foi planejada.
- Nenhuma regra depende de interpretação de IA.
- O escopo v2 é pequeno o suficiente para ser implementado e testado em incrementos.

---

# 8. Etapa 3.5 — Implementação incremental do contrato v2

Só iniciar depois de a proposta da Etapa 3.4 estar aprovada.

## Ordem recomendada

1. Criar tipos internos normalizados, sem alterar inicialmente a análise.
2. Fazer parser/validador do schema v2.
3. Adicionar testes de campos obrigatórios, desconhecidos, duplicados, paths inválidos e referências inexistentes.
4. Normalizar contratos v1 e v2 para o mesmo modelo interno.
5. Implementar validação de camadas.
6. Implementar agrupamento/limites de domínio, se aprovados na proposta.
7. Implementar exceções explícitas, se aprovadas.
8. Integrar regras ao motor de auditoria.
9. Expor os achados no JSON, Markdown, terminal e desktop sem duplicar lógica nas interfaces.
10. Validar que a extensão continua consumindo `findings` sem regras próprias.

### Testes obrigatórios

- Contrato v1 antigo continua produzindo o mesmo resultado nas fixtures existentes.
- Contratos v2 válidos são aceitos.
- Contratos v2 inválidos são rejeitados com mensagem e caminho do campo.
- Mapeamento ambíguo não produz violação definitiva.
- Dependência entre camadas permitida é aceita.
- Dependência contra a direção declarada gera `error` apenas quando a aresta está resolvida.
- Domínio não mapeado/ambíguo gera revisão ou limitação, não conclusão falsa.
- Ordenação do TOML não muda os resultados.
- Saídas texto, JSON e Markdown são determinísticas.
- Desktop e extensão exibem os novos achados sem reimplementar as regras.

### Critérios de aceite da Etapa 3.5

- V1 continua funcionando.
- V2 valida camadas/domínios efetivamente implementados.
- Não há alteração silenciosa de contratos existentes.
- Todas as saídas apresentam os mesmos achados e severidades.
- O README e o exemplo TOML descrevem somente campos já implementados.

---

# 9. Etapa 3.6 — Cobertura estruturada no histórico

**Objetivo:** permitir comparar a cobertura de análise entre execuções históricas, caso a validação de uso confirme essa necessidade.

Atualmente, `coverage` e `skip_summary` são apresentados nos relatórios, mas o roadmap registra que o SQLite ainda guarda as limitações em texto, sem colunas/campos estruturados para esses resumos.

Antes de implementar, confirmar que comparar cobertura ao longo do tempo é um requisito real e não apenas uma melhoria estética.

Se aprovado:

- projetar migração SQLite versionada;
- armazenar cobertura agregada por execução;
- armazenar motivos de exclusão com rótulos estáveis e contagens;
- evitar armazenar conteúdo integral de arquivos;
- manter exemplos limitados e não aumentar o volume do banco desnecessariamente;
- permitir comparar alterações de cobertura entre duas execuções;
- testar banco antigo, migração, rollback transacional em falha e banco já migrado.

Se não aprovado, manter a limitação documentada e não alterar o banco nesta etapa.

---

# 10. Etapa 3.7 — Validação ponta a ponta do desktop e da extensão

**Objetivo:** conferir o fluxo como usuário, além dos testes de núcleo e build.

## Desktop

Executar em `apps/desktop`:

```powershell
npm run tauri dev
```

Verificar manualmente:

- aplicação inicia sem erro;
- usuário consegue escolher a pasta de um projeto;
- auditoria pode ser iniciada manualmente;
- resumo, cobertura, exclusões e achados correspondem ao JSON da CLI;
- filtros de severidade/regra funcionam;
- grafo apresenta as mesmas arestas que o núcleo;
- contrato é somente leitura;
- histórico pode ser consultado e registrado por ação explícita;
- erros operacionais são exibidos sem travar a janela;
- diretório do banco pode ser configurado conforme a implementação atual.

## Extensão VS Code

Executar em `extensions/vscode`:

```powershell
npm test
npm run package
```

Se o comando `code` estiver disponível e o usuário desejar validar a instalação local:

```powershell
code --install-extension .\vibe-coding-auditor-0.1.0.vsix --force
```

Verificar manualmente:

- detecção de `auditor.toml` na pasta aberta;
- comando de auditoria pela paleta;
- diagnósticos no painel Problems;
- severidade e `rule_id` coerentes com CLI;
- clique no achado abre arquivo/linha corretos;
- estado da última auditoria aparece na barra de status;
- código de saída `0` e `1` são tratados como resultados válidos.

## Comparação de resultados

Para um mesmo projeto e contrato, comparar:

- JSON gerado pela CLI;
- resultados apresentados no desktop;
- diagnósticos apresentados no VS Code.

As quantidades de achados, severidades e identificadores devem coincidir. Diferenças de apresentação são permitidas; diferenças de regra não.

### Critérios de aceite da Etapa 3.7

- Checklist manual preenchido.
- Capturas de tela opcionais guardadas em relatório de validação, sem incluir dados privados do projeto.
- Nenhuma regra duplicada na UI ou extensão.
- O desktop e a extensão refletem o mesmo resultado do núcleo.
- Limitações da inspeção visual e do ambiente estão registradas.

---

# 11. Verificação final completa

Executar na raiz do repositório depois de todas as modificações aprovadas:

```powershell
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings
git diff --check
```

Depois:

```powershell
Push-Location apps/desktop
npm ci
npm test
npm run build
Pop-Location

Push-Location extensions/vscode
npm ci
npm test
npm run package
Pop-Location
```

Executar novamente as fixtures relevantes da Etapa 2 e as novas fixtures da Etapa 3. Não exigir que todas as contagens antigas permaneçam idênticas se uma correção legítima mudar o resultado; nesse caso, explicar cada diferença e atualizar a expectativa com teste de regressão.

### Smoke mínimo obrigatório

- `simple`: resumo de exclusões preservado.
- `partial_coverage`: arquivos não analisados e cobertura parcial explícitos.
- `ambiguous_mapping_a` e `ambiguous_mapping_b`: resultado independente da ordem de declaração.
- `rust_dependencies`: dependências externas e dependências locais do workspace corretamente distinguidas.
- `workspace_glob` e `workspace_glob_unsupported`: expansão suportada e limitações explícitas.
- `typescript` e `typescript_scanner`: imports reconhecidos sem arestas inventadas.
- `python`: imports e limitações documentados.
- `dependencies`: dependências proibidas/ciclos com severidades corretas.
- fixtures novas de identidade de achado, v2 e histórico, conforme implementadas.

---

# 12. Atualização documental exigida

Ao fechar cada subetapa, atualizar:

- `docs/ROADMAP.md`: status, data, critérios e comandos realmente executados;
- `docs/DECISIONS.md`: decisões arquiteturais, alternativas rejeitadas e impacto;
- `README.md`: somente se comandos, capabilities ou limitações públicas mudarem;
- `auditor.example.toml`: somente quando o schema/exemplo mudar e os campos estiverem implementados;
- `AGENTS.md`: somente se as regras persistentes de trabalho precisarem mudar.

Criar relatórios em `docs/validation/etapa-3/`:

- `baseline.md`;
- `self-audit.md`;
- `finding-identity.md`;
- `second-project.md` (se houver projeto-alvo);
- `desktop-vscode.md`;
- `final-verification.md`.

Não preencher relatórios com resultados esperados como se fossem resultados executados.

---

# 13. Definição de pronto da Etapa 3

A Etapa 3 só pode ser declarada concluída quando:

- [ ] baseline reproduzível registrado;
- [ ] dependências `path` do workspace foram testadas e classificadas corretamente;
- [ ] autoauditoria executada com contrato revisado;
- [ ] identificadores de achados testados contra deslocamento de linhas e ocorrências duplicadas;
- [ ] histórico preservado e migrações testadas, se necessárias;
- [ ] segundo projeto real validado ou bloqueio documentado por falta de projeto-alvo;
- [ ] proposta de contrato v2 documentada e aprovada;
- [ ] v2 implementado somente se a proposta foi aprovada;
- [ ] compatibilidade v1 comprovada;
- [ ] persistência estruturada de cobertura implementada somente se requisito confirmado;
- [ ] desktop e extensão validados ponta a ponta;
- [ ] todas as verificações finais executadas;
- [ ] README, ROADMAP e DECISIONS atualizados com fatos comprovados.

Se o segundo projeto real estiver bloqueado, a Etapa 3 pode concluir as tarefas técnicas independentes, mas deve permanecer explicitamente marcada como **validação externa pendente**. Não declarar confiabilidade em projetos reais sem essa evidência.

---

# 14. Prompt inicial para o OpenCode

Copie e envie este texto com o repositório aberto:

> Leia integralmente `AGENTS.md`, `README.md`, `docs/ROADMAP.md`, `docs/DECISIONS.md` e `PLANO_ETAPA_3_MODIFICACOES_E_EXECUCOES.md`.
>
> Vamos iniciar a Etapa 3 do Vibe Coding Auditor. Não implemente funcionalidades novas imediatamente. Comece exclusivamente pela Etapa 3.0: verifique o estado do Git, versões do ambiente, execute os testes Rust, testes/build do desktop e testes da extensão conforme o documento. Não descarte alterações locais, não instale ferramentas e não modifique código nesta primeira execução.
>
> Registre os resultados reais em `docs/validation/etapa-3/baseline.md`, incluindo comandos, códigos de saída e falhas. Depois me apresente o relatório e aguarde minha orientação para iniciar a Etapa 3.1.
>
> Respeite rigorosamente o escopo e as regras de `AGENTS.md`. Não marque tarefas como concluídas sem testes. Não invente resultados.

---

## Referências do repositório usadas para compor este plano

- [README.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/README.md)
- [AGENTS.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/AGENTS.md)
- [docs/ROADMAP.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/docs/ROADMAP.md)
- [docs/DECISIONS.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/docs/DECISIONS.md)
- [Cargo.toml](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/Cargo.toml)
- [auditor-cli/Cargo.toml](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/crates/auditor-cli/Cargo.toml)
- [desktop backend Cargo.toml](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/apps/desktop/src-tauri/Cargo.toml)
- [desktop/package.json](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/apps/desktop/package.json)
- [extension/package.json](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/extensions/vscode/package.json)

