# Todo teste cobre um risco registrado, e a máquina confere

Agentes escrevem testes com facilidade, e o teste mais fácil é o que passa por construção: confere
uma regra lógica que ninguém pediu, não tem afirmação, compara um valor com ele mesmo, só confere a
chamada de um mock ou fica desligado. O P5 pedia teste pela interface, mas nada dizia **o que**
merece teste, e nenhuma máquina conferia. Cobertura de linha não resolve: ela mede que a linha rodou,
não que algo foi afirmado, e vira teto quando é meta. O humano decidiu que regras de teste fazem
parte do método e são validadas pelos hooks. Havia três caminhos: deixar com o revisor, medir por
cobertura, ou ligar **cada teste a um risco aprovado pelo humano** e conferir por máquina o que tem
forma fixa. Escolhemos o terceiro. A pesquisa
`desenvolvimento/research/2026-10-23-testes-com-valor.md` o sustenta: *Software Engineering at
Google* (caps. 11 e 12: testar o que não pode quebrar, comportamento e estado em vez de método e
interação, cobertura como teto), os *test smells* do tsDetect (*Unknown Test*, *Redundant
Assertion*, *Ignored Test*) e os estudos de cobertura contra mutação (Inozemtseva e Holmes, 2014;
Just et al., 2014; Google, 2018 e 2021).

**Consequências:** o P5 passa a ser "teste cobre um risco, pela interface" e, como o E1 (ADR 0013),
vira regra dura da constituição (regra 15): o projeto o adapta, nunca o afasta, e a linha dele na
tabela de cobertura cita uma regra com a checagem `teste-cobre-risco`. Os 22 continuam 22. Cada
projeto ganha o registro `docs/riscos.md` (riscos `RK<n>` com origem, nascidos no mapa de Regras de
código, na spec e no triage de bug); o agente propõe risco, o humano aprova. O motor ganha a
checagem `teste-cobre-risco` (teste sem `RK<n>`, risco fora do registro, risco sem origem e risco não
pendente sem teste) e três modelos de ast-grep para TypeScript, JavaScript, Rust e Java (teste sem
afirmação, afirmação trivial, teste desligado ou focado). O hook novo `vigia-teste` (PostToolUse)
roda essas regras no teste que o agente acabou de editar e devolve o motivo na hora; o validador as
roda no CI e na revisão. O que não tem forma fixa (o teste cita o risco e não o exercita, o esperado
copia o código, o teste só confere interação) fica com o revisor e o conferente. Teste de mutação no
diff (Stryker, cargo-mutants, PIT) é receita que o grilling adota para módulos de risco alto, não
portão global, pelo custo. Projeto que já fechou o mapa reabre a linha do P5 por ticket no mapa de
Regras de código e escreve o registro.
