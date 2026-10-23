# Observação: mapas retroativos, triage e passagem para planejamento

Os watchdogs só avisam nesta etapa. Use o tempo para pôr o projeto no formato do método.

## 4. Mapas iniciais retroativos

Uma sessão por mapa, nesta ordem: Arquitetura, Protótipo, Fases, Infra
(`docs/agents/mapas-iniciais.md`). O mapa é **retroativo**: registra decisões que o projeto já tomou.

1. Grilling para confirmar com o humano como o projeto está hoje naquela frente.
2. Cada decisão difícil de reverter que ele confirmar vira ADR pelo `/domain-modeling`.
3. Crie o mapa (`wayfinder:map` + `mapa-inicial:<tipo>`); *Decisions so far* aponta as ADRs e os
   documentos que já existem.
4. O que ninguém decidiu de verdade vira ticket ou névoa. Sem nada em aberto, **feche o mapa**.

Na sessão de Arquitetura, crie também o `GLOSSARY.md` de partida pelo `/domain-modeling`, a partir do
código e da conversa. Pronto, por mapa, quando ele existe e cada linha de *Decisions so far* aponta
uma ADR, um documento ou um ticket fechado.

## 5. Triage das issues abertas

`/triage` em lotes de até 10 issues por sessão, cada lote decidido pelo humano. A régua é a definição
de pronto (`docs/agents/caminho-ate-ready-for-agent.md`): cumpre → `ready-for-agent` ou
`ready-for-human`; falta algo → `needs-info` ou `needs-triage`; perdeu o sentido → `wontfix`. Issue
antiga só entra na orquestração depois daqui.

Pronto quando nenhuma issue aberta está sem label de estado.

## 6. Passagem para planejamento

Mostre ao humano, com evidência, cada condição:

- os quatro mapas iniciais existem;
- Protótipo, Arquitetura e Fases estão fechados;
- o `GLOSSARY.md` existe;
- o passo 5 está pronto.

Com as quatro certas, ele faz a PR que muda `etapaDeAdocao` para `planejamento`. Pronto quando ela
está mergeada; o próximo ramo é [PLANEJAMENTO.md](PLANEJAMENTO.md).
