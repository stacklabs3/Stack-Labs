---
name: setup-stacklabs
description: Instala o método StackLabs num projeto novo, ou atualiza um projeto que já o segue.
disable-model-invocation: true
---

Leva ao projeto o `modelo-de-projeto` (constituição, `docs/agents/`, watchdogs), as skills da casa e
as skills publicadas do Matt Pocock, todas em `.claude/skills/`, para que cada pessoa e cada sessão
na nuvem tenham as mesmas. Quem copia é o instalador em Rust desta pasta (`src/`), que o
[`rodar`](rodar) compila na primeira vez; você conduz, confere e mescla.

O script é idempotente: num projeto em dia não muda nada, e depois de o Matt ou o Stack-Labs mudarem
ele faz a **atualização**. As skills do Matt vêm sempre do `plugin.json` mais novo de
`mattpocock/skills`; skill que saiu da fonte sai do projeto.

## Passos

### 1. Diagnóstico

Confira, na raiz do projeto:

- é um repositório git com `origin` no GitHub;
- `gh auth status` passa (o script cria as labels do método);
- `cargo --version` responde (o instalador e os watchdogs são compilados na máquina pelo cargo);
- qual é o **caso**: **instalação** (repositório sem código nem histórico além do inicial) ou
  **atualização** (há `.claude/skills/ORIGEM.json`).

Projeto com código em andamento que não segue o método é **adoção**: pare e indique o
`/adotar-metodo-stack-labs`.

Pronto quando os quatro itens estão respondidos para o humano. Item que falha bloqueia: diga o que
falta e pare.

### 2. Simulação

Rode `sh <pasta desta skill>/rodar --simular` e mostre o relatório ao humano, agrupado por
tipo de linha. Se o clone do Stack-Labs falhar, peça o caminho de um clone local e rode com
`STACKLABS_ORIGEM=<caminho>`.

Pronto quando o humano disse **sim** à lista. Ela inclui criar labels no GitHub.

### 3. Instalação

Rode `sh <pasta desta skill>/rodar` sem `--simular`. O
`--atualizar-watchdogs` é do humano (o `guarda-bash` barra): ele entra no passo 4.

Pronto quando o script terminou sem erro e imprimiu o relatório.

### 4. Pendências do relatório

Trate as linhas `MESCLAR`, `PULADO`, `PRE-PUSH inativo` e `FALHA`; as demais são informativas.

- **`MESCLAR`**: o modelo novo está em `<arquivo>.stacklabs-novo`; junte e apague o `.stacklabs-novo`.
  No `.gitignore`, acrescente as linhas que faltam. No `AGENTS.md` e no `CLAUDE.md`, o modelo novo é
  a base, e a seção `## Regras deste projeto` do arquivo antigo volta inteira no fim. No
  `.claude/settings.json`, que é protegido, entregue o JSON junto ao humano.
- **`PULADO`** (watchdog ausente ou desatualizado) e **`PRE-PUSH inativo`**: entregue ao humano o
  comando exato e espere que ele rode:
  `sh <pasta desta skill>/rodar --atualizar-watchdogs`. Com essa opção, os watchdogs antigos em
  node (`.claude/hooks/` e `.claude/validador/`) saem do projeto (linhas `REMOVIDA`).
- **`FALHA`** (os watchdogs não compilaram): rode `sh .claude/watchdogs/rodar --version` e
  mostre o erro do cargo ao humano: o passo 5 só passa com os watchdogs compilados.

Pronto quando `find . -name '*.stacklabs-novo' -not -path './.git/*'` não lista nada, ou lista só
arquivos de watchdog entregues ao humano.

### 5. Verificação

- `sh .claude/watchdogs/rodar --version` termina com código 0 (compila os watchdogs, se preciso);
- `sh -n .claude/githooks/pre-push` passa;
- `.claude/hooks/` e `.claude/validador/` não existem;
- `git config core.hooksPath` responde `.claude/githooks`;
- toda pasta em `.claude/skills/` tem `SKILL.md`, e `ORIGEM.json` lista as skills do Matt e as da casa;
- com `docs/regras-de-codigo.json`, `sh .claude/watchdogs/rodar validar` termina em
  `VALIDADOR: APROVADO`, ou cada violação virou ajuste no `.json` ou ticket. Depois da ADR 0012, o
  `lint-cobre` reprova regra desligada, comentada ou ligada só por preset, que antes passava.

Pronto quando os seis passam. Falha é defeito da instalação: corrija antes de seguir.

### 6. Commit e PR

Numa branch `chore/setup-metodo` (ou `chore/atualizar-metodo`), um commit com tudo o que o script e
a mesclagem mudaram, push e PR. O corpo da PR traz os commits de origem de `ORIGEM.json` e o que foi
mesclado à mão. Num repositório sem `main`, diga ao humano que a PR depende de ele criar a `main`.

Pronto quando a PR existe, ou o humano sabe exatamente o que a impede.

### 7. Próximo passo

Os hooks só carregam no início de uma sessão: gere com o `/nova-sessao` o kickoff da **janela nova**
para o humano colar lá. O próximo trabalho depende do caso:

- **Instalação:** o `sessao-inicio` mostra o setup ✅ e os mapas iniciais ❌; o próximo trabalho é
  abrir os cinco mapas iniciais (`docs/agents/mapas-iniciais.md`).
- **Atualização:** se o `sessao-inicio` mostrar ❌ num mapa inicial que o projeto ainda não tinha (o
  de Regras de código, em projeto anterior à ADR 0007), avise o humano que nenhuma issue será ativada
  até esse mapa existir e fechar. O próximo trabalho é abri-lo como mapa retroativo.

Pronto quando o humano tem o kickoff e sabe qual é o próximo trabalho.
