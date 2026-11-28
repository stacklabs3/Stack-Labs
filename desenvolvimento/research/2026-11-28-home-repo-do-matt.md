# Home repo do Matt: o que é, o que muda no Stack-Labs e o que fazer

Pesquisa de 2026-11-28. Pergunta: no vídeo `hWe-j78vNpc`, o Matt Pocock propõe o **home repo**, um
repositório único de onde o agente sempre roda e alcança os outros por caminho. Ele resolve dois
problemas: manter skills em sincronia em 30–40 repositórios e o diretório do usuário ser invisível e
pessoal. Isso serve à StackLabs, que hoje copia o método para dentro de cada projeto? O que quebraria,
e o que o piloto de verificação formal de 27/11 já mostrou sobre sessões que cruzam repositórios?

A fonte do vídeo é a transcrição em português `transcricao_hWe-j78vNpc.md` (543 linhas), lida
inteira. As citações dele trazem a linha desse arquivo (`l.N`). A transcrição é automática e
traduzida: "claw.md" é o `CLAUDE.md` e "clawed skills" é o `.claude/skills/` (l.195, l.201).

Legenda: **[fonte]** = dito pela fonte citada; **[medido]** = conferido nesta máquina com o comando
dado; **[inferido]** = conclusão minha a partir das fontes; **[promessa]** = o vídeo afirma, mas não
mostra funcionando; **[confirmar]** = não achei a fonte primária nesta passada.

## Resultado

| Ponto | Conclusão |
|---|---|
| O problema de sincronia que o home repo resolve | A casa já o resolve de outro jeito: instalador único, versão em `ORIGEM.json` e atualização sem humano (ADR 0020, ADR 0024). O atraso existe e é medido: o excurfly está no método 4.6.0, o Stack-Labs no 4.7.0 |
| O problema do diretório do usuário | Existe aqui e é maior que o vídeo descreve: 54 das 67 skills do projeto excurfly têm homônima em `~/.claude/skills/`, 23 com conteúdo diferente, e a doc do Claude Code diz que a pessoal vence a do projeto |
| Hooks e watchdogs num home repo | Quebram: o Claude Code lê o `.claude/settings.json` só da pasta de trabalho principal, e os watchdogs acham o projeto por `$CLAUDE_PROJECT_DIR`. O piloto mediu o efeito: `rodar custo 2910` deu 0 |
| Recomendação | **Opção (c)**: manter a cópia por projeto e corrigir os quatro atritos medidos. Home repo só volta à mesa se sessões que cruzam repositórios virarem rotina |

## 1. Como o Matt usa o home repo

### O que mora ali

- **Um repositório com tudo do agente.** "um único repositório" onde residem "todas as minhas
  habilidades", os "arquivos de direcionamento", "um wiki e várias ferramentas" (l.157-165). [fonte]
- **O agente roda sempre ali.** "eu o executo sempre dentro do home repo" (l.167-171). [fonte]
- **Daí alcança os outros.** O agente "consegue acessar a vitrine e a API de pagamentos" e
  "operar em vários repositórios diferentes ao mesmo tempo" (l.173-185). [fonte]
- **Antes do home repo, ele já descrevia a sessão multi-diretório:** uma sessão na vitrine que
  abre PR na vitrine e na API de pagamentos (l.57-75). O problema não é alcançar o outro repositório,
  é o que cada sessão carrega (l.77-85). [fonte]

### Os dois problemas que ele quer resolver

- **Cópia em cada repositório:** skills, ferramentas, `agents.md` e links de wiki "duplicados em
  vários repositórios" (l.87-95), e mantê-los "sincronizados" em "30 ou 40 deles" é "um enorme
  transtorno" (l.105-113). [fonte]
- **Diretório do usuário:** "fica meio invisível para o usuário" e as skills "ficarão defasadas"
  (l.123-131). Além disso, ele vê o diretório do usuário como o lugar da forma "como ele trabalha",
  não "como a equipe quer" (l.133-143). [fonte]

### Onde ficam as skills: públicas, privadas e do usuário

- **Privadas no home repo.** As skills em `.claude/skills/` do home repo "são privadas" e "sensíveis
  para o meu negócio" (l.199-211), e por isso "disponíveis em todas as sessões" (l.211-213). [fonte]
