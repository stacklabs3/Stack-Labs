# Orquestração de issues: a fila implementada com revisão

Quando as issues chegam a `ready-for-agent`, o agente recomenda a **orquestração**: numa janela
própria, um **orquestrador** implementa a fila **uma issue de cada vez**: cada issue ganha uma sessão própria, em que
papéis de uma tarefa só (implementador, revisor, conferente e diagnosticador) a levam até o fim. Cada issue entra na **branch de integração**
só com revisor e conferente aprovando, e tem até dois **ciclos de correção**; passou disso, volta para
triagem com os pareceres. Papéis e ciclo:
[docs/agents/orquestracao.md](../agentes/modelo-de-projeto/docs/agents/orquestracao.md). O porquê do
desenho: [ADR 0005](../adr/0005-orquestracao-de-issues-com-subagentes.md) e
[ADR 0016](../adr/0016-uma-sessao-por-issue-e-trava-da-maquina.md).

## Como usar

1. No fim do planejamento (`/to-tickets` ou `/triage`), o agente gera o kickoff com o `/nova-sessao`.
   Abra uma janela nova e cole. Num projeto que está
   [adotando o método](adotar-em-projeto-existente.md), a orquestração começa na etapa de planejamento.
2. O orquestrador te pergunta **quais issues** entram, **o nome da branch de integração**
   (`integracao/<nome>`) e a base dela, e avisa quais issues vão pedir sua autorização (navegador,
   conector, computador). Responda e autorize já: pedida no meio da rodada, ela para a issue até você
   voltar.
3. Ele trabalha sozinho. A cada issue aprovada, ele diz onde o código está em três linhas: a branch
   da issue, a de integração e a `main` (que só recebe tudo no fim, pela PR). Se a janela dele encher, ele mesmo gera o kickoff para continuar numa janela
   nova, do ponto onde parou.
4. No fim da fila, ele mostra a tabela da rodada e pergunta como levar à `main`: **uma PR só** com
   tudo, ou **uma PR por issue**, na ordem. O merge é sempre seu.

## Quando ele para e te chama

- Acabou a fila, ou não há mais issue livre e desbloqueada.
- **Duas issues seguidas** foram devolvidas: algo comum a elas precisa de você.

## Mais de uma fila ao mesmo tempo

Filas que não mexem nos mesmos arquivos podem rodar juntas: abra **um orquestrador por fila**, cada um
na sua janela, com a sua branch de integração. Os comandos pesados se revezam sozinhos: um por vez em
cada projeto (quem espera mostra o pid e o comando de quem está rodando), e projetos diferentes
rodam juntos até as vagas de memória da máquina (2 em 8 GB). Para
mudar as vagas, defina `STACKLABS_VAGAS` na máquina.

## Preparar a máquina para dois projetos

Uma vez por máquina, antes de rodar dois produtos juntos:

1. **Linux: ligue o swap.** Sem ele, faltar memória não deixa a máquina lenta, mata processos,
   inclusive o shell do agente. Confira com `swapon --show`; vazio, crie um de 8 GB:

   ```bash
   sudo fallocate -l 8G /swapfile && sudo chmod 600 /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile
   ```

   ```bash
   echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
   ```

   No Mac, pule: o macOS já comprime a memória e usa swap sozinho.
2. **Linux: confira o limite de memória por vaga.** Num projeto com o método, rode
   `sh .claude/watchdogs/rodar pesado true`. Sem nenhuma linha `[pesado] sem limite de memória`, cada
   comando pesado roda limitado ao tamanho da vaga e, se passar dela, morre sozinho sem derrubar o
   resto. Com o aviso, a máquina não tem systemd de usuário com o controlador de memória
   (comum no WSL sem systemd e em containers): os comandos rodam só com a trava e as vagas.
3. **Limite os workers em cada projeto**, na configuração dele: `experimental.cpus: 2` no
   `next.config`, `maxWorkers: 2` no Jest e no Vitest, `workers: 2` no Playwright. É isso, não o
   `--max-old-space-size`, que segura o pico de um build.

As vagas saem da memória total (2 em 8 GB, 5 em 16 GB). Para mudar só na sua máquina:
`STACKLABS_VAGAS` (número de vagas) e `STACKLABS_MEMORIA_POR_VAGA_MB` (tamanho de cada uma).

## Bons hábitos

- Issue boa para a orquestração tem critério de aceite claro e cabe numa sessão.
- Revise a branch de integração antes de levar à `main`: três agentes aprovando não substituem você.
- O comentário `## Contrato da fatia` de cada issue diz o que o agente entendeu antes de escrever.
  Uma linha errada ali é mais barata de corrigir do que um ciclo de correção depois.
