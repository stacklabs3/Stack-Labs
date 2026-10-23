# Testes com valor: risco registrado, checagens de máquina e teste de mutação

Pesquisa de 2026-10-23. Pergunta: agentes de IA escrevem testes que passam mas não protegem a
aplicação (regra trivial, sem afirmação, afirmação tautológica, só conferem chamada de mock,
testes desligados). As três regras propostas têm apoio em fonte primária? São elas:

1. todo teste cita um risco registrado em `docs/riscos.md` (`RK<n>`), decidido pelo humano;
2. checagens de máquina (ast-grep) para teste sem afirmação, afirmação trivial, teste desligado ou
   focado;
3. teste de mutação como receita opcional para módulos de risco alto.

## Resultado

| Regra | Fontes que sustentam | Fontes que contestam ou limitam | Conclusão |
|---|---|---|---|
| 1. Teste cita risco `RK<n>` | Glossário ISTQB, *risk-based testing* **[confirmar]**; *Software Engineering at Google* cap. 11 (testar o que não pode quebrar; cobertura vira teto) e cap. 12 (testar comportamento, não método; teste claro) | Nenhuma fonte exige um registro por teste; o SWE Book pede "test everything that you don't want to break", o que é mais largo que uma lista de riscos | **Sustentada**, com folga: o registro serve como "o comportamento que não pode quebrar" do cap. 11 |
| 2. Checagens de máquina | tsDetect (Peruma et al., FSE 2020): *Empty Test*, *Unknown Test*, *Ignored Test*, *Redundant Assertion* são exatamente os casos; van Deursen et al. 2001 criaram o conceito de *test smell*; Siddiq et al. (EASE 2024) acharam *smells* em testes gerados por LLM | Não pegam o teste que só confere mock nem o oráculo errado (Konstantinou et al. 2024): esses só o risco (regra 1) e a mutação (regra 3) pegam | **Sustentada**. As regras de ast-grep abaixo foram testadas no 0.51.0 |
| 3. Mutação opcional no risco alto | Inozemtseva & Holmes (ICSE 2014): cobertura não mede eficácia; Just et al. (FSE 2014): mutantes se correlacionam com falhas reais; Petrović et al. (ICSE 2021, Google): quem recebe mutantes escreve mais e melhores testes; Meta (FSE 2025, ACH): geração de teste por LLM guiada por mutação | Custo: o Google só roda no diff, com poda de linhas "áridas" (ICSE-SEIP 2018). Mutantes equivalentes impedem 100% (PIT, doc. de `mutationThreshold`) | **Sustentada** como receita para o diff e para o risco alto, não como portão global |

| Ferramenta | Versão no registro (2026-10-23) | Licença | Só alterados | Sai com código ≠ 0 quando |
|---|---|---|---|---|
| StrykerJS (`@stryker-mutator/core`) | 10.1.1 (publicada hoje); 10.0.0 de 2026-08-14 | Apache-2.0 | `--mutate <arquivos>`; `--incremental` | pontuação < `thresholds.break` (só no arquivo de config): código 1 |
| cargo-mutants | 27.1.0 (2026-06-02) | MIT | `--in-diff <arquivo.diff>`; `-f/--file <glob>` | qualquer mutante não pego: código 2 (3 = timeout) |
| PIT (`org.pitest:pitest-maven`) | 1.30.1 (2026-10-14); 1.30.0 de 2026-08-27 | Apache-2.0 | `targetClasses` (o goal `scmMutationCoverage` **foi removido** na 1.18.0) | pontuação < `mutationThreshold`: falha do build Maven |

## 1. Teste baseado em risco

