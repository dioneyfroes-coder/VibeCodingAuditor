# Plano de Desenvolvimento — Etapa 6

## Amadurecimento arquitetural, descoberta multi-ecossistema e distribuição

**Projeto:** Vibe Coding Auditor\
**Branch de referência:** `main`\
**Data da referência:** 27/11/2026\
**Atualização de status:** 28/11/2026 — Etapas 6.0, 6.1, 6.2, 6.3, 6.4.0 e 6.4.1 concluídas e publicadas em `main` (`ece58fe`, `426e125`, `5087fc4`, `8499680`, `072b948`); Etapa 6.5 em planejamento; 6.6–6.8 pendentes\
**Status deste documento:** plano em execução; subetapas 6.0–6.4.1 implementadas, aprovadas e verificadas; continue a partir da Etapa 6.5
**Público-alvo:** OpenCode e desenvolvedor responsável pelo projeto

> As seções 2.1 e 2.2 descrevem o estado de referência de 27/11/2026, antes das alterações. Os itens que foram superados estão registrados como concluídos nas seções 5 e no `docs/ROADMAP.md`; as limitações ainda vigentes permanecem explícitas.

---

## 1. Objetivo da etapa

Levar o Vibe Coding Auditor de uma aplicação funcional, validada no fluxo principal, para uma ferramenta mais abrangente e distribuível.

A Etapa 6 deverá trabalhar em quatro frentes:

1. **Descoberta multi-ecossistema:** reconhecer melhor a organização de projetos que usam JavaScript/TypeScript, Python, Rust e outros manifests, sem alegar que uma linguagem está analisada quando ainda não existe analisador para ela.
2. **Contrato arquitetural mais expressivo:** evoluir o modelo atual para representar domínios, limites e regras de dependência de maneira verificável, sem interpretar texto livre como regra.
3. **Autoauditoria mais completa:** ampliar a auditoria do próprio Vibe Coding Auditor para incluir seus componentes Rust, desktop e extensão, respeitando as diferenças entre dependências de código e integrações entre processos.
4. **Preparação para distribuição:** resolver consistência de documentação, dependências, CI, empacotamento, versão e instalação limpa no Windows antes de anunciar uma release.

A etapa deve ser executada em subetapas pequenas. Não implementar tudo em uma única solicitação ao agente.

## 2. Estado de referência encontrado no GitHub

Esta proposta foi escrita após consultar a branch `main` do repositório em 27/11/2026. O OpenCode deverá confirmar o `HEAD` atual antes de começar, pois a branch pode ter avançado desde a consulta.

### 2.1 Capacidades já entregues

O estado documentado no repositório já inclui:

- núcleo de auditoria em Rust, reutilizado pela CLI, desktop e extensão;
- CLI com `scan`, `analyze`, `audit`, `init` e histórico;
- modos `discovery` e `compliance`, com `auditor.toml` opcional;
- descoberta de candidatos, relações observadas, ambiguidades e limitações;
- Architecture Builder no desktop, com revisão, prévia de diff e aprovação explícita para gravar;
- contratos TOML v1 e v2; o Builder gera v2 e o `init` ainda gera v1;
- histórico SQLite schema v3, com modo e identidade do contrato;
- extensão VS Code que chama a CLI, sem duplicar regras de auditoria;
- testes determinísticos e suíte E2E para CLI, desktop e extensão;
- fluxo desktop com abas de Descoberta, Relatório, Mapa, Contrato e Histórico.

O `docs/ROADMAP.md` declara a nova Etapa 4 (descoberta e Builder) e a Etapa 5 (E2E) concluídas em 27/11/2026. Também registra ajustes de UX, nome da CLI e integração com a extensão realizados depois do fechamento inicial.

### 2.2 Limitações atuais registradas no próprio projeto

As limitações que justificam esta etapa incluem:

- a descoberta ainda não usa `package.json`, `pyproject.toml` ou `go.mod` para gerar candidatos arquiteturais;
- pacotes externos e aliases `tsconfig.paths` ainda não são resolvidos de forma suficiente;
- arquivos Go podem ser identificados como cobertura sem analisador; a presença de `go.mod` não significa que o código Go foi analisado;
- `architecture.style` continua limitado a `modular_monolith`;
- `responsibility` é texto descritivo e não deve ser tratado como regra executável;
- cada módulo do contrato possui atualmente um único `path`;
- o Builder é uma funcionalidade do desktop; não cria contrato diretamente pela extensão VS Code;
- o comando `init` ainda gera contrato v1, enquanto o Builder gera v2;
- o autoescopo do repositório ainda não representa completamente as relações entre Rust, React/TypeScript, Tauri e extensão;
- a aplicação compila, mas `bundle.active` está desativado no `tauri.conf.json`; compilar o executável não equivale a produzir um instalador;
- o CI de verificação principal está configurado para Windows; a suíte E2E documentada também é focada no Windows.

### 2.3 Inconsistências documentais/de configuração a verificar no baseline

A consulta ao `main` revelou pontos que precisam ser confirmados antes de qualquer alteração:

