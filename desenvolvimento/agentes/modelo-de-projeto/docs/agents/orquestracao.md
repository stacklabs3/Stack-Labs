# Orquestração de issues

Como a casa implementa a fila de issues `ready-for-agent` (ADRs 0005 e 0016 do método). Uma sessão própria,
aberta com `/orquestrar-issues <N>`, tem um **orquestrador** que dispara os papéis de
`.claude/agents/` numa **sessão do Claude por issue**, de janela limpa, que leva a issue até o fim
por um ciclo fixo, um papel por vez; só então abre a sessão da próxima. Os passos, o
esforço por ciclo e o estado da rodada estão na skill `/orquestrar-issues`; aqui está o porquê de
cada peça.

## Quando recomendar

Ao terminar um `/to-tickets` ou um `/triage` que deixou issues em `ready-for-agent`, recomende a
orquestração ao humano e gere, com o `/nova-sessao`, o kickoff da janela do orquestrador. Num projeto
em adoção, só a partir da etapa `planejamento` ([adocao.md](adocao.md)). O agente pode chamar a skill
sozinho (o kickoff colado não trava); a fila e a branch continuam decisão do humano, no passo 2.

- **Trilhas independentes** (filas sem arquivo em comum): uma sessão de orquestrador por trilha, cada uma
  com a sua branch de integração e o seu estado em `.claude/orquestracao/`.
- **Dentro de uma trilha, nada em paralelo:** uma issue por vez, até o fim. Paralelismo só entre
  trilhas, e só com filas sem arquivo, contrato ou migração em comum.
- Em qualquer caso, **todo comando pesado roda por `sh .claude/watchdogs/rodar pesado "<comando>"`**
  (detalhe na skill): um por vez em cada projeto, e projetos diferentes juntos até as vagas de
  memória da máquina (ADR 0025).

## Os papéis

| Papel | Faz | Edita? |
|---|---|---|
| `implementador` | `/tdd` na branch da issue; num ciclo de correção, só o plano do diagnosticador | Sim |
| `revisor` | Eixo *Standards* do `/code-review`: padrões do projeto, testes, vocabulário | Não |
| `conferente` | Eixo *Spec* do `/code-review`: o feito bate com a issue e o spec? | Não |
| `diagnosticador` | Causa raiz dos achados reprovados e plano de correção | Não |

Revisão e conferência são separadas de propósito: código que segue todos os padrões pode fazer a
coisa errada, e código que faz a coisa certa pode quebrar os padrões. Um eixo não esconde o outro.

Os quatro leem o mesmo **contrato da fatia**, publicado na issue pelo implementador antes do código
([contrato-da-fatia.md](contrato-da-fatia.md)): a primeira escrita já sai dos documentos, e o ciclo
de correção resolve o achado dentro do que o contrato fixa, em vez de deixar o código à deriva.

## O ciclo de uma issue

```
implementador → revisor → conferente
  ├─ dois APROVADO → merge na branch de integração → próxima issue
  └─ algum REPROVADO → diagnosticador → implementador (plano) → revisor → conferente → …
                        (até 2 ciclos de correção; depois, a issue volta para needs-triage)
```

Cada issue roda este ciclo numa sessão própria (o `start_session` do app; sem ele, subagentes como
antes). O orquestrador espera o aviso de fim dessa sessão, sem polling, confere os pareceres e a
branch, faz o merge e só então abre a sessão da próxima.

Duas issues devolvidas seguidas param a rodada: algo comum a elas precisa de um humano.

## Branches

O desenvolvedor escolhe o nome da **branch de integração** e a base. No fim da fila ele decide:
uma PR só da integração para a base, ou uma PR por issue, empilhadas na ordem. O merge na base é
sempre do desenvolvedor.

Cada orquestrador tem a **sua** branch de integração e o **seu** estado
(`.claude/orquestracao/<nome-da-integração>.json`) e não toca nos de outra trilha. A sessão de
cada issue trabalha na sua worktree, criada a partir da integração.

## Esforço

O primeiro ciclo roda barato, menos o implementador, que escreve o contrato da fatia e por isso já
roda em `medium` (ADR 0023); o diagnosticador e os ciclos de correção sobem o esforço. O esforço vai
em cada chamada, para subir sem editar os papéis.

## Watchdogs na orquestração

- Cada subagente aparece para os hooks com o próprio `agent_id`: a regra de uma issue ativa vale por
  subagente implementador, e os demais papéis não editam.
- No aviso da smart zone (450k), o orquestrador grava o estado da rodada e passa a rodada para uma
  janela nova com o `/nova-sessao`. Nas sessões normais o limite duro continua em 550k.
