# Proposta técnica — modo de descoberta e Architecture Builder

Status: aprovada e implementada nas Etapas 4.1–4.7. Este documento registra a proposta original da Etapa 4.0; o estado corrente está em `docs/ROADMAP.md` e `docs/DECISIONS.md`.
Data: 26/11/2026

Este documento responde aos itens 5 e 6 da Etapa 4.0 do plano. Ele descreve o menor refactor compatível com o estado atual, sem introduzir um segundo scanner, um segundo motor de regras, IA, rede ou novas dependências.

## 1. Motivo e princípios

Hoje a ausência de `auditor.toml` é um erro operacional em todos os pontos de entrada: a CLI carrega o contrato em `scan_project` (`main.rs:164`) e `analyze_project` (`main.rs:181`); `audit_project` carrega em `audit.rs:68`; o backend Tauri carrega em `analyze_project` (`lib.rs:27`) e lê o texto em `read_contract` (`lib.rs:35`); a extensão VS Code verifica `existsSync(root/auditor.toml)` antes de chamar `audit` e a desktop orienta o usuário a selecionar uma raiz que contenha o contrato.

A mudança necessária não é apenas tratar `NotFound` como vazio. Sem contrato não existe lista aprovada de dependências permitidas, logo não faz sentido emitir `dep.not_allowed`, `dep.layer_not_allowed`, `file.unmapped`, `module.unreferenced` ou qualquer outro achado que pressuponha módulos declarados. O resultado deve ser explicitamente de descoberta: estrutura observada, evidências, sugestões e limitações.

Princípios derivados do plano:

- Descoberta não é aprovação arquitetural.
- Observação, sugestão e violação são categorias distintas.
- O núcleo continua sendo a única fonte de regras, validação e serialização.
- O contrato pertence ao usuário; a aplicação nunca o cria ou altera sem confirmação explícita.
- Perfis do Builder devem ser representáveis pelo schema v2 e fiscalizáveis pelo motor atual.
- A auditoria e a descoberta permanecem locais, determinísticas e somente leitura até a gravação final aprovada.

## 2. APIs atuais afetadas

| Local | Assinatura ou comportamento atual | Impacto previsto |
|---|---|---|
| `contract::load_contract(path)` | lê e valida `auditor.toml`; `NotFound` é `ContractError::Io` | continua assim; ausência passa a ser decidida pelo chamador antes da chamada, não silenciada dentro do parser |
| `contract::parse_contract(text)` | valida schema 1 ou 2 e rejeita campos desconhecidos | continua sendo o validador único do Builder |
| `discovery::discover(root, &Contract)` | usa `scan.include`/`scan.exclude` e `DEFAULT_IGNORED_DIRS` | precisa aceitar opções de varredura sem contrato; manter os mesmos motivos de exclusão |
| `analysis::analyze(root, &Contract, &Inventory)` | classifica arestas como `Allowed`, `Forbidden` ou `Unresolved` | precisa separar coleta de observações da classificação por contrato |
| `rules::evaluate(&Contract, &Inventory, &Analysis)` | produz `Finding`s de conformidade | continua chamado somente no modo de conformidade |
| `report::AuditReport::from_pipeline(...)` | exige `&Contract`; campos atuais são a base do JSON | permanece para conformidade; novos campos serão aditivos |
| `audit::audit_project(root)` | carrega contrato internamente | passa a detectar modo ou receber modo explícito, sem duplicar regras |
| CLI `Scan`, `Analyze`, `Audit` | exigem contrato na raiz; saídas `0`, `1`, `2` | mesmos nomes; descoberta sem contrato retorna `0` e identifica o modo |
| Tauri `audit_project`, `analyze_project`, `read_contract` | exigem contrato | novos comandos finos para descoberta, prévia, validação e gravação |
| Extensão `runAudit` | bloqueia explicitamente pasta sem contrato em `extension.ts:174-177` | passará a chamar descoberta e a mostrar observações, sem diagnósticos de violação |

O mapeamento não encontrou nenhuma API pública de descoberta sem contrato. Hoje `contract::load_contract` e `contract::parse_contract` são as únicas portas públicas de leitura/validação; `Contract::validate` é privada. `history record` e o comando Tauri `history_record` também dependem de `audit_project`, portanto falham sem contrato. `diff::parse_unified_diff` e `diff::scope_report` não dependem de `Contract`, mas reutilizam `discovery::matches_pattern`: alterar esse matcher não pode mudar o comportamento do `--diff`.

