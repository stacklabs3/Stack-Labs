# Da ideia à issue pronta para o agente

`ready-for-agent` é a label que diz: um agente consegue fazer esta tarefa sozinho, do começo ao fim.
É ela que libera a [orquestração](orquestracao.md). Chegar a ela tem caminho certo, e ele depende de
onde a tarefa nasce. O detalhe que o agente segue está em
[docs/agents/caminho-ate-ready-for-agent.md](../agentes/modelo-de-projeto/docs/agents/caminho-ate-ready-for-agent.md).

```
projeto novo ──► 5 mapas iniciais ─────────────┐
                                               ▼
ideia grande ──► /wayfinder ──► (mapa claro) ──► /to-spec ──► /to-tickets ──► ready-for-agent ──► orquestração
ideia clara ───► /grill-with-docs ─────────────┘                ▲
ideia pequena ─► /grill-with-docs ──────────────────────────────┘ (uma fatia só)
bug / pedido ──► /triage ──────────────────────────────────────────────► ready-for-agent
```

## Qual caminho

- **Projeto novo:** primeiro os cinco mapas iniciais. Depois, cada fase vira uma ideia grande.
- **Ideia grande**, com muita coisa a decidir: `/wayfinder`. Um ticket de decisão por sessão, até o
  mapa não ter mais nada a decidir. Então `/to-spec` e `/to-tickets`.
- **Ideia clara**, mas grande para uma sessão: `/grill-with-docs`, depois `/to-spec` e `/to-tickets`.
- **Ideia clara e pequena**: `/grill-with-docs`, depois `/to-tickets` com uma tarefa só.
- **Bug ou pedido de cliente**: abra a issue e rode `/triage`.
- **Issue antiga de um projeto que está adotando o método**: passa pelo `/triage` em lotes, durante a
  [adoção](adotar-em-projeto-existente.md). Nenhuma vai para o agente sem esse triage.

## O que você faz em cada etapa

| Etapa | Você |
|---|---|
| Grilling (no `/wayfinder` e no `/grill-with-docs`) | Responde as perguntas e decide. Não aceite recomendação sem fonte |
| `/to-spec` | Confere o resumo; o agente troca a label do spec para `spec` |
| `/to-tickets` | Aprova a divisão em tarefas: o tamanho de cada uma e o que depende do quê |
| `/triage` | Diz a direção: pronta para o agente, para uma pessoa, precisa de mais informação ou não vai ser feita |

## Quando uma issue está pronta para o agente

Antes de marcar, o agente confere uma lista de dez itens. Os que mais importam para você:

- É uma **tarefa completa e verificável sozinha**, que cabe numa sessão.
- Tem **critérios de aceite** que dá para conferir.
- **Nada ficou "a definir"**: toda decisão já foi tomada e está escrita.
- Se mexe em tela, o **protótipo foi aprovado** por você.
- **Ninguém precisa entrar no meio.** Se precisar de acesso, conta, decisão de design ou teste manual,
  ela vai para `ready-for-human`, não `ready-for-agent`.

## Depois

Com issues em `ready-for-agent`, o agente recomenda a orquestração e te dá o kickoff para abrir a
janela do orquestrador. Veja [orquestracao.md](orquestracao.md).
