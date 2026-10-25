# A fila de issues roda numa sessão orquestradora, com subagentes em sequência e papéis separados

Substitui a ADR 0004. O loop por script abria uma sessão nova do `claude -p` para cada issue, e a
mesma sessão implementava e julgava o próprio trabalho. Queríamos o oposto: quem implementa não é
quem aprova. Escolhemos uma **sessão orquestradora**, aberta pelo kickoff do `/nova-sessao`, que
dispara subagentes **em sequência** com papéis separados: implementador, revisor (eixo *Standards*
do `/code-review`), conferente (eixo *Spec*) e diagnosticador nos ciclos de correção. Em paralelo,
como no `/implement-spec` do Matt, as sessões disputam a memória das máquinas da equipe. Por isso,
um subagente por vez. Cada subagente tem janela própria, e os hooks o identificam pelo `agent_id`
([documentação de hooks](https://code.claude.com/docs/en/hooks)). Assim os watchdogs continuam
valendo, com uma issue ativa por implementador em vez de por sessão.

**Consequências:** o trabalho da rodada acontece numa **branch de integração** escolhida pelo
desenvolvedor. Cada issue parte dela e volta por merge depois de aprovada, e no fim da fila o
desenvolvedor decide entre uma PR única e PRs por issue. Merge na `main` nunca é do agente. Cada
issue tem até dois ciclos de correção, e duas issues devolvidas seguidas param a rodada. Todos os
papéis usavam `opus` (a [ADR 0016](0016-sonnet-na-primeira-implementacao-e-na-conferencia.md) tirou dele o `conferente` e a 1ª implementação), em esforço `low` no primeiro ciclo e `medium` nas correções; o diagnosticador,
que abre cada correção, já começa em `medium`. O orquestrador passa a rodada adiante no aviso de 450k da smart zone; as outras sessões
seguem com o limite duro de 550k. Os papéis ficam em `.claude/agents/`, protegidos como os watchdogs.