`Contract::architecture.modules[].path` é um único padrão `String` nos schemas 1 e 2. Portanto, o Builder não pode oferecer “um ou mais paths” por módulo sem mudar o schema. Para MVP, cada candidato terá um `path`; diretórios adicionais viram candidatos separados ou ambiguidade explícita.

## 3. Formato de resultado proposto

### 3.1 Tipos no núcleo

Um tipo discriminado de execução, sem enum de regras:

```rust
enum ExecutionMode { Discovery, Compliance }

enum ContractStatus {
    Missing,
    Present { schema_version: u32, identity: String },
}

enum ProjectRun {
    Discovery(DiscoveryReport),
    Compliance(AuditReport),
}
```

`DiscoveryReport` será um tipo novo, separado de `AuditReport`. Campos mínimos:

- `mode: "discovery"`.
- `contract: { "status": "missing" }`.
- `project`: nome sugerido e origem da sugestão (`directory`, `manifest` ou `manual`).
- `languages`: extensões e quantidades, sem concluir tecnologia a partir de nome de pasta.
- `candidate_modules`: nome, `path`, responsabilidade sugerida, evidências e categoria de evidência.
- `observed_dependencies`: origem, destino bruto, resolução, arquivo, linha e evidência.
- `ambiguities`: candidatos ou destinos que não puderam ser decididos.
- `limitations`: analisadores ausentes, arquivos não analisados, manifests com padrão não suportado.
- `coverage` e `skip_summary`: mesmos modelos já usados em `AuditReport`.
- `observations`: texto curto, sem `rule_id` de conformidade.
- `findings: []`: preservado para consumidores atuais, mas sempre vazio em descoberta.

A categoria de evidência será textual e fechada: `direct` (manifesto/pacote reconhecido), `inferred_path` (diretório ou extensão), `ambiguous`. Não haverá porcentagem de confiança.

No modo de conformidade, `AuditReport` ganha apenas campos aditivos `mode: "compliance"` e `contract` com `status`, `schema_version` e `identity`. Todos os campos atuais — `project`, `summary`, `findings`, `limitations`, `coverage`, `skip_summary` — mantêm nome e significado. `identity` será um hash determinístico da forma canônica do contrato, produzido no núcleo, sem salvar o conteúdo do projeto.

### 3.2 Separação da análise

A menor mudança internamente é dividir `analysis::analyze` em duas etapas reutilizando o mesmo scanner:

1. Coleta neutra: lê manifests e arquivos suportados e produz pacotes, dependências declaradas, imports com arquivo/linha/evidência, declarações `mod`, duplicações, cobertura e limitações. Nenhum destino é classificado como permitido ou proibido.
2. Classificação com contrato: mapeia arquivos e destinos para módulos declarados e produz `Analysis` com `DependencyKind`.

`discovery::discover` continua com a assinatura atual e passa a ser um wrapper que delega para uma função interna com `ScanOptions` (`include`/`exclude`). `is_excluded_path` passa a receber as listas em vez de `&Contract`. Sem contrato, os defaults seguros incluem extensões com analisador conhecido nas pastas de código encontradas abaixo da raiz, mantêm `DEFAULT_IGNORED_DIRS`, nunca seguem links simbólicos e registram entradas inacessíveis como hoje. `SkipReason`, `matches_pattern` e o comportamento do `--diff` ficam intactos. Isso reaproveita o walker atual; não cria um segundo scanner.

## 4. Agrupamento de módulos candidatos

Ordem de evidências, da mais forte para a mais fraca:

1. **Pacote/manifesto explícito:** cada `Cargo.toml` com `[package]` e membro efetivo do workspace vira candidato `direct`. O nome vem de `[package].name`; o path vem do diretório do manifesto. Regras já existentes de `workspace.members`, `exclude` e expansão com `*` são reutilizadas.
2. **Fronteira de diretório com código:** Pastas que contêm arquivos com analisador e não pertencem a um pacote conhecido viram candidato `inferred_path`. Um único diretório não é automaticamente um módulo: o candidato é apresentado para confirmação.
3. **Dependência observada:** uma aresta cuja origem ou destino coincide com um candidato existente reforça a evidência; nunca cria permissão.
4. **Ambiguidade:** se dois candidatos cobrem o mesmo arquivo ou dois pacotes usam o mesmo nome curto, o registro `ambiguous` é preservado e bloqueia decisões obrigatórias no Builder.

