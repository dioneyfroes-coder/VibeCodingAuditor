# Vibe Coding Auditor

## Plano de início e instruções para o OpenCode

**Versão do documento:** 0.1  
**Status:** proposta inicial, sujeita a decisões explícitas do responsável pelo projeto  
**Plataforma inicial:** Windows 11  
**Objetivo:** construir uma ferramenta local que compare a arquitetura pretendida de um projeto com a arquitetura realmente implementada e identifique desvios antes que eles se tornem difíceis de corrigir.

---

## 1. Visão do produto

O Vibe Coding Auditor será uma aplicação desktop independente, com um motor de auditoria reutilizável e uma futura extensão para VS Code.

O usuário define, no início de cada projeto, um **contrato arquitetural**: organização desejada, módulos, responsabilidades, dependências permitidas, limites e restrições. O auditor inspeciona o repositório e compara o código com esse contrato.

O objetivo não é impedir a evolução da arquitetura. É distinguir:

- **Desvio indesejado:** uma alteração viola uma regra ou aumenta o acoplamento sem justificativa.
- **Evolução intencional:** uma mudança arquitetural necessária, explicada e aprovada pelo usuário.

O auditor deve apresentar evidências, não apenas opiniões. Cada achado precisa apontar a regra violada, os arquivos envolvidos e, quando possível, as linhas relevantes.

### Princípios inegociáveis

1. O usuário é a autoridade final sobre a arquitetura.
2. O contrato arquitetural não pode ser alterado silenciosamente pelo auditor ou pelo agente de programação.
3. A auditoria inicial é somente leitura: nunca modificar automaticamente o código-fonte.
4. Regras determinísticas devem ser preferidas a julgamentos de IA.
5. O auditor deve separar fatos, heurísticas e sugestões.
6. Uma alteração arquitetural legítima pode ser aprovada e registrada, atualizando o contrato por meio de uma ação explícita.
7. O projeto deve começar pequeno. Não criar abstrações, serviços ou módulos sem necessidade demonstrada.
8. O funcionamento básico não deve depender de conexão com a internet nem de um modelo de IA.

---

## 2. Escopo da primeira versão

### O MVP deve fazer

- Abrir e analisar um diretório de projeto local.
- Criar e validar um arquivo `auditor.toml` com as intenções arquiteturais.
- Ignorar diretórios gerados e configuráveis, como `.git`, `target`, `node_modules`, `dist`, `build` e ambientes virtuais.
- Fazer uma varredura inicial da estrutura de diretórios e arquivos.
- Construir um primeiro grafo de dependências para projetos Rust.
- Aplicar um conjunto pequeno de regras explícitas.
- Gerar achados com severidade, evidência e explicação.
- Exibir o resultado em terminal e exportar JSON e Markdown.
- Registrar execuções e histórico localmente em SQLite.
- Permitir comparar auditorias sucessivas.

### Não faz parte do MVP

- Um agente autônomo que modifica o projeto.
- Uma IA que decide sozinha qual arquitetura o projeto deveria ter.
- Suporte completo a todas as linguagens.
- Uma extensão VS Code funcional.
- Auditoria de segurança abrangente, análise de vulnerabilidades ou substituição de ferramentas especializadas.
- Refatoração automática.
- Bloqueio de salvamento ou de commits.
- Sistema de plugins.
- Execução de código desconhecido do projeto durante uma auditoria normal.
- Painel de métricas com uma nota geral arbitrária de qualidade.

Esses itens podem ser avaliados depois que o núcleo provar seu valor.

---

## 3. Decisões técnicas iniciais

As tecnologias abaixo são a direção inicial. Antes de instalar versões ou gerar o projeto, o OpenCode deve verificar a documentação oficial e a compatibilidade das versões estáveis disponíveis naquele momento. Não deve escolher versões antigas só porque aparecem em tutoriais.

| Parte | Tecnologia | Responsabilidade |
|---|---|---|
| Motor | Rust | Varredura, análise, regras e geração de achados |
| Aplicação desktop | Tauri 2 | Empacotamento e integração entre interface e núcleo Rust |
| Interface desktop | React + TypeScript | Dashboard, configurações e visualização de resultados |
| Persistência | SQLite | Histórico de auditorias, achados e decisões aprovadas |
| Contrato | TOML | Definição legível e versionável da arquitetura desejada |
| CLI inicial | Rust + Clap | Validar o motor sem depender da interface gráfica |
| Análise inicial de Rust | `syn` e metadados do Cargo | Ler módulos, imports e dependências do workspace |
| Testes | Rust test framework + fixtures | Validar as regras com projetos de exemplo |

