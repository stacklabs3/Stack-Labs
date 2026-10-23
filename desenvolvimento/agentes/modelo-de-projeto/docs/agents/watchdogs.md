# Watchdogs

Os watchdogs transformam as regras **[W]** do [AGENTS.md](../../AGENTS.md) em bloqueios. São três
camadas: hooks do Claude Code (o agente, na sessão), o `pre-push` do git (humano e agente, na
máquina) e o `vigia-do-metodo` do GitHub Actions (todos, na PR). O estado de cada sessão fica em
`.claude/state/sessoes/<session_id>.json`, fora do git.

| Watchdog | Onde | O que garante | Ação |
|---|---|---|---|
| `sessao-inicio` | SessionStart | Confere o setup e o pre-push, mostra a branch, a issue ativa, a situação dos **mapas iniciais** e os mapas abertos, e reancora a constituição | Injeta contexto |
| `vigia-prompt` | UserPromptSubmit | Mede o contexto: avisa em 450k e exige `/handoff` a partir de 550k (constantes em `.claude/hooks/lib/comum.mjs`) | Injeta alerta |
| `guarda-edicao` | PreToolUse `Write\|Edit\|NotebookEdit` | Protege os watchdogs, aplica a smart zone e só libera código de produção com issue ativa, fora da `main` | **Bloqueia** |
| `guarda-bash` | PreToolUse `Bash` | Protege os watchdogs; ativa issue (uma por sessão) só com os **quatro mapas iniciais** criados e Protótipo, Arquitetura e Fases fechados; exige claim antes de comentar ou fechar ticket; exige comentário de resolução **com `## Fontes`** em grilling, tarefa e pesquisa; um ticket de decisão por sessão; nada de commit na `main`, push forçado nem `--no-verify` | **Bloqueia** |
| `vigia-parada` | Stop | O turno não termina sem o ponteiro do ticket resolvido em *Decisions so far* | **Bloqueia o fim do turno** |
| `pre-push` | `git push` | Recusa push direto para `main`/`master` | **Bloqueia o push** |
| `vigia-do-metodo` | GitHub Actions, em toda PR | Branch do método; PR de `issue/<n>-*` com `Closes #<n>` e issue `ready-for-*`; `research/*` nunca entra na `main`; mudança nos watchdogs só com a label `altera-watchdogs` | **Reprova a PR** |

## Ativar uma issue

```bash
.claude/hooks/ativar-issue.sh 42
```

O `guarda-bash` intercepta o comando antes de ele rodar e confere a issue (aberta, `ready-for-agent`
ou `ready-for-human`, sem `wayfinder:*`) e os mapas iniciais. Uma sessão aceita uma única issue.

## Ativar o pre-push (uma vez por clone)

```bash
git config core.hooksPath .claude/githooks
```

O `sessao-inicio` avisa quando ele está inativo. Escape de emergência, **só humano**:
`STACKLABS_PERMITIR_PUSH_MAIN=1 git push origin main`.

## Quando o GitHub não responde

Os watchdogs que consultam o `gh` **falham fechados**: sem rede ou sem auth, bloqueiam.

## Desligar (só humano)

`STACKLABS_WATCHDOGS=desligado claude`. A variável vem do ambiente do processo do Claude Code; um
comando Bash do agente não consegue defini-la.

## Ajustar ao projeto

As constantes ficam em `.claude/hooks/lib/comum.mjs`: `MAPAS_INICIAIS`,
`MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR`, `TIPOS_QUE_EXIGEM_FONTES`, os limites da smart zone e os
caminhos livres sem issue. Mudança que vale para todos os projetos vai primeiro para o
`modelo-de-projeto` no repositório Stack-Labs.

## Limites conhecidos

- A proteção contra Bash é heurística: procura verbos mutantes junto dos caminhos protegidos. Barra o
  descuido, não um agente decidido a contornar; por isso a regra 20 também está escrita.
- O `guarda-bash` lê o **texto** do comando (e os arquivos de `--body-file`). Um comando guardado num
  script e executado por ele não é visto. A regra 21 cobre esse caso.
- A regra de fontes confere que a seção e o link existem, não que a fonte sustenta a recomendação.
  Isso é do humano no grilling e do `/auditar-metodo`.
- Grilling com o humano respondendo e TDD não dão para checar por máquina.
