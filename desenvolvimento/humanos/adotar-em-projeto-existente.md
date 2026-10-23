# Adotar o método num projeto que já está rodando

Um projeto em andamento entra no método em **três etapas**, conduzidas pela skill
**`/adotar-metodo-stack-labs`**, para os watchdogs não travarem o time no primeiro dia
([ADR 0006](../adr/0006-adocao-por-etapas.md)). O que cada etapa bloqueia, item por item:
[docs/agents/adocao.md](../agentes/modelo-de-projeto/docs/agents/adocao.md).

## As etapas

| Etapa | Como fica o dia a dia |
|---|---|
| **Observação** | Os agentes já seguem o método, mas os watchdogs só avisam. Só protegem a si mesmos e a `main` |
| **Planejamento** | Mapas, issues e branches passam a ser cobrados. O código ainda não exige issue ativa |
| **Completo** | O projeto segue o método inteiro, como um projeto novo |

O que já estava em andamento (branches e PRs abertas antes da adoção) termina do jeito antigo.

## O caminho

Rode `/adotar-metodo-stack-labs` no projeto: ele descobre em que ponto a adoção está e continua dali.

1. **Diagnóstico:** o agente mede o projeto contra o método e lista o que ele já tem.
2. **Plano:** você decide o que fica fora (o trabalho em andamento) e o destino das regras próprias
   que o projeto já tinha. No appetito_v2, por exemplo, os watchdogs próprios dão lugar aos do método,
   e o que só eles faziam vira regra do projeto ou proposta para o método.
3. **Instalação em observação:** uma PR com o método instalado. Você faz o merge.
4. **Mapas iniciais retroativos:** uma sessão por mapa. Você confirma as ADRs das decisões que o
   projeto **já tomou**; o que ninguém decidiu de verdade vira ticket. O de Regras de código começa
   pelo `/improve-codebase-architecture` e escreve em `docs/regras-de-codigo.md` as regras que o
   código já segue de fato. O E1 (todo nome diz o que é) entra sempre; nome curto no código antigo
   vira ticket, e o validador só cobra o que muda.
5. **Triage das issues abertas**, em lotes de até 10: só passam a `ready-for-agent` as que cumprem a
   [definição de pronto](caminho-ate-ready-for-agent.md).
6. **Passagem para planejamento** e, depois, **para completo**: o agente mostra que as condições
   estão cumpridas, e você muda a etapa por PR, com a label `altera-watchdogs`.
