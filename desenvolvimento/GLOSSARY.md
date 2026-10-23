# Método de desenvolvimento com IA da StackLabs

Linguagem comum do método que sócios, funcionários e agentes de IA usam em todo projeto da
StackLabs. É a fonte única dos termos: o guia dos humanos e a constituição dos agentes usam estas
palavras com estes sentidos. Não descreve implementação. Para isso servem os guias e o modelo de projeto.

## Language

### Planejamento

**Mapa**:
Uma issue do GitHub com a label `wayfinder:map` que leva um esforço grande demais para uma sessão
até o destino dele. É um índice: guarda o destino, as notas, as decisões tomadas (uma linha e um
link cada), a névoa e o que ficou fora de escopo. A decisão em si mora no ticket.
_Evitar_: Épico, roadmap, plano, backlog

**Mapa inicial**:
Um dos cinco mapas que todo projeto abre no primeiro dia, antes de qualquer código de produção:
Protótipo, Arquitetura, Fases, Regras de código e Infra. Leva a label `wayfinder:map` e a label
`mapa-inicial:<tipo>`.
_Evitar_: Kickoff, discovery, fase zero

**Regras de código**:
O documento `docs/regras-de-codigo.md` de cada projeto, destino do mapa inicial de mesmo nome: as
regras que todo código segue, cada uma com o porquê e um exemplo do projeto. É a régua do
implementador e do revisor. Muda só por ticket.
_Evitar_: Style guide, padrões, boas práticas, coding standards

**Princípios de código**:
Os 22 princípios da casa para escrever código, iguais em todo projeto (escrita E1 a E8, padrões P1 a
P9 e SOLID), em `docs/agents/principios-de-codigo.md`. Guiam o grilling do mapa de Regras de código,
que adota, adapta ou afasta cada um; o E1 (todo nome diz o que é) e o P5 (todo teste cobre um risco)
são regras da constituição e nunca se afastam.
_Evitar_: Boas práticas, clean code, style guide

**Prática da linguagem**:
Como um princípio de código aparece numa linguagem da casa (TypeScript, Java, Rust), com a regra de
máquina que o confere, ou uma prática que só aquela linguagem tem e nenhum dos 22 cobre (`TS<n>`,
`JV<n>`, `RS<n>`), em `docs/agents/principios-de-codigo/<linguagem>.md`. O grilling decide cada uma
como regra do projeto, fora da tabela dos 22.
_Evitar_: Boa prática, style guide, princípio da linguagem

**Validador**:
O motor do método, no binário Rust de `.claude/watchdogs/`, que confere por máquina as regras de código decididas em
`docs/regras-de-codigo.json`, a cobertura dos 22 princípios e das práticas da linguagem adotadas e o
registro de riscos contra os testes. O
projeto refina a configuração; o
motor é protegido como um watchdog.
_Evitar_: Linter (quando é o validador), checker

**Risco**:
O que, se der errado no produto, causa dano concreto a quem usa ou a quem mantém (cobrar duas vezes,
cupom vencido aceito, permissão vazada). Tem ID `RK<n>`, nasce aprovado pelo humano com a origem
escrita e fica no **registro de riscos** (`docs/riscos.md`). Todo teste cobre um risco e o cita;
teste sem risco não entra.
_Evitar_: Caso de teste, cenário (quando é o risco), requisito

**Registro de riscos**:
O documento `docs/riscos.md` de cada projeto: a tabela dos riscos que os testes cobrem, cada um com o
dano, onde mora, a origem e, enquanto não tem teste, `pendente #<issue>`. Nasce no mapa de Regras de
código e cresce pela spec e pelo triage de bug.
_Evitar_: Matriz de risco, plano de testes, test plan

**Destino**:
O que significa chegar ao fim de um mapa: uma spec, uma decisão travada ou uma mudança feita. É a
primeira coisa que se escreve num mapa e define o escopo dele.
_Evitar_: Objetivo, meta, entrega

**Ticket**:
Uma issue filha de um mapa que faz uma pergunta cuja resposta é uma decisão. Cabe numa sessão e tem
um tipo: pesquisa, protótipo, grilling ou tarefa. Ticket decide, não implementa.
_Evitar_: Tarefa (quando não é do tipo tarefa), card, história

**Fronteira**:
Os tickets abertos, sem bloqueio e sem responsável de um mapa. São os únicos que se pode pegar agora.
_Evitar_: Próximos passos, fila

