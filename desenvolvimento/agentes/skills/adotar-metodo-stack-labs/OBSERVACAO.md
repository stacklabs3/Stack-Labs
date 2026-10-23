# Observação: mapas retroativos, triage e passagem para planejamento

Nesta etapa os watchdogs barram só o que mexe neles e na `main`; o resto vira aviso
(`docs/agents/adocao.md`). Use o tempo para pôr o projeto no formato do método.

## 4. Mapas iniciais retroativos

Um **mapa retroativo** por sessão, nesta ordem: Arquitetura, Regras de código, Protótipo, Fases,
Infra (`docs/agents/mapas-iniciais.md`).

1. Grilling para confirmar com o humano como o projeto está hoje naquela frente.
2. Cada decisão difícil de reverter que ele confirmar vira ADR pelo `/domain-modeling`.
3. Crie o mapa (`wayfinder:map` + `mapa-inicial:<tipo>`); *Decisions so far* aponta as ADRs e os
   documentos que já existem.
4. O que ninguém decidiu de verdade vira ticket ou névoa. Sem nada em aberto, **feche o mapa**.

Dois mapas pedem trabalho a mais na sessão deles:

- **Arquitetura:** crie também o `GLOSSARY.md` de partida pelo `/domain-modeling`, a partir do
  código e da conversa.
- **Regras de código:** rode antes o `/improve-codebase-architecture` e escreva
  `docs/regras-de-codigo.md` com as regras que o código e o lint já seguem de fato; o atrito que ele
  achar vira ticket ou névoa, não regra inventada.

Pronto, por mapa, quando ele existe e cada linha de *Decisions so far* aponta uma ADR, um documento
ou um ticket fechado.

## 5. Triage das issues abertas

`/triage` em lotes de até 10 issues por sessão, cada lote decidido pelo humano. A régua é a definição
de pronto (`docs/agents/caminho-ate-ready-for-agent.md`): cumpre → `ready-for-agent` ou
`ready-for-human`; falta algo → `needs-info` ou `needs-triage`; perdeu o sentido → `wontfix`. Issue
antiga só entra na orquestração depois daqui.

Pronto quando nenhuma issue aberta está sem label de estado.

## 6. Passagem para planejamento

Mostre ao humano, com evidência, cada condição:

- os cinco mapas iniciais existem;
- Protótipo, Arquitetura, Fases e Regras de código estão fechados;
- `docs/regras-de-codigo.md` existe;
- o `GLOSSARY.md` existe;
- o passo 5 está pronto.

Com as cinco certas, ele faz a PR que muda `etapaDeAdocao` para `planejamento`. Pronto quando ela
está mergeada; o próximo ramo é [PLANEJAMENTO.md](PLANEJAMENTO.md).
