# Changelog do método

Cada versão do método StackLabs, a mais nova primeiro. A versão vive em [VERSAO](VERSAO), segue
[SemVer](https://semver.org/lang/pt-BR/) e cada uma ganha a tag `v<versão>` no Stack-Labs
([ADR 0020](adr/0020-metodo-versionado-e-regras-por-projeto.md)).

- **Maior:** muda o que o projeto precisa fazer à mão na atualização (regra dura nova que reprova
  código existente, arquivo que muda de lugar, watchdog que passa a bloquear algo novo).
- **Menor:** regra, skill ou watchdog novo que o instalador aplica sozinho.
- **Correção:** texto, defeito de watchdog ou de skill, sem regra nova.

Toda PR que muda `desenvolvimento/agentes/` sobe a versão e acrescenta a entrada aqui.

## 4.2.0 (2026-11-04)

- **Dois projetos rodam juntos na mesma máquina.** A trava global `~/.stacklabs/pesado.lock` deu
  lugar ao subcomando `sh .claude/watchdogs/rodar pesado "<comando>"`: um comando pesado por vez em
  cada projeto, e projetos diferentes juntos até as vagas da máquina (2 em 8 GB, 5 em 16 GB)
  ([ADR 0025](adr/0025-trava-por-projeto-e-vagas-de-memoria.md)). Os watchdogs passam a pedir Rust 1.89.
- No Linux com systemd de usuário, cada comando pesado roda com o limite de memória da vaga
  (cgroup): quem passa dele morre sozinho, não a máquina nem o shell do agente.
- Derrubar o `rodar pesado` derruba todos os processos do comando, e o que ele deixar para trás é
  encerrado com a trava: nenhum dev server órfão. Sem terminal (o agente), o comando não lê o
  teclado: um prompt de senha do git ou do ssh falha na hora em vez de travar; configure as
  credenciais antes.
- O heap do Node deixa de ser forçado (era 4096 MB); o pico se controla pelos workers de cada
  projeto. `STACKLABS_HEAP_MB` liga o teto quando a máquina precisa.
- **Push mais rápido:** o pre-push não roda a revisão por modelo (ela fica no `rodar ci`), não roda
  o CI de novo para um HEAD já aprovado contra a mesma base e não espera um dev server de pé no
  projeto (o CI pega uma vaga, não a trava do projeto).
- A revisão por modelo abre 1 `claude -p` por vez (eram 4, ~220 MB cada).
- O binário dos watchdogs fica guardado por versão em `~/.stacklabs/watchdogs/`: o segundo projeto
  copia em vez de compilar.
- A aprovação guardada do CI leva o hash do binário dos watchdogs: depois de atualizá-los, o push
  roda o CI de novo.
- **Instalador:** ao atualizar os watchdogs, apaga a compilação anterior do projeto
  (`.claude/watchdogs/target/`). Antes, com um clone local (`STACKLABS_ORIGEM`), a cópia guardava
  datas antigas, o cargo religava o artefato velho e o relatório dizia `COMPILADO` com o binário da
  versão anterior rodando.
- Guia humano: *Preparar a máquina para dois projetos* (swap, limite e workers) em
  `humanos/orquestracao.md`. Glossário: **comando pesado** e **vaga**. `docs/agents/watchdogs.md`:
  seção *Rodar um comando pesado* e os limites dela.
- **Base do CI:** o `pre-push`, o `rodar ci` sem `--base` e a regra `mensagem-de-commit` comparam
  com a branch padrão do remoto (`origin/HEAD`), não mais com `origin/main` fixo: projeto em que a
  `dev` faz o papel da `main` deixa de remendar o `pre_push.rs`. Sem `origin/HEAD`, continua a `main`;
  para gravá-lo, `git remote set-head origin -a`.

## 4.1.0 (2026-11-04)

- **Antes da primeira atualização, uma vez por máquina:** reinstale as skills com o comando do
  `AGENTS.md` do Stack-Labs. Depois disso, a skill se atualiza sozinha.
- O `/setup-stacklabs` atualiza sem intervenção: a pessoa só dispara a skill
  ([ADR 0024](adr/0024-atualizacao-do-metodo-sem-humano.md)). O `--atualizar-watchdogs` saiu, e o
  instalador atualiza os watchdogs por padrão, depois de compilar a fonte à parte.
- O instalador mescla `.gitignore` e `.claude/settings.json` sozinho e tira do projeto o arquivo de
  watchdog que saiu da fonte.
- Comandos novos: `--conferir-skill` (roda a cada `/setup-stacklabs` e `/adotar-metodo-stack-labs`),
  `--verificar` e `--concluir-pr <número>`.
- O `guarda-bash` barra `STACKLABS_ORIGEM`, `MATT_ORIGEM`, `--permitir-regressao` e a label
  `altera-watchdogs` aplicada direto pelo agente.

## 4.0.0 (2026-11-03)

- **Ao atualizar os watchdogs:** o `guarda-edicao` só libera código quando a issue ativa tem o
  comentário `## Contrato da fatia`: o que o diff toca e o vizinho que imita, as seams com os riscos,
  as regras de código que o alcançam, os termos, as ADRs e os critérios de aceite. Regra 24 do
  `AGENTS.md`, `docs/agents/contrato-da-fatia.md`, [ADR 0023](adr/0023-contrato-da-fatia-antes-do-codigo.md).
  Rode o `--atualizar-watchdogs`.
- Hook novo `vigia-codigo`: a cada edição de código, devolve na hora a regra de
  `docs/regras-de-codigo.json` que o arquivo quebra (`ast-grep`, `termo-evitado`, `todo-sem-issue`),
  com a regra de onde nasceu, em vez de esperar o validador no commit ou o `ci` no push.
- Os papéis da orquestração leem o contrato: o implementador o publica e implementa contra ele, o
  revisor e o conferente conferem o diff com ele, e o diagnosticador escreve o plano com a linha do
  contrato que cada passo preserva (passo `contrato:` quando precisa mudar uma).
- O implementador do primeiro ciclo roda em `medium`, porque escreve o contrato.

## 3.0.0 (2026-11-01)

- **Ao atualizar os watchdogs:** o `guarda-contexto` novo e o `guarda-bash` barram a leitura de código
  (`Read`, `Grep`, `Glob`, `cat`, `sed`, `grep`) enquanto a sessão ou o subagente não rodar
  `sh .claude/watchdogs/rodar contexto <consulta>`. Resposta vazia libera. Regra 23 do `AGENTS.md`,
  [ADR 0022](adr/0022-consulta-de-contexto-obrigatoria.md). Rode o `--atualizar-watchdogs`.
- O `rodar contexto` acha arquivo e pasta pelo nome, e enxerga `.claude/skills/` mesmo ignorada pelo git.

## 2.1.2 (2026-11-01)

- O `lint-cobre` mostra uma vez por execução o `analysis_options.yaml` por pasta, em vez de repeti-lo
  em cada regra.

## 2.1.1 (2026-11-01)

- Quem configura e quem audita acham o Dart: requisito e versão do preset no `dart.md`, Flutter e
  subpasta no `validador.md`, DT no mapa de Regras de código, e a `/auditar-metodo` vale para toda
  linguagem com arquivo de princípios.

## 2.1.0 (2026-11-01)

- O modelo Dart fica igual ao guia: caixa do E1, `avoid_print`, `implementation_imports` e as outras
  regras que o guia cita, DT5 e DT6 como `revisor`, `**/*.mocks.dart` fora do validador e do
  analisador, e a troca para `flutter analyze --fatal-infos` registrada.
- Uma receita só do OSV-Scanner, conferida na documentação da v2.7.0:
  `osv-scanner scan source --lockfile pubspec.lock`.
## 2.0.1 (2026-11-01)

- A revisão por modelo mostra o nome certo de `testWidgets`, `goldenTest` e `patrolTest`, também com a descrição na linha de
  baixo.
## 2.0.0 (2026-11-01)

- **Ao atualizar os watchdogs:** o `lint-cobre` passa a reprovar `analysis_options.yaml` versionado
  dentro da pasta de um `config` declarado e fora da lista de `config`, porque esse arquivo cala o
  lint da pasta. Um arquivo por pasta que passava pode reprovar: apague-o ou declare-o em `config`.
  Arquivo ignorado pelo git não dispara ([ADR 0021](adr/0021-dart-no-ci-local.md)).

## 1.0.0 (2026-10-25)

Primeira versão marcada: o método como estava no commit `c7c1fa1`, mais:

- `desenvolvimento/VERSAO` e este changelog; o instalador grava a versão em
  `.claude/skills/ORIGEM.json`, mostra a mudança de versão no relatório (`VERSÃO`) e instala uma
  versão marcada com `--versao <x.y.z>`.
- O `AGENTS.md` do modelo traz a seção `## Regras deste projeto`, com `### Acrescentadas` e
  `### Afastadas`. Na atualização, o instalador troca o resto do arquivo pelo modelo novo e mantém a
  seção (`MESCLADO`), sem mesclagem à mão.
- Instalação: label que o `gh` não cria (sem login ou sem `origin`) vira `FALHA` no relatório em vez
  de abortar no fim com o projeto já copiado; fora de um repositório git, o erro diz o que fazer; o
  `cp` das skills de entrada não depende de `{a,b}` do shell, e a pasta `.claude/skills` é criada.
- Watchdogs configuráveis por projeto: campo `ajustes` de `.claude/metodo.json` com
  `caminhosLivres`, `branchesDescartaveis` e `contexto` (limites da smart zone).
- O agente não abre mais PR no Stack-Labs com regra específica de um projeto: a regra fica no projeto
  (`AGENTS.md`, `metodo.json` ou `regras-de-codigo.json`); levar ao método só a pedido do humano.
