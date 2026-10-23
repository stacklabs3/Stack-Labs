---
name: setup-stacklabs
description: Instala ou atualiza o método StackLabs num projeto — modelo de projeto, watchdogs, skills da casa e as skills mais novas do Matt Pocock.
disable-model-invocation: true
---

Leva um repositório ao **método StackLabs**: o `modelo-de-projeto` (constituição, `docs/agents/`,
watchdogs), as skills da casa e as skills publicadas do Matt Pocock, todas em `.claude/skills/` do
projeto, para que cada pessoa e cada sessão na nuvem tenham as mesmas. Quem copia é o
[`instalar.mjs`](instalar.mjs) desta pasta; você conduz, confere e mescla.

O script é idempotente: rodar de novo num projeto em dia não muda nada, e rodar depois de o Matt ou
o Stack-Labs mudarem é a **atualização**. As skills do Matt vêm sempre do `plugin.json` mais novo de
`mattpocock/skills`; uma skill que saiu da fonte sai do projeto.

## Passos

### 1. Diagnóstico

Confira, na raiz do projeto:

- é um repositório git com `origin` no GitHub;
- `gh auth status` passa (o script cria as labels do método);
- `node --version` é 18 ou mais (os watchdogs rodam em node);
- qual é o **caso**: **instalação** (não há `.claude/skills/ORIGEM.json` nem `AGENTS.md`),
  **projeto existente** (há `AGENTS.md`, `CLAUDE.md` ou `.claude/settings.json` próprios) ou
  **atualização** (há `ORIGEM.json`).

Pronto quando os quatro itens estão respondidos para o humano. Item que falha bloqueia: diga o que
falta e pare.

### 2. Simulação

Rode `node <pasta desta skill>/instalar.mjs --simular` e mostre o relatório ao humano, agrupado por tipo
de linha. Se o clone do Stack-Labs falhar, peça o caminho de um clone local e rode com
`STACKLABS_ORIGEM=<caminho>`.

Pronto quando o humano disse **sim** à lista. Ela inclui criar labels no GitHub.

### 3. Instalação

Rode `node <pasta desta skill>/instalar.mjs` sem `--simular`. Nunca passe `--atualizar-watchdogs`: essa
opção é do humano, e o `guarda-bash` barra.

Pronto quando o script terminou sem erro e imprimiu o relatório.

### 4. Pendências do relatório

Trate cada linha do relatório que não seja `COPIADO`, `SKILL` ou `REMOVIDA`:

- **`MESCLAR`**: siga [MESCLAR.md](MESCLAR.md).
- **`PULADO`** (watchdog ausente ou desatualizado) e **`PRE-PUSH inativo`**: entregue ao humano o
  comando exato para ele rodar, e espere:
  `node <pasta desta skill>/instalar.mjs --atualizar-watchdogs`.

Pronto quando `find . -name '*.stacklabs-novo' -not -path './.git/*'` não lista nada, ou lista só
arquivos de watchdog entregues ao humano.

### 5. Verificação

- `node --check` passa em cada `.claude/hooks/**/*.mjs`;
- `node --check .claude/githooks/pre-push` passa;
- `git config core.hooksPath` responde `.claude/githooks`;
- toda pasta em `.claude/skills/` tem `SKILL.md`, e `ORIGEM.json` lista as skills do Matt e as da casa.

Pronto quando os quatro passam. Falha é defeito da instalação: corrija antes de seguir.

### 6. Commit e PR

Numa branch `chore/setup-metodo` (ou `chore/atualizar-metodo`), um commit com tudo o que o script e
a mesclagem mudaram, push e PR. O corpo da PR traz os commits de origem de `ORIGEM.json` e o que foi
mesclado à mão. Num repositório sem `main`, diga ao humano que a PR depende de ele criar a `main`.

Pronto quando a PR existe, ou o humano sabe exatamente o que a impede.

### 7. Próximo passo

O humano precisa de uma **janela nova**: os hooks só carregam no início de uma sessão. Gere o
kickoff com o `/nova-sessao` para ele colar lá. O `sessao-inicio` vai mostrar o setup ✅ e os mapas
iniciais ❌. Na **instalação**, o próximo
trabalho é abrir os quatro mapas iniciais (`docs/agents/mapas-iniciais.md`).
