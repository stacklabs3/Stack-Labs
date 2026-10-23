# Da ideia à issue pronta para o agente

`ready-for-agent` é a label que diz: um agente consegue fazer esta tarefa sozinho, do começo ao fim.
É ela que libera a [orquestração](orquestracao.md). O caminho até ela depende de onde a tarefa nasce.
A rota que o agente segue e a definição de pronto inteira estão em
[docs/agents/caminho-ate-ready-for-agent.md](../agentes/modelo-de-projeto/docs/agents/caminho-ate-ready-for-agent.md).

## Qual caminho

- **Projeto novo:** primeiro os cinco mapas iniciais. Depois, cada fase vira uma ideia grande.
- **Ideia grande**, com muita coisa a decidir: `/wayfinder`. Um ticket de decisão por sessão, até o
  mapa não ter mais nada a decidir. Então `/to-spec` e `/to-tickets`.
- **Ideia clara**, mas grande para uma sessão: `/grill-with-docs`, depois `/to-spec` e `/to-tickets`.
- **Ideia clara e pequena**: `/grill-with-docs`, depois `/to-tickets` com uma tarefa só.
- **Bug ou pedido de cliente**: abra a issue e rode `/triage`.
- **Issue antiga de um projeto que está adotando o método**: passa pelo `/triage` em lotes, durante a
  [adoção](adotar-em-projeto-existente.md), antes de ir para o agente.

## O que você faz em cada etapa

| Etapa | Você |
|---|---|
| Grilling (no `/wayfinder` e no `/grill-with-docs`) | Responde as perguntas e decide, aceitando só recomendação com fonte |
| `/to-spec` | Confere o resumo; o agente troca a label do spec para `spec` |
| `/to-tickets` | Aprova a divisão em tarefas: o tamanho de cada uma e o que depende do quê |
| `/triage` | Diz a direção: pronta para o agente, para uma pessoa, precisa de mais informação ou não vai ser feita |

## O que conferir ao aprovar as tarefas

O agente confere cada issue contra a **definição de pronto** antes de marcar. Os itens que pedem o
seu olho:

- É uma **tarefa completa e verificável sozinha**, que cabe numa sessão.
- Tem **critérios de aceite** que dá para conferir.
- Toda decisão já foi tomada e está escrita: nada "a definir".
- Se mexe em tela, o **protótipo foi aprovado** por você.
- Diz **quais riscos** do `docs/riscos.md` os testes dela cobrem, e todo risco novo foi aprovado por
  você.
- **Ninguém precisa entrar no meio.** Se precisar de acesso, conta, decisão de design ou teste manual,
  ela vai para `ready-for-human`.

## Depois

Com issues em `ready-for-agent`, o agente recomenda a orquestração e te dá o kickoff para abrir a
janela do orquestrador: [orquestracao.md](orquestracao.md).
