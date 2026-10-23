# Adotar o método num projeto que já está rodando

Um projeto em andamento não entra no método de uma vez: os watchdogs travariam o time no mesmo dia,
porque exigem mapas, issues prontas e branches no padrão que ele ainda não tem. Por isso a adoção
anda em **três etapas**, conduzidas pela skill **`/adotar-metodo-stack-labs`**
([ADR 0006](../adr/0006-adocao-por-etapas.md)).

## As etapas

| Etapa | Como fica o dia a dia |
|---|---|
| **Observação** | Os agentes já seguem o método, mas os watchdogs só avisam. Só protegem a si mesmos e a `main` |
| **Planejamento** | Mapas, issues e branches passam a ser cobrados. O código ainda não exige issue ativa |
| **Completo** | O projeto segue o método inteiro, como um projeto novo |

O que já estava em andamento (branches e PRs abertas antes da adoção) termina do jeito antigo.

## O caminho

1. **Diagnóstico:** o agente mede o projeto contra o método e lista o que ele já tem.
2. **Plano:** você decide o que fica fora (o trabalho em andamento) e o destino de regras próprias
   que o projeto já tinha. No appetito_v2, por exemplo, os watchdogs próprios dão lugar aos do método,
   e o que só eles faziam vira regra do projeto ou proposta para o método.
3. **Instalação em observação:** uma PR com o método instalado. Você faz o merge.
4. **Mapas iniciais retroativos:** uma sessão por mapa. Eles registram as decisões que o projeto
   **já tomou**, com as ADRs que você confirmar. O que ninguém decidiu de verdade vira ticket.
5. **Triage das issues abertas**, em lotes de até 10: só passam a `ready-for-agent` as que cumprem a
   [definição de pronto](caminho-ate-ready-for-agent.md).
6. **Passagem para planejamento** e, depois, **para completo**: o agente mostra que as condições
   estão cumpridas, e você muda a etapa por PR.

Rode `/adotar-metodo-stack-labs` no projeto: ele descobre em que ponto a adoção está e continua dali.
