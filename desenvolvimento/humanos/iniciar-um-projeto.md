# Checklist: começar um projeto

Marque na ordem. Cada passo diz quem faz.

## Dia 0: o repositório

- [ ] **Você:** crie o repositório no GitHub da StackLabs e clone ao lado do Stack-Labs.
- [ ] **Você:** tenha a skill `/setup-stacklabs` instalada na sua máquina
      ([instruções](../agentes/README.md#instalar-num-projeto-novo)).
- [ ] **Agente:** rode `/setup-stacklabs` no projeto. Ele mostra o que vai fazer, espera o seu sim,
      instala o método e as skills e abre a PR `chore/setup-metodo`. Faça o merge.
- [ ] **Você:** abra uma janela nova do Claude Code com o kickoff que o `/setup-stacklabs` gerou e
      confira o aviso do `sessao-inicio`: setup ✅ e pre-push ✅. Os mapas iniciais aparecem como ❌;
      é o esperado.

## Dia 1: os mapas iniciais

Uma sessão por mapa, com `/wayfinder`. Em cada uma, o agente faz o grilling do destino, depois o
grilling amplo, cria o mapa e os tickets e dispara as pesquisas. Você responde.

- [ ] **Mapa de Protótipo**: primeiro ticket é criar o protótipo único com `/novo-prototipo`.
- [ ] **Mapa de Arquitetura**: primeiro ticket são as restrições do projeto.
- [ ] **Mapa de Fases**: bloqueado pelas decisões de Arquitetura e pelas rotinas do Protótipo.
- [ ] **Mapa de Infra**: leve e informativo; não trava a implementação.
- [ ] **Você:** confira no GitHub que cada mapa tem as labels `wayfinder:map` e `mapa-inicial:<tipo>`.

## Semanas seguintes: resolver os mapas

- [ ] Uma sessão por ticket: `/wayfinder <mapa>` pega o próximo da fronteira.
- [ ] Pesquisas rodam sozinhas; você lê o resumo antes do grilling que depende delas.
- [ ] Protótipos em lote: você aprova por escrito.
- [ ] Mapa sem nada a decidir → `/to-spec` → `/to-tickets`.
- [ ] Protótipo (desenho inicial), Arquitetura e Fases fechados: a implementação está liberada.

## Durante todo o projeto

- [ ] Entre uma sessão e a próxima, `/nova-sessao`; assunto paralelo numa sessão satélite, de volta
      com o `/voltar-a-principal`.
- [ ] Uma sessão por issue de implementação, PR com `Closes #<n>`. Com issues
      `ready-for-agent` prontas, abra a orquestração (`/orquestrar-issues`) e revise as PRs do fim da fila.
- [ ] `/auditar-metodo` a cada marco e antes de entregar ao cliente.
- [ ] `/retro` ao fim de cada fase: o que ajustar no método vira PR no Stack-Labs.