1. O `README.md` ainda resume o status como Etapas 0–3 e Etapa 4 concluída com ressalvas, enquanto o `docs/ROADMAP.md` também declara a Etapa 5 E2E concluída e registra mudanças posteriores.
2. O `extensions/vscode/package.json` consultado ainda apresenta `cliPath` padrão `vibe-coding-auditor`, embora o `docs/ROADMAP.md` registre a renomeação do binário para `vibe-coding-auditor-cli`.
3. O mesmo manifest ainda condiciona o menu de contexto a `vibeCodingAuditor.projectConfigured`, enquanto o roadmap registra que o comando deve aparecer também sem `auditor.toml`.
4. `apps/desktop/src-tauri/tauri.conf.json` apresenta `bundle.active: false`. O comando `tauri build` pode compilar o executável sem produzir instaladores quando o bundle está desativado.
5. O CI principal `.github/workflows/ci.yml` executa verificação em `windows-latest`; a cobertura real dos demais workflows deve ser inventariada, não presumida.
6. O histórico da Etapa 5 menciona vulnerabilidades em dependências transitivas de desenvolvimento. A situação atual deve ser consultada novamente; números antigos não devem ser reutilizados como se ainda fossem atuais.

Esses pontos são **itens de verificação**, não autorização para alterar arquivos imediatamente. O agente deve reproduzir cada divergência e registrar o que encontrar.

## 3. Princípios que não podem ser quebrados

1. **O núcleo Rust continua sendo a fonte única da verdade.** React, Tauri e a extensão apenas apresentam dados e encaminham operações.
2. **Descoberta não é conformidade.** Observação, sugestão e violação continuam sendo conceitos diferentes.
3. **Ausência de evidência não é prova de violação.** Ambiguidade, falta de parser ou resolução incompleta devem gerar limitação/revisão explícita.
4. **Não executar o projeto auditado.** Não executar scripts npm, comandos Cargo, build scripts, testes ou código encontrado no projeto sob análise para descobrir sua arquitetura.
5. **Não instalar dependências do projeto analisado.** A descoberta deve ler manifests e arquivos existentes sem `npm install`, `cargo build` ou acesso de rede.
6. **Não alterar o projeto auditado durante descoberta ou auditoria.** A única gravação arquitetural permitida continua sendo a ação explícita do Builder, após prévia e confirmação.
7. **Não aprovar automaticamente relações observadas.** Toda dependência que se torne permitida no contrato exige decisão humana explícita.
8. **Não interpretar `responsibility` como linguagem natural executável.** Regras precisam ser estruturadas e validadas.
9. **Compatibilidade retroativa.** Contratos v1/v2 e histórico SQLite v3 existentes devem continuar sendo lidos; migrações precisam de testes e não podem apagar dados.
10. **Sem IA nesta etapa.** A análise e as regras continuam determinísticas e locais. Sugestões por IA permanecem fora do escopo.
11. **Sem microserviços no próprio auditor.** Manter o monólito modular atual, salvo decisão técnica documentada e aprovada.
12. **Sem atualização forçada de dependências.** Não executar `npm audit fix --force` nem comandos equivalentes que imponham breaking changes.
13. **Uma subetapa por vez.** O agente apresenta diagnóstico/plano, aguarda autorização e só então implementa a subetapa aprovada.
14. **Preservar o trabalho local.** Nunca executar `git reset --hard`, `git clean -fd`, checkout destrutivo ou sobrescrever arquivos modificados sem autorização explícita.

## 4. Resultado esperado ao final da Etapa 6

Ao concluir a etapa, o projeto deverá:

- reconhecer manifests relevantes de mais de um ecossistema e extrair metadados com origem/evidência;
- melhorar a resolução de imports locais, aliases e dependências declaradas sem produzir violações baseadas em suposições;
- oferecer um contrato arquitetural mais expressivo, com compatibilidade para v1/v2;
- permitir que o Builder configure as novas dimensões sem duplicar regras na interface;
- auditar uma parte substancial do próprio repositório, incluindo Rust e TypeScript/JavaScript, e explicitar o que ainda não cobre;
- ter documentação coerente com o comportamento real;
- ter dependências e workflows revisados;
- produzir um instalador Windows reproduzível, com estratégia clara para disponibilizar a CLI à extensão;
- possuir um processo de release versionado, com artefatos e checksums;
- passar pelas suítes determinísticas e E2E aplicáveis.

Não declarar cobertura universal de linguagens. A cobertura final deverá ser quantificada e as lacunas continuarão visíveis.

---

# 5. Execução por subetapas

## Etapa 6.0 — Baseline, inventário e reconciliação do estado atual

**Objetivo:** confirmar o estado real da branch e resolver divergências entre README, roadmap, manifests, testes e workflows antes de planejar mudanças de código.

### Tarefas

- [x] Verificar branch, `HEAD`, commits recentes e alterações locais.
- [x] Preservar qualquer alteração local; não limpar nem restaurar arquivos.
- [x] Ler `AGENTS.md`, `README.md`, `docs/ROADMAP.md`, `docs/DECISIONS.md` e os documentos de validação das Etapas 4 e 5.
- [x] Inventariar todos os arquivos de workflow em `.github/workflows/` e identificar quais executam CI, E2E, empacotamento ou publicação.
- [x] Conferir `Cargo.toml`, lockfiles, `apps/desktop/package.json`, `extensions/vscode/package.json` e `apps/desktop/src-tauri/tauri.conf.json`.
- [x] Verificar se o nome do binário da CLI, o `cliPath` padrão e a resolução automática da extensão estão alinhados.
- [x] Verificar se o menu/command da extensão realmente aparece quando não existe `auditor.toml`.
- [x] Distinguir build do executável, bundle Tauri e instalador distribuível.
- [x] Consultar vulnerabilidades atuais das dependências npm, separando produção e desenvolvimento.
- [x] Executar o baseline disponível sem atualizar versões de ferramentas.
- [x] Registrar resultados, falhas, warnings e comandos exatos em `docs/validation/etapa-6/baseline.md`.