### Por que começar pela CLI

Embora o produto final seja desktop, o primeiro protótipo deve ser executável pelo terminal. Isso permite validar a lógica de auditoria sem adicionar prematuramente problemas de interface, estado visual e integração com o Tauri.

A CLI e a aplicação desktop devem chamar o mesmo núcleo Rust. Não duplicar regras de auditoria na interface.

### Arquitetura: monólito modular

Começar com um único processo local e módulos bem separados. Não criar microserviços, servidor web local, comunicação distribuída ou plugins nesta fase.

A separação inicial deve ser lógica, dentro de poucos crates. Só criar novos crates quando houver uma fronteira real que justifique isso.

---

## 4. Arquitetura de referência do próprio auditor

A estrutura abaixo é uma direção, não uma ordem para criar todos os arquivos imediatamente. O OpenCode deve implementar somente as partes necessárias para a etapa atual.

```text
vibe-coding-auditor/
├── AGENTS.md
├── README.md
├── Cargo.toml
├── .gitignore
├── auditor.example.toml
├── apps/
│   └── desktop/                 # Adicionado em uma etapa posterior
│       ├── src/                 # React + TypeScript
│       └── src-tauri/           # Adaptador Tauri; chama o núcleo Rust
├── crates/
│   ├── auditor-core/            # Regras e lógica principal
│   │   └── src/
│   │       ├── contract/        # Leitura e validação do contrato
│   │       ├── discovery/       # Descoberta de arquivos e diretórios
│   │       ├── analysis/        # Analisadores por linguagem
│   │       ├── graph/           # Grafo de módulos e dependências
│   │       ├── rules/           # Avaliação das regras
│   │       ├── findings/        # Modelo e classificação dos achados
│   │       └── report/          # Relatórios JSON e Markdown
│   └── auditor-cli/             # Interface de linha de comando
├── docs/
│   ├── ARCHITECTURE.md
│   ├── DECISIONS.md
│   └── ROADMAP.md
├── tests/
│   └── fixtures/                # Pequenos projetos de teste
└── data/                        # Dados locais de desenvolvimento; não versionar
```

Não criar diretórios vazios só para combinar com o desenho. A estrutura deve crescer junto com funcionalidades implementadas.

### Responsabilidades e limites

- `auditor-core` não deve depender de Tauri, React ou da CLI.
- `auditor-cli` deve traduzir argumentos e apresentar resultados. Não deve implementar regras próprias.
- `apps/desktop/src-tauri` deve ser um adaptador fino para o mesmo núcleo Rust.
- A interface React não deve analisar código-fonte diretamente nem acessar o SQLite por conta própria.
- A camada de persistência deve ficar isolada do domínio e das regras.
- O contrato `auditor.toml` descreve o projeto auditado, não as configurações internas do próprio auditor.
- Relatórios devem ser dados estruturados. A interface decide como apresentá-los.

---

## 5. O contrato arquitetural dos projetos auditados

Cada projeto analisado poderá ter um arquivo `auditor.toml` na raiz. Ele será versionado junto com o código para que as decisões arquiteturais sejam revisáveis no Git.

O contrato deve ser explícito e validado por um schema com versão. Não aceitar propriedades desconhecidas silenciosamente.

### Exemplo conceitual inicial

O exemplo abaixo serve para orientar o desenho do formato. O OpenCode deve transformá-lo em um schema real, testado e documentado, sem adicionar opções que ainda não tenham implementação.

```toml
schema_version = 1

[project]
name = "projeto-exemplo"

[scan]
include = ["crates/**/*.rs"]
exclude = ["target/**", ".git/**", "vendor/**"]

[architecture]
style = "modular_monolith"

[[architecture.modules]]
name = "domain"
path = "crates/app/src/domain/**"
responsibility = "Modelos e regras centrais do domínio"
allowed_dependencies = []

[[architecture.modules]]
name = "application"
path = "crates/app/src/application/**"
responsibility = "Casos de uso e coordenação"
allowed_dependencies = ["domain"]

[[architecture.modules]]
name = "infrastructure"
path = "crates/app/src/infrastructure/**"
responsibility = "Acesso a banco de dados, arquivos e serviços externos"
allowed_dependencies = ["domain", "application"]
```

