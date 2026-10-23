# O caminho até `ready-for-agent`

Uma issue só chega a `ready-for-agent` por uma das rotas abaixo, e só depois de passar pela
**definição de pronto**. É a label que libera a orquestração (`/orquestrar-issues`). Rotas que
pulam etapas produzem issues que o implementador não consegue terminar sozinho.

## As rotas

| De onde vem | Caminho | Quem aplica a label |
|---|---|---|
| **Projeto novo** | os 5 mapas iniciais (`docs/agents/mapas-iniciais.md`); depois, cada fase segue a rota de ideia grande | — |
| **Ideia grande** (várias sessões, com névoa) | `/wayfinder` → um ticket de decisão por sessão → mapa sem nada a decidir → `/to-spec` → `/to-tickets` | `/to-tickets` |
| **Ideia clara, várias sessões** | `/grill-with-docs` → `/to-spec` → `/to-tickets` | `/to-tickets` |
| **Ideia clara que cabe numa sessão** | `/grill-with-docs` → `/to-tickets` com uma fatia só | `/to-tickets` |
| **Bug ou pedido de fora** | issue → `/triage` (verifica, grilling se preciso, agent brief) | `/triage` |
| **Issue antiga de projeto em adoção** | `/triage` em lotes de até 10, na etapa de observação (`/adotar-metodo-stack-labs`) | `/triage` |

Em todas: as decisões saem do **grilling com o humano respondendo**, e toda recomendação tem fonte
(`docs/agents/pesquisa-antes-de-recomendar.md`). Aparência de tela passa antes pelo mapa de
Protótipo; a fatia que toca tela aponta o protótipo aprovado.

## Spec não é fatia

O `/to-spec` publica o spec com `ready-for-agent`. Logo depois, **troque essa label por `spec`**: o
spec é o pai das fatias, não uma delas. Quem o implementa são as fatias do `/to-tickets`, publicadas
como sub-issues dele. Os watchdogs reforçam: o `guarda-bash` não ativa uma issue `spec`, e o
orquestrador pula specs e issues com sub-issues.

## `ready-for-agent` ou `ready-for-human`

`ready-for-agent` quando um agente termina a fatia sozinho, do começo ao fim. `ready-for-human`
quando ela exige alguém no meio: acesso externo (painel, conta, credencial), decisão de design que
ficou para a hora, teste manual em aparelho, ou dado que só uma pessoa tem. Na dúvida,
`ready-for-human`, com o motivo escrito na issue.

## Definição de pronto

Antes de aplicar `ready-for-agent`, confira a issue contra **cada** item. Item que falha volta para a
etapa da rota que o resolve; não marque "quase pronta".

1. **Fatia vertical:** atravessa todas as camadas necessárias e é verificável sozinha. Refatoração
   larga segue expandir → migrar → contrair, como no `/to-tickets`.
2. **Cabe numa janela limpa** de um implementador.
3. **O que construir**, do ponto de vista de quem usa, e **critérios de aceite** verificáveis, em
   lista de checagem.
4. **Bloqueios** ligados como dependências nativas, só os que de fato impedem o começo.
5. **Nenhuma decisão em aberto:** tudo o que ela pressupõe está num ticket de decisão fechado, numa
   ADR, no spec ou no agent brief. Nada de "a definir".
6. **Ligada à origem:** sub-issue do spec pai, ou com o agent brief do `/triage`.
7. **Vocabulário do `GLOSSARY.md`** e coerente com as ADRs; contradição apontada, nunca escondida.
8. **Sem caminho de arquivo nem código**, salvo o trecho de protótipo que registra uma decisão.
9. **Tela com protótipo aprovado**, quando toca interface: link do ticket com a aprovação explícita.
10. **Sem humano no meio** (senão, `ready-for-human`).

A implementação ainda depende dos mapas iniciais: com Protótipo, Arquitetura, Fases ou Regras de
código abertos, o watchdog não ativa nenhuma issue.

## Depois da label

Ao terminar um `/to-tickets` ou um `/triage` que deixou issues em `ready-for-agent`, recomende a
orquestração e gere com o `/nova-sessao` o kickoff da janela do orquestrador
(`docs/agents/orquestracao.md`).