### Comandos de baseline

Executar a partir da raiz do repositório, depois de verificar `git status`:

```bash
git status --short --branch
git rev-parse HEAD
git log -10 --oneline --decorate
rustc --version
cargo --version
node --version
npm --version

cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings
git diff --check

npm --prefix apps/desktop ci
npm --prefix apps/desktop test
npm --prefix apps/desktop run build

npm --prefix extensions/vscode ci
npm --prefix extensions/vscode test
npm --prefix extensions/vscode run package
```

Se houver scripts E2E locais, primeiro ler os `package.json` e configurações correspondentes; não inventar nomes de scripts. A validação E2E já executada no GitHub deve ser registrada com URL/run ID, commit, jobs e resultado.

Para segurança de dependências, consultar os dois projetos Node separadamente:

```bash
npm --prefix apps/desktop audit --json
npm --prefix extensions/vscode audit --json
```

Se algum comando de auditoria não estiver disponível na versão instalada, registrar a limitação. Não corrigir dependências ainda nesta subetapa.

### Critérios de aceite

- [x] `HEAD`, branch e estado local documentados.
- [x] Nenhuma alteração local foi perdida.
- [x] Todos os workflows foram inventariados.
- [x] Divergências de `README`, `ROADMAP`, manifest da extensão e config Tauri foram reproduzidas ou descartadas com evidência.
- [x] Baseline de Rust, desktop e extensão possui comandos, códigos de saída e contagem de testes.
- [x] Vulnerabilidades npm atuais foram classificadas, sem aplicar correções automáticas.
- [x] O agente apresentou uma proposta priorizada para 6.1; não iniciou implementação de funcionalidades.

**Gate:** aprovação explícita do usuário antes de iniciar 6.1.

## Etapa 6.1 — Documentação e configuração coerentes

**Objetivo:** fazer a documentação refletir o estado real do software e corrigir discrepâncias comprovadas de configuração.

### Tarefas

- [x] Atualizar o resumo de status do `README.md` para refletir as Etapas 4 e 5 concluídas e os ajustes posteriores realmente validados.
- [x] Atualizar `docs/ROADMAP.md` para registrar a Etapa 6 como planejada/em andamento, sem marcar tarefas futuras como concluídas.
- [x] Atualizar a documentação de comandos da CLI para refletir o modo automático: contrato presente = conformidade; ausência = descoberta; TOML inválido = erro explícito.
- [x] Confirmar o nome atual do executável e alinhar `cliPath`, texto de ajuda, testes, manifest e documentação.
- [x] Confirmar a condição de visibilidade do comando na extensão para projetos sem contrato.
- [x] Documentar com clareza que `tauri build` com `bundle.active = false` não representa uma release instalável.
- [x] Atualizar `AGENTS.md` somente se as instruções atuais não cobrirem as novas decisões; preservar as regras de segurança e aprovação humana.
- [x] Registrar decisões relevantes em `docs/DECISIONS.md`.

### Critérios de aceite

- [x] README, ROADMAP, AGENTS, manifest e código não se contradizem.
- [x] A extensão encontra a CLI por um mecanismo documentado e testado.
- [x] O comando da extensão está disponível em projeto sem `auditor.toml`.
- [x] O estado de build/empacotamento é descrito corretamente.
- [x] Nenhuma limitação foi removida da documentação sem existir implementação e teste correspondentes.

## Etapa 6.2 — Descoberta de manifests multi-ecossistema

**Objetivo:** permitir que o Discovery Engine entenda melhor os manifests do projeto sem executar ferramentas ou instalar dependências.

### Escopo inicial

**JavaScript/TypeScript — `package.json`**

- [x] Reconhecer manifests na raiz e em subprojetos.
- [x] Extrair `name`, `version` quando presentes e dependências declaradas.
- [x] Distinguir `dependencies`, `devDependencies`, `peerDependencies` e `optionalDependencies`.
- [x] Reconhecer workspaces em formato suportado e apontar padrões não suportados como limitação explícita.
- [x] Identificar relações `workspace:` quando presentes e não confundi-las com dependências externas.
- [x] Não executar scripts definidos em `package.json`.

**Python — `pyproject.toml`**

- [x] Reconhecer o padrão PEP 621 em `[project]`.
- [x] Extrair nome, versão, `dependencies` e `optional-dependencies` quando presentes.
- [x] Tratar formatos específicos de Poetry, uv ou outros gerenciadores somente se houver escopo e fixtures aprovados; não inferir que todos os formatos TOML são equivalentes.
- [x] Não executar ambientes virtuais, instaladores ou comandos Python do projeto.

**Go — `go.mod`**

