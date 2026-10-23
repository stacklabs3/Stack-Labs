# Nome com sentido é regra dura: o E1 não se afasta, e nome de uma letra nunca

A constituição já pedia que o nome de variável dissesse o que guarda (regra 20), e o princípio E1
pedia o mesmo para função e tipo, mas o grilling podia afastar o E1, o E1 aceitava letra em laço
curto, e nenhuma ferramenta conferia nome curto. O humano decidiu que isto é obrigatório em todo
projeto: todo nome diz o que é (`produto = valor - custo`, nunca `x = y - z`). Entre as exceções
possíveis (nenhuma, só `_`, ou também `i`, `j`, `k` de laço), escolheu **só `_`**, para valor
descartado, que a própria linguagem usa.

**Consequências:** a regra 20 vale para todo nome (variável, função, parâmetro, tipo, módulo e
parâmetro de tipo), e a checagem `cobertura` do validador reprova o E1 afastado: o grilling decide só
como o E1 se aplica (língua, termos do glossário). Cada linguagem confere por lint, e as três
ferramentas exigem configuração explícita, porque o padrão delas aceita letra solta: o Clippy
`min_ident_chars` libera `i`, `j`, `x`, `y`, `z`, `w` e `n` até a lista ficar vazia no `clippy.toml`;
os formatos padrão do Checkstyle aceitam uma letra; o `id-length` do ESLint precisa de `min: 2`. O
genérico de uma letra (`<T>`) nenhum lint de TypeScript ou de Rust confere, e ganhou o modelo de
ast-grep `parametro-de-tipo-sem-nome.yml`; em Java, o Checkstyle cobre. Projeto que já fechou o mapa
com o E1 afastado reabre a linha por ticket no mapa de Regras de código.
