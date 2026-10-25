# Orquestração de issues

Como a casa implementa a fila de issues `ready-for-agent` (ADR 0005 do método). Uma sessão própria,
aberta com `/orquestrar-issues <N>`, tem um **orquestrador** que dispara os papéis de
`.claude/agents/` **em sequência**, um subagente por vez, cada um com a janela limpa. Os passos, o
esforço por ciclo e o estado da rodada estão na skill `/orquestrar-issues`; aqui está o porquê de
cada peça.

## Quando recomendar

Ao terminar um `/to-tickets` ou um `/triage` que deixou issues em `ready-for-agent`, recomende a
orquestração ao humano e gere, com o `/nova-sessao`, o kickoff da janela do orquestrador. Num projeto
em adoção, só a partir da etapa `planejamento` ([adocao.md](adocao.md)).

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

O desenvolvedor escolhe o nome da **branch de integração** e a base. No fim da fila ele decide:
uma PR só da integração para a base, ou uma PR por issue, empilhadas na ordem. O merge na base é
sempre do desenvolvedor.

## Esforço e modelo

O primeiro ciclo roda barato; o diagnosticador e os ciclos de correção sobem o esforço. O esforço vai
em cada chamada, para subir sem editar os papéis. O modelo segue a mesma ideia (ADR 0016): a 1ª
implementação e a conferência rodam em Sonnet para poupar o limite semanal, e o julgamento (revisor,
diagnosticador) e as correções ficam em Opus. O `implementador` traz `sonnet` no arquivo, e o
orquestrador o chama com `opus` nos ciclos de correção.

## Watchdogs na orquestração

- Cada subagente aparece para os hooks com o próprio `agent_id`: a regra de uma issue ativa vale por
  subagente implementador, e os demais papéis não editam.
- No aviso da smart zone (450k), o orquestrador grava o estado da rodada e passa a rodada para uma
  janela nova com o `/nova-sessao`. Nas sessões normais o limite duro continua em 550k.