- [x] Reconhecer o módulo declarado e dependências `require`.
- [x] Tratar `replace`, `exclude`, comentários e blocos de dependências conforme o subconjunto formalmente suportado.
- [x] Se uma diretiva não for compreendida, produzir limitação específica em vez de descartar silenciosamente o manifest.
- [x] Não declarar arquivos `.go` analisados enquanto não existir analisador de código Go.

### Regras de implementação

- [x] Usar bibliotecas que já estejam no workspace quando forem adequadas (`serde_json`/TOML), evitando dependências novas sem justificativa.
- [x] Criar tipos explícitos para manifest, ecossistema, pacote local e dependência externa.
- [x] Manter caminho relativo, arquivo de origem do metadado e tipo da dependência como evidência.
- [x] Descoberta continua somente leitura e offline.
- [x] Manifest malformado não pode derrubar a varredura inteira; o relatório deve registrar erro/limitação localizado.
- [x] Distinguir manifest ausente, manifest inválido e formato não suportado.

### Fixtures mínimas

- [x] `package.json` simples com dependências runtime e dev.
- [x] workspace npm com dois pacotes locais.
- [x] `package.json` malformado.
- [x] `pyproject.toml` PEP 621 simples.
- [x] `pyproject.toml` com optional dependencies.
- [x] formato Poetry/uv não suportado, caso não entre no escopo: deve gerar limitação testada.
- [x] `go.mod` com `module`, `require` simples, bloco `require` e `replace`.
- [x] `go.mod` com diretiva não suportada.
- [x] projeto misto com manifests aninhados.
- [x] projeto sem manifest.

### Critérios de aceite

- [x] Os manifests são lidos sem executar comandos do projeto.
- [x] Os resultados são determinísticos e incluem evidência de origem.
- [x] Dependências runtime/dev/optional não são misturadas.
- [x] Relações de workspace local são diferenciadas de pacotes externos.
- [x] Arquivos de linguagens sem parser continuam contabilizados como não analisados.
- [x] Descoberta não gera `dep.not_allowed` nem `dep.layer_not_allowed`.
- [x] Fixtures comprovam manifests válidos, inválidos, ausentes e parcialmente suportados.

## Etapa 6.3 — Resolução de imports, aliases e dependências externas

**Objetivo:** reduzir dependências não resolvidas e relações perdidas sem classificar incorretamente pacotes externos como módulos internos.

### TypeScript/JavaScript

- [x] Definir e documentar uma ordem determinística de resolução de imports locais.
- [x] Resolver aliases de `tsconfig.json` por `baseUrl` e `paths` no subconjunto suportado.
- [x] Tratar `extends` local de configuração de forma explícita; configurações estendidas por pacote externo não devem ser carregadas pela rede.
- [x] Reconhecer workspaces e pacotes locais declarados em manifests.
- [x] Diferenciar built-ins, imports relativos, aliases locais e pacotes externos.
- [x] Extrair o nome-base de pacotes scoped (`@scope/name`) e subpaths sem confundir o subpath com outro pacote.
- [x] Classificar pacote externo declarado, não declarado ou não verificável sem transformar isso automaticamente em violação arquitetural.

### Python

- [x] Preservar resolução atual de imports relativos e absolutos.
- [x] Associar dependências declaradas somente quando o mapeamento entre nome importável e nome de distribuição for determinístico.
- [x] Para nomes diferentes (ex.: distribuição e módulo importável com nomes distintos), manter revisão/limitação; não inventar correspondência.

### Modelo comum

- [x] Definir estados explícitos para relação local resolvida, pacote externo declarado, pacote externo não confirmado, dependência não resolvida e relação ambígua.
- [x] Cada estado deve carregar evidência e origem.
- [x] A descoberta continua gerando observações, nunca violações.
- [x] Em conformidade, somente regras declaradas no contrato podem elevar uma relação a violação.
- [x] Não realizar download, consulta a registry ou leitura de `node_modules`/site-packages para inferir dependências instaladas.

### Testes

- [x] Alias TS com destino único.
- [x] Alias TS com múltiplos destinos possíveis.
- [x] Alias TS sem destino.
- [x] `tsconfig` com `baseUrl` e `paths`.
- [x] `tsconfig` com `extends` local.
- [x] Workspace package local.
- [x] Pacote npm scoped e import com subpath.
- [x] Built-in de Node.
- [x] Dependência Python com nome equivalente.
- [x] Dependência Python cuja distribuição e módulo têm nomes diferentes.
- [x] Regressões nos imports já suportados.

### Critérios de aceite

- [x] Resolução é determinística e testada.
- [x] Ambiguidade nunca vira escolha silenciosa.
- [x] Pacote externo não é tratado como dependência interna.
- [x] Pacote externo não resolvido não vira violação arquitetural por padrão.
- [x] A cobertura informa exatamente o que foi analisado.

## Etapa 6.4 — Contrato arquitetural v3: projeto, domínios e regras declarativas

**Objetivo:** ampliar a expressividade do contrato sem introduzir interpretação de linguagem natural ou código executável do usuário.

> Esta etapa deve começar com uma proposta de schema e exemplos. Não implementar o schema v3 antes da aprovação do desenho.

### 6.4.0 — Proposta formal