O schema final deve resolver claramente:

- Como um arquivo é associado a um módulo.
- Como declarar dependências permitidas e proibidas.
- O que fazer quando um arquivo não pertence a módulo declarado.
- Como tratar módulos ainda não mapeados.
- Como declarar exclusões e diretórios gerados.
- Como evoluir o formato sem quebrar contratos antigos.

**Regra:** não inferir uma arquitetura completa e tratá-la como aprovada. O sistema pode sugerir um contrato inicial, mas o usuário precisa revisá-lo e confirmá-lo.

---

## 6. Modelo de achado da auditoria

Todo achado deve ser estruturado, para que CLI, desktop e futura extensão mostrem a mesma informação.

Campos mínimos:

- `id`: identificador estável do tipo de problema ou da ocorrência.
- `rule_id`: regra que produziu o achado.
- `severity`: `error`, `warning`, `info` ou `needs_review`.
- `title`: descrição curta.
- `message`: explicação objetiva.
- `path`: arquivo ou caminho envolvido.
- `line_start` e `line_end`: quando disponíveis.
- `evidence`: o que foi observado.
- `expected`: o que o contrato exige.
- `suggestion`: possível correção, sem aplicá-la automaticamente.
- `confidence`: usar apenas quando a análise for heurística; não inventar precisão numérica.

Distinguir claramente:

- **Violação comprovada:** regra determinística foi violada.
- **Sinal de atenção:** métrica ou heurística ultrapassou um limite.
- **Revisão necessária:** o auditor não tem informação suficiente para decidir.

Não apresentar heurísticas como verdades absolutas.

---

## 7. Plano de implementação por etapas

Cada etapa deve ser concluída, testada e revisada antes de iniciar a seguinte. O OpenCode não deve tentar implementar o roadmap inteiro em uma única solicitação.

### Etapa 0 — Verificar o ambiente e preparar o repositório

**Objetivo:** descobrir se o ambiente Windows está pronto e criar uma base limpa.

Tarefas:

1. Verificar `git`, `rustc`, `cargo`, `rustup`, `node` e `npm`.
2. Verificar os pré-requisitos oficiais do Tauri para Windows, mas não instalar nada sem autorização.
3. Consultar a documentação oficial das versões estáveis de Rust, Tauri e ferramentas necessárias.
4. Criar o repositório Git e os arquivos mínimos de documentação.
5. Criar `AGENTS.md` com as regras deste projeto.
6. Registrar as decisões técnicas em `docs/DECISIONS.md`.

**Critério de conclusão:** ambiente documentado, repositório inicializado e nenhuma dependência desnecessária instalada.

### Etapa 1 — Criar o workspace Rust e a CLI mínima

**Objetivo:** ter uma aplicação que compile e execute, sem ainda analisar código.

Tarefas:

1. Criar um Cargo workspace.
2. Criar `auditor-core` como biblioteca Rust.
3. Criar `auditor-cli` como executável que depende de `auditor-core`.
4. Adicionar comandos iniciais: `--help` e `--version`.
5. Configurar formatação e lint com `cargo fmt` e `cargo clippy`.
6. Criar testes unitários mínimos.
7. Documentar os comandos de desenvolvimento.

**Critério de conclusão:** `cargo check`, `cargo test`, `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings` passam.

### Etapa 2 — Criar o modelo do contrato arquitetural

**Objetivo:** carregar, validar e representar o `auditor.toml`.

Tarefas:

1. Definir a primeira versão do schema TOML.
2. Criar modelos Rust para projeto, módulos, regras e exclusões.
3. Validar campos obrigatórios, caminhos e referências.
4. Rejeitar versões de schema não suportadas com mensagem clara.
5. Criar `auditor init` para gerar um exemplo de contrato, sem afirmar que ele está aprovado.
6. Criar fixtures válidas e inválidas.

**Critério de conclusão:** contratos válidos são carregados; contratos inválidos geram erros compreensíveis e testes reproduzíveis.

### Etapa 3 — Descobrir e inventariar o projeto

**Objetivo:** percorrer o diretório-alvo com segurança e produzir um inventário.

Tarefas:

1. Receber a raiz do projeto como argumento explícito.
2. Respeitar `include` e `exclude` do contrato.
3. Ignorar, por padrão, diretórios gerados e metadados de controle de versão.
4. Evitar seguir links simbólicos para fora da raiz sem uma opção explícita.
5. Registrar caminho relativo, extensão, tamanho e metadados úteis.
6. Lidar com arquivos inacessíveis sem interromper toda a auditoria.

