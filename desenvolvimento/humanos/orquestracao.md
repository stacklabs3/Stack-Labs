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
   (`integracao/<nome>`) e a base dela. Responda.
3. Ele trabalha sozinho. Se a janela dele encher, ele mesmo gera o kickoff para continuar numa janela
   nova, do ponto onde parou.
4. No fim da fila, ele mostra a tabela da rodada e pergunta como levar à `main`: **uma PR só** com
   tudo, ou **uma PR por issue**, na ordem. O merge é sempre seu.

## Quando ele para e te chama

- Acabou a fila, ou não há mais issue livre e desbloqueada.
- **Duas issues seguidas** foram devolvidas: algo comum a elas precisa de você.

## Mais de uma fila ao mesmo tempo

Filas que não mexem nos mesmos arquivos podem rodar juntas: abra **um orquestrador por fila**, cada um
na sua janela, com a sua branch de integração. Os comandos pesados se revezam sozinhos numa trava da
máquina, para que dois builds ao mesmo tempo não derrubem tudo.

## Bons hábitos

- Issue boa para a orquestração tem critério de aceite claro e cabe numa sessão.
- Revise a branch de integração antes de levar à `main`: três agentes aprovando não substituem você.
- O comentário `## Contrato da fatia` de cada issue diz o que o agente entendeu antes de escrever.
  Uma linha errada ali é mais barata de corrigir do que um ciclo de correção depois.
