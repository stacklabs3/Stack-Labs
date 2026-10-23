# Princípios de código da casa: o ponto de partida das regras de cada projeto

A ADR 0007 deu a cada projeto o seu `docs/regras-de-codigo.md` e recusou um guia de estilo único da
casa, porque as regras dependem da stack e do time. Faltava o outro lado: cada mapa de Regras de
código começava do zero, só com duas linhas da constituição e os *smells* do `/code-review`, e o
mesmo debate (o que é SOLID aqui, quando criar interface, como tratar erro) se repetia em todo
projeto. Havia três caminhos: seguir sem base, transformar a base em regra dura da casa, ou ter uma
base de **princípios** que cada projeto adota, adapta ou afasta. Escolhemos a terceira: o
`docs/agents/principios-de-codigo.md`, instalado pelo `modelo-de-projeto`, com a escrita de código
(E1 a E8), os padrões de desenvolvimento (P1 a P9) e SOLID lido com o vocabulário do
`/codebase-design`, para que ele não produza módulos rasos.

**Consequências:** a base é princípio, não estilo: vale para qualquer linguagem, cada item tem o
porquê, um exemplo e o que o revisor procura no diff, e cita a fonte primária. O mapa de Regras de
código passa por ela no grilling, item por item, e `docs/regras-de-codigo.md` continua sendo a régua
dura: cada regra dele cita o ID da base de onde nasceu. O que o projeto não adotou é julgamento para
o revisor, como os *smells*: só reprova com dano concreto. A precedência é `docs/regras-de-codigo.md`,
depois as ADRs do projeto, depois a base, depois os *smells*. Mudar a base é por PR no Stack-Labs,
como todo o método, e chega aos projetos pelo `/setup-stacklabs`.
