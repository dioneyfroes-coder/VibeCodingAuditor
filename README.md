# Vibe Coding Auditor

Ferramenta local para comparar a arquitetura pretendida de um projeto com a arquitetura realmente implementada. Cada achado deve apontar a regra violada, os arquivos envolvidos e, quando possível, as linhas relevantes. O contrato arquitetural (`auditor.toml`) pertence ao usuário e é versionado junto ao projeto auditado.

**Status atual:** Etapa 0 — verificação do ambiente e preparação inicial do repositório. Nenhuma funcionalidade de auditoria foi implementada ainda.

## Documentação

- Plano completo e critérios de cada etapa: [PLANO_VIBE_CODING_AUDITOR.md](PLANO_VIBE_CODING_AUDITOR.md)
- Decisões técnicas e ambiente verificado: [docs/DECISIONS.md](docs/DECISIONS.md)
- Progresso das etapas: [docs/ROADMAP.md](docs/ROADMAP.md)
- Regras para agentes: [AGENTS.md](AGENTS.md)

## Direção técnica definida no plano

- Núcleo de auditoria em Rust, reutilizado pela CLI e pela futura aplicação desktop.
- Interface desktop planejada com Tauri 2, React e TypeScript.
- Histórico local em SQLite; contrato dos projetos auditados em TOML.
- Processamento local e auditoria somente leitura por padrão.
- Regras determinísticas, sem dependência de IA ou de internet para o funcionamento básico.

## Ambiente

O resumo das versões encontradas e das referências oficiais consultadas está em [docs/DECISIONS.md](docs/DECISIONS.md). Nenhuma ferramenta foi instalada, atualizada ou removida.

## Execução

Ainda não há executável. O workspace Rust e a CLI mínima pertencem à Etapa 1 e só devem ser criados após a conclusão e a aprovação da Etapa 0.
