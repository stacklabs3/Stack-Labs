# Checklist: começar um projeto

Marque na ordem. Cada passo diz quem faz. Projeto que já está rodando começa pela
[adoção](adotar-em-projeto-existente.md).

## Dia 0: o repositório

- [ ] **Você:** tenha o Rust na máquina (`cargo --version`; sem ele, instale por https://rustup.rs).
      Os watchdogs são compilados nela na primeira vez que rodam ([ADR 0011](../adr/0011-watchdogs-em-rust.md)).
- [ ] **Você:** crie o repositório no GitHub da StackLabs e clone ao lado do Stack-Labs.
- [ ] **Você:** instale a skill `/setup-stacklabs` na sua máquina
      ([instruções](../agentes/README.md#instalar-num-projeto-novo)).
- [ ] **Você:** rode `/setup-stacklabs` no projeto. O agente mostra o que vai fazer, espera o seu sim,
      instala o método e as skills e abre a PR `chore/setup-metodo`. Faça o merge.
- [ ] **Você:** abra uma janela nova do Claude Code com o kickoff que o `/setup-stacklabs` gerou e
      confira o aviso do `sessao-inicio`: setup ✅ e pre-push ✅. Os mapas iniciais aparecem como ❌;
      é o esperado. Pre-push ❌: ative-o ([watchdogs.md](watchdogs.md#ativar-o-pre-push)).

## Dia 1: os mapas iniciais

Uma sessão por mapa, com `/wayfinder`: o agente faz o grilling, cria o mapa e os tickets e dispara as
pesquisas; você responde. O que cada mapa decide e de qual depende:
[mapas-iniciais.md](mapas-iniciais.md).

- [ ] Mapa de **Protótipo**
- [ ] Mapa de **Arquitetura**
- [ ] Mapa de **Fases**
- [ ] Mapa de **Regras de código**
- [ ] Mapa de **Infra**
- [ ] **Você:** confira no GitHub que cada mapa tem as labels `wayfinder:map` e `mapa-inicial:<tipo>`.

## Semanas seguintes: resolver os mapas

- [ ] Uma sessão por ticket: `/wayfinder <mapa>` pega o próximo da fronteira.
- [ ] Pesquisas rodam sozinhas; você lê o resumo antes do grilling que depende delas.
- [ ] Protótipos em lote: você aprova por escrito.
- [ ] Mapa sem nada a decidir → `/to-spec` → `/to-tickets`.
- [ ] Protótipo, Arquitetura, Fases e Regras de código fechados: a implementação está liberada.

## Durante todo o projeto

- [ ] Os [hábitos de sessão](README.md#hábitos-de-sessão): `/nova-sessao` entre sessões, sessão
      satélite para assunto paralelo.
- [ ] Com issues `ready-for-agent`, abra a [orquestração](orquestracao.md) e revise as PRs do fim
      da fila.
- [ ] `/auditar-metodo` a cada marco e antes de entregar ao cliente.
- [ ] `/retro` ao fim de cada fase: o que ajustar no método vira PR no Stack-Labs.
