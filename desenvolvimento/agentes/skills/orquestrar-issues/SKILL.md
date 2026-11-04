---
name: orquestrar-issues
description: Orquestra a fila de issues ready-for-agent numa sessão própria: uma sessão por issue, em sequência, numa branch de integração.
argument-hint: "N issues, ou --retomar <arquivo de estado>"
---

Você é o **orquestrador**: o código é dos subagentes. Você os dispara de `.claude/agents/`, lê os
pareceres, cuida das branches e do estado da rodada. **Uma issue por vez, até o fim:** cada issue
ganha uma sessão própria do Claude, de janela limpa, que segue o ciclo fixo do passo 3; só quando ela
termina (na integração ou devolvida) abre-se a sessão da próxima (seção *Uma sessão por issue*). Passe a
eles **ponteiros** (número da issue, nomes das branches, o plano); o conteúdo eles mesmos leem. O porquê de cada peça está em
`docs/agents/orquestracao.md`.

## Esforço por ciclo

Todos os papéis rodam no modelo `opus`. O esforço vai na chamada de cada subagente:

| Papel | Ciclo 1 | Ciclos de correção |
|---|---|---|
| `implementador` | `medium` | `medium` |
| `revisor` | `low` | `medium` |
| `conferente` | `low` | `medium` |
| `diagnosticador` | `medium`, já no primeiro diagnóstico | `medium` |

## Estado da rodada

`.claude/orquestracao/<nome-da-integração>.json`, regravado depois de **cada** subagente: a branch
de integração e a base dela, e para cada issue da rodada o número, o título, a branch, o ciclo, a
situação (`em-andamento`, `na-integracao`, `devolvida`; estado antigo com `integrada` vale como
`na-integracao`) e o último parecer de cada papel; mais o contador
de issues devolvidas em sequência. É por ele que uma janela nova retoma a rodada.

## Vários orquestradores ao mesmo tempo

Cada trilha (uma fila de issues independente das outras) roda na **sua sessão**, com a **sua branch de integração** e o
**seu arquivo de estado** em `.claude/orquestracao/`. Um orquestrador nunca lê para decidir, grava nem
retoma o estado de outra trilha, nem faz merge na integração dela. As trilhas precisam de filas sem
arquivo, contrato de API ou migração em comum; quem combina isso é o desenvolvedor, no passo 2. A
trava da máquina (abaixo) vale entre todos os orquestradores e qualquer outro agente da máquina.

## A trava da máquina (obrigatória)

Várias sessões na mesma máquina disputam a memória, e sem swap o kernel mata processos, inclusive o
shell do agente (ADR 0016). Por isso todo **comando pesado** — build, dev server, suíte de testes,
e2e (Playwright), build nativo (Flutter, `cargo build`) — roda pelo `rodar pesado`:

```sh
sh .claude/watchdogs/rodar pesado "npm run build"
```

- **Um por vez em cada projeto, e projetos diferentes juntos** até as vagas da máquina (2 em 8 GB).
  Quem espera vê o motivo no terminal e segue sozinho quando a trava solta (ADR 0025).
- No Linux com systemd, o comando roda com o limite de memória da vaga. Saída 137 com o aviso
  `[pesado] o comando foi morto (SIGKILL)` é o limite agindo: reduza os workers do comando
  (`experimental.cpus` do Next, `--maxWorkers=2` do Jest e do Vitest, `--workers=2` do Playwright),
  nunca suba o teto por conta própria.
- `next dev` segura a trava do projeto enquanto estiver de pé: suba pelo `rodar pesado` e derrube-o
  ao terminar. Derrubar o `rodar pesado` derruba todos os processos do comando.
- O `rodar ci` e o pre-push pegam uma vaga, não a trava do projeto: o `git push` vai direto, mesmo
  com o dev server de pé, e não roda o CI de novo para um HEAD que ele já aprovou.
- Sem terminal, o comando não lê o teclado: um prompt de senha (git, ssh) falha na hora. Configure
  as credenciais antes (`gh auth setup-git`, chave ssh sem senha ou agente ssh).
- Leve: typecheck, lint, `git` e `gh` rodam fora dela.
- O prompt de **todo** subagente que roda comandos (implementador, e os outros papéis se rodarem
  testes, e o brief de cada sessão de issue) traz a frase: "todo comando pesado roda por
  `sh .claude/watchdogs/rodar pesado "<comando>"`: um por vez no projeto; sem swap, comandos além
  das vagas matam a máquina".

## Uma sessão por issue