- **As da organização também.** Lista de skills da organização se adiciona "dentro do home repo"
  (l.215-221). [fonte]
- **Públicas noutro repositório.** Mudança de skill vai para o repositório "AI skills", alcançado
  pelo ponteiro (l.245-249). [fonte] Ele não diz se a sessão carrega as skills públicas por plugin,
  por cópia ou pelo caminho. [confirmar]
- **Do usuário:** fica para a preferência pessoal (l.133-143). Ele não mostra o que guarda ali.

### Como o agente alcança os outros repositórios

- **Ponteiro é um caminho.** O `CLAUDE.md`/`agents.md` tem "ponteiros de navegação" para os
  repositórios do dia a dia (l.235-245); "Literalmente apenas um caminho dentro do agents.md"
  (l.271-273). [fonte]
- **Clona o que falta.** Numa máquina nova, se o repositório "não existir neste local, ele o clona"
  (l.275-279). [fonte]
- **Worktree por agente.** Com vários agentes ao mesmo tempo, cada um usa "uma árvore de trabalho
  dentro desse mesmo repositório" (l.279-287). Ele chama de "configuração zero" (l.287) e lembra da
  limpeza de worktrees (l.289-293). [fonte] A afirmação de configuração zero não vem com
  demonstração. [promessa]
- **Repositório novo é uma linha.** "basta editar o agents.md" (l.295-299). [fonte]
- **Escala.** Funciona para "20, 30, 40 repositórios" e não para 400 (l.299-307). Para 400, ele
  "adicionaria" divulgação progressiva: grupos front-end, back-end e internos no `agents.md`
  (l.307-333). [promessa]: é como faria, não como faz.
- **Monorepo primeiro.** Se tudo coubesse num repositório, "ótimo" (l.353-357); o home repo é para
  quando não cabe, como na mistura de públicos e privados (l.345-351, l.359-363). [fonte]

### `settings.json` versionado

- O home repo é onde configurar "de uma forma versionada" (l.221-225), com um `settings.json` que se
  aplica "imediatamente a toda a sua organização" (l.227-235). [promessa]: ele não abre o arquivo no
  vídeo nem diz que hooks ou permissões guarda ali. A doc do Claude Code limita o alcance: o
  `.claude/settings.json` vale para quem trabalha na pasta que o contém, e é lido da pasta de
  trabalho principal da sessão (*Settings files and precedence*, seção *Settings files and who they
  affect* e *Where Claude Code keeps the local file*). [fonte] Ou seja: vale para a organização só
  porque todos rodam no home repo, e não entra nos repositórios que a sessão alcança. [inferido]

### Wiki e ferramentas

- **Wiki pessoal.** Ele chama o home repo de "minha wiki pessoal" (l.371-377): markdown para LLM,
  "estilo Carpathy" (l.379-387), consultada para criar cursos e conteúdo (l.395-403). [fonte]
- **Como gera e mantém a wiki:** "em um vídeo futuro" (l.387-393). [promessa]
- **Ferramentas.** Mostra a ajuda da CLI do wiki, com integrações de Gmail, Slack, Discord, YouTube,
  GitHub e Gcal (l.419-431). [fonte] Cresceram "organicamente" porque ele centralizou tudo
  (l.437-447). [fonte]
- **Fluxo entre repositórios:** achar uma mensagem no Discord, ir ao repositório do curso e corrigir
  "usando todas as habilidades" (l.449-468). [promessa]: descrito, não executado no vídeo.
- **Montar o seu:** um artigo, "Home Repo Pattern", com um prompt que entrevista e move as coisas
  (l.485-505). Não li o artigo. [confirmar]

### O que o repositório público do Matt mostra

Em `mattpocock/skills` @ `bbadaa631c37af594e5a05c9cefacd9956136b18` (HEAD de 2026-11-27) não há skill
nem documento com "home repo" (busca de código do GitHub por `home repo` no repositório; a árvore
inteira lida por `gh api .../git/trees/HEAD?recursive=1`). O parente mais próximo é a skill em
andamento `skills/in-progress/chief-of-staff/SKILL.md`: guarda memória "in a **wiki**" e manda
procurar "a shared wiki first" antes de criar uma (seção *Remember*). [fonte]

## 2. Comparação com o Stack-Labs de hoje

