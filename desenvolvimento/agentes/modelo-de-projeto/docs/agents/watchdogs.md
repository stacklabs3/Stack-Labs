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
| `vigia-prompt` | UserPromptSubmit | Mede o contexto: avisa em 450k e exige `/nova-sessao` a partir de 550k. Em sessão de grilling, relembra a **linguagem simples** a cada resposta do humano. Com a issue ativa acima do orçamento, relembra o custo | Injeta alerta |
| `vigia-skill` | PostToolUse `Skill` | Quando uma skill de grilling carrega, marca o grilling na sessão e injeta a regra de linguagem simples (regra 10) | Injeta contexto |
| `guarda-edicao` | PreToolUse `Write\|Edit\|NotebookEdit` | Protege os watchdogs, aplica a smart zone e só libera código de produção com issue ativa, fora da `main` e depois do **contrato da fatia** publicado na issue (regra 24, [contrato-da-fatia.md](contrato-da-fatia.md)); os caminhos livres da regra 3 passam. O contrato é procurado uma vez por sessão e por subagente e fica no estado | **Bloqueia** |
| `guarda-bash` | PreToolUse `Bash` | Protege os watchdogs pelo **alvo da escrita** (o destino de `>`, `>>`, `tee`, `cp`, `mv`, `rm`, `sed -i`, `find -delete`, `sh -c` e da gravação por `python` ou `node`): ler um protegido, gravar no `$TMPDIR` ou citá-lo no texto de um heredoc passa (ADR 0027); ativa issue (uma por sessão ou por subagente, e uma por comando) só com os **cinco mapas iniciais** criados e Protótipo, Arquitetura, Fases e Regras de código fechados, e nunca um `spec`; exige claim antes de comentar ou fechar ticket; exige comentário de resolução **com `## Fontes`** em grilling, tarefa e pesquisa; um ticket de decisão por sessão; barra commit na `main`, push forçado, `--no-verify`, mudança no `core.hooksPath`, `STACKLABS_PERMITIR_PUSH_MAIN`, fonte local do instalador (`STACKLABS_ORIGEM`, `MATT_ORIGEM`), `--permitir-regressao` e a label `altera-watchdogs` aplicada direto | **Bloqueia** |
| `guarda-contexto` | PreToolUse `Read\|Grep\|Glob`, e o `guarda-bash` para `cat`, `head`, `tail`, `less`, `sed`, `grep` e `rg` | Código só se lê depois de uma consulta `rodar contexto` da sessão ou do subagente; a primeira leitura fora dos citados recebe aviso. Passam livres os caminhos da regra 3, os de `caminhosLivres`, documento e configuração por extensão (`.md`, `.json`, `.yaml`, `.toml`...), `*.config.*` e arquivo oculto | **Bloqueia** |
| `vigia-teste` | PostToolUse `Write\|Edit\|MultiEdit\|Bash` | No teste que acabou de ser editado (também por `sed -i`, heredoc ou `python`, pelo alvo da escrita), roda as regras do P5 de `docs/regras-de-codigo.json`: teste sem `RK<n>` ou com risco fora do registro, sem afirmação, com afirmação trivial, desligado ou focado ([riscos.md](riscos.md)). Projeto sem `docs/regras-de-codigo.json` passa; na etapa de observação, só avisa | **Devolve ao agente para corrigir** |
| `vigia-codigo` | PostToolUse `Write\|Edit\|MultiEdit\|Bash` | No código que acabou de ser editado (também pelo Bash, como o `vigia-teste`), roda as regras de `docs/regras-de-codigo.json` fora do P5 que têm forma fixa (`ast-grep`, `termo-evitado`, `todo-sem-issue`) e devolve cada violação com a regra de onde nasceu; `comando` (lint, typecheck) fica para o `ci`. Projeto sem `docs/regras-de-codigo.json` passa; na etapa de observação, só avisa | **Devolve ao agente para corrigir** |
| `vigia-saida` | PostToolUse `Bash\|Read\|Grep\|Glob\|WebFetch` | Saída de ferramenta acima de 20k caracteres (uns 5k tokens) pede filtro da próxima vez: trecho com `offset`/`limit`, `grep`/`tail`, padrão mais estreito ou subagente | Injeta alerta |
| `vigia-agente` | PreToolUse `Agent\|Task` | Subagente de busca (`Explore`, `claude-code-guide`) lançado sem `model` recebe o lembrete de usar haiku ou sonnet | Injeta alerta |
| `vigia-custo` | Stop e SubagentStop | Soma o uso de tokens de cada turno, da sessão e dos subagentes, na issue ativa, em `.claude/state/custos/issue-<n>.json`. Ao cruzar o **orçamento por issue**, avisa o humano; dali em diante o `vigia-prompt` relembra o agente a cada prompt. Relatório: `sh .claude/watchdogs/rodar custo [<n>]`, que vai na Evidence da PR | Avisa o humano |
| `vigia-parada` | Stop | O turno só termina com o ponteiro do ticket resolvido em *Decisions so far* | **Bloqueia o fim do turno** |
| `orquestrar-issues` | Sessão do orquestrador, com os papéis de `.claude/agents/` | Ciclo e limites da [orquestração](orquestracao.md); nunca faz merge na base | **Devolve a issue / para a rodada** |
| `validador` | CI local (`ci`), CI do projeto, implementador e revisor | As regras de `docs/regras-de-codigo.json` e a cobertura dos 22 princípios e das práticas da linguagem e o registro de riscos contra os testes ([validador.md](validador.md)); o motor é o mesmo binário dos watchdogs, protegido como eles | **Reprova a fatia / o CI** |
| `pre-push` | `git push` | Recusa push direto para `main`/`master`; depois roda o `ci` contra a branch padrão do remoto (`<remoto>/HEAD`; sem ela, `<remoto>/main`): lint e typecheck (regras `comando`) e o validador, sem a revisão por modelo, e só se o mesmo conteúdo ainda não passou ([validador.md](validador.md#onde-roda)). Na etapa de observação, o `ci` vermelho só avisa | **Bloqueia o push** |
| `vigia-do-metodo` | GitHub Actions, em toda PR | Branch do método; PR de `issue/<n>-*` com `Closes #<n>` e issue `ready-for-*`; `research/*` nunca entra na `main`; mudança nos watchdogs só com a label `altera-watchdogs`, de um humano ou do `--concluir-pr` do `/setup-stacklabs` | **Reprova a PR** |

Num projeto em adoção, a etapa de `.claude/metodo.json` decide quais destes bloqueios valem
([adocao.md](adocao.md)). Sem o arquivo, vale tudo.

## Consultar o contexto

`sh .claude/watchdogs/rodar contexto <símbolo | termo | caminho | #issue>`, **um nome por consulta**, liga o nome ao que o
projeto já escreveu sobre ela e devolve cada ligação com `caminho:linha`: onde o símbolo é definido,
os termos do `GLOSSARY.md`, as ADRs que o citam, os riscos `RK<n>` dos testes que o tocam, quem o
chama, os arquivos que mudam junto e as issues do histórico. Uma `#issue` traz os arquivos dos
commits que a citam em qualquer branch (cada merge contra o primeiro pai) e os da branch
`issue/<n>-*` que ainda não entrou. Uma frase vira uma consulta por nome, com as respostas juntas: os
identificadores dela (`snake_case`, `camelCase`, caminho, `#issue`) e, sem nenhum, as palavras que
podem ser termo do glossário, até 6. Cada nome custa uma consulta inteira; o nome solto é mais rápido
e mais certo. Tudo sai do repositório e do git a cada chamada, sem modelo e sem índice
guardado, então não envelhece (ADR 0018). Uma consulta cujo texto é o nome de arquivo ou de pasta
traz esses arquivos, inclusive em `.claude/skills/`. Leia a resposta antes de abrir arquivos e abra só
os trechos citados; `--limite <n>` muda as 5 ligações por seção. A regra 23 torna a consulta
obrigatória (ADR 0022): resposta vazia ou insuficiente libera a leitura direta.

## Ativar uma issue

`sh .claude/watchdogs/rodar ativar-issue <n>`. O `guarda-bash` intercepta o comando antes de ele rodar e
confere a issue (regra 3) e os mapas iniciais (regra 1). Uma sessão aceita uma única issue.

## Rodar um comando pesado

`sh .claude/watchdogs/rodar pesado "<comando>"` para build, dev server, suíte de testes, e2e e build
nativo; typecheck, lint, `git` e `gh` rodam direto. Um comando pesado por vez em cada projeto, e
projetos diferentes juntos até as vagas da máquina (ADR 0025). Quem espera vê o motivo e,
na trava do projeto, quem a segura (`[pesado] <projeto> já roda um comando pesado (pid <n>: <comando>)`)
e segue sozinho. Saída 137 com `[pesado] o comando foi morto (SIGKILL)` é o
limite de memória da vaga: reduza os workers do comando, nunca suba o teto. O `rodar ci` e o
`pre-push` usam o mesmo caminho por dentro, com uma vaga e sem a trava do projeto.

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
- **Disparar a instalação ou a atualização:** `/setup-stacklabs`. Depois disso, o agente atualiza os
  watchdogs pelo instalador, sem pedir mais nada ([ADR 0024](../../../../adr/0024-atualizacao-do-metodo-sem-humano.md)). Constantes (mapas iniciais, tipos que exigem fontes,
  skills de grilling, limites da smart zone, caminhos livres) ficam em `.claude/watchdogs/src/comum.rs`.
  O projeto ajusta o que precisa sem mudar o código: veja *Ajustes do projeto*.

## Ajustes do projeto

O que um projeto muda nos watchdogs fica no campo `ajustes` de `.claude/metodo.json`, que só o
humano grava (o arquivo é protegido). A atualização do método não toca nele. Campo ausente ou
inválido fica no padrão.

```json
{
  "ajustes": {
    "caminhosLivres": ["infra/", "scripts/deploy.sh"],
    "branchesDescartaveis": ["spike/"],
    "contexto": { "aviso": 300000, "duro": 400000 }
  }
}
```

- **`caminhosLivres`**: soma aos caminhos editáveis sem issue ativa. Pasta termina em `/`. Não
  libera a infraestrutura dos watchdogs.
- **`branchesDescartaveis`**: soma prefixos aos `prototype/` e `research/`, onde se edita sem issue.
- **`contexto`**: os limites da smart zone em tokens (`aviso` menor que `duro`); padrão 450k e 550k.
- **`orcamentoPorIssue`** (na raiz do arquivo, não em `ajustes`): veja o `vigia-custo`.

Regra específica do projeto não vira PR no Stack-Labs: ela fica aqui, na seção
`## Regras deste projeto` do `AGENTS.md` ou no `docs/regras-de-codigo.json`. Mudança no método só
quando o humano pede.

## Limites conhecidos

- A proteção contra Bash é heurística: procura verbos mutantes junto dos caminhos protegidos. Barra o
  descuido, não um agente decidido a contornar; por isso a regra 21 também está escrita.
- O `guarda-bash` lê o **texto** do comando (e os arquivos de `--body-file`). Um comando guardado num
  script e executado por ele não é visto. A regra 21 cobre esse caso: contornar um watchdog é
  contrariá-la.
- O `guarda-edicao` confere que a issue tem um comentário com `## Contrato da fatia`, não que o
  contrato está completo nem que o diff o cumpre. Isso é do revisor e do conferente. O `vigia-codigo`
  roda só as regras de forma fixa no arquivo editado: o lint, o typecheck e as checagens do
  repositório inteiro continuam no `ci`.
- O `guarda-contexto` confere que a consulta foi chamada, não que a resposta foi lida. O registro
  acontece antes de o comando rodar. Leitura dentro de script, por `awk`, `git show` ou `python` não é
  vista. A regra 23 cobre esse caso.
- A regra de fontes confere que a seção e o link existem, não que a fonte sustenta a recomendação.
  Isso é do humano no grilling e do `/auditar-metodo`.
- Grilling com o humano respondendo e TDD não dão para checar por máquina.
- O `vigia-teste` e o validador conferem que o teste cita um risco do registro e tem afirmação que
  não é trivial; não conferem que ele exercita o risco que cita, nem que o esperado não copia o
  código. Isso é do revisor, que parte da lista da `revisao-por-modelo` quando o projeto a liga; o
  teste de mutação, quando adotado, mede.
- A `revisao-por-modelo` é consultiva e **falha aberta**, ao contrário dos watchdogs: sem `claude` no
  PATH, sem rede ou sem login, o `ci` avisa e segue. Ela lê só o trecho do teste e a linha do risco,
  não o código de produção, e erra nos dois sentidos; por isso não reprova. Os hooks nunca chamam o
  modelo, pela latência a cada edição.
- O custo por issue mede em **tokens equivalentes de entrada**: entrada 1x, escrita de cache 1,25x
  (5 min) ou 2x (1 h), leitura de cache 0,1x e saída 5x, os pesos da tabela de preços da Anthropic.
  Conta só turnos com issue ativa; planejamento, grilling e revisões por `claude -p` ficam de fora.
  O orçamento padrão é 3M; o humano o calibra em `.claude/metodo.json` (`"orcamentoPorIssue"`)
  olhando as issues já medidas.
- O `rodar pesado` só limita a memória por cgroup no Linux com systemd de usuário; sem ele (macOS,
  WSL sem systemd, container), avisa e roda com a trava e as vagas. SIGKILL no próprio `rodar pesado`
  não se intercepta: a trava solta e o comando segue vivo. Sem terminal, o comando não lê o
  teclado, e um prompt de senha falha. O cache do `ci` não enxerga o que o git ignora.
- O `guarda-bash` e os vigias leem o comando com um leitor de shell pequeno: o caminho montado numa
  variável (`p = '.claude/agents/x'; open(p, 'w')`) não aparece como alvo. Script que grava e cita
  um caminho protegido, e `xargs` ou `find -exec` que gravam num comando que cita um, são barrados
  pelo texto.
- O `rodar contexto` acha a definição por expressão regular (a palavra que declara e o nome), não
  por tree-sitter: um símbolo gerado por macro ou declarado de forma incomum não aparece. Termo do
  glossário casa pelos 5 primeiros caracteres de cada palavra, o que puxa vizinho de grafia parecida.
  O "muda junto" usa os últimos 2.000 commits, ignora commit com mais de 50 arquivos e some quando
  há mais de 10 arquivos em foco (uma issue larga muda junto com tudo).
