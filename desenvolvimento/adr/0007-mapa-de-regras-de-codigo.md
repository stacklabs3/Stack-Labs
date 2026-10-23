# Regras de código é o quinto mapa inicial, e trava a implementação

Os quatro mapas iniciais decidiam o que construir (Protótipo), com o quê (Arquitetura), em que ordem
(Fases) e onde roda (Infra), mas não **como o código é escrito**. Cada sessão de implementação
decidia isso sozinha, e o revisor da orquestração só tinha a lista genérica de smells do
`/code-review` e uma regra solta de nomes na constituição. Havia três caminhos: deixar as regras
nascerem nas fatias, ter um guia de estilo único da casa para todo projeto, ou um mapa por projeto.
Escolhemos um **mapa inicial de Regras de código**, com a label `mapa-inicial:regras-de-codigo`,
porque as regras dependem da linguagem e da stack que a Arquitetura escolheu e do time que vai
manter, e porque regra decidida depois do primeiro código vira refatoração.

**Consequências:** o mapa fecha com `docs/regras-de-codigo.md` escrito e aprovado: cada regra com o
porquê e um exemplo do projeto, e o que o lint garante marcado como tal. Ele depende da Arquitetura e
trava a implementação como Protótipo, Arquitetura e Fases (o `guarda-bash` barra). O grilling usa
`/human-architect-mindset`, `/domain-modeling` e `/codebase-design`, e `/improve-codebase-architecture`
quando já existe código. Lint e formatador passam por pesquisa; configurá-los é a primeira fatia da
primeira fase, não ticket do mapa. O implementador e o revisor da orquestração leem o documento, e o
revisor reprova a fatia que quebra uma regra dele. Projeto que já seguia o método passa a ver o mapa
faltando depois de atualizar os watchdogs e o abre como **retroativo** (ADR 0006): registra as regras
que o código já segue e fecha na hora quando nada está em aberto.