**Névoa**:
As perguntas que se percebe que virão, mas que ainda não se sabe formular com precisão. Fica na
seção *Not yet specified* do mapa e vira ticket quando a pergunta fica precisa.
_Evitar_: Pendências, TODO, dúvidas

**Fora de escopo**:
O que está além do destino. Não é névoa e nunca vira ticket deste mapa.
_Evitar_: Backlog, depois

**Spec**:
O documento que fecha um mapa: tudo o que foi decidido, pronto para ser fatiado. Sai do `/to-spec`
e leva a label `spec`: é o pai das fatias verticais, nunca implementado inteiro.
_Evitar_: PRD, requisito, documento de requisitos

**Definição de pronto**:
A lista de itens que uma fatia vertical cumpre antes de receber `ready-for-agent`. A lista vive em
`docs/agents/caminho-ate-ready-for-agent.md`.

**Fatia vertical**:
Uma issue de implementação que atravessa todas as camadas necessárias e entrega algo verificável.
Sai do `/to-tickets` com a label `ready-for-agent` ou `ready-for-human`.
_Evitar_: Task, subtarefa, camada

### Decisão

**Grilling**:
O interrogatório que o agente faz ao humano em rodadas: todas as perguntas independentes de uma vez,
numeradas, cada uma com a recomendação do agente e a fonte que a sustenta. O humano responde; o
agente nunca responde por ele.
_Evitar_: Brainstorm, conversa, alinhamento

**Pesquisa**:
A investigação em fontes primárias (documentação oficial, código, especificação, preço publicado)
que vira um arquivo em `docs/research/` com cada afirmação citando a fonte. Feita por um subagente
com `/research`.
_Evitar_: Benchmark de blog, opinião, "pelo que eu sei"

**Recomendação fundamentada**:
Toda sugestão do agente que aponta a pesquisa ou a fonte primária que a sustenta. Recomendação sem
fonte não é recomendação, é palpite, e não pode ser oferecida.
_Evitar_: Sugestão, dica, boa prática

**Fontes**:
A seção `## Fontes` do comentário de resolução de um ticket, com ao menos um link para
`docs/research/` ou uma fonte primária.

**ADR**:
O registro curto de uma decisão difícil de reverter, surpreendente sem contexto e resultado de uma
troca real. Fica em `docs/adr/`. Contradizer uma ADR exige apontar a contradição, nunca ignorar.
_Evitar_: Ata, decisão (solta), RFC

**Glossário do projeto**:
O `GLOSSARY.md` de cada projeto, com a linguagem do domínio do cliente. Nasce e cresce com o
`/domain-modeling`.
_Evitar_: Dicionário, wiki

### Protótipo

**Protótipo**:
Artefato barato e descartável que responde "como deve parecer" ou "como deve se comportar". A
aparência se prototipa com a família `*-prototipo`; o comportamento, com `/prototype`.
_Evitar_: MVP, POC, mockup

**Protótipo único**:
O protótipo visual do projeto, criado uma vez pelo `/novo-prototipo` em `prototipos/<projeto>/`, onde
cada rotina nova entra pelo `/integrar-ao-prototipo`.
_Evitar_: Protótipo da tela X, Figma (como fonte da verdade)

**Rotina**:
Uma sequência de telas que uma pessoa percorre para fazer uma coisa, com o caminho e os toques contados.
_Evitar_: Fluxo, jornada, feature

**Lote de protótipo**:
Um conjunto de rotinas construídas, acabadas e auditadas juntas, que o humano aprova de uma vez.
_Evitar_: Sprint, onda (sozinho)

**Aprovação explícita**:
O "aprovado" escrito pelo humano no comentário de resolução do ticket de protótipo. Só ela libera o
`/portar-prototipo`. "Legal", emoji ou silêncio não são aprovação.

### Execução

**Issue ativa**:
A única fatia vertical em que uma sessão de agente pode editar código de produção, ativada com
`sh .claude/watchdogs/rodar ativar-issue <n>`.

**Sessão**:
Uma janela de contexto de um agente, do kickoff ao `/nova-sessao` que abre a próxima. Uma sessão
resolve um ticket de decisão, implementa uma issue ou conduz uma fase.
_Evitar_: Chat, conversa

