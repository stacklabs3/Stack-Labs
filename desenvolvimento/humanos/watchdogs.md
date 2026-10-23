# Watchdogs: o que barra e por quê

Os watchdogs garantem que o método é seguido, por agentes e por pessoas. Um bloqueio não é erro: é o
método dizendo qual é o próximo passo. A lista técnica completa fica em
[docs/agents/watchdogs.md](../agentes/modelo-de-projeto/docs/agents/watchdogs.md).

## As três camadas

| Camada | Vigia | Onde |
|---|---|---|
| Hooks do Claude Code | O agente, durante a sessão | `.claude/hooks/` |
| `pre-push` do git | Você e o agente, na sua máquina | `.claude/githooks/` |
| `vigia-do-metodo` | Toda PR, de qualquer pessoa | `.github/workflows/` |
| `/auditar-metodo` | O histórico do projeto inteiro, quando você pedir | skill |

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

## Escapes (só humanos, só em emergência)

- Desligar os hooks numa sessão: `STACKLABS_WATCHDOGS=desligado claude`.
- Push direto na `main`: `STACKLABS_PERMITIR_PUSH_MAIN=1 git push origin main`.

Usou um escape? Avise os sócios e registre o motivo na issue ou na PR. O `/auditar-metodo` vai achar.

## Projeto que está adotando o método

Num projeto que já existia, os watchdogs ligam em etapas: na **observação** quase tudo só avisa, no
**planejamento** passam a cobrar mapas, issues e branches, e no **completo** vale tudo. Veja
[adotar-em-projeto-existente.md](adotar-em-projeto-existente.md).

## Atualizar os watchdogs de um projeto

Rode `/setup-stacklabs` no projeto. O que for watchdog ele não sobrescreve sozinho: entrega a você o
comando com `--atualizar-watchdogs`, que só você roda.

Projeto que entrou no método antes do mapa de Regras de código
([ADR 0007](../adr/0007-mapa-de-regras-de-codigo.md)) passa a ver "faltam mapas iniciais" depois da
atualização. Abra o mapa como retroativo: ele registra as regras que o código já segue e fecha na
hora quando nada está em aberto.

## Mudar um watchdog

Por PR no Stack-Labs (`desenvolvimento/agentes/modelo-de-projeto/`), depois propague para os
projetos. Mudança só num projeto é exceção dele e fica escrita no `AGENTS.md` desse projeto.