- [x] Documentar os limites do schema atual: `style = modular_monolith`, módulos com um `path`, camadas v2 e `responsibility` descritivo.
- [x] Definir a relação entre perfil de arquitetura e regras efetivas. Um perfil é um preset, não uma afirmação de que o motor compreende qualquer estilo arquitetural.
- [x] Avaliar presets inicialmente representáveis por regras determinísticas: monólito modular, monólito em camadas e Clean/Hexagonal como conjunto de camadas/direções explícitas.
- [x] Não anunciar microsserviços como suportados nesta etapa; o modelo atual não representa adequadamente serviços implantáveis, comunicação remota e contratos de runtime.
- [x] Projetar domínios/contextos delimitados e sua relação com módulos e camadas.
- [x] Definir se um módulo pode pertencer a um domínio, a vários ou a nenhum; justificar e testar a semântica.
- [x] Definir regras de dependência por módulo, camada e domínio, com precedência determinística.
- [x] Definir exceções justificadas com escopo exato (regra/origem/destino), justificativa obrigatória e, se aplicável, data de expiração.
- [x] Exceções não podem apagar o histórico: o relatório deve informar que uma regra foi dispensada e por qual motivo.
- [x] Manter regras personalizadas declarativas; não aceitar scripts JavaScript, Lua, Python, WASM ou `eval` nesta etapa.
- [x] Avaliar se múltiplos paths por módulo são realmente necessários. Não ampliar o schema apenas por conveniência; usar fixtures para provar a necessidade.
- [x] Definir estratégia de identidade canônica do contrato para histórico e comparação.
- [x] Definir compatibilidade de leitura v1/v2 e escrita v3, sem regravar contratos existentes automaticamente.
- [x] Verificar se o histórico SQLite v3 comporta a nova `contract_schema_version` sem migração. Se precisar migrar, criar proposta separada e testes de preservação.

### 6.4.1 — Implementação após aprovação

- [x] Implementar os tipos/validadores/serializador no `auditor-core`.
- [x] Fazer parser rejeitar campos desconhecidos, referências órfãs, regras contraditórias e ciclos não permitidos.
- [x] Implementar avaliação de regras no núcleo, com IDs estáveis, evidência e severidade consistente.
- [x] Preservar leitura de contratos v1/v2.
- [x] Manter identidade estável e documentada; mudança de regra deve alterar a identidade do contrato quando semanticamente relevante.
- [x] Atualizar a CLI para validar e exibir os novos tipos de regra.
- [x] Atualizar o Builder para os presets e as novas dimensões sem duplicar validação no React.
- [x] Alinhar `auditor init` com a versão de contrato recomendada após a decisão; manter comportamento de não sobrescrever.
- [x] Atualizar exemplos, documentação e fixtures.

### Critérios de aceite

- [x] Proposta de schema revisada e aprovada antes do código.
- [x] Contratos v1/v2 existentes continuam sendo lidos sem reescrita.
- [x] Contrato v3 inválido é rejeitado com mensagem que identifica o campo/regra.
- [x] Domínios e camadas não se confundem; as regras entre eles têm precedência explícita.
- [x] Exceções são visíveis e nunca ocultam achados silenciosamente.
- [x] Nenhum texto livre é interpretado como política executável.
- [x] CLI, desktop e extensão apresentam os mesmos achados do núcleo.
- [x] Histórico só compara execuções compatíveis com identidade de contrato conhecida.

## Etapa 6.5 — Autoauditoria multi-stack do próprio Vibe Coding Auditor

**Objetivo:** usar o próprio produto para fiscalizar uma parte maior do repositório, sem forçar relações artificiais entre componentes diferentes.

### Escopo

Incluir no plano de autoauditoria:

- crates Rust do workspace;
- frontend desktop React/TypeScript;
- extensão VS Code TypeScript;
- manifests e limites de cada pacote;
- integrações explícitas entre frontend e backend Tauri;
- integração entre extensão e CLI.

### Tarefas

- [ ] Revisar o `auditor.toml` raiz e registrar o escopo atual realmente coberto.
- [ ] Separar grafo de imports internos de relações entre aplicações/processos.
- [ ] Representar desktop e extensão como componentes distintos; não criar dependência de runtime entre eles apenas por estarem no mesmo repositório.
- [ ] Detectar chamadas `invoke("...")` no frontend Tauri e comparar com comandos Tauri registrados no backend, dentro de um subconjunto sintático comprovável.
- [ ] Se a extração de um comando for ambígua (macros, geração ou código condicional), marcar revisão/limitação, não violação definitiva.
- [ ] Verificar a integração extensão → CLI: nome do binário, configuração `cliPath`, resolução automática, argumentos e formato JSON.
- [ ] Incluir limites de pacote/workspace no contrato, quando a implementação de 6.2/6.4 permitir.
- [ ] Criar relatório de cobertura por componente e linguagem.
- [ ] Executar a autoauditoria em descoberta e em conformidade e comparar os dois resultados sem misturar seus modos.
- [ ] Revisar manualmente falsos positivos e relações ausentes.

### Critérios de aceite

- [ ] Cobertura Rust e TypeScript/JavaScript aparece explicitamente.
- [ ] Componentes desktop e extensão permanecem separados no modelo.
- [ ] Relações frontend–Tauri usam um tipo de evidência diferente de import estático.
- [ ] Nenhuma limitação conhecida é escondida.
- [ ] Achados de autoauditoria apontam arquivo/linha quando disponível.
- [ ] A autoauditoria não reporta conformidade total enquanto existir parte relevante fora do escopo.
- [ ] Existe um relatório versionado em `docs/validation/etapa-6/` com commit e comandos exatos.