Evidência do piloto: `desenvolvimento/agentes/experimentos/verificacao-formal/README.md`, seção
*Ruído* (l.125-147) e tabela *O custo* (l.96-103). Cada fato abaixo foi conferido no arquivo.

| Peça | Onde mora hoje | Onde moraria num home repo | O que muda |
|---|---|---|---|
| Skills | Copiadas para `.claude/skills/` de cada projeto pelo instalador, com origem em `.claude/skills/ORIGEM.json` (`AGENTS.md` l.25-28; `setup-stacklabs/src/main.rs` l.882-981). Três skills de máquina em `~/.claude/skills/` (`AGENTS.md` l.18-20; `main.rs` l.30) | `.claude/skills/` do home repo | Some a cópia por projeto e o `ORIGEM.json` por projeto. Volta o problema que o `ORIGEM.json` resolveu: saber qual versão cada projeto roda (ADR 0020) |
| `AGENTS.md`/`CLAUDE.md` | A constituição em cada projeto, com a seção `## Regras deste projeto` (`modelo-de-projeto/AGENTS.md` l.146-177; ADR 0020). O `AGENTS.md` do Stack-Labs só mapeia este repositório | Um `AGENTS.md` com ponteiros por caminho (l.271-273) | A constituição de cada projeto continua precisando morar no projeto: as regras deste projeto são por projeto (ADR 0020) e o Claude Code carrega o `CLAUDE.md` da pasta onde a sessão começa [inferido] |
| `settings.json` | `.claude/settings.json` de cada projeto, protegido pela regra 21 (`modelo-de-projeto/AGENTS.md` l.112-117), com 12 watchdogs em 13 entradas de hook (`modelo-de-projeto/.claude/settings.json`). O Stack-Labs não versiona nenhum: só tem `.claude/settings.local.json`, com permissões [medido: `git ls-files .claude` vazio] | Um `settings.json` no home repo | Os hooks do projeto alvo deixam de carregar: o Claude Code lê o `settings.json` da pasta de trabalho principal (*Settings files and precedence*) |
| Watchdogs | Hooks chamados como `sh "$CLAUDE_PROJECT_DIR/.claude/watchdogs/rodar" hook <nome>` (`settings.json` l.8-134). O binário acha o projeto por `$CLAUDE_PROJECT_DIR`, e só sem ela pelo `cwd` do hook (`watchdogs/src/comum.rs` l.179-189). A ADR 0027 **não mudou isso**: mudou o `guarda-bash` e os vigias para decidir pelo **alvo da escrita** (`adr/0027` l.14-23; `escrita_por_bash.rs` l.47-60). O binário fica guardado por versão em `~/.stacklabs/watchdogs/<hash>/` (ADR 0025 l.47-49; `rodar` l.3-5) | Hooks do home repo | Com `$CLAUDE_PROJECT_DIR` no home repo, todo watchdog lê o estado, a etapa e os caminhos livres do home repo, não do projeto que se edita [inferido de `comum.rs` l.179-189]. O piloto: a sessão "nasceu no Stack-Labs e mudou de pasta" e os hooks do excurfly "não a acompanharam" (*Ruído*, l.136-137) |
| Custo (ADR 0017) | `vigia-custo` em Stop e SubagentStop grava `.claude/state/custos/issue-<n>.json` do projeto (`watchdogs/src/custo.rs` l.1-3, l.229-230; `watchdogs.md` l.26), e conta só turnos com issue ativa (`watchdogs.md` l.149) | Estado do home repo | O número da issue é por repositório: dois projetos com a #74 dividiriam o mesmo arquivo [inferido]. O piloto mediu 0: `rodar custo 2910` deu "0 tokens, 0 sessões", e o custo real (≈ 5,8 M) saiu do transcript (*O custo*, l.100-103) |
| Wiki e pesquisas | Do método: `desenvolvimento/GLOSSARY.md`, `desenvolvimento/adr/`, `desenvolvimento/research/` (`AGENTS.md` l.38-42). De cada projeto: `GLOSSARY.md`, `docs/adr/`, `docs/riscos.md` e `docs/research/` (ADR 0003 l.9-10; `modelo-de-projeto/AGENTS.md` l.139-144). A pesquisa de 2026-10-25 já chama isso de "segundo cérebro" (§ 6.4) | Uma wiki só no home repo | O que é do domínio do projeto continua no projeto, porque o `rodar contexto` liga termo, ADR e risco ao código do próprio repositório (ADR 0018; `watchdogs.md` l.41-58). Só o que cruza projetos (pesquisas da casa) caberia no home repo, e já cabe no Stack-Labs |
| Worktrees | Uma por sessão de issue, criada a partir da integração (`docs/agents/orquestracao.md` l.61-64; `orquestrar-issues/SKILL.md` l.73, l.83-98). A trava de comando pesado é por raiz do git, e cada worktree conta como projeto (ADR 0025 l.18-20; `trava.rs` l.94-95) | `.claude/worktrees/` dentro do home repo (l.279-287) | O caminho relativo começa em `.claude/`, que é caminho livre do `guarda-edicao` (`comum.rs` l.72-85; regra 3, `modelo-de-projeto/AGENTS.md` l.56-58). O piloto: "Uma worktree em `.claude/worktrees/` cairia em caminho livre do watchdog" (*Ruído*, l.138-140) |

