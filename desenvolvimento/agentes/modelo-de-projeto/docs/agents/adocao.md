# Adoção do método num projeto que já existia

Projeto que existia antes do método entra nele por **etapas** (ADR 0006 do método), conduzidas pela
skill `/adotar-metodo-stack-labs`. A etapa fica em `.claude/metodo.json`, ao lado da data de adoção.
Sem esse arquivo, o projeto nasceu no método e vale tudo.

| Etapa | O que bloqueia | O que ainda não bloqueia |
|---|---|---|
| `observacao` | Mexer nos watchdogs, `--no-verify`, `core.hooksPath`, push forçado, push na `main`, contexto acima de 550k | Ativação de issue, regras do wayfinder e das fontes, commit na `main`, código sem issue ativa; no CI, as regras de branch e de issue |
| `planejamento` | Tudo de `observacao`, mais ativação de issue (com mapas iniciais), wayfinder (claim, fontes, um ticket por sessão, ponteiro no mapa), commit na `main` e as regras de branch e de issue no CI | Código sem issue ativa |
| `completo` | Tudo | — |

## Numa sessão de um projeto em adoção

- O `sessao-inicio` mostra a etapa. **O que ainda não bloqueia também vale:** a constituição é a
  mesma em toda etapa; a etapa só decide o que a máquina barra.
- PRs abertas antes da data de adoção terminam fora das regras de branch e de issue do CI.
- `.claude/metodo.json` é protegido. Mudar de etapa é uma PR do humano, com a label `altera-watchdogs`,
  depois que o `/adotar-metodo-stack-labs` mostrou que as condições da etapa seguinte estão cumpridas.
