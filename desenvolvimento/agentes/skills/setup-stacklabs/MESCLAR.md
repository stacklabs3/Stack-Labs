# Mesclar o modelo num arquivo que o projeto já tem

Para cada linha `MESCLAR <arquivo>` do relatório, o modelo novo está em `<arquivo>.stacklabs-novo`
ao lado do arquivo do projeto. O resultado fica no arquivo original e o `.stacklabs-novo` é apagado.

## `AGENTS.md`

O modelo é a base; o que o projeto tinha de próprio entra no fim, numa seção
`## Regras deste projeto`.

1. Copie o `.stacklabs-novo` inteiro como novo `AGENTS.md`.
2. Leve para `## Regras deste projeto` cada regra antiga que o modelo não cobre.
3. Regra antiga que **contradiz** o modelo não entra: liste-a para o humano, com a regra do modelo
   que ela contradiz, e pergunte se vira exceção registrada ou se cai.
4. Decisão antiga que não é regra de agente (escolha de stack, de arquitetura) vira ADR em
   `docs/adr/`, se for difícil de reverter, surpreendente sem contexto e resultado de uma troca real.

## `CLAUDE.md`

A primeira linha é `@AGENTS.md`. O resto do arquivo antigo fica abaixo dela, sem o que já está no
`AGENTS.md`.

## `.gitignore`

Acrescente ao arquivo do projeto as linhas do `.stacklabs-novo` que faltam. Nenhuma linha antiga sai.

## `.claude/settings.json`

Junte os dois JSONs: o bloco `hooks` vem do modelo; `permissions`, `env` e o resto vêm do projeto.
Hook do projeto que não é do método entra ao lado dos do modelo, no mesmo evento. Com watchdogs já
ativos, esse arquivo é protegido: entregue o JSON mesclado ao humano para ele gravar.

Pronto quando nenhum `.stacklabs-novo` resta e o humano respondeu às contradições do passo 3.
