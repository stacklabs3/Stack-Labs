# Orquestração de issues: a fila implementada com revisão

Quando o planejamento termina e as issues chegam a `ready-for-agent`, o agente recomenda a
**orquestração**. Ela roda numa janela própria: um agente **orquestrador** pega a fila e, **uma issue
de cada vez**, chama outros agentes, cada um com uma tarefa só. A
[ADR 0005](../adr/0005-orquestracao-de-issues-com-subagentes.md) explica o porquê do desenho.

## Quem faz o quê

| Agente | Papel |
|---|---|
| Implementador | Escreve o código da issue, com testes |
| Revisor | Confere se o código segue os padrões do projeto |
| Conferente | Confere se o código faz o que a issue e o spec pedem |
| Diagnosticador | Quando algo é reprovado, acha a causa e monta o plano de correção |

Se o revisor e o conferente aprovam, a issue entra na branch de integração e a fila anda. Se algum
reprova, o diagnosticador acha a causa, o implementador corrige e os dois conferem de novo. São até
**duas correções por issue**; depois disso, a issue volta para triagem com os pareceres.

## Como usar

1. No fim do planejamento (`/to-tickets` ou `/triage`), o agente recomenda a orquestração e gera um
   kickoff com o `/nova-sessao`. Abra uma janela nova e cole. Num projeto que está
   [adotando o método](adotar-em-projeto-existente.md), a orquestração só começa na etapa de planejamento.
2. O orquestrador te pergunta **quais issues** entram e **o nome da branch de integração**
   (`integracao/<nome>`) e a base dela. Responda.
3. Ele trabalha sozinho, uma issue por vez. Se a janela dele encher, ele mesmo gera o kickoff para
   continuar numa janela nova, do ponto onde parou.
4. No fim da fila, ele mostra a tabela da rodada e pergunta como você quer levar à `main`: **uma PR
   só** com tudo, ou **uma PR por issue**, na ordem. O merge é sempre seu.

## Quando ele para sozinho

- Acabou a fila, ou não há mais issue livre e desbloqueada.
- **Duas issues seguidas** foram devolvidas: algo comum a elas precisa de você.

## Bons hábitos

- Issue boa para a orquestração tem critério de aceite claro e cabe numa sessão.
- Revise a branch de integração antes de levar à `main`: três agentes aprovando não substituem você.
