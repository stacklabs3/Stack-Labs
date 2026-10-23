# Como a StackLabs trabalha com IA

Guia para sócios e funcionários. Vale em **todo projeto**. A versão que os agentes seguem fica em
[../agentes/](../agentes/README.md); as duas usam as palavras do [glossário](../GLOSSARY.md).

## A ideia em uma frase

**A IA executa; você decide.** O agente pesquisa, pergunta, prototipa e escreve código. Toda decisão
passa por você, e toda recomendação dele vem com a fonte que a sustenta. O método é o do Matt Pocock
([ADR 0001](../adr/0001-metodo-matt-pocock-com-wayfinder.md)), mais quatro regras da casa: os mapas
iniciais, a pesquisa antes de recomendar, a família `*-prototipo` e todo nome que diz o que é
(`produto = valor - custo`, nunca `x = y - z`). O projeto de referência é o
[appetito_v2](https://github.com/stacklabs1/appetito_v2).

## Quem faz o quê

| Você | O agente |
|---|---|
| Responde o grilling e decide | Pergunta, recomenda e mostra a fonte |
| Aprova protótipo por escrito | Constrói, audita e leva a aprovação até você |
| Revisa e faz merge da PR | Implementa uma issue por sessão, com testes |
| Escolhe a fila e a branch de integração, e decide como ela chega à `main` | Orquestra a fila: implementa, revisa, confere e corrige, uma issue por vez |
| Abre a janela nova e cola o kickoff | Gera o kickoff com o `/nova-sessao` |
| Muda os watchdogs, se precisar | Nunca mexe neles |

## A rota

```
projeto novo ──► 5 mapas iniciais
                    │
ideia grande ──► /wayfinder ──► /to-spec ──► /to-tickets ──► ready-for-agent ──► orquestração ──► PR
                (decisões, uma    (fecha o    (fatias                            (implementa, revisa
                 por sessão)       mapa)       verticais)                          e confere)
```

- **Outras entradas** (ideia clara, bug ou pedido de cliente) e o que você aprova em cada etapa:
  [caminho-ate-ready-for-agent.md](caminho-ate-ready-for-agent.md).
- **Projeto que já está rodando:** entra por etapas, com `/adotar-metodo-stack-labs`
  ([adotar-em-projeto-existente.md](adotar-em-projeto-existente.md)).
- **Não sabe por onde ir:** `/ask-matt`.

## Os cinco mapas iniciais

Todo projeto abre, no primeiro dia, cinco mapas no GitHub: **Protótipo**, **Arquitetura**,
**Fases**, **Regras de código** e **Infra** ([ADR 0002](../adr/0002-quatro-mapas-iniciais.md),
[ADR 0007](../adr/0007-mapa-de-regras-de-codigo.md)). O que cada um decide e as perguntas típicas:
[mapas-iniciais.md](mapas-iniciais.md). O grilling de Regras de código roda em torno dos
[princípios de código](principios-de-codigo.md) da casa.

A implementação começa só com **Protótipo** (o desenho inicial), **Arquitetura**, **Fases** e
**Regras de código** fechados; o watchdog barra antes disso. **Infra** é informativo e não trava nada.

## Grilling: como responder

Em todo mapa, as decisões saem de um **grilling**.

- Responda por número. **"Sim"** aceita a recomendação; qualquer outra coisa, explique.
- Aceite só recomendação com fonte primária: documentação oficial, página de preço, texto da lei.
  Sem fonte, ou com blog e "todo mundo usa", peça a pesquisa
  ([ADR 0003](../adr/0003-nenhuma-recomendacao-sem-pesquisa.md)).
- Se você não sabe responder, diga. A pergunta vira um ticket de pesquisa, de protótipo ou de tarefa.
- A resposta é sempre sua. Se o agente responder por você, interrompa.

## Protótipo: aprovar é escrever

O agente leva os prints e os pontos até o ticket. Você aprova **por escrito** no comentário de
resolução ("Aprovado", com o que mudar, se for o caso): é a **aprovação explícita**, a única que
libera o protótipo para virar código. Etapas do protótipo: o
[roteiro de protótipo](../agentes/modelo-de-projeto/docs/agents/roteiro-de-prototipo.md).

## Implementação

- Uma issue por sessão. Para trocar de issue, abra outra sessão.
- A `main` só muda por PR. A PR cita a issue (`Closes #42`) e passa no `vigia-do-metodo`.
- "Pronto" é teste passando, não o agente dizendo que terminou.
- Com issues em `ready-for-agent`, o agente recomenda a **orquestração**:
  [orquestracao.md](orquestracao.md).

## Hábitos de sessão

- **Uma janela só do grilling até o `/to-tickets`**, sem `/compact` nem `/clear` no meio.
- **Trabalho novo ou continuação: janela nova.** No fim de uma fase, ou quando o watchdog avisa que
  a janela encheu, rode **`/nova-sessao`**, abra uma janela nova e cole o kickoff. Se algo mudou
  desde que ele foi gerado, o pré-voo faz a sessão nova parar e perguntar.
- **Assunto paralelo: sessão satélite.** Discuta lá e rode **`/voltar-a-principal`**: ele atualiza o
  `HANDOFF-<projeto>.md` do tópico e entrega uma mensagem curta para colar na sessão principal.
- **Um ticket de decisão por sessão** no wayfinder. Contexto limpo decide melhor.
- **Use o glossário do projeto.** Se o agente usa outra palavra para a mesma coisa, corrija.

## Quando um watchdog barra

Leia o motivo: ele diz o que falta e qual é a rota certa. Watchdog errado se corrige no watchdog,
por PR com a label `altera-watchdogs`, nunca na conversa. Mensagens e escapes:
[watchdogs.md](watchdogs.md).

## Começar um projeto

Siga o [checklist](iniciar-um-projeto.md).
