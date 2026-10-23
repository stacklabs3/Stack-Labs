# Loop de issues: várias issues sem você olhando

Quando há várias issues `ready-for-agent` prontas, o loop implementa até N delas sozinho, **uma de
cada vez**, cada uma numa sessão nova do Claude Code. Você dispara, vai fazer outra coisa e volta para
revisar PRs. É o padrão "Ralph" do Matt Pocock
([ADR 0004](../adr/0004-loop-de-issues-sequencial.md) explica por que é em sequência, e não em paralelo).

## Como usar

1. Numa sessão do Claude Code no projeto: **`/loop-de-issues 5`**. O agente confere os
   pré-requisitos, mostra quais issues entrariam na rodada e aponta as que parecem arriscadas.
2. Ele te entrega um comando. Rode **você mesmo, num terminal**:
   `.claude/loop/loop-de-issues.sh 5 --orcamento 5`
3. Volte depois e peça **`/loop-de-issues resultado`**: as PRs para revisar e as issues devolvidas,
   com o motivo de cada uma.

## O que ele faz, e o que nunca faz

- Pega as issues livres e desbloqueadas, da mais antiga para a mais nova, e se coloca como
  responsável por cada uma antes de começar.
- Cada sessão segue o `AGENTS.md` inteiro, com os watchdogs ativos, e termina numa **PR**.
  **Merge, nunca:** esse é sempre seu.
- Se a sessão enche o contexto (a smart zone), ela salva o trabalho, deixa um resumo e uma sessão
  nova continua a mesma issue. Até 4 sessões por issue; passou disso, a issue era grande demais.
- Se a sessão não consegue terminar, ela **devolve** a issue: comenta o motivo, tira a issue da fila
  (`needs-triage`) e segue para a próxima.

## Quando ele para sozinho

- Chegou ao número de issues que você pediu.
- Não há mais issue livre e desbloqueada.
- **Duas issues seguidas** saíram sem PR: algo comum a elas precisa de você.
- Uma sessão deixou trabalho sem commit: ele para para você conferir e não descarta nada.

## Bons hábitos

- Issue boa para o loop tem critério de pronto claro e cabe numa sessão. Issue vaga volta como bloqueada.
- Uma issue que depende de outra só entra depois do **merge** da outra. Revise as PRs da rodada antes
  de rodar a próxima.
- Cada sessão tem um teto de gasto (`--orcamento`, em dólares). Comece baixo.
