# Instruções do projeto: Vibe Coding Auditor

## Objetivo
Construir uma aplicação desktop local para comparar a arquitetura pretendida de projetos com a arquitetura implementada. O usuário mantém a decisão final.

## Arquitetura definida
- Núcleo de auditoria em Rust.
- Aplicação desktop em Tauri 2 com React e TypeScript.
- CLI Rust para validar o núcleo antes da interface desktop.
- SQLite para histórico local.
- Contrato arquitetural dos projetos analisados em `auditor.toml`.
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
- Registrar decisões relevantes em `docs/DECISIONS.md`.
- Manter `docs/ROADMAP.md` atualizado conforme etapas realmente concluídas.

## Verificação
- Executar testes focados após cada mudança.
- Executar a suíte completa antes de concluir uma etapa.
- Usar `cargo fmt` e `cargo clippy` para Rust.
- Revisar paradigma, nomes descritivos, documentação de código e testes dos helpers após cada mudança.
- Não declarar uma etapa concluída se os testes relevantes falharem.
- Relatar exatamente os comandos executados e seus resultados.

## Segurança e privacidade
- Processamento local por padrão.
- Não enviar código-fonte para serviços externos.
- Não executar arquivos do projeto durante a varredura normal.
- Não seguir links simbólicos para fora da raiz sem autorização explícita.
- Não armazenar conteúdo integral dos arquivos no histórico.

## Comandos de desenvolvimento
- `cargo check`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p auditor-cli -- --help` e `cargo run -p auditor-cli -- --version`
- `cargo run -p auditor-cli -- init <diretorio>` (cria modelo de contrato; nunca sobrescreve `auditor.toml`)
- `cargo run -p auditor-cli -- scan <raiz>` (inventaria arquivos; somente leitura)
- `cargo run -p auditor-cli -- analyze <raiz>` (grafo inicial de dependências; somente leitura)
- `cargo run -p auditor-cli -- audit <raiz>` (regras e relatório; somente leitura)
- `cargo run -p auditor-cli -- audit <raiz> --diff <patch>` (filtra achados por diff salvo do Git; somente leitura)
- `cargo run -p auditor-cli -- history record <raiz>` (grava execução no SQLite local)
- `cargo run -p auditor-cli -- history list|show|compare` (consulta e comparação; somente leitura do banco)
- Se o OneDrive causar avisos de sessão incremental: `cargo clippy --workspace --all-targets --config 'build.incremental=false' -- -D warnings`

## Referências
- O plano integral está em `PLANO_VIBE_CODING_AUDITOR.md` e deve ser preservado.
