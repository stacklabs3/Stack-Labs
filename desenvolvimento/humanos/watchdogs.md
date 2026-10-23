# Watchdogs: o que barra e por quê

Os watchdogs garantem que o método é seguido, por agentes e por pessoas. Um bloqueio é o método
dizendo qual é o próximo passo. O que cada watchdog garante, tecnicamente:
[docs/agents/watchdogs.md](../agentes/modelo-de-projeto/docs/agents/watchdogs.md).

## Quem vigia

Três camadas barram; o `/auditar-metodo` confere depois.

| Camada | Vigia |
|---|---|
| Hooks do Claude Code | O agente, durante a sessão |
| `pre-push` do git | Você e o agente, na sua máquina |
| `vigia-do-metodo` (CI) | Toda PR, de qualquer pessoa |
| `/auditar-metodo` | O histórico do projeto inteiro, quando você pedir |

## O que cada bloqueio quer dizer

| Mensagem | O que fazer |
|---|---|
| "faltam mapas iniciais" | Abra os mapas que faltam com `/wayfinder` |
| "mapas iniciais ... ainda estão abertos" | Feche Protótipo (desenho inicial), Arquitetura, Fases e Regras de código antes de implementar |
| "exige uma issue ativa" | Ative uma issue `ready-for-*` com `node .claude/hooks/ativar-issue.mjs <n>`, ou passe por `/to-tickets` |
| "seria fechado sem ## Fontes" | A decisão precisa citar a pesquisa ou a fonte primária |
| "não foi reivindicado" | O ticket precisa de responsável antes do trabalho |
| "o comando ativa mais de uma issue" | Uma issue por sessão: ative uma e abra outra janela para a próxima |
| "esta sessão já resolveu" | Um ticket de decisão por sessão: abra outra |
| "contexto ... acima da smart zone" | `/nova-sessao` e cole o kickoff numa janela nova |
| "push direto para main" | Suba a branch e abra PR |
| "a PR não fecha a issue da branch" | Acrescente `Closes #<n>` no corpo da PR |
| "a PR altera os watchdogs" | Um sócio revisa e aplica a label `altera-watchdogs` |

Sem rede ou sem `gh` autenticado, os watchdogs que consultam o GitHub bloqueiam: confira o
`gh auth status`.

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

Rode `/setup-stacklabs` no projeto. Os watchdogs ele não sobrescreve sozinho: entrega a você o
comando, que só você roda:

```bash
node ~/.claude/skills/setup-stacklabs/instalar.mjs --atualizar-watchdogs
```

Projeto que entrou no método antes do mapa de Regras de código
([ADR 0007](../adr/0007-mapa-de-regras-de-codigo.md)) passa a ver "faltam mapas iniciais" depois da
atualização. Abra o mapa como **mapa retroativo**.

## Mudar um watchdog

Por PR no Stack-Labs (`desenvolvimento/agentes/modelo-de-projeto/`), depois propague para os
projetos. Mudança só num projeto é exceção dele e fica escrita no `AGENTS.md` desse projeto.