## Etapa 6.6 — Segurança, dependências e consistência do CI

**Objetivo:** reduzir riscos operacionais antes de distribuir binários.

### Tarefas

- [ ] Reexecutar `npm audit` para desktop e extensão em separado.
- [ ] Classificar cada alerta por pacote direto/transitivo, severidade, alcance (produção/desenvolvimento) e correção disponível.
- [ ] Não usar contagens históricas como estado atual.
- [ ] Atualizar somente dependências cuja correção seja justificada e compatível; revisar diffs de lockfiles.
- [ ] Não usar `npm audit fix --force`.
- [ ] Avaliar o toolchain Rust fixado no CI (atualmente o workflow consultado fixa Rust 1.98.1) e decidir se permanece ou é atualizado; a atualização precisa de baseline integral.
- [ ] Conferir versões de Node e Actions em todos os workflows; atualizar somente após testar compatibilidade.
- [ ] Corrigir avisos de Actions depreciadas encontrados no baseline.
- [ ] Verificar permissões mínimas de `GITHUB_TOKEN` nos workflows.
- [ ] Garantir que logs e artefatos E2E não exponham caminhos privados, conteúdo de projeto ou segredos.
- [ ] Avaliar matriz de verificação Linux/macOS para Rust e frontend, sem prometer E2E desktop nessas plataformas nesta etapa.
- [ ] Manter E2E Windows enquanto plataforma de referência para o instalador inicial.

### Critérios de aceite

- [ ] Todos os alerts atuais estão classificados e documentados.
- [ ] Correções não introduzem breaking changes não revisadas.
- [ ] CI passa com toolchain explicitamente fixado.
- [ ] Workflows têm permissões mínimas e nenhum segredo nos logs.
- [ ] Se a matriz multiplataforma for aprovada, cada sistema executa apenas verificações para as quais seus pré-requisitos estão configurados.

## Etapa 6.7 — Empacotamento e estratégia de instalação

**Objetivo:** transformar os binários atuais em artefatos instaláveis e utilizáveis fora da árvore de desenvolvimento.

### 6.7.0 — Decisão de distribuição

Antes de alterar `tauri.conf.json`, decidir e registrar:

- [ ] Qual formato Windows será suportado inicialmente: NSIS, MSI ou ambos.
- [ ] Se a CLI será instalada junto da aplicação desktop, distribuída separadamente ou disponibilizada por opção do instalador.
- [ ] Como a extensão VS Code localizará a CLI numa instalação normal, fora do repositório.
- [ ] Como o usuário poderá configurar `vibeCodingAuditor.cliPath` sem editar arquivos manualmente.
- [ ] Onde o SQLite ficará armazenado em instalação normal (diretório de dados do usuário), evitando depender do diretório de trabalho atual ou de uma pasta protegida.
- [ ] Como atualizações preservarão o banco e os contratos dos projetos.
- [ ] Como desinstalação tratará dados do usuário; não apagar histórico sem consentimento explícito.
- [ ] Se assinatura de código será requisito já na primeira release ou ficará documentada para fase posterior.

### 6.7.1 — Preparar bundle Tauri

- [ ] Rever `apps/desktop/src-tauri/tauri.conf.json`; hoje `bundle.active` está desativado.
- [ ] Definir `bundle.targets`, metadados de produto, ícones, identificador, versão e instalador.
- [ ] Garantir que a CSP atual continue restritiva e que empacotamento não introduza permissões desnecessárias.
- [ ] Decidir como a CLI será incluída, localizada e atualizada sem procurar executáveis arbitrários.
- [ ] Confirmar diretório de dados gravável para SQLite.
- [ ] Validar que o instalador não modifica projetos auditados nem cria `auditor.toml` automaticamente.
- [ ] Criar testes/checagens para versão consistente entre `Cargo.toml`, `apps/desktop/package.json`, `tauri.conf.json` e `extensions/vscode/package.json`.

### 6.7.2 — Pipeline de release

- [ ] Criar workflow separado de release, acionado por tag versionada (`vX.Y.Z`).
- [ ] Exigir CI e E2E aprovados antes de publicar.
- [ ] Gerar instalador Windows da aplicação desktop.
- [ ] Gerar binário CLI release para Windows e pacote separado caso essa seja a estratégia escolhida.
- [ ] Gerar `.vsix` da extensão.
- [ ] Gerar checksums SHA-256 de todos os artefatos.
- [ ] Publicar notas de release com mudanças, limitações conhecidas e instruções de instalação/atualização.
- [ ] Não publicar artefato de build local como release oficial sem reproduzir a geração pelo workflow.

### 6.7.3 — Teste de instalação limpa

- [ ] Instalar em ambiente Windows limpo ou VM sem o repositório clonado.
- [ ] Abrir o desktop e selecionar um projeto sem contrato.
- [ ] Executar descoberta e verificar ausência de violações de conformidade indevidas.
- [ ] Criar `auditor.toml` pelo Builder, revisar diff e confirmar gravação.
- [ ] Rodar auditoria de conformidade.
- [ ] Instalar a extensão `.vsix` e confirmar a resolução da CLI instalada.
- [ ] Abrir projeto sem contrato pela extensão e verificar canal de saída/status.
- [ ] Abrir projeto com contrato e verificar diagnostics/Problems.
- [ ] Reiniciar aplicação e verificar persistência do histórico.
- [ ] Atualizar de uma versão anterior preservando histórico e configurações.
- [ ] Desinstalar e comprovar que os dados do usuário só são removidos conforme escolha explícita.