Nomes e paths são ordenados lexicograficamente. Nenhuma inferência vira regra sem ação do usuário. Dependências externas declaradas em Cargo permanecem em `declared_dependencies` e não viram módulo interno.

## 5. CLI proposta

Nomes de comandos e flags atuais permanecem. A seleção de modo será automática pela presença de `auditor.toml`, sem nova flag obrigatória:

- `scan <raiz>`: com contrato, comportamento atual; sem contrato, inventário com defaults seguros e cabeçalho `Modo: descoberta (sem contrato)`.
- `analyze <raiz>`: com contrato, `Analysis` atual; sem contrato, grafo de observações sem rótulos permitida/proibida/não resolvida e com os mesmos dados de cobertura e limitações.
- `audit <raiz> [--format ...] [--output ...] [--diff ...]`: com contrato, relatório de conformidade atual; sem contrato, relatório de descoberta nos mesmos três formatos. `--diff` continua aplicável apenas a achados; em descoberta, o relatório informa que não houve filtro de achados porque não há conformidade.
- `init <diretorio>`: inalterado por enquanto. Continua criando um modelo v1 que exige revisão. A eventual substituição pelo Builder será registrada antes da Etapa 4.4.

Códigos de saída propostos:

| Código | Significado |
|---:|---|
| `0` | operação bem-sucedida, inclusive descoberta com observações |
| `1` | conformidade com pelo menos um achado; inalterado |
| `2` | falha operacional: raiz inacessível, TOML inválido, rascunho inválido ou gravação falha |

Ausência de contrato nunca produz `2`. TOML presente mas inválido continua produzindo `2`, sem fallback para descoberta.

## 6. Modelo do Architecture Builder

Novo módulo do núcleo, sem dependência de Tauri, com tipos de rascunho que correspondem 1:1 ao schema v2:

```rust
struct ContractDraft {
    project_name: String,
    include: Vec<String>,
    exclude: Vec<String>,
    style: ArchitectureStyle, // apenas ModularMonolith nesta fase
    modules: Vec<DraftModule>,
    layers: Vec<DraftLayer>,
    pending_decisions: Vec<PendingDecision>,
}
```

Operações puras propostas:

- `draft_from_discovery(&DiscoveryReport) -> ContractDraft` — importa candidatos como sugestões editáveis e preenche `pending_decisions` para cada relação observada não aprovada.
- `empty_draft(project_name: &str) -> ContractDraft` — para projeto novo ou vazio, sem evidências de implementação.
- `validate_draft(&ContractDraft) -> Result<Contract, ContractError>` — converte para `Contract`, usa `parse_contract`/validação real e devolve erros por campo.
- `render_contract(&Contract) -> Result<String, ContractError>` — serialização determinística.
- `contract_identity(&Contract) -> String` — hash da forma canônica para histórico.

Para `render_contract`, a mudança mínima é derivar `Serialize` nos tipos já existentes de `contract`, garantindo ordem estável dos campos pela ordem das structs, e testar `parse_contract(render_contract(contract)) == contract`. Nenhum campo novo será gravado no TOML. `responsibility` continua texto livre; o motor não o interpreta.

Regras do Builder:

- Perfis oferecidos: “monólito modular sem camadas” e “monólito modular em camadas”. Ambos preenchem `schema_version = 2`; o primeiro com `layers = []`.
- Cada módulo tem um `path`; sobreposição e nomes duplicados são bloqueados pelo validador do núcleo.
- Camadas só aparecem quando o perfil em camadas for escolhido; módulos pertencem a exatamente uma camada.
- Dependências observadas aparecem como sugestões, nunca como `allowed_dependencies` aprovadas. Cada relação exige decisão explícita ou exclusão consciente.
- `pending_decisions` impede a geração final, mas não é gravado no contrato.

## 7. Backend Tauri e desktop