### Os quatro fatos do piloto, conferidos

1. **A sessão no Stack-Labs não tinha os watchdogs do excurfly.** O README diz menos que isso: diz
   que os hooks **de custo** não acompanharam a sessão (l.136-137). O resto se confirma no
   repositório: o Stack-Labs não versiona `.claude/settings.json` [medido], então a sessão não tinha
   hook nenhum do método. [inferido]
2. **Worktree em `.claude/worktrees/` cairia em caminho livre.** Confere (l.138-140), e o código diz
   por quê: `.claude/` está em `CAMINHOS_LIVRES_SEM_ISSUE` (`comum.rs` l.72-85).
3. **O clone dentro do Stack-Labs nasceu sem `core.hooksPath`.** Confere (l.140-142). O pre-push se
   ativa "uma vez por clone", e quem ativa é o humano (`watchdogs.md` l.83); o `guarda-bash` barra
   "mudança no `core.hooksPath`" (`watchdogs.md` l.20) e a regra 21 a proíbe ao agente
   (`modelo-de-projeto/AGENTS.md` l.112-114).
4. **`rodar custo 2910` mediu 0.** Confere (*O custo*, l.100; *Ruído*, l.136-137).

Um quinto fato do mesmo *Ruído* toca o home repo: `rodar ativar-issue 74` ativou a #74 do excurfly
em vez da #74 do Stack-Labs (l.143-145). Número de issue sem o repositório é ambíguo entre projetos.

### O diretório do usuário nesta máquina

O vídeo diz que skill no diretório do usuário fica defasada (l.123-131). Aqui ela também **manda**:

- A doc do Claude Code (*Extend Claude with skills*, tabela de precedência): "personal over
  project. With `deploy` in both `~/.claude/skills/` and the project's `.claude/skills/`, `/deploy`
  runs the personal one." [fonte]
- No excurfly (`~/Pessoal/excurfly-app/excurfly`, método 4.6.0 pelo `ORIGEM.json`), 54 das 67 skills
  de `.claude/skills/` têm pasta com o mesmo nome em `~/.claude/skills/`, e 23 delas diferem
  (`diff -rq` por skill). `/tdd` e `/to-tickets` estão entre as diferentes. [medido]
- Então, nesta máquina, a sessão no excurfly roda a cópia pessoal dessas 23, não a versão que o
  instalador registrou. [inferido da doc e da medição]

## 3. Opções para o Stack-Labs

### (a) O Stack-Labs vira o home repo da casa

Toda sessão abre no Stack-Labs. As skills ficam em `desenvolvimento/agentes/skills/` (ou num
`.claude/skills/` na raiz), o `AGENTS.md` ganha ponteiros para excurfly, organiza-ai e os outros, e o
agente os alcança por caminho e por worktree.

- **Prós:** uma cópia das skills, sem instalador por projeto (l.105-113); pesquisas, ADRs e
  glossário do método à mão em toda sessão (`AGENTS.md` l.38-42); o fluxo entre repositórios do
  piloto vira o caso normal (*Ruído*, l.136-142).