### Critérios de aceite

- [ ] Instalador Windows é gerado em workflow reproduzível.
- [ ] Instalação limpa funciona sem repositório clonado e sem Rust/Node instalados.
- [ ] A extensão encontra a CLI pelo mecanismo aprovado.
- [ ] Desktop, CLI e extensão reportam a mesma versão compatível.
- [ ] O banco fica em local gravável e sobrevive a atualização.
- [ ] Artefatos publicados possuem checksum e notas de versão.

## Etapa 6.8 — Fechamento, documentação e release candidata

**Objetivo:** consolidar tudo que foi realmente implementado e validado.

### Tarefas

- [ ] Atualizar `README.md` com instalação, requisitos, modos de execução, limitações e troubleshooting.
- [ ] Atualizar `docs/ROADMAP.md` marcando somente subetapas verificadas.
- [ ] Atualizar `docs/DECISIONS.md` com decisões finais e alternativas rejeitadas.
- [ ] Atualizar `AGENTS.md` com comandos e limites que permanecerão relevantes para próximos desenvolvimentos.
- [ ] Criar `CHANGELOG.md` ou documento equivalente se ainda não existir, com versão e mudanças.
- [ ] Criar relatório final `docs/validation/etapa-6/fechamento.md`.
- [ ] Registrar número de testes por camada, comandos, códigos de saída, workflows, commit e links das execuções.
- [ ] Registrar limitações que permanecerem.
- [ ] Só marcar Etapa 6 como concluída depois da instalação limpa e dos critérios de aceite.

### Verificação final sugerida

Executar os comandos abaixo e adaptar apenas quando o baseline confirmar comandos diferentes:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings

git diff --check

npm --prefix apps/desktop ci
npm --prefix apps/desktop test
npm --prefix apps/desktop run build

npm --prefix extensions/vscode ci
npm --prefix extensions/vscode test
npm --prefix extensions/vscode run package
```

Além desses comandos, executar os workflows E2E existentes, o build do instalador e os testes manuais de instalação limpa. `tauri build` só será considerado empacotamento se o bundle estiver habilitado e o artefato instalador for verificado.

---

# 6. Ordem de execução recomendada

| Ordem | Subetapa | Dependência | Gate |
|---|---|---|---|
| 1 | 6.0 Baseline e reconciliação | Nenhuma | Aprovação do diagnóstico |
| 2 | 6.1 Documentação/configuração | 6.0 | Configuração coerente |
| 3 | 6.2 Manifests multi-ecossistema | 6.0 | Fixtures e cobertura |
| 4 | 6.3 Imports/aliases/dependências | 6.2 | Resolução determinística |
| 5 | 6.4 Contrato v3 | 6.2 e 6.3 | RFC/schema aprovado |
| 6 | 6.5 Autoauditoria multi-stack | 6.2, 6.3 e 6.4 | Cobertura honesta |
| 7 | 6.6 Segurança e CI | 6.0, pode avançar em paralelo após baseline | CI verde |
| 8 | 6.7 Empacotamento/release | 6.1 e 6.6 | Instalação limpa |
| 9 | 6.8 Fechamento | Todas as anteriores | Release candidata validada |

A ordem não autoriza executar subetapas em paralelo automaticamente. O usuário aprova cada subetapa antes de sua implementação. A etapa 6.6 pode ser investigada em paralelo, mas mudanças em dependências e workflows também precisam de aprovação.

# 7. Arquivos e áreas provavelmente envolvidos

O OpenCode deve confirmar os caminhos reais antes de editar. A lista é indicativa:

| Área | Caminhos prováveis | Tipo de alteração |
|---|---|---|
| Núcleo Rust | `crates/auditor-core/src/` | manifests, descoberta, resolução, contrato, regras, Builder |
| CLI | `crates/auditor-cli/` | comandos, JSON e integração de novas funções |
| Desktop | `apps/desktop/src/` | apresentação de domínios/perfis/descoberta |
| Backend Tauri | `apps/desktop/src-tauri/` | comandos finos e configuração de bundle |
| Extensão | `extensions/vscode/` | localização da CLI, status e diagnostics |
| Fixtures | `tests/fixtures/projects/` e testes dos crates | cenários multi-ecossistema e regressões |
| Autoauditoria | `auditor.toml` | escopo real do próprio repositório |
| CI | `.github/workflows/` | matriz de validação, E2E e release |
| Documentação | `README.md`, `AGENTS.md`, `docs/ROADMAP.md`, `docs/DECISIONS.md`, `docs/architecture/` | status, RFCs, decisões e instruções |

Não criar arquivos para corresponder cegamente a esta tabela. Reutilizar a organização existente quando apropriado.

# 8. Fora do escopo desta etapa

- Integração com LLM, sugestões por IA ou agentes autônomos.
- Refatoração automática de código do projeto auditado.
- Aprovação automática das dependências encontradas.
- Execução de builds/testes/scripts do projeto analisado durante descoberta.
- Suporte completo a microsserviços e topologias distribuídas.
- Parser completo de Go, Java, C#, C/C++ ou outras linguagens sem estudo específico e critérios próprios.
- Reescrita total da interface desktop.
- Sincronização em nuvem ou envio de código para serviços externos.
- Regras personalizadas por script executável.
- Limpeza automática do histórico SQLite.

Novas ideias devem ser registradas para uma etapa posterior, sem aumentar o escopo desta.

# 9. Regras de trabalho para o OpenCode

1. Ler `AGENTS.md`, `README.md`, `docs/ROADMAP.md` e `docs/DECISIONS.md` antes de qualquer tarefa.
2. Verificar `git status` e registrar mudanças locais antes de editar.
3. Na primeira execução, realizar somente a Etapa 6.0.
4. Não implementar 6.1 ou qualquer outra subetapa antes de apresentar o diagnóstico e receber aprovação explícita.
5. Para cada mudança arquitetural, apresentar motivo, alternativas, impacto, arquivos previstos e estratégia de teste.
6. Aplicar TDD quando houver lógica nova: criar primeiro fixture/teste que reproduza o comportamento esperado, observar falha, implementar e validar.
7. Não mover regras de auditoria para React, Tauri ou extensão.
8. Não adicionar dependência sem explicar necessidade, alternativas, manutenção e impacto no lockfile.
9. Nunca atualizar dependências de forma forçada.
10. Após cada subetapa, executar testes focados, suíte afetada e, quando aplicável, suíte completa.
11. Revisar nomes, limites de módulos, documentação dos helpers e falsos positivos.
12. Registrar comandos exatos e resultados, inclusive falhas.
13. Atualizar `docs/ROADMAP.md` somente depois que os critérios forem comprovados.
14. Não declarar concluído com base apenas em build; E2E e validação manual devem ser distinguidos.
15. Parar ao final de cada subetapa e entregar resumo para revisão humana.

# 10. Prompt inicial para o OpenCode

Copiar e enviar com este documento disponível na raiz do repositório:

```text
Leia integralmente o arquivo PLANO_ETAPA_6_AMADURECIMENTO_ARQUITETURAL_E_DISTRIBUICAO.md e as instruções do projeto em AGENTS.md.