**Critério de conclusão:** inventário consistente, testes com árvore de diretórios controlada e nenhuma execução de arquivos do projeto.

### Etapa 4 — Implementar a primeira análise de dependências Rust

**Objetivo:** produzir um grafo inicial de módulos e dependências.

Tarefas:

1. Ler metadados do workspace pelo ecossistema Cargo.
2. Analisar declarações `mod` e imports `use` nos arquivos Rust.
3. Relacionar os arquivos aos módulos definidos no contrato.
4. Construir um grafo direcionado.
5. Detectar dependências que violam `allowed_dependencies`.
6. Identificar dependências não resolvidas e caminhos não mapeados como revisão necessária.
7. Guardar a origem de cada aresta para permitir apontar a evidência.

**Limitação explícita:** a primeira versão não precisa resolver todos os casos dinâmicos, macros complexas ou dependências geradas. Quando não conseguir concluir, deve sinalizar a limitação em vez de inventar uma conclusão.

**Critério de conclusão:** fixtures com dependências permitidas, proibidas e não resolvidas são classificadas corretamente.

### Etapa 5 — Criar o motor de regras e os relatórios

**Objetivo:** transformar dados analisados em achados úteis.

Começar com poucas regras:

1. Dependência proibida entre módulos.
2. Arquivo Rust fora dos módulos declarados.
3. Dependência circular entre módulos, quando detectável com segurança.
4. Módulo declarado, mas sem arquivos correspondentes.
5. Arquivo excedendo um limite explícito definido pelo usuário, se essa regra estiver configurada.

Tarefas:

- Separar descoberta, análise e avaliação das regras.
- Atribuir identificadores estáveis às regras.
- Produzir objetos de achado conforme a seção 6.
- Gerar saída legível no terminal.
- Exportar JSON e Markdown.
- Definir códigos de saída da CLI para sucesso, achados e falha operacional.

**Critério de conclusão:** os mesmos dados geram relatórios consistentes, e cada achado aponta evidência verificável.

### Etapa 6 — Registrar histórico local em SQLite

**Objetivo:** comparar a evolução entre auditorias.

Tarefas:

1. Salvar data, raiz do projeto, versão do contrato e resumo da execução.
2. Salvar achados associados à execução.
3. Permitir listar e consultar execuções anteriores.
4. Comparar duas auditorias e identificar achados novos, resolvidos e persistentes.
5. Definir política de migração do banco.
6. Não armazenar o conteúdo completo dos arquivos do usuário.

**Critério de conclusão:** o histórico sobrevive ao fechamento da aplicação e comparações são reproduzíveis.

### Etapa 7 — Provar o núcleo antes da interface gráfica

**Objetivo:** evitar construir uma interface para um motor que ainda não funciona.

Tarefas:

1. Criar pelo menos três projetos fictícios de teste: arquitetura correta, violações intencionais e estrutura parcialmente desconhecida.
2. Executar toda a auditoria pela CLI.
3. Avaliar falsos positivos e limitações.
4. Revisar mensagens e evidências.
5. Ajustar o contrato se o formato estiver difícil de usar, sem introduzir campos sem implementação.

**Critério de conclusão:** o motor é útil sem interface gráfica e seus limites estão documentados.

### Etapa 8 — Criar a aplicação desktop com Tauri

**Objetivo:** oferecer uma interface local para operar o núcleo já validado.

Telas iniciais:

- Selecionar e abrir um projeto.
- Resumo da última auditoria.
- Lista de achados filtrável por severidade e regra.
- Visualização do grafo de dependências.
- Editor ou visualizador do contrato arquitetural.
- Histórico de auditorias.
- Tela de configurações.

Regras de implementação:

1. A interface não deve conter a lógica das regras.
2. O backend Tauri chama funções do núcleo Rust.
3. A interface recebe modelos estruturados e renderiza os dados.
4. Toda operação que altera contrato ou configuração pede confirmação explícita.
5. Não implementar execução automática em segundo plano até haver controles claros de pausa e escopo.
6. Seguir as recomendações de segurança do Tauri e restringir permissões da interface.

**Critério de conclusão:** a aplicação abre um projeto, executa a mesma auditoria da CLI e apresenta os mesmos achados.