- **Contras e o que quebraria:**
  - **Watchdogs:** os hooks do projeto alvo não carregam, porque o `settings.json` vem da pasta de
    trabalho principal (*Settings files and precedence*). Os watchdogs achariam o home repo por
    `$CLAUDE_PROJECT_DIR` (`comum.rs` l.179-189). Para valerem no alvo, todo watchdog teria de achar
    o projeto pela raiz do git do arquivo tocado, o que a ADR 0027 fez só para o **alvo** da escrita,
    não para o **projeto** (`adr/0027` l.14-23). Reescrita da ADR 0011.
  - **Regra 21:** ela protege `.claude/watchdogs/`, `.claude/settings.json` e `.claude/state/` **do
    projeto** (`modelo-de-projeto/AGENTS.md` l.112-117). Do home repo, esses caminhos do alvo ficam
    fora de qualquer hook. [inferido]
  - **`ORIGEM.json` e versão por projeto:** somem. O projeto deixa de saber em que versão do método
    está, e a atualização sem humano perde o objeto que confere (ADR 0020 l.9-14; ADR 0024 l.9-15).
  - **Regras deste projeto:** cada projeto tem as suas (ADR 0020 l.12-17). Um `AGENTS.md` único não
    as carrega; cada projeto precisaria continuar com o seu.
  - **Trava por projeto:** continua certa, porque `trava.rs` usa a raiz do git do comando
    (l.94-95) e não o `$CLAUDE_PROJECT_DIR`. [fonte]
  - **Custo por issue:** grava no `.claude/state/` do home repo, com números de issue que se repetem
    entre projetos (`custo.rs` l.229-230; *Ruído*, l.143-145).
  - **Worktrees** em `.claude/worktrees/` caem em caminho livre (`comum.rs` l.72-85).
  - O Stack-Labs é o repositório **dos processos** (`AGENTS.md` l.1-4); misturar o trabalho diário
    de todos os produtos nele junta PRs do método com sessões de produto.

### (b) Home repo só para sessões que cruzam repositórios

O projeto mantém o próprio método (cópia, watchdogs, `ORIGEM.json`). Um repositório separado, ou o
Stack-Labs, serve de base só para o trabalho que lê vários projetos (pesquisa, piloto, auditoria).
Edição de código continua numa sessão aberta no projeto.

- **Prós:** nada do método por projeto muda (ADR 0020, ADR 0024); a sessão de leitura cruzada tem um
  lugar certo; o `--add-dir` já carrega `.claude/skills/`, `.claude/commands/` e `.claude/agents/` do
  diretório somado (*Extend Claude with skills*, seção *Skills in added directories*). [fonte]
- **Contras e o que quebraria:**
  - A fronteira "só lê" é regra escrita, não watchdog: o `--add-dir` não traz hooks, e o
    `permissions.additionalDirectories` só dá acesso a arquivo (*Extend Claude with skills*). [fonte]
    Uma edição feita dali no projeto alvo passa sem `guarda-edicao`, sem contrato da fatia
    (regra 24) e sem custo. Foi o que o piloto fez (*Ruído*, l.136-142).
  - O `.claude/skills/` do alvo somado carrega skills de mesmo nome que as do home repo; a do
    diretório somado ganha prefixo (*Extend Claude with skills*, *Skills in added directories*). [fonte]
  - Não resolve o diretório do usuário, que vence as duas (seção 2).

### (c) Manter a cópia por projeto e corrigir os atritos medidos

Fica tudo como está, e o método corrige o que o piloto e a medição desta pesquisa mostraram.

1. **Sessão que edita um projeto abre nele.** Trabalho que começa no Stack-Labs e passa a editar
   outro projeto fecha com `/nova-sessao` e continua numa janela aberta no projeto (regra 12,
   `modelo-de-projeto/AGENTS.md` l.83-85). Clonar um projeto dentro do Stack-Labs sai da rota: o
   clone nasce sem `core.hooksPath` e o agente não pode ligá-lo (*Ruído*, l.140-142; regra 21). A doc
   do Claude Code abre outra rota: depois de `/cd`, a sessão lê os arquivos de projeto da pasta nova
   (v2.1.246+) (*Settings files and precedence*). Se isso move também o `$CLAUDE_PROJECT_DIR` dos
   hooks, o caso do piloto se resolve sem sessão nova. [confirmar]
