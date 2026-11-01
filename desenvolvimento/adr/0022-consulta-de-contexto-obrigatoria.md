# A consulta de contexto é obrigatória antes de ler código

Emenda a [ADR 0018](0018-busca-deterministica-e-ligacoes-do-metodo.md), que deixava a consulta
`rodar contexto` como item da rota, sem nada que barrasse quem a pulava. Em 2026-11-01, a partir do
Excurfly, o Matheus decidiu: "temos que fazer isso como obrigatoriedade sempre". A decisão foi tomada
sem a medição que a ADR 0018 pede a cada passo; a medição fica para depois, com `rodar custo`.

**Consequências:** a regra 23 e o `guarda-contexto` bloqueiam `Read`, `Grep` e `Glob` em código, e o
`guarda-bash` bloqueia `cat`, `head`, `tail`, `less`, `sed`, `grep` e `rg`, até a sessão ou o subagente
rodar uma consulta. Uma consulta basta para cada um, e a primeira leitura fora dos arquivos citados
recebe um aviso que não bloqueia: bloquear por trecho travaria o trabalho quando a resposta é parcial.
Resposta vazia ou insuficiente libera a leitura direta. Passam livres os caminhos da regra 3, documento
e configuração. A consulta enxerga `.claude/skills/`, porque no Excurfly `rodar contexto
orquestrar-issues` voltou vazia com a skill presente. O bloqueio vale a partir da etapa `planejamento`.
O `guarda-bash` lê só o texto do comando, então leitura dentro de script escapa, e o registro acontece
antes de a consulta rodar. Granularidade e caminhos livres são propostas ainda a confirmar pelo Matheus.
