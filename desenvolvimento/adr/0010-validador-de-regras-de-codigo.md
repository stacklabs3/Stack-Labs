# Validador de regras de código: um motor do método, configurado por projeto

Depois das ADRs 0007 e 0008, as regras de código de cada projeto só eram conferidas pelo revisor da
orquestração e pelo lint que o projeto escolhesse; nada garantia por máquina as regras de desenho
(imports do domínio, teste de contrato por port), nem que o mapa fechasse com os 22 princípios
decididos. Havia três caminhos: escrever um validador em cada projeto, deixar tudo com o revisor, ou
um **motor único do método** que cada projeto configura. Escolhemos o motor: `.claude/validador/`,
instalado pelo `modelo-de-projeto`, lê `docs/regras-de-codigo.json` e roda um catálogo de checagens
parametrizáveis. O agente refina a configuração, não o motor. A pesquisa
`desenvolvimento/research/2026-10-23-ferramentas-do-validador.md` aplicou a regra dada pelo humano,
"ferramenta que não cobra a gente usa; a que cobra a gente escreve", e mediu tempo e memória, porque o
validador roda em todo CI e em toda revisão: o motor chama o ast-grep (Rust, MIT) para padrões de
código e, pelo tipo `comando`, jscpd e dependency-cruiser (MIT). O Semgrep ficou de fora pelos planos
pagos e pela licença das regras; o Opengrep, grátis, ficou de fora por custar 25 vezes o tempo e 28
vezes a memória do ast-grep no mesmo repositório. Checagens próprias só
para o que nenhuma ferramenta faz: cobertura, termo evitado do glossário, teste de contrato por port.

**Consequências:** o motor é protegido como os watchdogs (hooks, `vigia-do-metodo` e instalador o
tratam assim), para que a régua não se afrouxe na sessão que ela avalia. A tabela de cobertura do
`.md` continua dona das decisões; o `.json` diz como cada regra se verifica, e a checagem
`cobertura` confere os dois juntos. O mapa de Regras de código fecha com `--so-cobertura` aprovado; o
validador entra no CI na primeira fatia, junto com o lint; implementador e revisor o rodam, e o
revisor trata validador vermelho como violação dura. Cada princípio ganha a linha **No validador**.
Os princípios de julgamento (S, O, I, P1, P2, P6, P7) continuam com o revisor. Para gastar o mínimo
de tempo e memória, o CI, o implementador e o revisor rodam o validador com `--base`, que lê só os
arquivos alterados; o repositório inteiro fica para a auditoria. Checagem nova que sirva a outros projetos entra no motor por PR no Stack-Labs.
