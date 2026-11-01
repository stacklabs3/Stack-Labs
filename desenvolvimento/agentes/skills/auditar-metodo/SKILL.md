---
name: auditar-metodo
description: Audita, só lendo o histórico, se um projeto da StackLabs segue o método.
disable-model-invocation: true
---

Os watchdogs barram a ação na hora; esta auditoria olha o histórico inteiro do projeto e acha o que
passou por fora (um humano sem hooks, um watchdog desligado, uma regra que máquina não checa).
Você só **lê**: o resultado é um relatório, e o que fazer com cada desvio o humano decide.

## 1. Colete

Rode cada verificação abaixo com `gh` e `git` no clone do projeto, num subagente se o histórico for
grande. Pronto quando todo item abaixo tem ✅, ⚠️ ou ❌ e a evidência (nome do ticket com link, hash,
caminho).

**Etapa de adoção**
- Se `.claude/metodo.json` existe, informe a etapa e a data. Meça todos os critérios mesmo assim, e
  marque como ⚠️ (esperado nesta etapa) o que a etapa ainda não cobra (`docs/agents/adocao.md`).

**Setup**
- `AGENTS.md`, `CLAUDE.md`, `docs/agents/*.md`, `.claude/settings.json`, `.claude/watchdogs/`,
  `.claude/agents/`, `.github/workflows/vigia-do-metodo.yml` existem e batem com o `modelo-de-projeto` do repositório
  Stack-Labs (diff resumido do que divergiu).
- As labels do método existem (`docs/agents/issue-tracker.md`, seção *Labels*).

**Mapas iniciais**
- Os cinco existem, com `wayfinder:map` + `mapa-inicial:<tipo>`.
- Cada um tem *Destination* preenchido e *Decisions so far* com um ponteiro por ticket fechado.
- Nenhuma issue `ready-for-*` foi fechada antes de Protótipo, Arquitetura, Fases e Regras de
  código fecharem (compare as datas).
- `docs/regras-de-codigo.md` existe, cada regra tem o porquê e um exemplo, e o mapa de Regras de
  código aponta para ele. Amostra de 3 PRs mergeadas: o diff segue as regras.
- `docs/regras-de-codigo.md` tem a tabela de cobertura com os 22 princípios de
  `docs/agents/principios-de-codigo.md`, cada um adotado, adaptado ou afastado com o porquê (o E1 e o P5 nunca afastados), e cada regra cita o princípio de onde nasceu. Cada ticket de grupo do mapa
  fechou com uma linha de decisão por princípio. Projeto anterior à ADR 0008 sem cobertura: ⚠️ e o
  ticket que a completa.
- Para cada linguagem do projeto com arquivo em `docs/agents/principios-de-codigo/`,
  `docs/regras-de-codigo.md` tem a tabela *Práticas da linguagem* com cada prática desse arquivo
  adotada ou afastada. Projeto anterior à ADR 0012 sem a tabela, ou projeto Dart anterior à ADR 0019
  sem ela: ⚠️ e o ticket que a completa.
- `docs/riscos.md` existe, cada risco tem origem com a aprovação do humano, e o P5 da tabela de
  cobertura cita a regra com `teste-cobre-risco`. Amostra de 3 PRs mergeadas: cada teste novo exercita
  o risco que cita. Projeto anterior à ADR 0014 sem o registro: ⚠️ e o ticket que o escreve.
- `sh .claude/watchdogs/rodar validar` (o repositório inteiro) termina em `VALIDADOR: APROVADO` e roda no CI do projeto. Quem audita precisa das ferramentas do projeto na máquina: saída 127 é ferramenta ausente, não violação.
  Projeto anterior à ADR 0010 sem `docs/regras-de-codigo.json`: ⚠️ e o ticket que o escreve.

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

**Caminho até ready-for-agent**
- Toda issue `ready-for-agent` nasceu de uma rota de `docs/agents/caminho-ate-ready-for-agent.md`
  (sub-issue de spec, ou agent brief do `/triage`); amostra de 5 conferida contra a definição de pronto.
- Todo spec tem a label `spec` e nenhum spec está em `ready-for-agent`.

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

Pronto quando o relatório cobre todo item da seção 1 e o humano recebeu o caminho dele.