**Orquestrador**:
O agente principal da sessão aberta com `/orquestrar-issues`. Não escreve código: dispara em
sequência os subagentes implementador, revisor, conferente e diagnosticador, um por vez, e cuida da
branch de integração.
_Evitar_: Loop, gerente, coordenador

**Branch de integração**:
A branch `integracao/<nome>` de uma rodada de orquestração. Cada issue parte dela e volta para ela
por merge depois de aprovada; só no fim da fila ela chega à `main`, por PR.

**Ciclo de correção**:
Uma volta de diagnosticador → implementador → revisor → conferente depois de um parecer reprovado.
Cada issue tem até dois; passou disso, a issue é devolvida para triagem.

**Smart zone**:
A faixa da janela de contexto em que o agente ainda raciocina bem. Passou dela, a próxima ação é o `/nova-sessao`.

**Kickoff**:
A mensagem autossuficiente de copiar e colar que o `/nova-sessao` gera para abrir a próxima janela:
pré-voo com as premissas a conferir, skills a invocar, decisões já tomadas e critério de aceite.
_Evitar_: Resumo, prompt de continuação

**Handoff**:
O documento que guarda o estado de um trabalho para quem o retoma. Três formas: o
`HANDOFF-<projeto>.md` na raiz, que o `/voltar-a-principal` mantém para cada tópico de sessão
satélite; o do `/handoff` do Matt; e o estado da rodada que o orquestrador de issues grava.
Para abrir uma janela nova com humano, o padrão é o kickoff.

**Sessão principal**:
A sessão onde o trabalho em curso acontece e cujo contexto se protege.

**Sessão satélite**:
Uma sessão aberta à parte para discutir ou explorar algo fora do foco da sessão principal, sem gastar
nem corromper o contexto dela. Termina no `/voltar-a-principal`, que atualiza o `HANDOFF-<projeto>.md`
e gera a mensagem de retorno para colar na principal.
_Evitar_: Sessão lateral, sessão paralela (que soa como trabalho simultâneo), subagente

**Watchdog**:
Um hook do Claude Code, um hook do git ou um check de CI que transforma uma regra do método em
bloqueio. Um bloqueio de watchdog é uma informação, não um obstáculo a contornar.
_Evitar_: Guard-rail, lint (quando é do método)

**Adoção**:
A entrada no método de um projeto que já existia antes dele, pela `/adotar-metodo-stack-labs`. Anda
em etapas e não refaz o trabalho que estava em andamento.
_Evitar_: Migração, implantação

**Etapa de adoção**:
`observacao`, `planejamento` ou `completo`, gravada em `.claude/metodo.json`. Decide só o que os
watchdogs bloqueiam; as regras do método valem inteiras desde o primeiro dia.

**Mapa retroativo**:
Mapa inicial de um projeto em adoção, que registra decisões já tomadas, aponta as ADRs que o humano
confirmou e fecha na hora quando nada está em aberto.

**Constituição**:
O `AGENTS.md` de cada projeto: as regras duras que todo agente segue em toda sessão.
_Evitar_: Prompt, instruções

## Relationships

- Um **projeto** abre cinco **mapas iniciais** e pode abrir outros **mapas** depois.
- Um **mapa** tem um **destino**, muitos **tickets**, uma **névoa** e uma lista **fora de escopo**.
- Um **ticket** de grilling ou tarefa só fecha com **fontes**; um ticket de protótipo só libera o porte com **aprovação explícita**.
- Um **mapa** fechado vira uma **spec**, que vira **fatias verticais**; cada fatia é a **issue ativa** de uma **sessão**.
- Um **watchdog** vigia uma regra da **constituição**.
- Toda **fatia vertical** segue as **regras de código**; o revisor reprova a que quebra uma.
- As **regras de código** de um projeto partem dos **princípios de código** da casa, e cada regra cita o princípio de onde nasceu.

## Flagged ambiguities

- "Mapa" sempre quer dizer a issue `wayfinder:map`. Mapa mental, mapa de telas ou mapa de mesas do cliente não são mapas do método.
- "Fronteira" sem qualificador é a do mapa. A fronteira de decisões do grilling (as perguntas
  independentes de uma rodada) e a fronteira de issues `ready-for-agent` da orquestração se escrevem
  com o qualificador.
- "Tarefa" é só o tipo de ticket que faz um trabalho para destravar uma decisão. Trabalho de implementação é **fatia vertical**.
