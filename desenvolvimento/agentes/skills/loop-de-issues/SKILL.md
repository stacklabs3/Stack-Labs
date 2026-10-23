---
name: loop-de-issues
description: Prepara uma rodada do loop de issues (N issues ready-for-agent em sequência, uma sessão nova por issue) ou lê o resultado da última.
disable-model-invocation: true
argument-hint: "N issues, ou 'resultado' para ler a última rodada"
---

Conduz o **loop de issues** do projeto: o padrão Ralph do Matt Pocock, com uma sessão nova do Claude
Code por issue, em sequência. Como ele funciona está em `docs/agents/loop-de-issues.md`; leia antes.
O motor é `.claude/loop/loop-de-issues.sh`, e quem o roda é o **humano**, num terminal: você prepara
e lê, nunca dispara.

Dois caminhos: com o argumento `resultado`, vá direto para **Resultado**; senão, **Preparar**.

## Preparar

1. **Pré-requisitos.** Confira e diga ao humano o que falta:
   - working tree limpo e na `main`;
   - `gh auth status` passa;
   - `claude --version` é 2.1.259 ou mais (o script usa `--permission-prompts`);
   - `.claude/loop/loop-de-issues.sh` existe (senão, o projeto precisa do `/setup-stacklabs`).

   Pronto quando os quatro passam.

2. **Fila.** Rode `.claude/loop/loop-de-issues.sh <N> --simular` e mostre as issues que a rodada
   pegaria, **pelo nome**, na ordem. Aponte as que parecem grandes para uma sessão (o loop as devolve
   depois de 4 handoffs) e as sem critério de pronto claro (o loop as devolve como bloqueadas).

   Pronto quando o humano viu a lista.

3. **Combine a rodada.** N (padrão: o argumento recebido) e o orçamento por sessão (padrão 5 USD).

4. **Entregue o comando** para o humano rodar num terminal, fora desta sessão:

   ```
   .claude/loop/loop-de-issues.sh <N> --orcamento <USD>
   ```

   Diga que ele pode fechar esta sessão: o loop roda sozinho, uma issue por vez, e para nas travas
   do guia. Depois, `/loop-de-issues resultado`.

## Resultado

1. Leia `resumo.txt` da pasta mais nova em `.claude/state/loop/`.
2. Para cada `PR`: o link e o estado do CI (`gh pr checks`). Estas esperam a revisão do humano.
3. Para cada `DEVOLVIDA`: o motivo e o que o humano precisa decidir. Motivo de "passou de 4
   sessões" pede fatiar a issue de novo (`/to-tickets`).
4. Se a rodada parou por mudanças sem commit, mostre `git status` e o diff resumido. Não descarte nada.

Pronto quando cada issue da rodada aparece com o próximo passo de quem.