2. **Trabalho não commitado na raiz do projeto:** a rota é a worktree **ao lado** do projeto, que a
   `/setup-stacklabs` já usa (`setup-stacklabs/SKILL.md` l.39-41), não `.claude/worktrees/`, que é
   caminho livre (`comum.rs` l.72-85).
3. **Skills pessoais com nome de skill do método:** a cópia em `~/.claude/skills/` vence a do
   projeto (*Extend Claude with skills*). O método diz que só as três skills de máquina moram ali
   (`AGENTS.md` l.18-20; `main.rs` l.30), e o `sessao-inicio` pode avisar quando uma skill do
   `ORIGEM.json` tem homônima pessoal. [inferido]
4. **Custo de sessão que nasceu noutra pasta:** o `rodar custo` lê só o estado do projeto
   (`custo.rs` l.229-230). O relato do piloto já usou o transcript com os pesos do `custo.rs`
   (*O custo*, l.101). Esse cálculo pode virar um subcomando que aceita o transcript. [inferido]

- **Prós:** nenhuma ADR se reabre; os watchdogs seguem achando o projeto como hoje (`comum.rs`
  l.179-189); `ORIGEM.json`, regras deste projeto, trava por projeto e custo por issue ficam como
  estão (ADR 0020, 0024, 0025, 0017). Ataca o problema de sincronia que existe aqui de fato: o
  diretório do usuário (seção 2) e o atraso de versão, que a ADR 0024 já reduz ao único gesto
  `/setup-stacklabs` (l.9-15).
- **Contras e o que quebraria:** nada quebra. Fica o custo que o Matt descreve: cada projeto tem a
  sua cópia e precisa de `/setup-stacklabs` para andar (o excurfly está um minor atrás: `ORIGEM.json`
  4.6.0 contra `desenvolvimento/VERSAO` 4.7.0) [medido]. Sessão que cruza repositórios continua
  sendo duas sessões.

### Recomendação

**Opção (c).** O problema que o home repo resolve para o Matt, sincronizar 30–40 cópias à mão
(l.105-113), a casa já resolve com o instalador e a atualização sem humano (ADR 0024 l.9-15; ADR 0020
l.9-14). O que o home repo custaria é o núcleo do método: os watchdogs acham o projeto pela pasta da
sessão (`comum.rs` l.179-189), o Claude Code só carrega hooks dessa pasta (*Settings files and
precedence*), e o piloto mostrou o resultado, custo medido em 0 e commits sem pre-push (*Ruído*,
l.136-142). Os quatro atritos medidos têm correção local, listada em (c). A opção (b) fica registrada
para quando sessões que cruzam repositórios deixarem de ser exceção: hoje a evidência é um piloto.

## 4. Perguntas para o humano

1. **Sessão que passa a editar outro projeto abre janela nova nele?** Recomendação: sim, pelo
   `/nova-sessao` (regra 12; *Ruído* l.136-137). Exemplo: no piloto, a sessão do Stack-Labs editou o
   excurfly e o `rodar custo 2910` mediu 0; aberta no excurfly, o custo teria entrado na #2910.
2. **Clonar um projeto dentro do Stack-Labs fica fora da rota?** Recomendação: sim; com trabalho não
   commitado na raiz, worktree ao lado do projeto (`setup-stacklabs/SKILL.md` l.39-41). Exemplo: o
   clone do excurfly no Stack-Labs nasceu sem `core.hooksPath` e os commits passaram sem pre-push.
3. **Testo se o `/cd` leva os hooks e o `$CLAUDE_PROJECT_DIR` para o projeto novo?** Recomendação:
   sim, um teste curto antes de mudar a rota (*Settings files and precedence*, v2.1.246+). Exemplo:
   começar no Stack-Labs, `/cd` para o excurfly e ver se o `sessao-inicio` do excurfly roda.
4. **As skills do método saem de `~/.claude/skills/`, ficando só as três de máquina?** Recomendação:
   sim, porque a pessoal vence a do projeto (*Extend Claude with skills*). Exemplo: no excurfly, o
   `/tdd` que roda é o pessoal, diferente do instalado pela 4.6.0.
5. **O `sessao-inicio` avisa quando uma skill do projeto tem homônima pessoal?** Recomendação: sim,
   só aviso, como os vigias da ADR 0017. Exemplo: hoje 54 das 67 skills do excurfly têm homônima.
