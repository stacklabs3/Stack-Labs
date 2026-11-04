# Watchdogs: o que barra e por quê

Os watchdogs garantem que o método é seguido, por agentes e por pessoas. Um bloqueio é o método
dizendo qual é o próximo passo. O que cada watchdog garante, tecnicamente:
[docs/agents/watchdogs.md](../agentes/modelo-de-projeto/docs/agents/watchdogs.md).

## Quem vigia

Três camadas barram; o `/auditar-metodo` confere depois.

| Camada | Vigia |
|---|---|
| Hooks do Claude Code | O agente, durante a sessão |
| `pre-push` do git | Você e o agente, na sua máquina: a `main` e o CI local |
| `vigia-do-metodo` (CI) | Toda PR, de qualquer pessoa |
| `/auditar-metodo` | O histórico do projeto inteiro, quando você pedir |

## O que cada bloqueio quer dizer

| Mensagem | O que fazer |
|---|---|
| "faltam mapas iniciais" | Abra os mapas que faltam com `/wayfinder` |
| "mapas iniciais ... ainda estão abertos" | Feche Protótipo (desenho inicial), Arquitetura, Fases e Regras de código antes de implementar |
| "exige uma issue ativa" | Ative uma issue `ready-for-*` com `sh .claude/watchdogs/rodar ativar-issue <n>`, ou passe por `/to-tickets` |
| "ainda não tem o contrato da fatia" | O agente publica o comentário `## Contrato da fatia` na issue antes de escrever código. Você não faz nada, a não ser ler o contrato se quiser |
| "quebra regra de código" (`vigia-codigo`) | O agente acabou de editar um arquivo que quebra uma regra do `docs/regras-de-codigo.md`; a mensagem diz qual. Ele corrige na hora |
| "exige a consulta de contexto" | O agente roda `sh .claude/watchdogs/rodar contexto <consulta>` antes de abrir código; resposta vazia libera a leitura. Você não faz nada |
| "seria fechado sem ## Fontes" | A decisão precisa citar a pesquisa ou a fonte primária |
| "não foi reivindicado" | O ticket precisa de responsável antes do trabalho |
| "o comando ativa mais de uma issue" | Uma issue por sessão: ative uma e abra outra janela para a próxima |
| "esta sessão já resolveu" | Um ticket de decisão por sessão: abra outra |
| "contexto ... acima da smart zone" | `/nova-sessao` e cole o kickoff numa janela nova |
| "push direto para main" | Suba a branch e abra PR |
| "a PR não fecha a issue da branch" | Acrescente `Closes #<n>` no corpo da PR |
| "a PR altera os watchdogs" | Atualização do método: o agente roda o `--concluir-pr`. Outra mudança: um sócio revisa e aplica a label `altera-watchdogs` |
| "tem teste sem valor" (`vigia-teste`) | O agente escreveu teste sem risco do `docs/riscos.md`, sem afirmação ou desligado. Ele corrige; se faltar risco no registro, ele te pergunta, e quem aprova é você |
| `VALIDADOR: REPROVADO` | O código quebra uma regra de código; a lista diz qual, onde e de que princípio ela nasceu |
| "o CI local reprovou" (`pre-push`) | Lint, typecheck ou validador vermelho na sua branch. A saída acima diz qual regra e onde; corrija e suba de novo |

O **validador** de regras de código também barra: ele roda no CI do projeto e na revisão de cada
issue, e reprova o que quebra uma regra que a máquina consegue conferir
([ADR 0010](../adr/0010-validador-de-regras-de-codigo.md)).

Sem rede ou sem `gh` autenticado, os watchdogs que consultam o GitHub bloqueiam: confira o
`gh auth status`.

## O CI local no push

Todo `git push` roda o CI local do projeto antes de subir
([ADR 0015](../adr/0015-ci-local-com-revisao-por-modelo.md)): lint e typecheck, o validador e, se o
projeto ligou a regra `revisao-por-modelo`, uma revisão dos testes alterados por um modelo barato
(o Haiku). Você vê três blocos, `[ci] 1/3`, `2/3` e `3/3`, e a última linha `CI: APROVADO` ou
`CI: REPROVADO`. Só os dois primeiros barram o push. No push, a revisão por modelo fica de fora (rode
`sh .claude/watchdogs/rodar ci` para vê-la), e um conteúdo que já passou no CI contra a mesma base
sobe sem rodar de novo ([ADR 0025](../adr/0025-trava-por-projeto-e-vagas-de-memoria.md)).

A revisão por modelo só aconselha. Uma linha como
`⚠️  tests/cupom.rs:3 rk2_recusa_cupom_vencido (RK2): não exercita o risco que cita.` diz que o teste
cita o risco mas passaria mesmo se ele acontecesse. Ela não barra: quem decide é o revisor da issue.

Para ela rodar, o `claude` do terminal precisa estar logado (`claude auth login`). Sem ele, o CI
avisa e segue sem a revisão. Ela usa o seu plano, sem chave de API: cerca de US$ 0,001 e 3 s por
teste novo, e o mesmo teste não é perguntado de novo.

## Ativar o pre-push

Uma vez por clone, se o `sessao-inicio` avisar que ele está inativo:

```bash
git config core.hooksPath .claude/githooks
```

## Escapes (só humanos, só em emergência)

- Desligar os hooks numa sessão: `STACKLABS_WATCHDOGS=desligado claude`.
- Push direto na `main`: `STACKLABS_PERMITIR_PUSH_MAIN=1 git push origin main`.

Usou um escape? Avise os sócios e registre o motivo na issue ou na PR. O `/auditar-metodo` vai achar.

## Projeto que está adotando o método

Os watchdogs ligam em etapas: [adotar-em-projeto-existente.md](adotar-em-projeto-existente.md).

## Atualizar os watchdogs de um projeto

Rode `/setup-stacklabs` no projeto. O resto é do agente: ele confere se a skill está em dia, atualiza
os watchdogs, valida, abre a PR e faz o merge. Ele só instala da fonte oficial, e a PR fica para você
quando traz algo que não veio de lá, quando ele mesclou um arquivo à mão ou quando o CI não fica verde
([ADR 0024](../adr/0024-atualizacao-do-metodo-sem-humano.md)).

Voltar a uma versão anterior é só seu, porque afrouxa os watchdogs:
`sh ~/.claude/skills/setup-stacklabs/rodar --versao <x.y.z> --permitir-regressao`.

Projeto que entrou no método antes do mapa de Regras de código
([ADR 0007](../adr/0007-mapa-de-regras-de-codigo.md)) passa a ver "faltam mapas iniciais" depois da
atualização. Abra o mapa como **mapa retroativo**.

## Mudar um watchdog

Por PR no Stack-Labs (`desenvolvimento/agentes/modelo-de-projeto/`), depois propague para os
projetos. Mudança só num projeto é exceção dele e fica escrita no `AGENTS.md` desse projeto.
