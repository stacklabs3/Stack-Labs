# Sonnet na 1ª implementação e na conferência, Opus no julgamento e nas correções

A ADR 0005 pôs todos os papéis da orquestração em `opus`. Rodando as filas do excurfly, o limite
semanal do plano passou a ser o gargalo. Havia três caminhos: manter tudo em Opus, trocar tudo por
Sonnet, ou **dividir por papel**. O humano escolheu o terceiro em 2026-10-25: a 1ª implementação de
cada issue e a conferência (o eixo *Spec*, que compara o diff com a issue) vão para o Sonnet, e o
que exige julgar ou consertar fica no Opus: o `revisor`, o `diagnosticador` e as correções do
`implementador`.

**Consequências:** `conferente.md` e `implementador.md` passam a `model: sonnet`; `revisor.md` e
`diagnosticador.md` continuam `model: opus`. Como o `implementador` é um arquivo só, o
`/orquestrar-issues` o chama com `model: "opus"` nos ciclos de correção; a tabela de modelo e esforço
por ciclo da skill é a fonte. O esforço não muda. O risco é a 1ª implementação em Sonnet gerar mais
ciclos de correção; quem mede é a tabela da rodada (ciclos por issue) ao fim de cada fila, e voltar
ao Opus é trocar uma linha. Os agentes instalados nos projetos mudam quando o `/setup-stacklabs`
atualiza o `.claude/agents/`.
