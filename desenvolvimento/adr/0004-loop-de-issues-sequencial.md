# Issues sem ninguém olhando rodam em sequência, uma sessão nova por issue

Para implementar várias issues `ready-for-agent` sem um humano olhando, havia dois caminhos do Matt
Pocock: o `/implement-spec`, em que um orquestrador lança subagentes em paralelo, cada um numa
worktree, e o padrão Ralph, um laço em shell que abre uma sessão nova do `claude -p` para cada tarefa,
uma de cada vez. Escolhemos o **laço em sequência**. Em paralelo, as sessões disputam a memória da
máquina, e notebooks e PCs da equipe travaram várias vezes assim. Além disso, uma sessão por issue é
o que o watchdog `guarda-bash` já exige (uma issue ativa por sessão): o orquestrador com subagentes
roda tudo na mesma sessão e seria barrado na segunda issue.

**Consequências:** o motor é `.claude/loop/loop-de-issues.sh`, que um humano dispara num terminal. A
smart zone também vale dentro do loop: a sessão que bate no teto deixa um handoff e uma sessão nova
continua a mesma issue (até 4 por issue). Duas issues seguidas sem PR param a rodada. O merge
continua sendo do humano, e uma issue bloqueada só entra depois do merge da que a bloqueia. Rodar em
paralelo exige reabrir esta ADR e mudar a regra de uma issue por sessão.
