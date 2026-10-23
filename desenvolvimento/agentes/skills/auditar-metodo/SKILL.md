---
name: auditar-metodo
description: Audita se um projeto da StackLabs segue o método de desenvolvimento com IA (wayfinder, mapas iniciais, pesquisa antes de recomendar, protótipos aprovados, PRs ligadas a fatias). Use quando pedirem para auditar o método, conferir a saúde do processo de um projeto ou antes de uma entrega ao cliente.
disable-model-invocation: true
---

Os hooks barram a ação na hora; esta auditoria olha o histórico inteiro do projeto e acha o que
passou por fora (um humano sem hooks, um watchdog desligado, uma regra que máquina não checa).
Você só **lê**: não corrige, não fecha, não comenta. O resultado é um relatório.

## 1. Colete

Rode cada verificação abaixo com `gh` e `git` no clone do projeto, num subagente se o histórico for
grande. Cada item termina com ✅, ⚠️ ou ❌ e a evidência (nome do ticket com link, hash, caminho).

**Setup**
- `AGENTS.md`, `CLAUDE.md`, `docs/agents/*.md`, `.claude/settings.json`, `.claude/hooks/`,
  `.github/workflows/vigia-do-metodo.yml` existem e batem com o `modelo-de-projeto` do repositório
  Stack-Labs (diff resumido do que divergiu).
- As labels do método existem (`docs/agents/issue-tracker.md`, seção *Labels*).

**Mapas iniciais**
- Os quatro existem, com `wayfinder:map` + `mapa-inicial:<tipo>`.
- Cada um tem *Destination* preenchido e *Decisions so far* com um ponteiro por ticket fechado.
- Nenhuma issue `ready-for-*` foi fechada antes de Arquitetura e Fases fecharem (compare as datas).

**Pesquisa antes de recomendar**
- Todo ticket fechado de grilling, tarefa e pesquisa tem `## Fontes` com link no comentário de resolução.
- Abra uma amostra de até 5 tickets de escolha técnica e confira que a fonte citada **sustenta** a
  decisão (existe, é primária, tem data). Este é o item que nenhum hook checa.
- Arquivos de `docs/research/` citam fonte por afirmação e marcam **[confirmar]** o que falta.

**Grilling e wayfinder**
- Tickets de grilling têm rodadas com perguntas numeradas, recomendação e resposta do humano (amostra de 3).
- Nenhuma sessão fechou dois tickets de decisão (tickets do mesmo mapa fechados pelo mesmo autor em
  minutos são o sinal).
- Tickets citados pelo nome no mapa, não por `#número` solto.

**Protótipo**
- Todo `/portar-prototipo` (commit ou PR que leva tela de `prototipos/` para produção) tem antes a
  aprovação explícita no ticket.

**Implementação**
- Nenhum commit direto na `main` (`git log --first-parent main` só com merges de PR).
- Toda PR mergeada de `issue/*` tem `Closes #<n>` para uma issue `ready-for-*`.
- PRs têm testes junto do código (amostra de 5).
- Termos novos no código existem no `GLOSSARY.md`; nenhuma PR contradiz uma ADR sem apontar.

## 2. Relate

Escreva `.scratch/auditoria-do-metodo-<data>.md` com:

1. Um resumo de três linhas: o projeto segue, segue com desvios ou não segue o método.
2. A tabela dos itens acima com o status e a evidência.
3. Os desvios em ordem de gravidade, cada um com a regra do `AGENTS.md` que ele fere e o que o
   humano precisa decidir (corrigir agora, aceitar e registrar, ou ajustar o método).

Pronto quando todo item da seção 1 tem status e evidência, e o humano recebeu o caminho do relatório.
