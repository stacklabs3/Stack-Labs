# Changelog do método

Cada versão do método StackLabs, a mais nova primeiro. A versão vive em [VERSAO](VERSAO), segue
[SemVer](https://semver.org/lang/pt-BR/) e cada uma ganha a tag `v<versão>` no Stack-Labs
([ADR 0020](adr/0020-metodo-versionado-e-regras-por-projeto.md)).

- **Maior:** muda o que o projeto precisa fazer à mão na atualização (regra dura nova que reprova
  código existente, arquivo que muda de lugar, watchdog que passa a bloquear algo novo).
- **Menor:** regra, skill ou watchdog novo que o instalador aplica sozinho.
- **Correção:** texto, defeito de watchdog ou de skill, sem regra nova.

Toda PR que muda `desenvolvimento/agentes/` sobe a versão e acrescenta a entrada aqui.

## 2.1.1 (2026-11-01)

- Quem configura e quem audita acham o Dart: requisito e versão do preset no `dart.md`, Flutter e
  subpasta no `validador.md`, DT no mapa de Regras de código, e a `/auditar-metodo` vale para toda
  linguagem com arquivo de princípios.

## 2.1.0 (2026-11-01)

- O modelo Dart fica igual ao guia: caixa do E1, `avoid_print`, `implementation_imports` e as outras
  regras que o guia cita, DT5 e DT6 como `revisor`, `**/*.mocks.dart` fora do validador e do
  analisador, e a troca para `flutter analyze --fatal-infos` registrada.
- Uma receita só do OSV-Scanner, conferida na documentação da v2.7.0:
  `osv-scanner scan source --lockfile pubspec.lock`.

## 1.0.0 (2026-10-25)

Primeira versão marcada: o método como estava no commit `c7c1fa1`, mais:

- `desenvolvimento/VERSAO` e este changelog; o instalador grava a versão em
  `.claude/skills/ORIGEM.json`, mostra a mudança de versão no relatório (`VERSÃO`) e instala uma
  versão marcada com `--versao <x.y.z>`.
- O `AGENTS.md` do modelo traz a seção `## Regras deste projeto`, com `### Acrescentadas` e
  `### Afastadas`. Na atualização, o instalador troca o resto do arquivo pelo modelo novo e mantém a
  seção (`MESCLADO`), sem mesclagem à mão.
- Instalação: label que o `gh` não cria (sem login ou sem `origin`) vira `FALHA` no relatório em vez
  de abortar no fim com o projeto já copiado; fora de um repositório git, o erro diz o que fazer; o
  `cp` das skills de entrada não depende de `{a,b}` do shell, e a pasta `.claude/skills` é criada.
- Watchdogs configuráveis por projeto: campo `ajustes` de `.claude/metodo.json` com
  `caminhosLivres`, `branchesDescartaveis` e `contexto` (limites da smart zone).
- O agente não abre mais PR no Stack-Labs com regra específica de um projeto: a regra fica no projeto
  (`AGENTS.md`, `metodo.json` ou `regras-de-codigo.json`); levar ao método só a pedido do humano.