### Etapa 9 — Criar a extensão VS Code

**Objetivo:** levar alertas para dentro do editor, sem duplicar o motor.

Primeiras funções:

- Detectar que a pasta aberta contém um projeto configurado.
- Acionar uma auditoria manual.
- Mostrar achados no painel Problems/Diagnostics.
- Abrir o arquivo e a linha associados ao achado.
- Mostrar o estado da última auditoria.

A extensão deve consumir uma interface pública e estável do núcleo, inicialmente pela CLI ou por um protocolo local bem definido. Não criar um daemon residente apenas por conveniência.

**Critério de conclusão:** os achados mostrados no VS Code correspondem aos relatórios do núcleo.

### Etapa 10 — Expandir linguagens e recursos

Somente após o MVP estar estável, avaliar:

- TypeScript e JavaScript.
- Python.
- Análise de complexidade e concentração de responsabilidades.
- Detecção de duplicação e código sem referências.
- Regras personalizadas.
- Sugestões de arquitetura assistidas por IA.
- Integração com Git para auditar diffs antes de commits.
- Auditoria agendada ou acionada por alterações.

Cada novo recurso deve ter um caso de uso real, testes, impacto de desempenho conhecido e justificativa arquitetural.

---

## 8. Regras para impedir que o próprio projeto saia dos trilhos

Estas regras devem ser incluídas no `AGENTS.md` e aplicadas pelo OpenCode em todas as etapas.

1. Não implementar mais de uma etapa do roadmap por solicitação, salvo autorização explícita.
2. Antes de alterar a arquitetura, apresentar o problema, a alternativa proposta e o impacto.
3. Não adicionar dependências sem explicar por que a biblioteca padrão ou dependências existentes não bastam.
4. Não criar camadas, traits, interfaces, plugins ou abstrações apenas por antecipação.
5. Não transformar o projeto em microserviços.
6. Não introduzir IA no núcleo determinístico do MVP.
7. Não adicionar suporte a outra linguagem antes de fechar os critérios de conclusão da linguagem atual.
8. Não editar código gerado diretamente.
9. Não ignorar testes quebrados nem reduzir sua cobertura para fazer o build passar.
10. Não atualizar automaticamente `auditor.toml` para silenciar uma violação.
11. Não alterar uma decisão registrada em `docs/DECISIONS.md` sem registrar a substituição e sua justificativa.
12. Quando uma solicitação do usuário contrariar o contrato, explicar o conflito e aguardar a decisão do usuário.
13. Ao terminar uma etapa, informar arquivos alterados, decisões tomadas, testes executados, resultado e limitações restantes.
14. Não afirmar que uma auditoria é completa quando houver arquivos, linguagens ou construções não analisadas.
15. Preferir correções pequenas e localizadas a refatorações amplas.

---

## 9. Conteúdo inicial recomendado para AGENTS.md

O OpenCode deve criar um `AGENTS.md` na raiz do repositório, usando as instruções abaixo como base. Ele pode acrescentar comandos reais depois que o workspace existir, mas não deve remover as restrições arquiteturais.

```markdown
# Instruções do projeto: Vibe Coding Auditor

## Objetivo
Construir uma aplicação desktop local para comparar a arquitetura pretendida de projetos com a arquitetura implementada. O usuário mantém a decisão final.

## Arquitetura definida
- Núcleo de auditoria em Rust.
- Aplicação desktop em Tauri 2 com React e TypeScript.
- CLI Rust para validar o núcleo antes da interface desktop.
- SQLite para histórico local.
- Contrato arquitetural dos projetos analisados em auditor.toml.
- Extensão VS Code somente depois que o núcleo e a aplicação desktop estiverem estáveis.
- Começar como monólito modular. Não criar microserviços.

## Limites
- O núcleo não depende de Tauri, React ou da CLI.
- CLI e desktop reutilizam o mesmo núcleo.
- A interface não implementa regras de auditoria.
- A auditoria é somente leitura por padrão.
- Nunca modificar código-fonte ou contrato silenciosamente.
- Não depender de IA para as regras determinísticas do MVP.

## Processo de trabalho
- Implementar uma etapa do ROADMAP por vez.
- Antes de mudanças arquiteturais, explicar motivo, alternativas e impacto.
- Não adicionar dependências sem justificativa.
- Não criar abstrações preventivas ou funcionalidades fora do escopo atual.
- Não ocultar limitações ou falsos positivos conhecidos.
- Registrar decisões relevantes em docs/DECISIONS.md.
- Manter docs/ROADMAP.md atualizado conforme etapas realmente concluídas.

## Verificação
- Executar testes focados após cada mudança.
- Executar a suíte completa antes de concluir uma etapa.
- Usar cargo fmt e cargo clippy para Rust.
- Não declarar uma etapa concluída se os testes relevantes falharem.
- Relatar exatamente os comandos executados e seus resultados.

## Segurança e privacidade
- Processamento local por padrão.
- Não enviar código-fonte para serviços externos.
- Não executar arquivos do projeto durante a varredura normal.
- Não seguir links simbólicos para fora da raiz sem autorização explícita.
- Não armazenar conteúdo integral dos arquivos no histórico.
```