- **ISTQB.** O glossário define *risk-based testing* como teste em que atividades e recursos de
  teste são organizados por tipo e nível de risco, e *risk level* como a importância do risco dada por
  probabilidade e impacto. Só consegui ler essa definição em site de terceiros que a reproduz
  (softwaretestpilot.com/istqb/glossary/risk-based-testing); o glossário oficial
  (<https://glossary.istqb.org>) e o syllabus CTFL 4.0, seção 5.2 (<https://www.istqb.org>), estão
  bloqueados na rede desta pesquisa **[confirmar]**. Leitura para o método: a definição é do *nível*
  do processo de teste (priorizar e escolher testes por risco); ela não pede que cada teste cite um
  risco. A regra 1 é uma concretização nossa, compatível com ela.
- **Software Engineering at Google**, livro aberto em <https://abseil.io/resources/swe-book>, lido
  no fonte do site (github.com/abseil/abseil.github.io, `resources/swe-book/html/ch11.html` e
  `ch12.html`, clonado em 2026-10-23):
  - Cap. 11, *Testing Overview*, seção "The Beyoncé Rule": "test everything that you don't want to
    break" ([ch11](https://abseil.io/resources/swe-book/html/ch11.html)). É o critério de escolha do
    que testar, e é o que o registro de riscos torna explícito e decidido pelo humano.
  - Cap. 11, "A Note on Code Coverage": cobertura "only measures that a line was invoked, not what
    happened as a result"; metas de 80% viram teto ("engineers treat it like a ceiling"); a
    alternativa é "think about the behaviors that are tested. Do you have confidence that everything
    your customers expect to work will work?" Sustenta a regra 1 e a recusa de cobertura como portão.
  - Cap. 12, *Unit Testing* ([ch12](https://abseil.io/resources/swe-book/html/ch12.html)): resumo do
    capítulo, literal: "Test behaviors, not methods. Structure tests to emphasize behaviors. Name tests
    after the behavior being tested. Don't put logic in tests. Write clear failure messages. Follow DAMP
    over DRY when sharing code for tests."
  - Cap. 12, "Don't Put Logic in Tests": "Clear tests are trivially correct upon inspection [...] if
    you feel like you need to write a test to verify your test, something has gone wrong!" Lógica é
    "operators, loops, and conditionals". Base para um aviso futuro de condicional em teste (o
    *Conditional Test Logic* do tsDetect).
  - Cap. 12, "Test State, Not Interactions": "Interaction tests tend to be more brittle than state
    tests [...] interaction tests check how a system arrived at its result, whereas usually you should
    care only what the result is." É a fonte contra o teste que só confere chamada de mock.
  - Cap. 12, "Clear Tests": "A clear test is one whose purpose for existing and reason for failing is
    immediately clear to the engineer diagnosing a failure." Citar `RK<n>` responde ao "purpose for
    existing".
- **Kent Beck, Test Desiderata.** Doze propriedades que se trocam umas pelas outras: *Isolated,
  Composable, Deterministic, Fast, Writable, Readable, Behavioral, Structure-insensitive, Automated,
  Specific, Predictive, Inspiring*. Fonte primária: o texto de Beck em
  <https://medium.com/@kentbeck_7670/test-desiderata-94150638a4b3> e o site <https://testdesiderata.com>,
  ambos bloqueados aqui **[confirmar a lista e a redação]**; a busca confirmou só o post do boletim
  dele que trata as propriedades como "sliders"
  ([Desirable Unit Tests](https://newsletter.kentbeck.com/p/desirable-unit-tests)). As que importam
  para nós: *Behavioral* (o teste muda quando o comportamento muda) e *Predictive* (se todos passam,
  o código serve para produção). Um teste sem afirmação não é *Behavioral*; um teste fora de risco
  pouco contribui para *Predictive*.

## 2. Cobertura de linha não mede eficácia; mutação mede melhor

Os quatro artigos abaixo foram localizados por busca, e o que segue vem dos resumos publicados nas
páginas dos autores ou dos eventos. O texto integral está bloqueado nesta rede **[confirmar números
além dos resumos]**.

- **Inozemtseva & Holmes, "Coverage is not strongly correlated with test suite effectiveness", ICSE
  2014, pp. 435–445**, [PDF dos autores](https://cs.ubc.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf),
  DOI 10.1145/2568225.2568271. 31.000 suítes geradas para cinco sistemas Java de até 724 mil linhas;
  eficácia medida por mutação. Achado: correlação entre cobertura e eficácia "low to moderate" quando
  se controla o número de testes, e coberturas mais fortes (decisão, MC/DC) não acrescentam. Conclusão
  dos autores: cobertura serve para achar código pouco testado, mas "should not be used as a quality
  target". Prêmio de artigo mais influente do ICSE em 2024
  ([UBC](https://www.cs.ubc.ca/award/2024/04/icse-most-influential-paper-award)).
- **Just, Jalali, Inozemtseva, Ernst, Holmes, Fraser, "Are mutants a valid substitute for real faults
  in software testing?", FSE 2014, pp. 654–665**,
  [resumo](https://homes.cs.washington.edu/~rjust/publ/JustJIEHF2014b-abstract.html),
  [PDF](https://homes.cs.washington.edu/~mernst/pubs/mutation-effectiveness-fse2014.pdf). 357 falhas
  reais de 5 aplicações (base Defects4J). Achado: correlação estatisticamente significativa entre
  detectar mutantes e detectar falhas reais, **independente da cobertura**. É o que autoriza usar a
  pontuação de mutação como medida da regra 3.
- **Petrović & Ivanković, "State of Mutation Testing at Google", ICSE-SEIP 2018**,
  [Google Research](https://research.google/pubs/pub46584/). Mutação por diff, probabilística, que
  descarta linhas sem cobertura e linhas "áridas" (log, configuração), com os mutantes que sobrevivem
  mostrados na revisão de código. Usado em revisão obrigatória por milhares de engenheiros. Limita a
  regra 3: no Google a mutação roda **só no diff** e entra como sugestão ao revisor, não como portão.
  O seguimento "Practical Mutation Testing at Scale: A view from Google" (TSE 2021,
  [PDF](https://homes.cs.washington.edu/~rjust/publ/practical_mutation_testing_tse_2021.pdf)) detalha
  o limite de mutantes mostrados por mudança **[confirmar o número]**.
- **Petrović, Ivanković, Fraser, Just, "Does mutation testing improve testing practices?", ICSE 2021,
  pp. 910–921**, [resumo](https://homes.cs.washington.edu/~rjust/publ/PetrovicIFJ2021b-abstract.html),
  arXiv 2103.07189. Quase 15 milhões de mutantes. Achados do resumo: quem recebe mutantes escreve mais
  testes e testes melhores ao longo do tempo, e mutantes estão acoplados a falhas reais de alta
  prioridade. Estudo observacional, sem grupo de controle.

## 3. Test smells: nomes exatos e definições

- **van Deursen, Moonen, van den Bergh, Kok, "Refactoring Test Code", XP2001**, pp. 92–95,
  [CWI](https://ir.cwi.nl/pub/4324). Introduz o termo *test smell* e um catálogo de *smells* com
  refatorações próprias (entre eles *Mystery Guest*, *Resource Optimism*, *General Fixture*, *Eager
  Test*, *Lazy Test*, *Assertion Roulette*, *Sensitive Equality*). A lista exata não foi conferida no
  texto, que está bloqueado **[confirmar]**. Os quatro *smells* que nos interessam **não** estão
  nesse artigo: vêm do catálogo posterior do tsDetect.
- **Peruma, Almalki, Newman, Mkaouer, Ouni, Palomba, "tsDetect: An Open Source Test Smells Detection
  Tool", ESEC/FSE 2020**, DOI [10.1145/3368089.3417921](https://doi.org/10.1145/3368089.3417921).
  Definições lidas no fonte do site oficial (github.com/TestSmells/TestSmells.github.io,
  `pages/testsmells.html`, publicado em <https://testsmells.org/pages/testsmells.html>) e regras de
  detecção no código da ferramenta (github.com/TestSmells/TestSmellDetector,
  `src/main/java/testsmell/smell/`, licença **GPL-3.0**, só Java/JUnit):

| Smell (nome exato) | Definição (resumo do texto oficial) | Detecção do tsDetect | Nossa regra |
|---|---|---|---|
| **Empty Test** | "Occurs when a test method does not contain executable statements"; é "more dangerous than not having a test case at all since JUnit will indicate that the test passes" | método de teste sem nenhuma instrução executável | `*-teste-sem-afirmacao` (o vazio é caso particular) |
| **Unknown Test** | método de teste escrito "sans an assertion statement"; o JUnit o mostra como aprovado se nada lançar exceção | "does not contain a single assertion statement and @Test(expected) annotation parameter" | `*-teste-sem-afirmacao` |
| **Ignored Test** | testes suprimidos com `@Ignore` do JUnit 4 somam custo de compilação e de leitura | método ou classe com `@Ignore` (o código checa só `Ignore`, não o `@Disabled` do JUnit 5) | `*-teste-desligado` (cobre `@Disabled`, `#[ignore]`, `.skip`) |
| **Redundant Assertion** | "assertion statements that are either always true or always false"; criadas para depurar e esquecidas | "expected and actual parameters are the same"; o código também pega `assertTrue(true)`, `assertFalse(false)`, `assertNull(null)` (`RedundantAssertion.java`) | `*-afirmacao-trivial` |

  Outros *smells* do catálogo que ficam para depois: *Conditional Test Logic* (casa com o "Don't put
  logic in tests" do SWE Book), *Assertion Roulette*, *Magic Number Test*, *Sleepy Test*. O teste
  **focado** (`.only`) não está no catálogo; o incluímos porque ele desliga em silêncio todos os
  outros testes do arquivo.

## 4. Testes gerados por LLM

- **Alshahwan, Chheda, Finegenova, Gokkaya, Harman, Harper, Marginean, Sengupta, Wang, "Automated Unit
  Test Improvement using Large Language Models at Meta", FSE 2024 (Industry)**, arXiv
  [2402.09171](https://arxiv.org/abs/2402.09171),
  [página do FSE](https://2024.esec-fse.org/details/fse-2024-industry/18/Automated-Unit-Test-Improvement-using-Large-Language-Models-at-Meta).
  O TestGen-LLM só entrega um teste gerado depois de passar por filtros que "assure measurable
  improvement over the original test suite": (1) **builds**, o que não compila é descartado; (2)
  **passes**, que descarta teste que falha ou é instável, por execução repetida (uma fonte que cita o
  artigo diz cinco execuções **[confirmar no texto]**); (3) **improves coverage**, só fica o teste que
  aumenta a cobertura. Resultado no Instagram (Reels e Stories): 75% compilaram, 57% passaram de forma
  confiável, 25% aumentaram a cobertura; nos *test-a-thons*, 73% das recomendações foram aceitas.
  Lição para nós: a Meta não confia no teste gerado sem portão de máquina, e o portão final dela é
  "acrescenta algo que a suíte não tinha". O critério dela é cobertura, que a seção 2 mostra ser
  fraco; o trabalho seguinte da própria Meta troca cobertura por mutação:
- **Harman et al., "Mutation-Guided LLM-based Test Generation at Meta", FSE 2025 (Industry)**, arXiv
  [2501.12862](https://arxiv.org/abs/2501.12862),
  [página do FSE](https://conf.researchr.org/details/fse-2025/fse-2025-industry-papers/16/Mutation-Guided-LLM-based-Test-Generation-at-Meta).
  O sistema ACH gera poucos mutantes, ligados a uma preocupação definida (o exemplo é privacidade), e
  pede ao LLM testes que os matem: 10.795 classes Kotlin, 9.095 mutantes, 571 testes. É o desenho
  mais próximo das nossas regras 1 e 3 juntas: o risco escolhe onde mutar, o mutante prova que o teste
  protege. A taxa de aceitação por engenheiros não foi conferida no texto **[confirmar]**.
- **Konstantinou, Degiovanni, Papadakis, "Do LLMs generate test oracles that capture the actual or
  the expected program behaviour?"**, arXiv [2410.21136](https://arxiv.org/abs/2410.21136) (2024), 24
  repositórios Java. Achado: o LLM tende a escrever o oráculo a partir do comportamento **atual** do
  código, não do esperado. Ou seja, se o código tem erro, o teste gerado afirma o erro. É o caso que
  nenhuma checagem sintática pega; a defesa é o risco escrito pelo humano (regra 1) dizer o
  comportamento esperado antes do teste.
- **Siddiq et al., "Using Large Language Models to Generate JUnit Tests: An Empirical Study", EASE
  2024**, arXiv [2305.00418](https://arxiv.org/abs/2305.00418). Testes gerados por Codex, GPT-3.5 e
  StarCoder (os modelos variam entre versões do artigo) avaliados por compilação, correção, cobertura
  e *test smells* medidos com o tsDetect; os resumos citam *smells* como *Empty Test*, *Duplicate
  Assert*, *Assertion Roulette* e *Magic Number Test* **[confirmar frequências no texto]**.

Não incluí afirmações sobre "LLM escreve teste sem asserção em X% dos casos": nenhuma fonte primária
com esse número foi verificada.

## 5. Ferramentas de mutação

Versão e licença conferidas em 2026-10-23: `npm view @stryker-mutator/core version license time`,
API do crates.io (`/api/v1/crates/cargo-mutants`) e `cargo info cargo-mutants`, e
`repo1.maven.org/maven2/org/pitest/pitest-maven/maven-metadata.xml` com a licença lida no POM.
Documentação lida no repositório de origem de cada uma, clonado na mesma data:
`stryker-mutator/stryker-js` (`docs/`), `sourcefrog/cargo-mutants` (`book/src/`), `hcoles/pitest`
(fonte e `README.md`) e `hcoles/pitest-site` (`quickstart/maven.markdown`).

Critério de versão nas linhas abaixo: a mais nova publicada há pelo menos 14 dias. Stryker 10.1.1
saiu hoje e PIT 1.30.1 há 9 dias (só atualiza dependências, pelo `README.md` do PIT); ambas podem
substituir as fixadas depois.

### StrykerJS (TypeScript/JavaScript)

- Registro: `@stryker-mutator/core` 10.1.1, Apache-2.0; 10.0.0 de 2026-08-14. Exige Node >= 22.
  Os executores (`@stryker-mutator/vitest-runner`, `jest-runner`) têm a mesma versão.
- Só alterados: `--mutate` aceita lista de arquivos ou globs separados por vírgula ("`--mutate
  src/app/home/home.component.ts`, for one specific file",
  [configuration.md](https://stryker-mutator.io/docs/stryker-js/configuration#mutate-string)).
  `--incremental` guarda o resultado em `reports/stryker-incremental.json` e só refaz o que mudou em
  código ou teste ([incremental.md](https://stryker-mutator.io/docs/stryker-js/incremental)).
- Limite: `thresholds.break`; "mutation score < break: Error! Stryker will exit with exit code 1".
  **Não tem flag de linha de comando** ("Command line: _none_"), só arquivo de config; o padrão é
  `null`, que nunca falha.

```sh
npm install --save-dev @stryker-mutator/core@10.0.0 @stryker-mutator/vitest-runner@10.0.0
# stryker.config.json: { "testRunner": "vitest", "thresholds": { "high": 80, "low": 60, "break": 60 } }
ARQUIVOS_ALTERADOS=$(git diff --name-only --diff-filter=AM origin/main...HEAD -- 'src/*.ts' \
  | grep -v -E '\.(test|spec)\.ts$' | paste -sd, -)
[ -n "$ARQUIVOS_ALTERADOS" ] && npx stryker run --mutate "$ARQUIVOS_ALTERADOS" --incremental
```

### cargo-mutants (Rust)

- Registro: 27.1.0 (2026-06-02), MIT, exige Rust 1.88.
- Só alterados: `--in-diff DIFF_FILE` "tests only mutants that overlap with regions changed in the
  diff" ([in-diff.md](https://mutants.rs/in-diff.html)); o livro traz um workflow de PR pronto
  (`examples/workflows/in-diff.yml`). Por arquivo: `-f GLOB`, `--file GLOB`
  ([skip_files.md](https://mutants.rs/skip_files.html)).
- Limite: não há percentual. Código de saída ([exit-codes.md](https://mutants.rs/exit-codes.html)):
  0 todos pegos; **2** "Found some mutants that were not covered by tests"; 3 timeout; 4 testes já
  falham sem mutação. Um mutante aceito como equivalente se marca com `#[mutants::skip]` (crate
  `mutants` 0.0.4, dependência normal) ou `exclude_re` em `.cargo/mutants.toml`
  ([attrs.md](https://mutants.rs/attrs.html)), com comentário dizendo o porquê.

```sh
cargo install --locked cargo-mutants@27.1.0
git diff origin/main...HEAD > alteracoes.diff
cargo mutants --in-diff alteracoes.diff --no-shuffle     # sai com 2 se algum mutante sobreviver
cargo mutants --file src/pagamento.rs                     # alternativa: módulo de risco alto inteiro
```

### PIT (Java, Maven)

- Registro: `org.pitest:pitest-maven` 1.30.1 (2026-10-14), 1.30.0 (2026-08-27); POM declara "The
  Apache Software License, Version 2.0". JUnit 5 pede o plugin `org.pitest:pitest-junit5-plugin`
  1.2.3 como dependência do plugin (FAQ do pitest-site: "JUnit 5 is not supported out of the box").
- Só alterados: o goal **`scmMutationCoverage` não existe mais**. `README.md` do PIT, versão 1.18.0:
  "#1379 Fully remove deprecated scm maven goal". O sucessor por git (`+GIT(from[HEAD~1])` em
  `extraFeatures`) é do **Arcmutate**, produto comercial à parte (maven.markdown, seções
  `extraFeatures` e "Arcmutate"); preço não conferido **[confirmar]**. Sem pagar: montar
  `targetClasses` a partir do diff (propriedade `targetClasses` em `PitMojo.java`) e usar
  `-DwithHistory` para reaproveitar a análise anterior.
- Limite: `mutationThreshold`, "Mutation score threshold at which to fail the build" (0–100,
  propriedade `mutationThreshold`, padrão 0). O doc avisa: "your build may contain equivalent
  mutations. Careful thought must therefore be given when selecting a threshold." Existe também
  `testStrengthThreshold` (mortos / (mortos + sobreviventes), ignora o não coberto).

```sh
CLASSES_ALTERADAS=$(git diff --name-only --diff-filter=AM origin/main...HEAD -- 'src/main/java/*.java' \
  | sed -e 's#^src/main/java/##' -e 's#\.java$##' -e 's#/#.#g' | paste -sd, -)
[ -n "$CLASSES_ALTERADAS" ] && mvn test-compile org.pitest:pitest-maven:1.30.0:mutationCoverage \
  -DtargetClasses="$CLASSES_ALTERADAS" -DmutationThreshold=80 -DwithHistory
```

## 6. ast-grep 0.51.0: sintaxe conferida e regras testadas

Fonte: <https://ast-grep.github.io/reference/rule.html> e `reference/yaml.html`, lidos no fonte do
site (github.com/ast-grep/ast-grep.github.io, último commit 2026-10-14). Testado com
`@ast-grep/cli@0.51.0` (MIT) em amostras TS, Rust e Java fora do repositório.

- **`has` / `not` com `stopBy: end`**: `stopBy` aceita `"neighbor"` (padrão), `"end"` ou um objeto
  de regra; `end` busca "to the leaf node for `has`, to the first sibling for `follows`". Sem
  `stopBy: end`, o `has` só olha os filhos diretos e a regra de "teste sem afirmação" não acha o
  `expect` aninhado.
- **`follows`** para atributo Rust: o `#[test]` é um nó `attribute_item` irmão anterior do
  `function_item`. `follows` não tem `field`. Com `stopBy` objeto (`not: {kind: attribute_item}`), a
  busca atravessa vários atributos seguidos (`#[test]` + `#[cfg(...)]`) e para na primeira coisa que
  não é atributo; "`stopBy` is inclusive".
- **`constraints` com `regex`**: `constraints` é `HashMap<String, Rule>` com a chave sendo o nome da
  metavariável sem `$`; só vale para metavariável simples, não `$$$`; é aplicado **depois** da regra
  e "usually do[es] not work inside `not`". `regex` é Rust regex, casa em qualquer parte do texto do
  nó; use `^...$` para casar inteiro.
- Metavariável repetida (`$VALOR` duas vezes) exige o mesmo texto nos dois lugares: é o que pega
  `assert_eq!(total, total)` e `expect(true).toBe(true)`.
- Cuidado de YAML: `message: Teste desligado com #[ignore].` perde o texto após ` #`, que vira
  comentário; ponha a mensagem entre aspas.

As regras que saíram desta seção são os modelos `teste-sem-afirmacao.yml`, `afirmacao-trivial.yml`
e `teste-desligado.yml` em `agentes/modelo-de-projeto/.claude/watchdogs/modelos/ast-grep/`, com
documentos para TypeScript, Tsx, JavaScript, Rust e Java e o teste
`ast_grep_modelos_de_teste_acham_teste_sem_valor` do crate dos watchdogs, que planta cada caso.

Limites conhecidos, vistos no teste: um teste que só faz `expect(enviar).toHaveBeenCalled()` ou
`verify(...)` passa nas regras (conta como afirmação); um teste Rust que retorna `Result` e só usa
`?` é marcado como sem afirmação; afirmação em função auxiliar só é vista quando o nome começa por `assert`, `expect`, `verify` ou
`conferir` (`conferir_pedido(...)`), a convenção que os modelos adotaram em vez de exceção por teste.

## Recomendação para o método

1. **Regra 1 (risco `RK<n>`) entra como regra dura.** É a única defesa contra o oráculo copiado do
   código atual (Konstantinou et al.) e contra o teste de mock sem estado (SWE Book, cap. 12); o
   risco escrito pelo humano diz o comportamento esperado antes do teste. Formular a regra pelo
   comportamento ("o que não pode quebrar", cap. 11), não por método.
2. **Regra 2 entra como as nove regras de ast-grep acima**, pelo tipo `ast-grep` do validador, com
   os nomes do tsDetect na mensagem (*Unknown Test*, *Redundant Assertion*, *Ignored Test*) e o
   `.only` incluído. Lista de auxiliares de afirmação configurável por projeto.
3. **Regra 3 fica como receita opcional, só no diff e só em módulo com risco alto**, como faz o
   Google. Portão por linguagem: Stryker `thresholds.break`, cargo-mutants código 2, PIT
   `mutationThreshold`. Não usar cobertura de linha como portão (Inozemtseva & Holmes). Para Java,
   anotar que o modo por git do PIT é pago (Arcmutate) e usar `targetClasses` do diff.
4. Antes de virar ADR, abrir as fontes marcadas **[confirmar]** numa rede sem bloqueio: glossário e
   syllabus do ISTQB, Test Desiderata, lista de *smells* de van Deursen e os números do TestGen-LLM e
   do ACH.