- **Abrir:** para a issue da vez, abra uma sessão própria com o `start_session` do app (contexto
  `fresh`, worktree própria; o brief dá a issue, a branch da issue, a branch de integração e manda
  seguir o ciclo do passo 3, itens 2 a 5, até `na-integracao` ou `devolvida`). Sem `start_session` no
  ambiente, rode o mesmo ciclo com os subagentes de `.claude/agents/`, um por vez, como antes.
- **Workflow fixo dentro da sessão:** implementador → revisor → conferente → (diagnosticador →
  implementador → revisor → conferente, até 2 ciclos de correção). Um papel por vez; nada pula etapa.
- **Uma por vez:** nunca duas sessões de issue abertas pelo mesmo orquestrador. A próxima só abre
  depois que a anterior terminou e o orquestrador conferiu.
- **Acompanhar sem polling:** espere o aviso de fim da sessão. Então confira o resultado de verdade
  (os pareceres, a branch da issue, o commit na integração), grave o estado e siga.
- **O merge** na integração é do orquestrador, um por vez, depois da conferência.
- **Trabalhe onde a branch está.** O git não deixa a mesma branch aberta em duas worktrees: antes
  de `git switch`, rode `git worktree list`. Branch já aberta noutra worktree (a da sessão da issue,
  a de um subagente) se usa lá, com `git -C <caminho da worktree>`; worktree de sessão que já
  terminou se fecha com `git worktree remove <caminho>`. Criar outra branch, copiar commits ou forçar
  o checkout deixa duas versões da mesma issue.
- O brief traz a regra da trava da máquina (seção acima), com a frase pronta.
- O brief e o prompt de **todo** subagente trazem a frase: "antes de abrir arquivos, rode
  `sh .claude/watchdogs/rodar contexto <consulta>` e leia a resposta; vazia ou insuficiente, leia direto".

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
   e a base (padrão: `main`). Com outra trilha rodando, nome e arquivo de estado são outros.

Na mesma rodada, **avise o que só o humano autoriza**: issue da fila que precisa de navegador
(e2e, Playwright, conferência visual) ou de outra permissão da sessão (conector, computador) pede a
autorização dele agora, na sessão, e não quando a issue chegar. Diga quais issues e o que cada uma
vai pedir.

Crie a branch a partir da base, `git push -u origin integracao/<nome>`, e grave o estado.

Pronto quando o desenvolvedor respondeu e autorizou o que a fila vai pedir, a branch de integração está no `origin` e o estado da rodada
está gravado.

### 3. Uma issue por vez, numa sessão própria

**Fronteira:** issues abertas com `ready-for-agent`, sem responsável, sem label `wayfinder:*` nem
`spec`, sem sub-issues (issue com filhas é um spec, mesmo sem a label) e sem bloqueio aberto, da mais
antiga para a mais nova. Um bloqueador já `na-integracao` nesta rodada conta
como resolvido: o código dele já está na branch de integração.

Para a próxima issue da fronteira, na sessão dela (seção *Uma sessão por issue*):

1. Reivindique (`gh issue edit <n> --add-assignee @me`) e crie `issue/<n>-<slug>` a partir da branch
   de integração.
2. **`implementador`**: issue, branch da issue, branch de integração e, num ciclo de correção, o
   plano do diagnosticador. `RESULTADO: BLOQUEADO` → **devolva** a issue.
3. **`revisor`** e depois **`conferente`**, cada um com a issue, a branch dela e a de integração.
4. Os dois com `PARECER: APROVADO` → integre: `git switch integracao/<nome>`,
   `git merge --no-ff issue/<n>-<slug>`, `git push`. Comente na issue e relate ao desenvolvedor
   onde o código está, sempre nestas três linhas:
   ```
   Branch: issue/<n>-<slug> @ <commit>
   Integração: integracao/<nome> @ <commit do merge>
   main: ainda não; chega pela PR do fim da fila
   ```
   Zere o contador de devolvidas e siga para a próxima.
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

Mostre ao desenvolvedor a tabela da rodada (cada issue com a situação, os ciclos, o motivo das
devolvidas e as três linhas de onde o código está) e pergunte como as issues na integração chegam à `main`:

- **(A) Uma PR só:** `integracao/<nome>` → base, com `Closes #<n>` de cada issue na integração.
- **(B) Uma PR por issue:** empilhadas na ordem da rodada (a primeira contra a base, cada seguinte
  contra a branch da anterior), para o merge ser feito na ordem.

Abra as PRs da opção escolhida. O **merge é sempre do desenvolvedor**.

Pronto quando as PRs estão abertas e o desenvolvedor tem a tabela da rodada.
