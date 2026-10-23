# Orquestração de issues

Como a casa implementa a fila de issues `ready-for-agent` (ADR 0005 do método). Uma sessão própria,
aberta pelo kickoff do `/nova-sessao` com `/orquestrar-issues <N>`, tem um agente **orquestrador** que
dispara os papéis de `.claude/agents/` **em sequência**, um subagente por vez, cada um com a janela
limpa. Os passos estão na skill `/orquestrar-issues`; aqui está o porquê de cada peça.

## Quando recomendar

Ao terminar um `/to-tickets` ou um `/triage` que deixou issues em `ready-for-agent`, recomende a
orquestração ao humano e gere, com o `/nova-sessao`, o kickoff da janela do orquestrador. Num projeto
em adoção, só a partir da etapa `planejamento` (`docs/agents/adocao.md`).

## Os papéis

| Papel | Faz | Edita? |
|---|---|---|
| `implementador` | `/tdd` na branch da issue; num ciclo de correção, só o plano do diagnosticador | Sim |
| `revisor` | Eixo *Standards* do `/code-review`: padrões do projeto, testes, vocabulário | Não |
| `conferente` | Eixo *Spec* do `/code-review`: o feito bate com a issue e o spec? | Não |
| `diagnosticador` | Causa raiz dos achados reprovados e plano de correção | Não |

Revisão e conferência são separadas de propósito: código que segue todos os padrões pode fazer a
coisa errada, e código que faz a coisa certa pode quebrar os padrões. Um eixo não esconde o outro.

## O ciclo de uma issue

```
implementador → revisor → conferente
  ├─ dois APROVADO → merge na branch de integração → próxima issue
  └─ algum REPROVADO → diagnosticador → implementador (plano) → revisor → conferente → …
                        (até 2 ciclos de correção; depois, a issue volta para needs-triage)
```

Duas issues devolvidas seguidas param a rodada: algo comum a elas precisa de um humano.

## Branches

O desenvolvedor escolhe, antes de começar, o nome da **branch de integração**
(`integracao/<nome>`) e a base. Cada issue parte dela (`issue/<n>-<slug>`) e volta para ela por merge
depois de aprovada. No fim da fila, o desenvolvedor decide: uma PR só da integração para a base, ou
uma PR por issue, empilhadas na ordem. **Merge na `main`, nunca pelo agente.**

## Modelo e esforço

Todos os papéis usam `opus`. No primeiro ciclo, implementador, revisor e conferente rodam em `low`;
o diagnosticador, que abre cada ciclo de correção, roda sempre em `medium`, e nos ciclos de correção
todos sobem para `medium`. O esforço vai em cada chamada, para subir sem
editar os agentes.

## Watchdogs na orquestração

- Cada subagente aparece para os hooks com o próprio `agent_id`: a regra de uma issue ativa vale por
  subagente implementador, e os demais papéis não editam.
- O orquestrador grava o andamento em `.claude/orquestracao/<nome>.json` (fora do git) e, no aviso
  da smart zone (450k), passa a rodada para uma janela nova com o `/nova-sessao`. Nas sessões normais
  o limite duro continua em 550k.
- `.claude/agents/` é infraestrutura protegida: o agente não edita os papéis.
