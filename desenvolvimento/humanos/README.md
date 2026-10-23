# Como a StackLabs trabalha com IA

Guia para sócios e funcionários. Vale em **todo projeto**. A versão que os agentes seguem fica em
[../agentes/](../agentes/README.md); as duas usam as palavras do [glossário](../GLOSSARY.md).

## A ideia em uma frase

**A IA executa; você decide.** O agente pesquisa, pergunta, prototipa e escreve código. Toda decisão
passa por você, e toda recomendação dele vem com a fonte que a sustenta. O método é o do Matt Pocock
([ADR 0001](../adr/0001-metodo-matt-pocock-com-wayfinder.md)), mais três regras da casa: os mapas
iniciais, a pesquisa antes de recomendar e a família `*-prototipo`. O projeto de referência é o
[appetito_v2](https://github.com/stacklabs1/appetito_v2).

## Quem faz o quê

| Você | O agente |
|---|---|
| Responde o grilling e decide | Pergunta, recomenda e mostra a fonte |
| Aprova protótipo por escrito | Constrói, audita e leva a aprovação até você |
| Revisa e faz merge da PR | Implementa uma issue por sessão, com testes |
| Abre a janela nova e cola o kickoff | Gera o kickoff com o `/nova-sessao` |
| Muda os watchdogs, se precisar | Nunca mexe neles |

## A rota

```
projeto novo ──► 4 mapas iniciais
                    │
ideia grande ──► /wayfinder ──► /to-spec ──► /to-tickets ──► /implement (1 issue por sessão) ──► PR
                (decisões, uma    (fecha o    (fatias          + /tdd + /code-review
                 por sessão)       mapa)       verticais)
```

- **Ideia clara, várias sessões:** `/grill-with-docs` → `/to-spec` → `/to-tickets`.
- **Ideia clara, uma sessão:** `/grill-with-docs` → `/implement`.
- **Bug ou pedido de cliente:** `/triage` → `/implement`.
- **Não sabe por onde ir:** `/ask-matt`.

## Os quatro mapas iniciais

Todo projeto abre, no primeiro dia, quatro mapas no GitHub
([ADR 0002](../adr/0002-quatro-mapas-iniciais.md)). Detalhes e perguntas típicas em
[mapas-iniciais.md](mapas-iniciais.md).

1. **Protótipo**: o desenho inicial do produto, com a família `*-prototipo`.
2. **Arquitetura**: linguagem, stack e módulos.
3. **Fases**: como o desenvolvimento se divide, decidido pelo desenvolvedor que conduz o projeto.
4. **Infra**: o que vamos usar, quanto custa e o que teremos que construir em volta.

Ninguém começa a implementar antes de **Protótipo** (o desenho inicial), **Arquitetura** e **Fases**
fecharem. O watchdog barra. **Infra** é informativo: dá conhecimento de custos e tecnologias e não trava nada.

## Grilling: como responder

Em todo mapa, as decisões saem de um **grilling**: o agente faz várias perguntas de uma vez,
numeradas, cada uma com a recomendação dele.

- Responda por número. **"Sim"** aceita a recomendação; qualquer outra coisa, explique.
- Se a recomendação não tem fonte, **não aceite**. Peça a pesquisa
  ([ADR 0003](../adr/0003-nenhuma-recomendacao-sem-pesquisa.md)). Fonte boa é primária:
  documentação oficial, página de preço, texto da lei. Blog e "todo mundo usa" não contam.
- Se você não sabe responder, diga. A pergunta vira um ticket de pesquisa, de protótipo ou de tarefa.
- O agente nunca responde por você. Se ele fizer isso, interrompa.

## Protótipo: aprovar é escrever

O agente leva os prints e os pontos para aprovação até o ticket. Você aprova **por escrito** no
comentário de resolução ("Aprovado", com o que mudar, se for o caso). "Legal", emoji ou silêncio não
liberam o protótipo para virar código. Veja o [roteiro de protótipo](../agentes/modelo-de-projeto/docs/agents/roteiro-de-prototipo.md).

## Implementação

- Uma issue por sessão. Para trocar de issue, abra outra sessão.
- A `main` só muda por PR. A PR cita a issue (`Closes #42`) e passa no `vigia-do-metodo`.
- "Pronto" é teste passando, não o agente dizendo que terminou.

## Várias issues sem você olhando

Com várias issues `ready-for-agent` prontas, o **loop de issues** implementa até N delas em sequência,
cada uma numa sessão nova, e entrega PRs para você revisar. Veja [loop-de-issues.md](loop-de-issues.md).

## Hábitos de sessão

- **Uma janela do grilling até o `/to-tickets`.** Nada de `/compact` ou `/clear` no meio.
- **Trabalho novo ou continuação: janela nova.** No fim de uma fase, ou quando o watchdog avisa que
  a janela encheu, rode **`/nova-sessao`**. Ele confere que nada ficou de fora, escolhe as skills e
  entrega uma mensagem de kickoff; abra uma janela nova e cole. A mensagem traz um pré-voo: se algo
  mudou desde que ela foi gerada, a sessão nova para e pergunta.
- **Assunto paralelo: sessão satélite.** Uma dúvida que foge do foco não entra na sessão principal.
  Abra outra sessão, discuta lá e rode **`/voltar-a-principal`**: ele atualiza o
  `HANDOFF-<projeto>.md` do tópico e te entrega uma mensagem curta para colar na sessão principal,
  com só o que a satélite fez e decidiu. A principal não gasta contexto com o desvio.
- **Um ticket de decisão por sessão** no wayfinder. Contexto limpo decide melhor.
- **Use o glossário do projeto.** Se o agente usa outra palavra para a mesma coisa, corrija.

## Quando um watchdog barra

Leia o motivo: ele diz o que falta e qual é a rota certa. Se o watchdog estiver errado, corrija o
watchdog (por PR, com a label `altera-watchdogs`), não a regra na conversa. Veja [watchdogs.md](watchdogs.md).

## Começar um projeto

Siga o [checklist](iniciar-um-projeto.md).