6. **Home repo (opção b) fica para depois?** Recomendação: sim, até haver mais de um caso de trabalho
   entre repositórios (*Ruído* é o único registro). Exemplo: uma auditoria que lê excurfly e
   organiza-ai juntos seria o próximo caso a contar.
7. **O custo de sessão que nasceu noutra pasta entra no método?** Recomendação: um subcomando que
   calcula pelo transcript com os pesos do `custo.rs`, sem mudar o `vigia-custo` (*O custo* l.101).
   Exemplo: os ≈ 5,8 M do piloto, hoje contados à mão.

## Fontes

- Matt Pocock, vídeo do home repo: <https://youtu.be/hWe-j78vNpc>, pela transcrição
  `transcricao_hWe-j78vNpc.md` (raiz do Stack-Labs, 543 linhas, lida inteira). O artigo "Home Repo
  Pattern" citado no vídeo (l.485-505) não foi lido. [confirmar]
- `mattpocock/skills` @ `bbadaa631c37af594e5a05c9cefacd9956136b18` (lido por `gh api`): árvore
  inteira; `skills/in-progress/chief-of-staff/SKILL.md`; `CHANGELOG.md` (entrada `5beb35b` do
  `chief-of-staff`); `.changeset/chief-of-staff-shared-wiki-procedures.md`.
- Claude Code, *Settings files and precedence*: <https://code.claude.com/docs/en/settings> (seções
  *Settings files and who they affect*, *Where Claude Code keeps the local file in a git repository*,
  *Settings precedence*), lida em 2026-11-27.
- Claude Code, *Extend Claude with skills*: <https://code.claude.com/docs/en/skills> (tabela de onde
  as skills carregam, precedência, *Skills in added directories*), lida em 2026-11-27 até o
  caractere 100.000.
- Neste repositório:
  - `AGENTS.md`; `desenvolvimento/agentes/README.md` (tabela de donos, *Mudar o método*);
    `desenvolvimento/VERSAO`; `desenvolvimento/CHANGELOG.md` (4.7.0, 4.6.0).
  - `desenvolvimento/agentes/skills/setup-stacklabs/SKILL.md` e `src/main.rs` (l.1-40, l.640-680,
    l.860-1005); `desenvolvimento/agentes/skills/adotar-metodo-stack-labs/SKILL.md`;
    `desenvolvimento/agentes/skills/orquestrar-issues/SKILL.md` e `preparar-worktree.sh`.
  - `desenvolvimento/agentes/modelo-de-projeto/AGENTS.md` (regra 2, l.47-51; regra 3, l.53-59;
    regra 12, l.83-85; regra 21, l.111-117; regra 24, l.128-135; *Regras deste projeto*, l.146-177);
    `.gitignore`; `.claude/settings.json`; `docs/agents/watchdogs.md`; `docs/agents/orquestracao.md`;
    `.claude/agents/implementador.md`.
  - `desenvolvimento/agentes/modelo-de-projeto/.claude/watchdogs/`: `rodar`; `src/comum.rs`;
    `src/escrita_por_bash.rs`; `src/custo.rs`; `src/trava.rs`; `src/hooks/guarda_bash.rs`;
    `src/hooks/guarda_edicao.rs`; `src/hooks/sessao_inicio.rs`.
  - `desenvolvimento/agentes/experimentos/verificacao-formal/README.md` (*O custo*, *Ruído*, *O que
    o humano precisou ler e decidir*).
  - `desenvolvimento/adr/`: títulos de 0001 a 0027; lidas inteiras 0003, 0006, 0009, 0011, 0016,
    0017, 0020, 0024, 0025 e 0027.
  - `desenvolvimento/research/2026-11-04-memoria-e-tempo-das-validacoes.md` e
    `2026-10-25-busca-deterministica-para-contexto.md` (formato; § 6.4).
- Medido nesta máquina em 2026-11-27: `git ls-files .claude` e `.claude/settings.local.json` do
  Stack-Labs; `.claude/skills/ORIGEM.json` e `diff -rq` de cada skill de `.claude/skills/` contra
  `~/.claude/skills/` no clone do excurfly (`bb491f665`, 2026-11-23).
