# Como funcionam as skills de verificação formal do pstack

Pesquisa de 2026-11-25, ticket #70 do mapa #69. Pergunta: como funcionam por dentro
`create-formal-verification-skill`, `maintain-formal-verification-skill`, a skill `verify` que elas
deixam no repo e a `statechart`, do pstack de Lauren Tan; o que tem evidência e o que é promessa; e o
que serve ao método (ADR 0014, registro de riscos).

**Fonte primária:** o repositório [`cursor/plugins`](https://github.com/cursor/plugins), pasta
[`pstack/`](https://github.com/cursor/plugins/tree/main/pstack), lido no commit `b1fbf4b`
(2026-11-25), pstack versão `0.15.48` (`pstack/plugin.json`). As issues e PRs citadas são desse
repositório. O caso do crashlens foi lido no repositório
[`mingleiw/crashlens`](https://github.com/mingleiw/crashlens), commit `1c94575`. Os forks e ports de
terceiros (`backnotprop/pstack`, `michael-denyer/pstack-claude` e outros) não foram usados.

Neste texto, **invariante** é a propriedade que o model checker confere (o que o vídeo chama de
"regra"; o próprio pstack usa "property" e, na `statechart`, "rule").

## Resumo

- As três skills são **só texto de instrução** (43, 58 e 82 linhas de `SKILL.md`), com
  `disable-model-invocation: true`: rodam apenas quando o humano as chama pelo nome. A única com
  código é a `statechart` (um compilador em TypeScript que roda o TLC e desenha Mermaid).
- A skill `verify` **não existe no pstack**: é gerada no repo do usuário, com nome e conteúdo
  livres. O que o pstack fixa é o padrão que ela deve cumprir: uma invariante, um comando, a tabela
  modelo→código, o commit de sincronia e como separar bug de modelo errado.
- O mecanismo central ("uma checagem que você nunca viu falhar não prova nada") está no texto e tem
  evidência: o eval da PR #716 e o caso crashlens. A falha que o vídeo cita (o typo que prova uma
  invariante vazia, issue #1235) é real e **continua aberta**: a `statechart` ainda dá "Rules that
  hold" sem aviso; as duas PRs que corrigem (#1254, #1303) não foram mescladas.
- "Beta" não aparece em lugar nenhum do pstack. É afirmação do vídeo, sem fonte primária.

## `create-formal-verification-skill`

Texto: `pstack/skills/create-formal-verification-skill/SKILL.md`. Criada na PR #709 (2026-10-18),
ajustada nas #721, #716 e #794. Os passos:

1. **Escolher o alvo pelo código** (linhas 11–15). Lê o repo e pergunta ao usuário só o que não dá
   para observar; se o usuário nomeou uma área ou um bug, começa por ali. O alvo bom é "uma promessa
   que o código faz, onde a quebra prejudica usuários e os testes raramente alcançam o caso que a
   quebra", e a skill manda escolher **o mais forte, não o mais fácil de verificar**. Não há
   critério além desse parágrafo: a escolha é julgamento do agente.
2. **Deixar a pergunta escolher a ferramenta** (linha 19). Ordem de eventos → model checker sobre um
   modelo pequeno do desenho, **TLA+ por padrão em qualquer linguagem**. Comportamento do código
   para toda entrada (memória, overflow, invariante de estrutura) → verificador que roda no código.
   Sem verificador para a linguagem → dizer isso e testar o código real com entradas geradas, "não
   um modelo à mão da função". Propriedade em todo tamanho → prova. Preferir ferramenta que os devs
   do repo instalem e rodem, e justificar a escolha. A PR #721 mostra por que o fallback existe: num
   eval numa biblioteca JavaScript sem verificador, 6 de 9 runs com o texto antigo "provaram" a
   afirmação só numa cópia escrita à mão da função; com o texto novo, 9 de 9 checaram o código real.
   Kani não é nomeado no `SKILL.md`; o vídeo usa como exemplo.
3. **Os critérios do resultado** (linhas 21–29):
   - *Fiel ao código:* cada parte do modelo aponta para código que um revisor abre; o que o modelo
     trata como passo atômico é atômico no código; os nomes vêm do repo.
   - *Capaz de falhar:* "A check that you have never seen fail proves nothing." Mostrar **cada**
     invariante falhando numa variante quebrada do código que ela guarda (uma checagem removida, um
     passo reordenado).
   - *Achado continua achado:* se o código atual quebra a invariante, reportar com o contraexemplo
     nos termos do código; não enfraquecer invariante nem modelo, e não mexer em código de produto
     sem o usuário pedir.
   - *Honesto sobre escopo:* dizer o que foi verificado, sob que limites e suposições, e o que ficou
     de fora.
   - *Legível sem a ferramenta:* a invariante e o dano em uma frase simples cada; o contraexemplo
     como passos nas palavras do código.
   - *Barato de rodar:* um comando, versão da ferramenta reproduzível, run curto.
   - *Em passo com o código:* registrar arquivos e funções cobertos e o commit em que se confirmou
     pela última vez que a verificação os descreve.
4. **Gerar a skill `verify`** (linhas 31–35). Skill local com frontmatter (sem ele, "a skill nunca
   registra"). Só com ela, o próximo agente precisa saber se a mudança toca o código verificado,
   rodar a checagem, separar bug real de erro de modelagem e atualizar a verificação quando a
   mudança é intencional. O registro de cobertura e o commit de sincronia ficam dentro da skill, e o
   commit só avança depois de confirmar que o modelo descreve o código no commit novo. Sem
   placeholders. Se for TLA+, mandar ler todo resultado com `/statechart`, citada pelo nome e sem
   copiar comandos dela (PR #794).
5. **Provar antes de entregar** (linhas 37–39). Seguir a skill gerada uma vez, como o próximo agente
   faria, numa mudança de rascunho que quebra a invariante no código real. Ela tem de pegar a quebra.
   Depois descartar o rascunho e confirmar que a checagem voltou ao estado inicial. Se não pegar,
   conserta a skill ou a verificação, nunca o rascunho.
6. **Entregar** (linhas 41–43): o que foi verificado, ferramenta e porquê, limites, o achado em
   palavras simples, como rodar, o que não cobre; apontar a `maintain` para depois.

**O que ela grava:** a skill `verify` (com tabela de cobertura e commit de sincronia), o modelo e o
script de checagem. O `SKILL.md` não fixa caminho nem nome; no crashlens ficaram em
`.claude/skills/verify-serve-ask-ai/SKILL.md` e `verification/serve-ai/`.

## `maintain-formal-verification-skill`

Texto: `pstack/skills/maintain-formal-verification-skill/SKILL.md`, PR #716 (2026-10-19). Serve
para as mudanças que **pularam** a skill `verify`: merges, edições sem ela, deriva acumulada. O
caso perigoso, nas palavras dela, é o *pass* sobre código que não existe mais, "porque parece uma
prova".

Cada checagem termina num de quatro resultados: **in sync**, **updated**, **bug**, **blocked**
(pode ser *updated* e *bug* ao mesmo tempo). Só edita a verificação (modelos, harness, invariantes,
a própria skill `verify`), nunca código de produto sem pedido. Os passos (linhas 26–44):

1. **Achar as checagens** pelo que fazem, não pelo nome. Sem commit de sincronia registrado, usa o
   último commit que mexeu nos arquivos da verificação e diz isso. Sem skill `verify`, para e
   aponta a `create`.
2. **Rodar como está**, antes de mudar nada. Pass aqui só diz que a descrição antiga ainda vale.
3. **Ler cada mudança no código coberto**, do commit de sincronia até agora, mais as mensagens de
   commit, código movido ou renomeado e código novo que escreve o mesmo estado. Classificar cada
   mudança em: *sem efeito* (conferido contra o modelo, não contra a mensagem do commit); *o modelo
   não descreve mais o código* (passo atômico virou dois, ramo novo, faixa de valor mudou); *a
   promessa mudou de propósito* (com evidência: doc comment, testes, changelog, mensagem); *a
   promessa pode ter quebrado* (sem evidência, então candidato a bug).
4. **Atualizar a partir do código, não do resultado.** Escrever a mudança do modelo antes de
   rodar. Invariante só muda por promessa mudada de propósito, com redação antiga, nova e evidência.
   Reduzir limite só dizendo no relatório.
5. **Rodar e julgar.** Pass: quebrar o código novo do jeito que a invariante guarda e confirmar que
   a checagem pega. Fail: **reexecutar o contraexemplo no código real, passo a passo**. Se algum
   passo não pode acontecer no código, o modelo está errado (deriva); se todos podem, é bug: nomear
   o commit que o introduziu, contar os passos, o dano em uma frase, e anexar teste ou reprodução
   barata como evidência, sem commitá-lo no produto.
6. **Atualizar a skill `verify`** e mover o commit de sincronia, exceto para checagem *blocked*.

O relatório separa explicitamente deriva de bug, e as mudanças da verificação vão num commit
separado de qualquer correção de produto.

**É assim que ela separa modelo defasado de bug real:** pelo replay do contraexemplo no código, não
pelo resultado do checker. É o mesmo critério da `statechart` (linha 82: "se um bug não ocorre, pense
que o modelo está errado até mostrar que não está").

## A skill `verify` gerada (caso crashlens)

Não há modelo de `verify` no pstack. O único exemplo público achado é o do crashlens:
`.claude/skills/verify-serve-ask-ai/SKILL.md` (93 linhas). Ele tem:

- duas invariantes com o dano em uma frase: `OneSuggestion` ("o relatório mostra duas correções da
  IA") e `OneApiCall` ("o usuário paga duas vezes pela mesma resposta");
- o que fica fora (dois relatórios diferentes, falha da API com retry, o conteúdo da resposta);
- um comando, `verification/serve-ai/check.sh`, que baixa o TLC 1.7.4, confere o sha256, roda em
  cerca de um segundo e exige Java 11+;
- a tabela ação do modelo → código (`Press(t)` → rota `POST /api/ai` até o `await suggestFix`),
  com a justificativa da atomicidade ("o Node roda a rota sem interrupção entre os `await`s");
- o commit de sincronia (`01f0b8c`) e cinco passos para a próxima mudança, terminando em "não mova o
  commit sem fazer os passos 1 a 4".

**O bug achado.** O commit `f19c7a5` (2026-11-03, autor "Claude", sessão do Claude Code) diz que o
estado *busy* da página só bloqueava o ponteiro; Enter no botão focado ou uma segunda aba mandavam o
mesmo relatório à API de novo, gerando uma segunda chamada paga. "Found with a TLA+ model ... the
shortest failing path (press, press, answer, answer) is replayed against the real server in
`test/server.test.ts`." O commit seguinte, `a7258cb`, registra a prova de entrega: removeu a guarda
de requisição em voo e a checagem falhou em `OneApiCall`, junto com o teste de replay.

**Um limite que o próprio caso mostra.** O commit `105b782` (2026-11-13) corrige a skill: nenhuma
invariante dependia da guarda `analyzed.includes(entry)`, "removê-la mantém a checagem verde". Só
um teste a segura. Ou seja, a primeira versão afirmava cobrir uma guarda que o modelo não conseguia
ver falhar; a correção veio de uma revisão posterior, não do passo "capaz de falhar" da `create`,
que testou só uma quebra.

## `statechart`

Texto: `pstack/skills/statechart/SKILL.md`; código em `pstack/skills/statechart/scripts/` (PR #741,
2026-10-22; reescrita em TypeScript na #977). Não é verificação nova: é um **leitor** do TLC para
quem decide o comportamento.

- **Rodar:** `scripts/statechart SPEC.tla [--config] [--view] [--out]`, numa pasta de rascunho fora
  do repo. Precisa de Node 22.18+ (ou 23.6+), Bun 1.1.5+ ou Deno 2+ (o script de entrada escolhe).
  Na primeira vez instala TLC e Java pelo **mise** e precisa de rede: `run.ts` fixa
  `tla2tools` 1.7.4 com sha256 e `java@temurin-25`; dá para apontar `TLA2TOOLS` e `JAVA` (Java 11+).
- **O spec:** cada invariante que o leitor aceita vira `INVARIANT`; "nunca enfraqueça uma regra para
  ela valer"; `Next` como disjunção de ações nomeadas; estado final ganha uma ação `Done` para não
  quebrar `NoDeadlock`.
- **A view:** um módulo com `state(vars)` (string = gráfico plano, array = estados pais, objeto =
  regiões paralelas), `step(action, from, to)` com o rótulo da seta nas palavras do usuário
  ("you press play"), `group`, `colors`, `title`. O agente escolhe as palavras; meta de cerca de 20
  setas.
- **O relatório:** estado que quebra invariante leva a classe `breaks` (ou `mayBreak`); para cada
  invariante quebrada, o caminho mais curto. O comando roda o TLC com um worker, remove a invariante
  quebrada e roda de novo até nenhuma quebrar; depois da primeira quebra, cada run para em 100.000
  estados e o que não foi checado aparece em "Rules not checked". Acima de 500 estados o gráfico só
  desenha os caminhos até as quebras.
- **Entrega:** o gráfico Mermaid e uma linha por bug; e, se o modelo descreve código, reexecutar
  cada caminho no código real.

**Issues abertas relevantes:**

- **#1235 (o typo do pagamento, aberta desde 2026-11-14).** Um modelo `Vac.tla` com
  `BankAccepts == sent > 0 /\ paid' = TRUE /\ UNCHANGED <<sent, paid>>`: a ação muda `paid` e ao
  mesmo tempo o declara inalterado, então nunca dispara. `NeverPaidTwice` aparece em "Rules that
  hold", sem seta de `BankAccepts` no gráfico. O único sinal é um aviso do TLC em `tlc.out`, que o
  `report.md` descarta. Com `UNCHANGED sent`, a invariante quebra. O autor diz ter caído nisso "no
  primeiro dia de um modelo" em uso real. Pede (1) levar os avisos do TLC ao relatório e (2) listar
  as ações de `Next` que nunca deram passo. As PRs #1254 e #1303 fazem cada parte e estão abertas.
  No código de `main`, os avisos só aparecem no backend `plain-text.ts`, não no relatório.
- **#783:** a ajuda e os docs prometiam mais do que os scripts faziam; um comentário de 2026-11
  diz que os cinco pontos foram resolvidos pela reescrita e sobra só a frase "o caminho mais curto
  para cada uma", que não vale para propriedade temporal.
- **#1338, #1170, #973, #923, #1164, #1141:** defeitos de limpeza de processo, Windows (CRLF,
  `taskkill`), caminho relativo de `TLA2TOOLS`, `TMPDIR` com symlink no macOS e typecheck estrito.
  Mostram um script novo e ainda instável fora do caminho feliz.

Issue vizinha, de outra skill: **#1271** (fechada) relata que a `create-verification-skill` (a de
jornadas de app, não a formal) só provava que a skill roda: 7 de 12 quebras passaram. Com "quebre
de propósito e exija a falha", 16 de 16. É a mesma lição da linha "capaz de falhar" da skill formal.
A **#1245** (aberta) pede que a verificação seja usada só para o que os testes existentes não provam,
pelo custo.

## O que se sustenta e o que é promessa

| Afirmação (vídeo ou pstack) | Situação | Fonte |
|---|---|---|
| Escolhe o alvo pelo código, o de maior dano | No texto; é julgamento do agente, sem critério verificável | `create…/SKILL.md` l. 15 |
| TLA+ por padrão para ordem de eventos; fallback com entradas geradas | No texto, com eval preregistrado | PR #721 |
| Quebra o código de propósito e vê a checagem falhar | No texto; eval mostra efeito na `maintain` (11 de 11 runs com o skill contra 5 de 10 e 4 de 10 sem) | PR #716 |
| A `maintain` separa deriva de bug | No texto (replay do contraexemplo); eval: 18 de 18 runs separaram | PR #716 |
| A `maintain` melhora a correção | **Não** mostrou ganho real: corrigidos dois artefatos de gabarito, os três braços dão 18 de 18 | PR #716 |
| O registro de commit faz o agente atualizar a sincronia | Sim no eval: 18 de 18 com registro contra 4 de 18 sem | PR #716 |
| A `statechart` mostra o caminho mais curto em palavras simples | Sim para invariantes; para propriedade temporal dá o contraexemplo do TLC | SKILL.md, #783 |
| Um modelo com typo pode "provar" uma invariante vazia | **Real e aberto**; a ferramenta não avisa | #1235, PRs #1254 e #1303 abertas |
| crashlens: duas chamadas pagas achadas pelo checker | Sustentado pelo commit e por um teste de replay; o trabalho foi de um agente no próprio repo | crashlens `f19c7a5`, `a7258cb` |
| Lauren "já achou vários bugs" com isso | Sem fonte primária no repositório; não verificado | só o vídeo |
| "Está em beta" | Sem fonte primária; o pstack está em `0.15.48`, sem marca de beta | `plugin.json` |
| Uma invariante por vez | Coerente com o texto ("verifies one property"), não é limite técnico | `create…/SKILL.md` l. 9 |
| Precisa de Java | Sim: TLC é Java (11+); a `statechart` instala Temurin 25 via mise | `run.ts` l. 68–69 |

Duas ressalvas sobre a evidência: os evals das PRs #716 e #721 foram feitos pela própria autora, com
juiz por modelo, e o relato numérico está só no corpo das PRs, sem os dados brutos publicados. E o
caso crashlens é de um contribuidor do ecossistema (o mesmo `mingleiw` comenta na #783), não um
estudo independente.

## Licença, instalação e dependências

- **Licença:** MIT, `Copyright (c) 2026 Lauren Tan` (`pstack/LICENSE`; `"license": "MIT"` em
  `plugin.json`). O repositório `cursor/plugins` como um todo não declara licença na raiz (a API do
  GitHub dá `license: null`); a do pstack vale para a pasta dele.
- **Instalação no Claude Code** (`pstack/README.md`):
  `/plugin marketplace add cursor/plugins` e `/plugin install pstack@cursor-plugins`. O marketplace
  do Claude Code (`.claude-plugin/marketplace.json`) aponta para uma **cópia gerada** em
  `harness/pstack/claude-code/`, não para `pstack/`. A diferença nas duas skills formais é uma linha
  que manda ler o adaptador `docs/harness/claude-code.md` uma vez por sessão; a `statechart` é
  idêntica. No Claude Code os comandos ganham prefixo (`/pstack:create-formal-verification-skill`)
  e auto-update fica desligado para marketplace de terceiros. No Cursor: `/add-plugin pstack`.
- **O plugin vem inteiro:** são dezenas de skills, agentes e o `poteto-mode`; não há instalação só
  das três. A `/setup-pstack` oferece gerar a verificação formal (`docs/guide/01-setup.md` l. 41).
- **Dependências:** Java 11+ e `tla2tools.jar` (TLC); para a `statechart`, Node 22.18+/Bun/Deno e
  mise com rede na primeira vez (ou `TLA2TOOLS`/`JAVA` apontados à mão). Para outros alvos, a
  ferramenta que o agente escolher.
- **Ritmo:** as skills formais nasceram entre 2026-10-18 e 2026-10-27 e têm cinco commits em
  `create`; o plugin passou de 0.15.36 (citado na #1235) a 0.15.48 em onze dias.

## O que serve ao método

Olhando a ADR 0014 (todo teste cobre um risco `RK`, mutação como receita que o grilling adota) e o
registro `docs/riscos.md` (a classe "Tempo, ordem e concorrência" já existe, com o exemplo "dois
caixas fecham a mesma conta").

**Vira receita quase como está:**

- O **padrão da skill `verify`**: invariante e dano em uma frase cada, o que fica fora, um comando
  com versão fixada por hash, tabela ação do modelo → código, atomicidade justificada, commit de
  sincronia que só avança depois de reconferir. O crashlens é um modelo concreto a copiar. A
  invariante pode citar o `RK<n>` que cobre, e o "dano" já é a coluna do registro.
- **"Nunca viu falhar não prova nada"**, aplicado a **cada** invariante e **cada** guarda que o
  modelo diz cobrir. É a linha de base da mutação (4.4.0) levada ao modelo: a quebra de propósito é
  uma mutação manual do código que o modelo descreve.
- **Separar deriva de bug pelo replay do contraexemplo no código real**, com o replay virando teste
  `RK<n>` (crashlens fez isso em `test/server.test.ts`). Liga a verificação formal à regra 15: o
  achado do checker termina num teste que cobre o risco.
- **Não enfraquecer a invariante; mudança de promessa só com evidência** (redação antiga, nova,
  evidência). Casa com "o agente propõe, o humano aprova".

**Precisa de adaptação:**

- **Escolha do alvo:** no pstack é o agente que escolhe "o mais forte". No método, o alvo vem do
  registro: um `RK` de severidade alta da classe tempo, ordem e concorrência, aprovado pelo humano.
- **Invariante vazia (#1235):** a ferramenta não protege. A receita precisa exigir, além da quebra
  de propósito, que **toda ação de `Next` dê pelo menos um passo** e que avisos do TLC zerem ou sejam
  explicados. Dá para conferir por máquina lendo a saída do TLC (estatística por ação, `-coverage`),
  sem depender das PRs abertas.
- **Defasagem:** o pstack depende de o próximo agente ler a skill e de alguém rodar a `maintain`.
  No método, isso cabe ao motor: um watchdog ou o validador compara os arquivos da tabela de
  cobertura com o commit de sincronia e acusa código coberto que mudou sem o modelo mudar.
- **Caminho e nome:** o pstack deixa livres. O método precisa fixar (pasta do `.tla`, nome da skill
  `verify-*`, onde mora o registro de cobertura) para a máquina achar.
- **Instalação:** trazer o plugin inteiro conflita com as skills da casa e com o `ORIGEM.json` da
  `/setup-stacklabs`. Com licença MIT, o caminho viável é vendorizar as duas skills formais (texto
  curto) adaptadas, com a origem e o commit registrados.
- **Infra:** Java no CI local e na máquina; o TLC com um worker e o limite de estados mantêm o run
  curto, mas o custo de tokens por invariante não foi medido em fonte nenhuma (ADR 0017).

**Não serve, ou não agora:**

- A **`statechart`** como dependência: o script é novo, tem seis issues abertas de robustez e
  depende de mise e rede. A ideia (mostrar o contraexemplo nas palavras do usuário para o humano
  julgar a invariante) serve; o agente pode escrever o caminho em palavras sem a ferramenta.
- A **escolha livre de ferramenta** (Kani, provas, property-based): o mapa #69 limita o esforço a
  TLA+.
- O **`poteto-mode`** e o resto do plugin: fora do escopo.

## Fontes

- `cursor/plugins` @ `b1fbf4b`: `pstack/skills/create-formal-verification-skill/SKILL.md`,
  `pstack/skills/maintain-formal-verification-skill/SKILL.md`, `pstack/skills/statechart/SKILL.md`,
  `pstack/skills/statechart/scripts/run.ts`, `pstack/skills/statechart/scripts/statechart`,
  `pstack/README.md`, `pstack/LICENSE`, `pstack/plugin.json`, `pstack/docs/guide/01-setup.md`,
  `pstack/docs/guide/06-verify-and-ship.md`, `.claude-plugin/marketplace.json`,
  `harness/pstack/claude-code/`.
- PRs de `cursor/plugins`: [#709](https://github.com/cursor/plugins/pull/709),
  [#716](https://github.com/cursor/plugins/pull/716),
  [#721](https://github.com/cursor/plugins/pull/721),
  [#794](https://github.com/cursor/plugins/pull/794),
  [#1254](https://github.com/cursor/plugins/pull/1254),
  [#1303](https://github.com/cursor/plugins/pull/1303).
- Issues de `cursor/plugins`: [#1235](https://github.com/cursor/plugins/issues/1235),
  [#783](https://github.com/cursor/plugins/issues/783),
  [#1338](https://github.com/cursor/plugins/issues/1338),
  [#1271](https://github.com/cursor/plugins/issues/1271),
  [#1245](https://github.com/cursor/plugins/issues/1245),
  [#1170](https://github.com/cursor/plugins/issues/1170),
  [#973](https://github.com/cursor/plugins/issues/973).
- `mingleiw/crashlens` @ `1c94575`: `.claude/skills/verify-serve-ask-ai/SKILL.md`,
  `verification/serve-ai/check.sh`, commits `f19c7a5`, `a7258cb`, `105b782`.
- Transcrição do vídeo `bQCBfcZZj0Y` (raiz do repositório), só para as afirmações a conferir.