Vamos iniciar a Etapa 6 do Vibe Coding Auditor. Nesta primeira execução, você está autorizado a executar SOMENTE a Etapa 6.0 — Baseline, inventário e reconciliação do estado atual.

Regras obrigatórias:
- Não modifique código, manifests, workflows ou contratos nesta primeira execução.
- Não instale, atualize ou remova ferramentas/dependências.
- Não descarte, restaure ou sobrescreva alterações locais.
- Não execute git reset, git clean ou operações destrutivas.
- Leia o estado da branch main e registre o HEAD exato.
- Verifique todas as divergências listadas na seção 2.3, sem assumir que são bugs antes de reproduzi-las.
- Leia todos os workflows e os manifests envolvidos.
- Execute o baseline somente após confirmar que o ambiente e os lockfiles estão disponíveis.
- Consulte as vulnerabilidades npm atuais, mas não tente corrigi-las nesta subetapa.
- Registre comandos, códigos de saída, contagens de testes, warnings, limitações e URLs de CI em docs/validation/etapa-6/baseline.md.
- Não marque a Etapa 6.0 como concluída se algum teste relevante não tiver sido executado ou se uma divergência ficar sem classificação.

Ao terminar, apresente:
1. HEAD/branch e estado do working tree;
2. matriz dos componentes e workflows encontrados;
3. resultados dos testes com comandos e códigos de saída;
4. divergências confirmadas, descartadas e ainda inconclusivas;
5. riscos e prioridades;
6. proposta técnica objetiva para a Etapa 6.1.

Pare ao final. Não comece a Etapa 6.1 sem minha autorização explícita.
```

---

# 11. Referências do estado atual

Fontes consultadas na branch `main` em 27/11/2026. O OpenCode deve reabrir esses documentos no repositório local e confirmar se ainda correspondem ao `HEAD` atual.

- [Repositório VibeCodingAuditor](https://github.com/dioneyfroes-coder/VibeCodingAuditor)
- [README.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/README.md)
- [docs/ROADMAP.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/docs/ROADMAP.md)
- [docs/DECISIONS.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/docs/DECISIONS.md)
- [AGENTS.md](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/AGENTS.md)
- [apps/desktop/package.json](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/apps/desktop/package.json)
- [apps/desktop/src-tauri/tauri.conf.json](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/apps/desktop/src-tauri/tauri.conf.json)
- [extensions/vscode/package.json](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/extensions/vscode/package.json)
- [.github/workflows/ci.yml](https://github.com/dioneyfroes-coder/VibeCodingAuditor/blob/main/.github/workflows/ci.yml)

## Regra final

A Etapa 6 só será considerada concluída quando a ferramenta conseguir descobrir melhor projetos de ecossistemas diferentes, fiscalizar uma porção substancial da própria arquitetura e ser instalada em um Windows limpo com desktop, CLI e extensão funcionando juntos. Um README atualizado ou um build bem-sucedido, isoladamente, não satisfaz esse objetivo.
