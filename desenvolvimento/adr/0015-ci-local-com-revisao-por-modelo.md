# CI local no binário dos watchdogs, com uma revisão consultiva por modelo

Depois da ADR 0014, o que a máquina não via no P5 ficava todo com o revisor Opus da orquestração: o
teste que cita `RK<n>` e não exercita o risco. É o caso que a pesquisa
`desenvolvimento/research/2026-10-23-testes-com-valor.md` mais teme (Konstantinou et al., 2024: o
LLM escreve o oráculo a partir do comportamento atual do código), e o revisor só o via no fim da
issue. O validador, por sua vez, só rodava no CI do projeto quando alguém o configurava no mapa de
Regras de código. Havia três caminhos: CI no GitHub Actions, CI na raiz do Stack-Labs, ou **CI local
no binário dos watchdogs** do modelo de projeto, com um modelo barato lendo cada teste alterado antes
do revisor. O humano escolheu o terceiro: o Stack-Labs é guia, e o que governa o código de um projeto
chega a ele pelo `modelo-de-projeto`. O subcomando `sh .claude/watchdogs/rodar ci [--base <ref>]`
roda, em ordem, os `comando` de `docs/regras-de-codigo.json` (lint e typecheck), o `validar --base` e
a `revisao-por-modelo`; o `pre-push` o chama depois da regra do push na `main`. A revisão chama o
Haiku 5.5 pelo `claude -p`, do Rust, com uma chamada por teste alterado desde a base, em paralelo, e
uma resposta de esquema fixo (`exercita`, `motivo`). `--bare` ficou de fora por exigir
`ANTHROPIC_API_KEY`; `--setting-sources ""` e `STACKLABS_WATCHDOGS=desligado` impedem o filho de
disparar os hooks do projeto, `--tools ""` o deixa só lendo e `--max-budget-usd` põe teto por chamada.

**Consequências:** a revisão por modelo é **consultiva**: imprime os testes suspeitos com o motivo e
nunca reprova; o `ci` reprova só por lint, typecheck e validador. Ela não conta na `cobertura` (o P5
continua exigindo `teste-cobre-risco`), e o validador recusa a regra sem `"consultivo": true`. Falha
aberta: sem `claude` no PATH, sem rede ou sem login, avisa e segue, ao contrário dos watchdogs, que
falham fechados. O veredito fica em cache em `.claude/state/revisao-de-testes/`, pela soma SHA-256 do
teste, do risco, do modelo e do prompt: repetir não chama de novo, e o veredito não muda. Os hooks
nunca chamam o modelo, pela latência a cada edição. O revisor da orquestração lê a lista do Haiku
como ponto de partida, confirma ou descarta cada item, e o veredito continua dele. O `claude -p` usa
o login e o plano de quem roda, sem chave de API: medido em 2026-10-24, uma chamada leva uns 3 s e
custa US$ 0,001 no Haiku. O humano confirmou, em 2026-10-24, que isso cabe na regra "ferramenta que
não cobra a gente usa; a que cobra a gente escreve" (ADR 0010): o gasto sai do plano que a pessoa já
paga, sem chave de API nem cobrança à parte. A revisão vira bloqueante só depois de um piloto
medido contra teste de mutação e de outra ADR. Fica como próximo passo publicar o resultado do `ci`
como status `metodo/ci-local` no commit, pelo `gh api`.