---

## 10. Primeiro prompt para enviar ao OpenCode

Comece com este prompt. Não peça para ele implementar o produto inteiro.

```text
Leia integralmente o arquivo PLANO_VIBE_CODING_AUDITOR.md.

Quero iniciar o Vibe Coding Auditor seguindo esse documento como referência arquitetural.

Neste primeiro trabalho, execute somente a ETAPA 0:

1. Verifique o ambiente Windows e identifique as ferramentas disponíveis.
2. Verifique os pré-requisitos oficiais necessários para Rust e para Tauri no Windows.
3. Não instale, atualize nem remova ferramentas sem minha autorização.
4. Apresente a estrutura mínima do repositório e as decisões que precisam ser tomadas agora.
5. Crie apenas os arquivos mínimos da etapa 0, incluindo AGENTS.md, README.md e docs/DECISIONS.md, se ainda não existirem.
6. Preserve este plano. Não reescreva o escopo nem antecipe a implementação das próximas etapas.
7. Não crie código de funcionalidade ainda.

Antes de executar qualquer comando que altere o sistema ou instale dependências, explique o que ele fará e peça minha autorização.

Ao terminar, informe o que foi verificado, o que foi criado, o que falta e se a etapa 0 atende aos critérios de conclusão.
```

### Prompt padrão para cada etapa futura

Depois de aprovar a etapa anterior, use um prompt semelhante a este, substituindo o número e o nome:

```text
Vamos implementar somente a ETAPA [NÚMERO] — [NOME] do PLANO_VIBE_CODING_AUDITOR.md.

Antes de editar:
1. Leia o AGENTS.md e os documentos relevantes.
2. Confira o estado atual do repositório e os testes existentes.
3. Resuma o que será criado ou alterado e como isso respeita a arquitetura definida.
4. Aponte conflitos, ambiguidades ou dependências novas antes de prosseguir.

Implemente apenas o necessário para os critérios de conclusão desta etapa. Não antecipe funcionalidades de etapas posteriores.

Ao terminar:
1. Execute os testes e verificações relevantes.
2. Corrija regressões introduzidas por esta etapa.
3. Atualize a documentação realmente afetada.
4. Liste arquivos alterados, decisões, comandos executados, resultados e limitações.
5. Não declare a etapa concluída se algum critério não tiver sido atendido.
```

---

## 11. Critério de qualidade do projeto

O sucesso não será medido pela quantidade de funcionalidades ou pelo tamanho do código. O projeto estará evoluindo corretamente quando:

- O contrato arquitetural for simples de entender e modificar.
- As regras forem previsíveis e testáveis.
- Os achados tiverem evidências rastreáveis.
- Falsos positivos e limitações forem explícitos.
- A CLI e a interface desktop apresentarem os mesmos resultados.
- O núcleo permanecer independente das interfaces.
- Mudanças arquiteturais forem deliberadas e registradas.
- O auditor for útil sem IA e sem conexão com a internet.

**Diretriz final:** construir primeiro um auditor pequeno, determinístico e confiável. A interface desktop, a extensão VS Code, o suporte a novas linguagens e os recursos de IA vêm depois, quando houver uma base funcional que justifique sua complexidade.

---

## 12. Referências oficiais

- OpenCode, instruções persistentes com `AGENTS.md`: https://opencode.ai/docs/instructions
- OpenCode, inicialização do projeto e `/init`: https://opencode.ai/docs
- Tauri, documentação: https://tauri.app/start/
- Rust, instalação e toolchain: https://www.rust-lang.org/tools/install
- Cargo, referência oficial: https://doc.rust-lang.org/cargo/
