---
name: orquestrar-issues
description: Orquestra a fila de issues ready-for-agent numa sessão própria, com subagentes em sequência numa branch de integração.
disable-model-invocation: true
argument-hint: "N issues, ou --retomar <arquivo de estado>"
---

Você é o **orquestrador**: o código é dos subagentes. Você os dispara de `.claude/agents/`, lê os
pareceres, cuida das branches e do estado da rodada. Um subagente por vez, sempre em primeiro plano
(espere cada um terminar), cada um com a janela limpa. Passe a eles **ponteiros** (número da issue, nomes das
branches, o plano); o conteúdo eles mesmos leem. O porquê de cada peça está em
`docs/agents/orquestracao.md`.

## Esforço por ciclo

O modelo e o esforço vão na chamada de cada subagente (ADR 0016). O `implementador` e o `conferente`
têm `model: sonnet` no arquivo; o `revisor` e o `diagnosticador`, `model: opus`. O `implementador` só
fica em Sonnet na 1ª implementação: **nos ciclos de correção, chame-o com `model: "opus"`**.

| Papel | Ciclo 1 | Ciclos de correção |
|---|---|---|
| `implementador` | `sonnet`, `low` | `opus`, `medium` |
| `revisor` | `opus`, `low` | `opus`, `medium` |
| `conferente` | `sonnet`, `low` | `sonnet`, `medium` |
| `diagnosticador` | `opus`, `medium`, já no primeiro diagnóstico | `opus`, `medium` |

## Estado da rodada

`.claude/orquestracao/<nome-da-integração>.json`, regravado depois de **cada** subagente: a branch
de integração e a base dela, e para cada issue da rodada o número, o título, a branch, o ciclo, a
situação (`em-andamento`, `integrada`, `devolvida`) e o último parecer de cada papel; mais o contador
de issues devolvidas em sequência. É por ele que uma janela nova retoma a rodada.

## Passos

### 1. Pré-voo

- Com `--retomar <arquivo>`: leia o estado, confira que as branches existem e vá para o passo 3,
  continuando da issue `em-andamento` (do começo do ciclo dela).
- Sem ele: working tree limpo, `gh auth status` passa, e os quatro papéis existem em
  `.claude/agents/` (senão, o projeto precisa do `/setup-stacklabs`, ou do
  `/adotar-metodo-stack-labs` se já existia antes do método).
- Projeto em adoção na etapa `observacao` (`.claude/metodo.json`): **pare**. As issues antigas ainda
  não passaram pelo triage; a orquestração começa na etapa `planejamento`.

Pronto quando tudo passa, ou o humano sabe o que falta.

### 2. Combinar a rodada com o desenvolvedor

Pergunte numa rodada só, com recomendação em cada pergunta, e **espere as respostas**:

1. **A fila:** as issues da fronteira (regra no passo 3), pelo nome, na ordem, até N.
2. **A branch de integração:** o nome (`integracao/<nome>`, sugira um a partir do spec ou do mapa)
   e a base (padrão: `main`).

Crie a branch a partir da base, `git push -u origin integracao/<nome>`, e grave o estado.

Pronto quando o desenvolvedor respondeu, a branch de integração está no `origin` e o estado da rodada
está gravado.

### 3. Uma issue por vez

**Fronteira:** issues abertas com `ready-for-agent`, sem responsável, sem label `wayfinder:*` nem
`spec`, sem sub-issues (issue com filhas é um spec, mesmo sem a label) e sem bloqueio aberto, da mais
antiga para a mais nova. Um bloqueador já `integrada` nesta rodada conta
como resolvido: o código dele já está na branch de integração.

Para a próxima issue da fronteira:

1. Reivindique (`gh issue edit <n> --add-assignee @me`) e crie `issue/<n>-<slug>` a partir da branch
   de integração.
2. **`implementador`**: issue, branch da issue, branch de integração e, num ciclo de correção, o
   plano do diagnosticador. `RESULTADO: BLOQUEADO` → **devolva** a issue.
3. **`revisor`** e depois **`conferente`**, cada um com a issue, a branch dela e a de integração.
4. Os dois com `PARECER: APROVADO` → integre: `git switch integracao/<nome>`,
   `git merge --no-ff issue/<n>-<slug>`, `git push`. Comente na issue que ela foi integrada (branch e
   commit). Zere o contador de devolvidas e siga para a próxima.
5. Algum `PARECER: REPROVADO` → se esta issue ainda não teve **2 ciclos de correção**: chame o
   **`diagnosticador`** com os pareceres, some um ao ciclo e volte ao item 2 com o plano dele. Se já
   teve, **devolva** a issue.

**Devolver:** comente na issue o motivo e os últimos pareceres, tire o responsável, troque
`ready-for-agent` por `needs-triage` e deixe a branch da issue como está, para um humano olhar.
Some um ao contador; **duas devolvidas seguidas** param a rodada.

Pronto quando: a fila acabou, a fronteira esvaziou, ou a rodada parou por duas devolvidas seguidas.

### 4. Smart zone

O aviso de smart zone do watchdog é o seu limite. Ao recebê-lo, deixe o subagente em curso
terminar, grave o estado e rode o `/nova-sessao` em vez de começar outra issue. O kickoff abre uma
janela nova com `/orquestrar-issues --retomar .claude/orquestracao/<nome>.json`.

Pronto quando o estado está gravado e o desenvolvedor tem o kickoff.

### 5. Fim da fila

Mostre ao desenvolvedor a tabela da rodada (cada issue com a situação, os ciclos e o motivo das
devolvidas) e pergunte como as integradas chegam à `main`:

- **(A) Uma PR só:** `integracao/<nome>` → base, com `Closes #<n>` de cada integrada.
- **(B) Uma PR por issue:** empilhadas na ordem da rodada (a primeira contra a base, cada seguinte
  contra a branch da anterior), para o merge ser feito na ordem.

Abra as PRs da opção escolhida. O **merge é sempre do desenvolvedor**.

Pronto quando as PRs estão abertas e o desenvolvedor tem a tabela da rodada.
