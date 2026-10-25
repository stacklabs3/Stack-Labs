# Custo de tokens medido por issue, com avisos que não bloqueiam

O método já segurava o contexto de uma sessão (smart zone, ADR 0004), mas ninguém sabia quanto uma
issue custava somando as sessões e os subagentes, nem onde o gasto ia: saída de ferramenta enorme
relida em todo turno e subagente de busca rodando no modelo da sessão. Para cada um havia três
caminhos: medir, avisar ou bloquear. O humano escolheu, em 2026-10-25: **medir o custo por issue e
avisar** ao passar do orçamento, **avisar depois** de uma saída grande e **avisar** quando um
subagente de busca sai sem `model`.

**Consequências:** três watchdogs novos, todos só de aviso ([watchdogs.md](../agentes/modelo-de-projeto/docs/agents/watchdogs.md)):
`vigia-custo` (Stop e SubagentStop), `vigia-saida` (PostToolUse) e `vigia-agente` (PreToolUse do
`Agent`); o `vigia-prompt` relembra o custo da issue que passou do orçamento. Bloquear foi descartado
porque trava o time no meio de uma entrega e porque o limite ainda não está calibrado. O custo mede
em tokens equivalentes de entrada, com os pesos da tabela de preços, para que leitura de cache barata
não pareça cara; o orçamento padrão é 3M, e o humano o ajusta em `.claude/metodo.json` depois de ver
as primeiras issues medidas com `rodar custo`. O relatório entra na Evidence da PR. Os hooks leem o
transcript a partir de onde pararam e contam cada mensagem uma vez (ela se repete em várias linhas
com o mesmo `id`); não chamam modelo nem rede. A escolha de modelo da sessão continua com o humano:
o hook só vê o subagente que está sendo lançado.
