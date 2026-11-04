# Conferente em Sonnet; implementador, revisor e diagnosticador em Opus

Emenda a ADR 0005, que pôs todos os papéis da orquestração em `opus`. Rodando as filas do Excurfly, o
limite semanal do plano virou gargalo, e a PR 33 propôs dividir por papel: a 1ª implementação e a
conferência em Sonnet, o julgamento e as correções em Opus. Depois dela veio a ADR 0023: o
implementador do 1º ciclo passou a escrever o contrato da fatia e subiu para `medium`, porque o
contrato é leitura e julgamento, e um contrato fraco custa ciclos de correção.

O humano decidiu (2026-11-04) levar a divisão só até onde ela não desfaz a 0023: o `conferente` vai
para o Sonnet, em todos os ciclos. Ele compara o diff com a issue e com a coluna *Critérios de aceite*
do contrato, um trabalho de conferir lista contra lista. O `implementador`, o `revisor` e o
`diagnosticador` ficam no Opus.

**Consequências:** `conferente.md` passa a `model: sonnet`; a tabela de esforço do
`/orquestrar-issues` mostra o modelo de cada papel. O risco é o conferente deixar passar requisito
pela metade; quem mede é a tabela da rodada (issues que voltam por achado de *Spec* depois de
integradas). Voltar ao Opus é trocar uma linha. Os projetos recebem a mudança quando o
`/setup-stacklabs` atualiza `.claude/agents/`. A PR 33 fica fechada como superada por esta ADR.