Comandos finos propostos no adaptador, todos delegando ao núcleo:

- `discover_project(path) -> DiscoveryReport`
- `load_project(path) -> ProjectRun` — detecta contrato e escolhe modo
- `validate_contract_draft(draft) -> ValidationResult`
- `preview_contract(draft) -> String`
- `save_contract(path, draft, confirmation) -> ContractStatus` — recusa sobrescrita sem confirmação
- `audit_project(path)` — existente, continuando a servir o modo de conformidade

Na interface, a ação “Definir arquitetura” aparece quando `contract.status == "missing"` e “Revisar arquitetura” quando `present`. O assistente terá as sete etapas do plano com estado local de rascunho, navegação anterior/próxima, prévia TOML somente leitura, diff para contrato existente e estados explícitos de carregamento, validação, salvamento, sucesso e falha. Toda validação e serialização virá dos comandos do núcleo; React apenas apresenta e envia escolhas.

A gravação usará arquivo temporário no mesmo diretório, validação do texto gravado e renomeação quando suportada. No Windows, se `fs::rename` não substituir um destino existente, a Etapa 4.6 deve apresentar uma decisão explícita antes de adicionar dependência ou renunciar à atomicidade; isso não será resolvido silenciosamente nesta proposta.

## 8. Extensão VS Code e histórico

Extensão: quando o contrato estiver ausente, `runAudit` chamará `audit --format json`, lerá `mode` e `contract.status`, exibirá as observações como mensagens informativas e não criará diagnósticos de violação. O comando continuará desabilitado no menu de contexto quando não houver contrato, mas poderá ser executado pela paleta para descoberta. Um aviso orientará a usar o desktop para o Builder. Nenhuma regra será duplicada na extensão.

Histórico: `RunSummary.schema_version` hoje é a versão do contrato, não do modo; `History::compare` compara apenas IDs de achado e não distingue execuções. Para impedir comparação enganosa, propõe-se registrar `execution_mode` e `contract_identity` por execução. Isso exige uma proposta separada de migração SQLite v3 antes da Etapa 4.7, com transação, compatibilidade com bancos v1/v2 e testes de legado. Até lá, `history record` e `history_record` não serão chamados para descoberta: nenhuma execução será gravada como se fosse conformidade. `history compare` deverá rejeitar execuções de modos diferentes ou com identidades de contrato diferentes, com mensagem explícita.

## 9. Riscos e pontos que exigem decisão

- Separar coleta e classificação sem alterar a semântica atual das arestas é o maior risco de regressão; testes de paridade com fixtures v1/v2 são obrigatórios antes de mudar `analyze`.
- Manter `findings` no JSON de descoberta pode ser lido como conformidade vazia; por isso `mode` e `contract.status` serão obrigatórios na saída e a extensão será atualizada antes do fechamento da fase.
- Um diretório com vários pacotes ou pacotes com nomes curtos iguais pode gerar candidatos errados; a categoria `ambiguous` e a decisão humana são obrigatórias.
- `path` único por módulo limita a representação de pastas múltiplas; ampliar o schema está fora do escopo e exigiria proposta própria.
- `init` hoje grava `auditor.example.toml`, que é v1, enquanto o Builder prevê v2 por padrão. Manter `init` inalterado até a Etapa 4.4 evita mudança silenciosa, mas exige comunicação clara na interface.
- `diff::scope_report` reutiliza `discovery::matches_pattern`; qualquer mudança no matcher precisa de teste de paridade do `--diff`.
- Escrita atômica no Windows precisa de decisão explícita na Etapa 4.6.
- A migração SQLite é necessária para comparabilidade segura, mas não deve ser implementada sem proposta e testes de banco legado.

## 10. Impacto esperado e aprovação

Nenhuma mudança de `rule_id`, severidade, campos existentes de `findings`, schema de contrato v1/v2, comandos atuais, códigos de saída `1`/`2`, dependências Rust ou npm foi proposta. Mudanças aditivas ficam restritas a novos tipos de descoberta, novos campos de modo/contrato no JSON, comandos finos no backend e etapas de gravação aprovada.

A proposta foi registrada na Etapa 4.0. A implementação da Etapa 4.1 depende da aprovação explícita do usuário.
