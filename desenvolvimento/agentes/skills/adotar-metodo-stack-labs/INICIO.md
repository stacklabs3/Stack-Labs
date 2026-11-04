# Início: diagnóstico, plano e instalação em observação

Três passos, um por sessão.

## 1. Diagnóstico

Só leitura. Meça o projeto contra **cada** critério de `../auditar-metodo/SKILL.md` e inventarie o
que ele já tem: `AGENTS.md` e `CLAUDE.md` próprios, hooks próprios com o que cada um garante, labels,
issues abertas, branches e PRs abertas, ADRs, glossário e protótipos.

Escreva `.scratch/adocao/diagnostico.md`. Pronto quando todo critério e todo item do inventário têm
status (✅, ⚠️ ou ❌) e evidência (caminho, link ou comando).

## 2. Plano

Grilling com o humano, as três perguntas numa rodada:

1. Quais branches e PRs abertas terminam fora do método (padrão: todas as do diagnóstico).
2. O destino de cada garantia dos hooks antigos que nenhum watchdog do modelo cobre (seção *Watchdogs
   que o projeto já tinha* de [MESCLAR.md](MESCLAR.md)).
3. Quem responde os grillings dos mapas retroativos.

Escreva `.scratch/adocao/plano.md` com cada resposta e os passos que faltam. Pronto quando as três
perguntas têm resposta do humano escrita no plano.

## 3. Instalação em observação

1. `sh <pasta da skill setup-stacklabs>/rodar --adocao --simular` e mostre o relatório.
2. Com o sim do humano, rode sem `--simular`. O `--adocao` grava `.claude/metodo.json` em
   `observacao`, com a data de hoje, antes de copiar os watchdogs.
   Hooks antigos que o projeto já tinha saem nessa hora: o humano precisa ter decidido as garantias
   deles no passo 2 ([MESCLAR.md](MESCLAR.md)).
3. Linhas `MESCLAR`: siga [MESCLAR.md](MESCLAR.md).
4. Branch `chore/adotar-metodo`, um commit, PR.

Pronto quando a PR da adoção está aberta e `find . -name '*.stacklabs-novo' -not -path './.git/*'`
não lista nada. O merge é do humano; depois dele, o próximo ramo é [OBSERVACAO.md](OBSERVACAO.md).
