# Checklist: começar um projeto

Marque na ordem. Cada passo diz quem faz.

## Dia 0: o repositório

- [ ] **Você:** crie o repositório no GitHub da StackLabs e clone ao lado do Stack-Labs.
- [ ] **Você:** copie o modelo de projeto e ative o pre-push
      ([instruções](../agentes/README.md#instalar-num-projeto-novo)).
- [ ] **Você:** crie as labels do método.
- [ ] **Você:** instale as skills do Matt Pocock, a família `*-prototipo` e o `/auditar-metodo`.
- [ ] **Você:** abra uma sessão do Claude Code e confira o aviso do `sessao-inicio`: setup ✅ e
      pre-push ✅. Os mapas iniciais aparecem como ❌; é o esperado.

## Dia 1: os mapas iniciais

Uma sessão por mapa, com `/wayfinder`. Em cada uma, o agente faz o grilling do destino, depois o
grilling amplo, cria o mapa e os tickets e dispara as pesquisas. Você responde.

- [ ] **Mapa de Protótipo**: primeiro ticket é criar o protótipo único com `/novo-prototipo`.
- [ ] **Mapa de Arquitetura**: primeiro ticket são as restrições do projeto.
- [ ] **Mapa de Fases**: bloqueado pelas decisões de Arquitetura e pelas rotinas do Protótipo.
- [ ] **Mapa de Infra**: bloqueado pela stack.
- [ ] **Você:** confira no GitHub que cada mapa tem as labels `wayfinder:map` e `mapa-inicial:<tipo>`.

## Semanas seguintes: resolver os mapas

- [ ] Uma sessão por ticket: `/wayfinder <mapa>` pega o próximo da fronteira.
- [ ] Pesquisas rodam sozinhas; você lê o resumo antes do grilling que depende delas.
- [ ] Protótipos em lote: você aprova por escrito.
- [ ] Mapa sem nada a decidir → `/to-spec` → `/to-tickets`.
- [ ] Arquitetura e Fases fechados: a implementação está liberada.

## Durante todo o projeto

- [ ] Uma sessão por issue de implementação, PR com `Closes #<n>`.
- [ ] `/auditar-metodo` a cada marco e antes de entregar ao cliente.
- [ ] `/retro` ao fim de cada fase: o que ajustar no método vira PR no Stack-Labs.
