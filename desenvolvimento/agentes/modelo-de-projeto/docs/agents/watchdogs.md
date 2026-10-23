# Watchdogs

Os watchdogs transformam as regras **[W]** do [AGENTS.md](../../AGENTS.md) em bloqueios, em três
camadas: hooks do Claude Code (o agente, na sessão), o `pre-push` do git (humano e agente, na
máquina) e o `vigia-do-metodo` do GitHub Actions (todos, na PR). O estado de cada sessão fica em
`.claude/state/sessoes/<session_id>.json`, fora do git.

Todos os watchdogs e o validador são subcomandos de um binário Rust, com o fonte em
`.claude/watchdogs/` (ADR 0011 do método). O lançador `.claude/watchdogs/rodar` compila com o `cargo`
na primeira vez e sempre que o fonte muda, e o binário fica em `.claude/watchdogs/target/`, fora do
git. Sem o `cargo` na máquina, o lançador sai com 2 e os watchdogs **falham fechados**: o humano
instala o Rust (https://rustup.rs). Testes: `cargo test --release --manifest-path .claude/watchdogs/Cargo.toml`.

| Watchdog | Onde | O que garante | Ação |
|---|---|---|---|
| `sessao-inicio` | SessionStart | Confere o setup e o pre-push, mostra a branch, a etapa de adoção, a issue ativa, a situação dos **mapas iniciais** e os mapas abertos, e reancora a constituição | Injeta contexto |
| `vigia-prompt` | UserPromptSubmit | Mede o contexto: avisa em 450k e exige `/nova-sessao` a partir de 550k. Em sessão de grilling, relembra a **linguagem simples** a cada resposta do humano | Injeta alerta |
| `vigia-skill` | PostToolUse `Skill` | Quando uma skill de grilling carrega, marca o grilling na sessão e injeta a regra de linguagem simples (regra 10) | Injeta contexto |
| `guarda-edicao` | PreToolUse `Write\|Edit\|NotebookEdit` | Protege os watchdogs, aplica a smart zone e só libera código de produção com issue ativa e fora da `main` (os caminhos livres da regra 3 passam) | **Bloqueia** |
| `guarda-bash` | PreToolUse `Bash` | Protege os watchdogs; ativa issue (uma por sessão ou por subagente, e uma por comando) só com os **cinco mapas iniciais** criados e Protótipo, Arquitetura, Fases e Regras de código fechados, e nunca um `spec`; exige claim antes de comentar ou fechar ticket; exige comentário de resolução **com `## Fontes`** em grilling, tarefa e pesquisa; um ticket de decisão por sessão; barra commit na `main`, push forçado, `--no-verify`, mudança no `core.hooksPath`, `STACKLABS_PERMITIR_PUSH_MAIN` e `--atualizar-watchdogs` | **Bloqueia** |
| `vigia-parada` | Stop | O turno só termina com o ponteiro do ticket resolvido em *Decisions so far* | **Bloqueia o fim do turno** |
| `orquestrar-issues` | Sessão do orquestrador, com os papéis de `.claude/agents/` | Ciclo e limites da [orquestração](orquestracao.md); nunca faz merge na base | **Devolve a issue / para a rodada** |
| `validador` | CI do projeto, implementador e revisor | As regras de `docs/regras-de-codigo.json` e a cobertura dos 22 princípios e das práticas da linguagem ([validador.md](validador.md)); o motor é o mesmo binário dos watchdogs, protegido como eles | **Reprova a fatia / o CI** |
| `pre-push` | `git push` | Recusa push direto para `main`/`master` | **Bloqueia o push** |
| `vigia-do-metodo` | GitHub Actions, em toda PR | Branch do método; PR de `issue/<n>-*` com `Closes #<n>` e issue `ready-for-*`; `research/*` nunca entra na `main`; mudança nos watchdogs só com a label `altera-watchdogs` | **Reprova a PR** |

Num projeto em adoção, a etapa de `.claude/metodo.json` decide quais destes bloqueios valem
([adocao.md](adocao.md)). Sem o arquivo, vale tudo.

## Ativar uma issue

`sh .claude/watchdogs/rodar ativar-issue <n>`. O `guarda-bash` intercepta o comando antes de ele rodar e
confere a issue (regra 3) e os mapas iniciais (regra 1). Uma sessão aceita uma única issue.

## Quando o GitHub não responde

Os watchdogs que consultam o `gh` **falham fechados**: sem rede ou sem auth, bloqueiam. Peça ao
usuário para conferir o `gh auth status` e a rede.

## O que só o humano faz

Ao dar com um destes, entregue o comando ao usuário:

- **Ativar o pre-push**, uma vez por clone: `git config core.hooksPath .claude/githooks`. O
  `sessao-inicio` avisa quando está inativo.
- **Push na `main` de emergência:** `STACKLABS_PERMITIR_PUSH_MAIN=1 git push origin main`.
- **Desligar os watchdogs:** `STACKLABS_WATCHDOGS=desligado claude`. A variável vem do ambiente do
  processo do Claude Code; um comando Bash do agente não a define.
- **Instalar ou atualizar:** `/setup-stacklabs`. Constantes (mapas iniciais, tipos que exigem fontes,
  skills de grilling, limites da smart zone, caminhos livres) ficam em `.claude/watchdogs/src/comum.rs`.
  Mudança que vale para todos os projetos vai primeiro para o `modelo-de-projeto` do repositório Stack-Labs.

## Limites conhecidos

- A proteção contra Bash é heurística: procura verbos mutantes junto dos caminhos protegidos. Barra o
  descuido, não um agente decidido a contornar; por isso a regra 21 também está escrita.
- O `guarda-bash` lê o **texto** do comando (e os arquivos de `--body-file`). Um comando guardado num
  script e executado por ele não é visto. A regra 21 cobre esse caso: contornar um watchdog é
  contrariá-la.
- A regra de fontes confere que a seção e o link existem, não que a fonte sustenta a recomendação.
  Isso é do humano no grilling e do `/auditar-metodo`.
- Grilling com o humano respondendo e TDD não dão para checar por máquina.
