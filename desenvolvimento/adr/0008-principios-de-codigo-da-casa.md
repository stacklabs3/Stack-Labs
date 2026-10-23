# Princípios de código da casa: o guia do grilling do mapa de Regras de código

A ADR 0007 deu a cada projeto o seu `docs/regras-de-codigo.md` e recusou um guia de estilo único da
casa, porque as regras dependem da stack e do time. Faltava o outro lado: cada mapa de Regras de
código começava do zero, só com duas linhas da constituição e os *smells* do `/code-review`, e o
mesmo debate (o que é SOLID aqui, quando criar interface, como tratar erro) se repetia em todo
projeto, com o risco de um tema importante nunca ser perguntado. Havia três caminhos: seguir sem
base, transformar a base em regra dura da casa, ou ter uma base de **princípios** em torno da qual o
grilling do mapa roda, com cada projeto decidindo cada um. Escolhemos a terceira: o
`docs/agents/principios-de-codigo.md`, instalado pelo `modelo-de-projeto`, com a escrita de código
(E1 a E8), os padrões de desenvolvimento (P1 a P9) e SOLID lido com o vocabulário do
`/codebase-design`, para que ele não produza módulos rasos.

**Consequências:** a base é princípio, não estilo: vale para qualquer linguagem, e cada item tem o
porquê, um exemplo, a pergunta-guia do grilling, o que o revisor procura no diff e a fonte primária.
O mapa de Regras de código é organizado por ela: depois das restrições, um ticket de grilling por
grupo de princípios, e todo princípio sai **adotado**, **adaptado** ou **afastado**, com o porquê.
O mapa só fecha quando `docs/regras-de-codigo.md` tem a tabela de cobertura com os 22 princípios.
Esse documento continua sendo a régua dura: cada regra cita o princípio de onde nasceu. Princípio
afastado não é cobrado; onde nenhuma regra alcança o trecho, o princípio é julgamento para o revisor,
como os *smells*, e só reprova com dano concreto. A precedência é `docs/regras-de-codigo.md`, depois
as ADRs do projeto, depois a base, depois os *smells*. Projeto que já tinha o mapa fechado antes desta
ADR completa a cobertura por um ticket no próprio mapa, sem travar a implementação. Mudar a base é
por PR no Stack-Labs, como todo o método, e chega aos projetos pelo `/setup-stacklabs`.
