# Princípios de bom desenvolvimento em Java para o validador

Pesquisa de 2026-10-23. Pergunta: quais práticas de Java (21+ LTS, Spring Boot) concretizam os 22
princípios da casa, e como cada uma se confere por máquina com ferramenta que não cobra?
Regra das ferramentas: **sem cobrança, usamos; com cobrança, escrevemos a nossa**. Semgrep e
Opengrep continuam fora.

Desfecho: [ADR 0012](../adr/0012-praticas-da-linguagem.md). As linhas desta pesquisa viraram
`docs/agents/principios-de-codigo/java.md` e os modelos de `.claude/watchdogs/modelos/` do
`modelo-de-projeto`; as práticas fora dos 22 viraram JV1 a JV6. Os achados da seção 6 foram
corrigidos no motor e nas receitas na mesma mudança (o contrato Maven usa `{dir_de_teste}`).

## Como foi feito, e o que a rede deixou ver

- **Nomes de regra conferidos no artefato, não em blog.** Baixei de Maven Central os jars das
  versões atuais e conferi cada nome citado: as categorias `category/java/*.xml` do `pmd-java`
  7.28.0, as classes `checks/**/<Nome>Check` do `checkstyle` 14.3.0, as classes de
  `com.google.errorprone.bugpatterns` do `error_prone_core` 2.51.0 (com a severidade lida da
  anotação `@BugPattern` e o grupo padrão lido de `BuiltInCheckerSuppliers`: `ENABLED_ERRORS`,
  `ENABLED_WARNINGS` ou `DISABLED_CHECKS`), o `findbugs.xml` do `spotbugs` 4.10.5 e
  `GeneralCodingRules`/`Architectures` do `archunit` 1.5.1.
- **Rodei de verdade**: PMD, Checkstyle, SpotBugs, Error Prone com NullAway, ArchUnit e Spotless em
  Maven 3.9.11; Checkstyle, PMD, SpotBugs e Error Prone em Gradle 8.14.3; jscpd 5.4.2 em Java; e as
  seis regras de ast-grep 0.51.0 abaixo. JDK 21.0.12. Projetos de teste
  mínimos em Maven e Gradle, fora do repositório.
- **Fontes lidas na íntegra**: Google Java Style Guide (`google/styleguide`, ramo `gh-pages`), Oracle
  Secure Coding Guidelines for Java SE, documentação do Spring Framework e do Spring Boot (os `.adoc`
  dos repositórios `spring-projects/*`), README e LICENSE dos repositórios das ferramentas, campo
  `<licenses>` dos POMs.
- **Bloqueados nesta rede**: `checkstyle.org`, `docs.pmd-code.org`, `errorprone.info`,
  `spotbugs.readthedocs.io`, `archunit.org`, `openjdk.org`, `docs.spring.io`. Os links da tabela
  apontam para essas páginas oficiais; o **conteúdo** de cada uma foi confirmado pelo artefato ou pelo
  fonte da documentação no repositório. Os JEPs foram confirmados pela busca restrita a
  `openjdk.org` (número, título e versão de entrega).
- **Effective Java, 3ª ed.** (Bloch, 2018, ISBN 9780134685991): o livro não está aberto na rede; os
  números dos itens vêm do sumário do livro e não foram conferidos numa página online
  **[confirmar: numeração dos itens de Effective Java]**.

Abreviações: **EJ** Effective Java 3ª ed.; **GJS** Google Java Style Guide; **OSCG** Oracle Secure
Coding Guidelines; **EP** Error Prone (entre parênteses, a severidade e se vem ligada por padrão).

## 1. Prática, fonte, princípio e verificação

Tipos do catálogo: `lint-cobre` (a regra está na configuração do Checkstyle, do PMD ou nos
`-Xep:` do `pom.xml`/`build.gradle`), `comando`, `ast-grep`, `revisor`.

### Escrita (E1 a E8)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Convenção de nomes: `UpperCamelCase` para tipo, `lowerCamelCase` para método, campo e variável, `UPPER_SNAKE_CASE` para constante; sigla como palavra (`XmlHttp`) | [GJS 5](https://google.github.io/styleguide/javaguide.html#s5-naming), EJ Item 68 | E1 | `lint-cobre` Checkstyle: `TypeName`, `MethodName`, `MemberName`, `LocalVariableName`, `ParameterName`, `ConstantName`, `AbbreviationAsWordInName` ([naming](https://checkstyle.org/checks/naming/index.html)). Vocabulário do glossário: `termo-evitado` (vale igual para Java) |
| Poucos parâmetros (o livro sugere quatro ou menos); muitos argumentos de construtor indicam classe com responsabilidade demais | EJ Item 51; [Spring Framework, "Constructor-based or setter-based DI?"](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html) | E2 (e S) | `lint-cobre` Checkstyle [`ParameterNumber`](https://checkstyle.org/checks/sizes/parameternumber.html) (`max`=3) ou PMD [`ExcessiveParameterList`](https://docs.pmd-code.org/latest/pmd_rules_java_design.html#excessiveparameterlist) (`minimum`=4). EP `TooManyParameters` existe mas vem desligado |
| Aninhamento raso e saída cedo | GJS não fixa; princípio da casa | E2 | `lint-cobre` Checkstyle [`NestedIfDepth`](https://checkstyle.org/checks/coding/nestedifdepth.html), `NestedTryDepth`; PMD `AvoidDeeplyNestedIfStmts` |
| Função curta e simples | princípio da casa | E2 | `lint-cobre` Checkstyle [`MethodLength`](https://checkstyle.org/checks/sizes/methodlength.html), [`CyclomaticComplexity`](https://checkstyle.org/checks/metrics/cyclomaticcomplexity.html); PMD [`CognitiveComplexity`](https://docs.pmd-code.org/latest/pmd_rules_java_design.html#cognitivecomplexity), `NcssCount` |
| Variável local declarada perto do uso, escopo mínimo | EJ Item 57; [GJS 4.8.2.2](https://google.github.io/styleguide/javaguide.html#s4.8.2.2-variables-limited-scope) | E2 | `lint-cobre` Checkstyle `VariableDeclarationUsageDistance` (já está no `google_checks.xml`); no mais, `revisor` |
| Tipo de valor como `record`: imutável, com validação no construtor compacto | [JEP 395](https://openjdk.org/jeps/395) (Java 16); EJ Item 49 (validar parâmetros) | E3, E5 | `revisor` (o "tipo próprio na borda" é julgamento; nenhuma ferramenta sabe o que é conceito do domínio) |
| Não usar `String` onde outro tipo cabe | EJ Item 62 | E3 | `revisor` |
| Nulidade no tipo: pacote `@NullMarked`, `@Nullable` só onde pode faltar; o compilador recusa desreferência insegura | [JSpecify 1.0](https://jspecify.dev/docs/start-here/) ; [NullAway](https://github.com/uber/NullAway) | E3 | `comando`: compilação com Error Prone + NullAway (`-Xep:NullAway:ERROR -XepOpt:NullAway:OnlyNullMarked=true`); `lint-cobre` com `config: pom.xml` e `regras: ["NullAway:ERROR"]` para provar que está ligado. Testado: pegou `nome.length()` com `nome` `@Nullable` |
| Validar entrada não confiável na borda | [OSCG 5-1 INPUT-1](https://www.oracle.com/java/technologies/javase/seccodeguide.html) | E3 | `revisor` (Bean Validation, `@Valid` no controller, é a forma comum em Spring; a ferramenta não sabe qual é a borda) |
| Nunca ignorar exceção; se não fazer nada é correto, o porquê vai num comentário ou se relança como `AssertionError` | EJ Item 77; [GJS 6.2](https://google.github.io/styleguide/javaguide.html#s6.2-caught-exceptions) | E4 | `ast-grep` **`catch-vazio-java`** (seção 4); `lint-cobre` PMD [`EmptyCatchBlock`](https://docs.pmd-code.org/latest/pmd_rules_java_errorprone.html#emptycatchblock) ou Checkstyle [`EmptyCatchBlock`](https://checkstyle.org/checks/blocks/emptycatchblock.html); EP [`EmptyCatch`](https://errorprone.info/bugpattern/EmptyCatch) (WARNING, ligado). Nota: o GJS aceita catch vazio **com comentário**; a regra da casa (E4) é mais dura e o modelo pega também esse caso |
| Não capturar `Exception`/`Throwable`/`RuntimeException` genérico | EJ Item 77 (indireto); princípio da casa | E4 | `lint-cobre` PMD [`AvoidCatchingGenericException`](https://docs.pmd-code.org/latest/pmd_rules_java_errorprone.html#avoidcatchinggenericexception) (no PMD 7.28 mudou para a categoria `errorprone`; a referência em `design` está obsoleta) e `AvoidCatchingThrowable`; Checkstyle [`IllegalCatch`](https://checkstyle.org/checks/coding/illegalcatch.html). A borda (`@ControllerAdvice`) fica fora por supressão |
| Não só imprimir a pilha e seguir | princípio da casa | E4 | EP [`CatchAndPrintStackTrace`](https://errorprone.info/bugpattern/CatchAndPrintStackTrace) (WARNING, ligado); PMD `AvoidPrintStackTrace`; `ast-grep` `catch-vazio-java` e `saida-padrao-java` |
| Exceção da abstração certa, com a causa encadeada e mensagem com o dado que falhou | EJ Itens 72, 73, 75 | E4 | `lint-cobre` PMD [`PreserveStackTrace`](https://docs.pmd-code.org/latest/pmd_rules_java_bestpractices.html#preservestacktrace), `AvoidThrowingRawExceptionTypes`; ArchUnit `GeneralCodingRules.NO_CLASSES_SHOULD_THROW_GENERIC_EXCEPTIONS` |
| Checked para erro recuperável, runtime para erro de programação; evitar checked desnecessária | EJ Itens 70, 71 | E4 (o "erro esperado na interface") | `revisor`; é a pergunta do grilling de E4 em Java |
| Ausência é resultado: devolver `Optional`, nunca `null` de coleção, e não chamar `get()` sem tratar | EJ Itens 54, 55 | E4 | `ast-grep` **`optional-get-direto-java`** (seção 4); EP [`NullOptional`](https://errorprone.info/bugpattern/NullOptional) (WARNING, ligado); PMD `ReturnEmptyCollectionRatherThanNull`. Nenhum checador gratuito testado pega `Optional.get()` sem checagem: o EP não tem regra para isso (testado, passou calado) |
| Valor de retorno não se ignora (`s.trim();` sozinho é bug) | EJ Item 77 (indireto); princípio da casa | E4, E5 | EP [`ReturnValueIgnored`](https://errorprone.info/bugpattern/ReturnValueIgnored) e `CheckReturnValue` (ERROR, ligados: quebram a compilação; testado); SpotBugs `RV_RETURN_VALUE_IGNORED` (testado) |
| Exceção e log sem dado sensível | [OSCG 2-1 CONFIDENTIAL-1, 2-2 CONFIDENTIAL-2](https://www.oracle.com/java/technologies/javase/seccodeguide.html) | E4 ("sem dado pessoal sensível") | `revisor` |
| Minimizar mutabilidade: campo `final`, classe imutável, `record` | EJ Item 17; [OSCG 6-1 MUTABLE-1](https://www.oracle.com/java/technologies/javase/seccodeguide.html); [Spring Boot: injeção por construtor deixa o campo `final`](https://docs.spring.io/spring-boot/reference/using/spring-beans-and-dependency-injection.html) | E5 | `lint-cobre` PMD [`ImmutableField`](https://docs.pmd-code.org/latest/pmd_rules_java_design.html#immutablefield); EP `FieldCanBeFinal` existe mas vem desligado (SUGGESTION) |
| Cópia defensiva; não expor coleção ou array mutável interno | EJ Item 50; OSCG 6-2 MUTABLE-2, 6-12 MUTABLE-12 | E5 | SpotBugs `EI_EXPOSE_REP`, `EI_EXPOSE_REP2`, `MS_MUTABLE_COLLECTION` ([descrições](https://spotbugs.readthedocs.io/en/latest/bugDescriptions.html)); EP `MutablePublicArray`, `MixedMutabilityReturnType` (WARNING, ligados) |
| Campo estático público é `final` e constante | OSCG 6-9, 6-10, 6-11 (MUTABLE-9 a 11) | E5 | `lint-cobre` PMD `MutableStaticState`; Checkstyle [`VisibilityModifier`](https://checkstyle.org/checks/design/visibilitymodifier.html) |
| Parâmetro não se reatribui | princípio da casa | E5 | `lint-cobre` PMD `AvoidReassigningParameters` |
| Função de stream sem efeito colateral | EJ Item 46 | E5 | `revisor` |
| `TODO` com link para a issue | [GJS 4.8.6.2](https://google.github.io/styleguide/javaguide.html#s4.8.6.2-todo-comments) | E6 | `todo-sem-issue` (já serve para Java: lê `//` e `/* */` linha a linha) |
| Código morto se apaga | princípio da casa | E6 (e P7) | EP [`UnusedVariable`](https://errorprone.info/bugpattern/UnusedVariable), `UnusedMethod` (WARNING, ligados); PMD `UnusedPrivateField`, `UnusedPrivateMethod`, `UnusedLocalVariable`, `UnusedAssignment` |
| Javadoc em toda API exposta | EJ Item 56; [GJS 7.3](https://google.github.io/styleguide/javaguide.html#s7.3-javadoc-where-required) | E6 (conflito) | Choca com E6 ("só o porquê"). Se o projeto adotar, `lint-cobre` Checkstyle `MissingJavadocMethod`/`MissingJavadocType`; o normal na casa é afastar para código de aplicação e adotar só em biblioteca publicada |
| Duplicação | [OSCG 0-2 FUNDAMENTALS-2](https://www.oracle.com/java/technologies/javase/seccodeguide.html) ("Avoid duplication") | E7 | `comando` `npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` (testado em Java: achou o clone); alternativa só Maven: `mvn pmd:cpd-check` (PMD CPD) |
| Número mágico vira constante; literal repetido vira constante | princípio da casa | E8 | `lint-cobre` Checkstyle [`MagicNumber`](https://checkstyle.org/checks/coding/magicnumber.html) (testado); PMD `AvoidLiteralsInIfCondition`, `AvoidDuplicateLiterals` |
| `enum` em vez de constante `int`/`String` | EJ Item 34 | E8, E3 | `revisor` |

### Padrões (P1 a P9) e SOLID

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Minimizar acessibilidade: classe de implementação package-private, só a interface pública | EJ Item 15; OSCG 4-1 EXTEND-1 | P1, I | `revisor`; com ArchUnit, regra do projeto: `classes().that().resideInAPackage("..adapters..").should().notBePublic()` |
| Esconder pacote interno com módulo (JPMS) ou com módulos do Spring Modulith | OSCG 4-2 EXTEND-2; [Spring Boot, "Structuring Your Code"](https://docs.spring.io/spring-boot/reference/using/structuring-your-code.html) (aponta o Spring Modulith para estrutura por domínio) | P1, P4 | `comando` com o teste do Spring Modulith (`ApplicationModules.of(App.class).verify()`) ou ArchUnit |
| Referir-se ao objeto pela interface; interface só para definir tipo | EJ Itens 64, 20, 22 | P2, I | `revisor`; PMD `LooseCoupling` pega o caso concreto (`ArrayList` na assinatura) |
| Injeção por construtor; dependência obrigatória entra no construtor, campo `final` | EJ Item 5; [Spring Framework](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html) ("The Spring team generally advocates constructor injection"); [Spring Boot](https://docs.spring.io/spring-boot/reference/using/spring-beans-and-dependency-injection.html) | P3, E5 | `ast-grep` **`injecao-por-campo-java`** (seção 4); ArchUnit `GeneralCodingRules.NO_CLASSES_SHOULD_USE_FIELD_INJECTION` (testado). O EP não tem regra para isso |
| Domínio não depende de framework, ORM, SQL nem SDK | princípio da casa (D, P3) | D, P3 | `ast-grep` **`dominio-sem-infra-java`** (seção 4); ArchUnit `noClasses().that().resideInAPackage("..dominio..").should().dependOnClassesThat().resideInAnyPackage(...)` (testado) ou `Architectures.onionArchitecture()`; Checkstyle [`ImportControl`](https://checkstyle.org/checks/imports/importcontrol.html) serve a `lint-cobre`. Comparação na seção 4.7 |
| Configuração por ambiente lida num ponto só, em objeto tipado (`@ConfigurationProperties`) | [Spring Boot, "Type-safe Configuration Properties"](https://docs.spring.io/spring-boot/reference/features/external-config.html#features.external-config.typesafe-configuration-properties) | P3, E3 | `ast-grep` **`ambiente-so-na-montagem-java`** (seção 4) |
| Organizar por conceito do domínio (`customer/`, `order/`), não por camada | [Spring Boot, "Structuring Your Code"](https://docs.spring.io/spring-boot/reference/using/structuring-your-code.html) (o layout típico é por domínio) | P4 | `comando` com ArchUnit `slices().matching("br.com.loja.(*)..").should().beFreeOfCycles()` ou Spring Modulith `verify()`; a decisão de layout é `revisor` |
| Teste pela interface pública; mock só no que não é nosso | princípio da casa | P5 | `ast-grep` `dominio-sem-infra-java` apontado para `src/test/java/**/dominio/**` com `org\.mockito` nos pacotes proibidos; nome do teste fica com o `revisor` |
| Um só formatador, sem discussão de estilo | [google-java-format](https://github.com/google/google-java-format) (implementa o GJS) | P8 | `comando` `mvn spotless:check` / `gradle spotlessCheck` (testado) |
| Uma só biblioteca de log e de datas | princípio da casa | P8 | ArchUnit `NO_CLASSES_SHOULD_USE_JAVA_UTIL_LOGGING`, `NO_CLASSES_SHOULD_USE_JODATIME`, `OLD_DATE_AND_TIME_CLASSES_SHOULD_NOT_BE_USED`; EP `JavaUtilDate` (WARNING, ligado; apareceu no teste em Gradle), `JavaTimeDefaultTimeZone`; `ast-grep` `saida-padrao-java` |
| Sem import com curinga | [GJS 3.3.1](https://google.github.io/styleguide/javaguide.html#s3.3.1-wildcard-imports) | P8 | `lint-cobre` Checkstyle `AvoidStarImport` |
| Classe que faz coisa demais | princípio da casa | S | `revisor`; sinais por `lint-cobre`: PMD `GodClass`, `TooManyMethods`, `CouplingBetweenObjects`; Checkstyle `ClassFanOutComplexity` |
| Composição em vez de herança; projetar para herança ou proibi-la | EJ Itens 18, 19; OSCG 4-5 EXTEND-5 | O, L | `revisor`; Checkstyle `FinalClass` (classe só com construtor privado) |
| Variante fechada com `sealed` + `switch` exaustivo sem `default`: variante nova quebra a compilação em todo `switch` | [JEP 409](https://openjdk.org/jeps/409) (Java 17), [JEP 441](https://openjdk.org/jeps/441) (Java 21), [JEP 440](https://openjdk.org/jeps/440); [GJS 4.8.4.3](https://google.github.io/styleguide/javaguide.html#s4.8.4.3-switch-default) | O | o próprio `javac` (exaustividade); EP `MissingCasesInEnumSwitch`, `StatementSwitchToExpressionSwitch` (WARNING, ligados); PMD `NonExhaustiveSwitch`. Cuidado: Checkstyle `MissingSwitchDefault` exige `default` e anula a exaustividade em `sealed`; não ligar junto |
| Adapter cumpre a port inteira | princípio da casa | L | `teste-de-contrato-por-port`, com a ressalva do layout Maven (seção 6) |
| `@Override` sempre | EJ Item 40; [GJS 6.1](https://google.github.io/styleguide/javaguide.html#s6.1-override-annotation) | L (indireto) | EP `MissingOverride` (WARNING, ligado); PMD `MissingOverride` |
| Commit | princípio da casa | P9 | `mensagem-de-commit` (independe da linguagem) |
| P6, P7 | princípio da casa | P6, P7 | `revisor` (P7 ganha sinal com os `Unused*` acima) |

## 2. Práticas que não cabem nos 22 (candidatas a princípio de Java)

1. **Recurso se fecha em todo caminho** (`try-with-resources`). EJ Item 9; OSCG 1-2 DOS-2 ("Release
   resources in all cases"). Nenhum dos 22 fala de vazamento de recurso. Verificação: PMD
   `CloseResource`, `UseTryWithResources`; SpotBugs `OBL_UNSATISFIED_OBLIGATION`,
   `ODR_OPEN_DATABASE_RESOURCE`.
2. **Contratos de `Object`: `equals` com `hashCode`, `compareTo` coerente com `equals`.** EJ Itens
   10, 11, 14. Está perto de L, mas L fala de adapter e port, não do contrato da plataforma.
   Verificação: EP `EqualsHashCode` (ERROR, ligado); PMD `OverrideBothEqualsAndHashcode`; SpotBugs
   `HE_EQUALS_USE_HASHCODE`. Em `record`, o compilador gera os dois.
3. **Concorrência: estado mutável compartilhado só com sincronização; executor em vez de `Thread`;
   documentar a segurança de thread.** EJ Itens 78 a 82. Verificação parcial: EP
   `FutureReturnValueIgnored` (WARNING, ligado), `InterruptedExceptionSwallowed` (desligado), a
   categoria `multithreading` do PMD, SpotBugs categoria `MT_CORRECTNESS`. Com threads virtuais
   ([JEP 444](https://openjdk.org/jeps/444), Java 21) o tema fica mais comum em Spring Boot.
4. **Injeção e desserialização: SQL nunca montado por concatenação; desserialização de dado não
   confiável só com filtro.** OSCG 3-2 INJECT-2, 8-6 SERIAL-6; EJ Item 85. Os 22 não têm princípio de
   segurança. Verificação: SpotBugs `SQL_NONCONSTANT_STRING_PASSED_TO_EXECUTE`; plugin
   [Find Security Bugs](https://github.com/find-sec-bugs/find-sec-bugs) 1.14.0 (LGPL-3.0) para o
   resto (SSRF, XXE, criptografia fraca); PMD `security` (`HardCodedCryptoKey`, `InsecureCryptoIv`).
5. **Classe que pode ser estendida não chama método sobrescrevível no construtor.** OSCG 7-4
   OBJECT-4; EJ Item 19. Verificação: SpotBugs `CT_CONSTRUCTOR_THROW` (parcial), PMD
   `ConstructorCallsOverridableMethod` (categoria `errorprone`).
6. **Primitivo em vez de boxed; nada de `==` em objeto.** EJ Item 61. Verificação: EP
   `ReferenceEquality` (WARNING, ligado); PMD `CompareObjectsWithEquals`.

Os três primeiros são os que valem virar princípio de Java (ou um grupo "Plataforma" no mapa de
Regras de código): têm fonte canônica e checagem gratuita pronta.

## 3. Ferramentas

| Ferramenta | Versão (Maven Central, 2026-10-23) | Licença (fonte) | Cobra? | Comando no CI: Maven | Comando no CI: Gradle |
|---|---|---|---|---|---|
| [Checkstyle](https://github.com/checkstyle/checkstyle) | 14.3.0; plugin Maven 3.6.0 | LGPL-2.1-or-later (POM e LICENSE) | Não | `mvn -B checkstyle:check` (plugin com `configLocation`; dependência `checkstyle` 14.3.0 para não ficar na versão antiga embutida) | `gradle checkstyleMain` (plugin `checkstyle` do Gradle, `toolVersion = '14.3.0'`, `maxWarnings = 0`) |
| [PMD](https://github.com/pmd/pmd) (inclui CPD) | 7.28.0; plugin Maven 3.28.0 | BSD-style (LICENSE do repositório; o POM não declara) | Não | `mvn -B compile pmd:check` (com `failurePriority` 5 para quebrar em toda prioridade) e `mvn -B pmd:cpd-check` | `gradle pmdMain` (plugin `pmd`, `toolVersion = '7.28.0'`, `ruleSets = []`, `ruleSetFiles = files('config/pmd.xml')`) |
| [SpotBugs](https://github.com/spotbugs/spotbugs) | 4.10.5; plugin Maven 4.10.5.0; plugin Gradle `com.github.spotbugs` 6.5.13 | LGPL-2.1 | Não | `mvn -B compile com.github.spotbugs:spotbugs-maven-plugin:4.10.5.0:check` | `gradle spotbugsMain` |
| [Error Prone](https://github.com/google/error-prone) | 2.51.0 (publicada em 2026-10-22) | Apache-2.0 (POM pai e COPYING) | Não | roda no `mvn compile`: `maven-compiler-plugin` com `-XDcompilePolicy=simple`, `-XDaddTypeAnnotationsToSymbol=true` (exigido no JDK 21), `--should-stop=ifError=FLOW`, `-Xplugin:ErrorProne -Xep:<Regra>:ERROR ...`, `error_prone_core` em `annotationProcessorPaths` e `.mvn/jvm.config` com os `--add-exports`/`--add-opens` de `jdk.compiler` | plugin `net.ltgt.errorprone` 5.1.1, `errorprone 'com.google.errorprone:error_prone_core:2.51.0'`, `options.errorprone.error('<Regra>')`; roda no `gradle compileJava` |
| [NullAway](https://github.com/uber/NullAway) | 0.14.3 | MIT (POM e LICENSE.txt) | Não | junto do Error Prone: `annotationProcessorPaths` + `-Xep:NullAway:ERROR -XepOpt:NullAway:OnlyNullMarked=true` | `errorprone 'com.uber.nullaway:nullaway:0.14.3'` + `options.errorprone.error('NullAway')` |
| [JSpecify](https://github.com/jspecify/jspecify) (anotações) | 1.0.0 | Apache-2.0 | Não | dependência comum | dependência comum |
| [ArchUnit](https://github.com/TNG/ArchUnit) | 1.5.1 (`archunit-junit5`) | Apache-2.0 (README e LICENSE) | Não | é teste: `mvn -B test -Dtest='*ArquiteturaTest'` | `gradle test --tests '*ArquiteturaTest'` |
| [Spring Modulith](https://github.com/spring-projects/spring-modulith) | 2.1.2 | Apache-2.0 (POM) | Não | é teste: `mvn -B test -Dtest='*ModulosTest'` | `gradle test --tests '*ModulosTest'` |
| [google-java-format](https://github.com/google/google-java-format) via [Spotless](https://github.com/diffplug/spotless) | gjf 1.37.0; Spotless Maven 3.10.4, Gradle 8.10.4 | Apache-2.0 os dois | Não | `mvn -B spotless:check` | `gradle spotlessCheck` |
| [Find Security Bugs](https://github.com/find-sec-bugs/find-sec-bugs) | 1.14.0 | LGPL-3.0 (LICENSE) | Não | plugin do SpotBugs (`<plugins>` do `spotbugs-maven-plugin`) | `spotbugsPlugins 'com.h3xstream.findsecbugs:findsecbugs-plugin:1.14.0'` |
| [jscpd](https://github.com/kucherenko/jscpd) | 5.4.2 (npm) | MIT | Não | `npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` | o mesmo |
| [ast-grep](https://github.com/ast-grep/ast-grep) | 0.51.0 (npm) | MIT | Não | tipo `ast-grep` do validador | o mesmo |
| [SonarQube Community Build](https://github.com/SonarSource/sonarqube) | não medido | plataforma LGPL-3.0; **o analisador de Java (`sonar-java`) está sob a Sonar Source-Available License v1.0 desde 29/11/2024** (README e LICENSE.txt do `sonar-java`) | A plataforma não; análise de branch e de PR só nas edições comerciais (README do `sonarqube`) | — | — |

**Decisão sugerida para o SonarQube: não usar.** Não cobra para rodar, mas exige um servidor, não
analisa PR na edição gratuita (que é onde o validador roda) e o analisador de Java deixou de ser
código aberto. Tudo o que ele pegaria nesta tabela já sai de PMD, Checkstyle, SpotBugs e Error Prone.

**Recomendação de conjunto mínimo para um projeto Spring Boot:** Error Prone + NullAway (no
compilador, sem passo extra), PMD com um ruleset curto, Spotless, ArchUnit para a arquitetura e as
seis regras de ast-grep. Checkstyle só se o projeto quiser as regras de tamanho e nome que o PMD não
cobre (`ParameterNumber`, `MagicNumber`, `ImportControl`, convenções de nome). SpotBugs pesa (14 s no
projeto de teste) e entra quando houver Find Security Bugs.

Observações dos testes:
- **Warning do Error Prone não quebra o build.** As regras ligadas por padrão como WARNING
  (`EmptyCatch`, `CatchAndPrintStackTrace`, `UnusedVariable`, `MissingOverride`...) só passam a
  reprovar com `-Xep:<Regra>:ERROR` ou `-Werror`. Listar cada uma com `:ERROR` também deixa a regra
  visível para o `lint-cobre` (ver seção 6).
- **Severidade e padrão no Error Prone 2.51.0**, lidos do jar: ERROR e ligado: `ReturnValueIgnored`,
  `CheckReturnValue`, `EqualsHashCode`. WARNING e ligado: `EmptyCatch`, `CatchAndPrintStackTrace`,
  `UnusedVariable`, `UnusedMethod`, `MissingOverride`, `ReferenceEquality`, `NullOptional`,
  `MutablePublicArray`, `MixedMutabilityReturnType`, `JavaTimeDefaultTimeZone`,
  `MissingCasesInEnumSwitch`, `StatementSwitchToExpressionSwitch`, `FutureReturnValueIgnored`,
  `PatternMatchingInstanceof`, `BadImport`. Desligados: `SystemOut`, `TooManyParameters`,
  `FieldCanBeFinal`, `CatchingUnchecked`, `InterruptedExceptionSwallowed`, `SystemExitOutsideMain`,
  `UnusedException`, `ReturnMissingNullable`, `UnnecessaryOptionalGet`, `IdentifierName`. Não
  existem no Error Prone: `OptionalGetWithoutIsPresent` (é inspeção do IntelliJ) e `FieldInjection`.
- **Spotless com `googleJavaFormat` fixado em 1.37.0 falhou** no JDK 21
  (`InvocationTargetException`); sem fixar a versão (o Spotless 3.10.4 baixou a 1.36.1) funcionou, e o
  gjf 1.37.0 chamado direto também funcionou. Não achei a causa em fonte primária **[confirmar]**.
- O PMD 7.28 marca `AvoidCatchingGenericException` em `design.xml` como `deprecated` (aponta para
  `errorprone.xml`); use `category/java/errorprone.xml/AvoidCatchingGenericException`.

## 4. Regras de ast-grep para Java

Todas testadas com ast-grep 0.51.0, cada uma num exemplo que deve pegar e num que deve passar
(pastas de teste fora do repositório). As três primeiras são o documento Java dos modelos que já existem:
entram como mais um `---` em `catch-vazio.yml`, `dominio-sem-infra.yml` e
`ambiente-so-na-montagem.yml` (testei o `catch-vazio.yml` atual com o documento Java acrescentado:
6 achados, os mesmos da regra sozinha). As três últimas são novas.

**Comando que funciona**: `npx -y -p @ast-grep/cli ast-grep scan --rule <arquivo> <pasta>`. O
comando do pedido, `npx -y @ast-grep/cli scan ...`, falha nesta máquina com "could not determine
executable to run", porque o pacote tem dois binários (`ast-grep` e `sg`). Com `npm i -g
@ast-grep/cli` (o que o validador.md manda no CI) não há problema.

### 4.1 `catch-vazio-java` (E4)

```yaml
# Princípio E4: erro nunca é engolido. Pega o catch vazio, o que só tem comentário e o que só loga,
# imprime ou chama printStackTrace e segue. Bloco que relança, retorna ou trata passa.
id: catch-vazio-java
language: Java
message: "E4: o erro é engolido. Trate o caso esperado ou deixe o erro subir até a borda."
severity: error
rule:
  kind: catch_clause
  has:
    field: body
    kind: block
    not:
      has:
        not:
          any:
            - regex: '^[{}]$'   # as chaves do bloco também são filhas
            - kind: line_comment
            - kind: block_comment
            - kind: expression_statement
              regex: '(?s)^((log|logger|LOG|LOGGER|System\.(out|err))\s*\.\s*\w+\s*\(.*\)|\w+\s*\.\s*printStackTrace\s*\(\s*\))\s*;$'
```

Deve pegar (6 de 6 pegos):
```java
try { g(); } catch (IOException e) { }
try { g(); } catch (IOException | RuntimeException e) { /* nada a fazer */ }
try { g(); } catch (Exception e) { log.error("falhou", e); }
try { g(); } catch (Exception e) { e.printStackTrace(); }
try { g(); } catch (Exception e) { LOGGER.warn("falhou ao cobrar pedido {}",
    pedido.id(), e); }                       // log em várias linhas
try (var in = abrir()) { ler(in); } catch (IOException e) { System.err.println(e); }
```
Deve passar (0 achados):
```java
try { g(); } catch (IOException e) { throw new UncheckedIOException("falha ao ler pedido", e); }
try { g(); } catch (IOException e) { log.error("falhou", e); throw new CobrancaFalhou(pedido.id(), e); }
try { return g(); } catch (CupomVencido e) { return Resultado.recusado(e.motivo()); }
try { g(); } finally { fechar(); }
```
Detalhe de implementação: em `has`, as chaves `{` `}` do bloco contam como filhas; sem a linha
`regex: '^[{}]$'` a regra não pega nada (testado). O nome do logger fica na regex; troque se o
projeto usa outro (`log|logger|LOG|LOGGER`).

### 4.2 `dominio-sem-infra-java` (D, P3)

```yaml
# Princípios D e P3: o domínio não importa infraestrutura. Copie para docs/regras-de-codigo/ast-grep/
# e troque os marcadores: <PASTA_DO_DOMINIO> (ex.: src/main/java/br/com/loja/dominio) e
# <PACOTES_DE_INFRA> (regex com pontos escapados, ex.: org\.springframework|jakarta\.persistence|
# org\.hibernate|com\.fasterxml\.jackson|java\.sql|software\.amazon). Pega import comum, estático e
# com curinga, e nome qualificado no código (anotação, tipo, new). Chamada estática qualificada
# (org.x.Classe.metodo()) escapa: o ArchUnit, que lê o bytecode, cobre.
id: dominio-sem-infra-java
language: Java
message: "D/P3: o domínio importa infraestrutura. Defina a port no domínio; o adapter vive fora."
severity: error
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  any:
    - kind: import_declaration
      regex: '^import\s+(static\s+)?(<PACOTES_DE_INFRA>)\.'
    - any:
        - kind: scoped_identifier
        - kind: scoped_type_identifier
      regex: '^(<PACOTES_DE_INFRA>)\.'
      not:
        any:
          - inside: { kind: scoped_identifier }
          - inside: { kind: scoped_type_identifier }
          - inside: { kind: import_declaration, stopBy: end }
          - inside: { kind: package_declaration, stopBy: end }
```

Teste com `<PACOTES_DE_INFRA>` = `org\.springframework|jakarta\.persistence|org\.hibernate|java\.sql`.
Deve pegar (7 de 7, um achado por linha, sem duplicar pelos nós aninhados), em `dominio/`:
```java
import jakarta.persistence.Entity;
import org.springframework.transaction.annotation.Transactional;
import static org.springframework.util.Assert.notNull;
import java.sql.Connection;
import org.springframework.*;
@org.springframework.stereotype.Service class Fatura {
  java.sql.Connection conexao; }
```
Deve passar (0 achados): `import java.util.List;`, `import java.time.Instant;`, `import
br.com.loja.dominio.portas.Cobranca;`, `import org.springframeworkish.Fake;` (o `\.` no fim evita
prefixo falso) e o mesmo `import jakarta.persistence.Entity;` num arquivo de `infra/` (fora de `files`).

Diferença do modelo de TS: em Java o pacote não é string, então a regra casa o texto do
`import_declaration` em vez de `string_fragment`. Atenção ao copiar: os globs de `files` e
`ignores` resolvem a partir do diretório de trabalho; com o arquivo da regra **fora** da árvore que
se varre (testei `--rule ../regra.yml`), `files` não casou nada. Dentro de
`docs/regras-de-codigo/ast-grep/`, como manda o validador.md, funciona.

### 4.3 `ambiente-so-na-montagem-java` (P3)

```yaml
# Princípio P3: configuração por ambiente se lê num ponto só, a montagem. Copie para
# docs/regras-de-codigo/ast-grep/ e troque <ARQUIVOS_DA_MONTAGEM> (ex.: src/main/java/**/config/**).
# Em Spring Boot, a montagem é a classe @Configuration ou um record @ConfigurationProperties.
id: ambiente-so-na-montagem-java
language: Java
message: "P3: leitura de ambiente fora da montagem. Receba o valor pelo construtor."
severity: error
ignores: ["<ARQUIVOS_DA_MONTAGEM>"]
rule:
  any:
    - pattern: System.getenv($$$)
    - pattern: System.getProperty($$$)
    - kind: annotation
      regex: '^@(org\.springframework\.beans\.factory\.annotation\.)?Value\s*\('
```
Deve pegar (5 de 5), fora da montagem:
```java
@Value("${gateway.chave}") private String chave;
private final String url = System.getenv("GATEWAY_URL");
String fuso() { return System.getProperty("user.timezone", "UTC"); }
var todas = System.getenv();
Cobranca(@org.springframework.beans.factory.annotation.Value("${x}") String x) { }
```
Deve passar (0 achados): `@ValueObject record Taxa(int v)`, `mapa.getenv("x")`,
`props.getProperty("y")`, construtor que recebe `Percentual limite`, e um `GatewayConfig` com
`@Bean ... @Value(...) ... System.getenv(...)` dentro de `<ARQUIVOS_DA_MONTAGEM>`. Não pega
`Environment#getProperty` injetado (precisa de tipo); fica com o revisor.

### 4.4 Nova: `injecao-por-campo-java` (P3, E5)

```yaml
# Princípios P3 e E5: dependência entra pelo construtor, em campo final. Pega @Autowired, @Inject e
# @Resource em campo. Em teste (@MockBean, @Autowired em teste do Spring) aponte `ignores` para
# src/test/**. Fonte: documentação do Spring Framework e do Spring Boot (injeção por construtor).
id: injecao-por-campo-java
language: Java
message: "P3/E5: injeção por campo. Receba a dependência pelo construtor, em campo final."
severity: error
ignores: ["src/test/**"]
rule:
  kind: field_declaration
  has:
    kind: modifiers
    has:
      any:
        - kind: marker_annotation
        - kind: annotation
      regex: '^@((org\.springframework\.beans\.factory\.annotation\.)?Autowired|((jakarta|javax)\.inject\.)?Inject|((jakarta|javax)\.annotation\.)?Resource)\b'
```
Deve pegar (4 de 4): `@Autowired private PedidoRepository pedidos;`, `@Autowired(required = false)
private Relogio relogio;`, `@Inject Cobranca cobranca;`, `@jakarta.annotation.Resource private Fila
fila;`. Deve passar (0): `@Autowired` no construtor, `@Autowired` em setter, `@Deprecated private
int contador;`, `@AutowiredLike private Outro outro;`.
Por que ast-grep além do ArchUnit (`NO_CLASSES_SHOULD_USE_FIELD_INJECTION`, que também pega):
roda sem compilar e só nos arquivos alterados.

### 4.5 Nova: `saida-padrao-java` (E4, P8)

```yaml
# Princípios E4 e P8: o registro de erro e de evento passa pelo logger que o projeto escolheu.
# System.out, System.err e printStackTrace somem do código de produção. Aponte `ignores` para o
# que é CLI de verdade (ex.: um main de ferramenta).
id: saida-padrao-java
language: Java
message: "E4/P8: saída padrão no código. Use o logger do projeto (ou trate o erro)."
severity: error
ignores: ["src/test/**"]
rule:
  any:
    - pattern: System.out.$M($$$)
    - pattern: System.err.$M($$$)
    - pattern: $E.printStackTrace()
```
Deve pegar (3 de 3): `System.out.println("total " + total);`, `System.err.printf("%d%n", 3);`,
`e.printStackTrace();`. Deve passar (0): `log.info("total {}", total);`,
`e.printStackTrace(writer);` (com argumento, vai para destino escolhido), `saida.out.println("x");`.
Equivalentes: PMD `SystemPrintln` + `AvoidPrintStackTrace`, EP `SystemOut` (desligado por padrão),
ArchUnit `NO_CLASSES_SHOULD_ACCESS_STANDARD_STREAMS`.

### 4.6 Nova: `optional-get-direto-java` (E4)

```yaml
# Princípio E4: ausência é resultado esperado e faz parte da interface. Pega `.get()` encadeado logo
# depois de uma chamada que devolve Optional (findById, findFirst, findAny, max, min, reduce,
# Optional.of/ofNullable). Use orElseThrow com o erro de domínio, orElse ou map. Sem tipo, o
# ast-grep não sabe que `x.get()` é Optional; por isso a regra olha só o encadeamento.
id: optional-get-direto-java
language: Java
message: "E4: Optional.get() sem tratar a ausência. Use orElseThrow(() -> new <ErroDoDominio>(...)), orElse ou map."
severity: error
rule:
  any:
    - pattern: $R.$M($$$).get()
    - pattern: $M($$$).get()
constraints:
  M:
    regex: '^(findById|findOne|findFirst|findAny|findBy\w+|max|min|reduce|of|ofNullable)$'
```
Deve pegar (5 de 5): `pedidos.findById(id).get()`, `itens.stream().filter(Item::ativo).findFirst().get()`,
`valores.stream().max(Integer::compare).get()`, `clientes.findByEmail(email).get()`,
`Optional.ofNullable(nome).get()`. Deve passar (0): `findById(id).orElseThrow(() -> new
PedidoNaoEncontrado(id))`, `mapa.get("x")`, `referencia.get()`, `futuro.get()`,
`findFirst().orElse(Item.VAZIO)`.
Limite: `Optional` guardado em variável e depois `opcional.get()` escapa (precisa de tipo). Foi a
escolha entre ruído e cobertura: `$X.get()` sem argumento pegaria também `Supplier`, `Future`,
`AtomicReference`. Nenhuma ferramenta gratuita testada cobre o caso com tipo; o EP não tem regra
para `Optional.get()` sem checagem.

Fora das seis: `catch (Exception e)` genérico não precisa de ast-grep, porque PMD
`AvoidCatchingGenericException` e Checkstyle `IllegalCatch` já pegam (testado) e aceitam supressão
na borda.

Tempo: as seis regras sobre os exemplos levam 0,4 s com o `npx` (quase tudo é arranque do npx).

### 4.7 ArchUnit ou ast-grep para "domínio sem infraestrutura" (D, P3)

| | ast-grep `dominio-sem-infra-java` | ArchUnit `noClasses()...resideInAnyPackage(...)` |
|---|---|---|
| Lê | o fonte, sem compilar | o bytecode compilado (README do ArchUnit: "analyzing given Java bytecode") |
| Pega | `import` (comum, estático, curinga), nome qualificado em anotação, tipo e `new` | toda dependência real: import, nome qualificado, chamada estática, herança, tipo em assinatura, anotação (testado: "Class Pedido is annotated with org.springframework.stereotype.Service") |
| Escapa | chamada estática totalmente qualificada (`org.x.Classe.metodo()`), dependência que chega por tipo de outra classe nossa | nada no nível de classe; não vê o que não compila |
| Incremental (`--base`) | sim, só os arquivos alterados, em décimos de segundo | não: importa todas as classes do pacote; precisa de `compile` + `test` (1,1 s o teste, mais o build) |
| Onde roda | tipo `ast-grep` do validador | tipo `comando` (`mvn -B test -Dtest='*ArquiteturaTest'`) e na suíte de testes do projeto |
| Vai além | — | camadas (`layeredArchitecture()`), cebola (`onionArchitecture()`), ciclos entre fatias (`slices()...beFreeOfCycles()`, P4), regras prontas em `GeneralCodingRules` |
| Licença | MIT | Apache-2.0 |

**Recomendação:** os dois, com papéis diferentes. O ast-grep fica como a checagem do validador
(rápida, incremental, sem compilar, a mesma forma do TS e do Python). O ArchUnit entra como teste do
projeto, na primeira fatia, porque é a única forma gratuita de garantir a regra **completa** e de
cobrir P4 (ciclos entre pacotes de domínio). No `docs/regras-de-codigo.json` do projeto Java, a regra
D pode ter as duas verificações em regras separadas (`R6` ast-grep, `R6b` comando ArchUnit).

## 5. Exemplo de `docs/regras-de-codigo.json` para um projeto Java

Mesmo formato do modelo; as regras de `lint-cobre` apontam para as configurações que o texto cita.

```json
{
  "versao": 1,
  "projeto": { "codigo": ["src/main/java/**", "src/test/java/**"], "ignorar": ["target/**", "build/**"] },
  "regras": [
    { "id": "R2", "principio": "E2", "texto": "Até 3 parâmetros e 2 níveis de if",
      "verificacao": { "tipo": "lint-cobre", "config": "config/checkstyle.xml", "regras": ["ParameterNumber", "NestedIfDepth"] } },
    { "id": "R3", "principio": "E4", "texto": "Nenhum catch vazio ou que só loga",
      "verificacao": { "tipo": "ast-grep", "regras": "docs/regras-de-codigo/ast-grep/catch-vazio.yml" } },
    { "id": "R3b", "principio": "E4", "texto": "Valor de retorno e catch vazio barrados no compilador",
      "verificacao": { "tipo": "lint-cobre", "config": "pom.xml", "regras": ["-Xep:EmptyCatch:ERROR", "-Xep:CatchAndPrintStackTrace:ERROR"] } },
    { "id": "R5", "principio": "E7", "texto": "Sem bloco duplicado de 8 linhas ou mais",
      "verificacao": { "tipo": "comando", "executar": "npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}" } },
    { "id": "R6", "principio": "D", "texto": "O domínio não importa infraestrutura",
      "verificacao": { "tipo": "ast-grep", "regras": "docs/regras-de-codigo/ast-grep/dominio-sem-infra.yml" } },
    { "id": "R6b", "principio": "D", "texto": "O domínio não depende de infraestrutura (bytecode)",
      "verificacao": { "tipo": "comando", "executar": "mvn -B -q test -Dtest='*ArquiteturaTest' -Dsurefire.failIfNoSpecifiedTests=false" } },
    { "id": "R10", "principio": "P3", "texto": "Dependência entra pelo construtor",
      "verificacao": { "tipo": "ast-grep", "regras": "docs/regras-de-codigo/ast-grep/injecao-por-campo.yml" } },
    { "id": "R11", "principio": "E8", "texto": "Sem número mágico",
      "verificacao": { "tipo": "lint-cobre", "config": "config/checkstyle.xml", "regras": ["MagicNumber"] } }
  ]
}
```

## 6. Achados sobre o motor e os modelos (fora do pedido, mas afetam Java)

1. **A receita do jscpd no modelo está quebrada na versão atual.** `regras-de-codigo.json` (R5) e o
   `validador.md` usam `--exitCode 1`. O jscpd 5.4.2 aceita só `--exit-code` e, com `--exitCode`,
   sai com **2 sempre**, até em pasta sem duplicação (testado). Como `comando` trata saída diferente
   de 0 como violação, a R5 do modelo reprova todo projeto. Troque por `--exit-code 1`.
2. **`teste-de-contrato-por-port` não serve ao layout Maven/Gradle.** O motor troca `{dir}` pela
   pasta inteira da port (`validador.rs`, linhas 427 a 455). Em Java a port fica em
   `src/main/java/br/com/loja/dominio/portas/Cobranca.java` e o teste em
   `src/test/java/br/com/loja/dominio/portas/CobrancaContratoTest.java`: nenhum modelo com `{dir}`
   chega lá. Sugestão para o motor: um marcador que troque a raiz (por exemplo
   `{dir:src/main/java=src/test/java}`) ou um `{pacote}` relativo a uma raiz declarada. Também o
   padrão de `ignorar` (`**/*.test.*`, `**/*.spec.*`) não reconhece `*Test.java`; não atrapalha
   porque as ports ficam em `src/main`, mas convém documentar.
3. **`lint-cobre` procura o nome como substring no arquivo.** Funciona com Checkstyle
   (`<module name="ParameterNumber">`), com PMD (`category/java/design.xml/ExcessiveParameterList`) e
   com `-Xep:Regra:ERROR` no `pom.xml`/`build.gradle`. Duas ressalvas para Java: regra ligada por
   **padrão** no Error Prone não aparece no arquivo (liste-a com `:ERROR`, o que também a torna
   bloqueante); e uma regra **comentada** no XML (`<!-- <module name="MagicNumber"/> -->`) passa na
   checagem.
4. **`npx -y @ast-grep/cli scan` não roda** ("could not determine executable to run"); o certo é
   `npx -y -p @ast-grep/cli ast-grep scan`. Vale corrigir onde o método citar a primeira forma.

## Fontes

- Joshua Bloch, *Effective Java*, 3ª ed., Addison-Wesley, 2018 (ISBN 9780134685991), página da
  editora: https://www.pearson.com/store/p/effective-java/P200000000138 **[confirmar: numeração dos
  itens]**.
- Google Java Style Guide: https://google.github.io/styleguide/javaguide.html (lido em
  https://raw.githubusercontent.com/google/styleguide/gh-pages/javaguide.html).
- Oracle, Secure Coding Guidelines for Java SE: https://www.oracle.com/java/technologies/javase/seccodeguide.html
- JEPs: [358](https://openjdk.org/jeps/358) (NPE úteis, Java 14), [394](https://openjdk.org/jeps/394)
  (pattern matching para `instanceof`, Java 16), [395](https://openjdk.org/jeps/395) (records, Java
  16), [409](https://openjdk.org/jeps/409) (sealed, Java 17), [440](https://openjdk.org/jeps/440)
  (record patterns, Java 21), [441](https://openjdk.org/jeps/441) (pattern matching para `switch`,
  Java 21), [444](https://openjdk.org/jeps/444) (threads virtuais, Java 21); índice em https://openjdk.org/jeps/0.
- Spring Framework, Dependency Injection:
  https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html
  (fonte: `framework-docs/.../factory-collaborators.adoc`, `spring-projects/spring-framework`).
- Spring Boot: injeção por construtor
  (https://docs.spring.io/spring-boot/reference/using/spring-beans-and-dependency-injection.html),
  estrutura do código (https://docs.spring.io/spring-boot/reference/using/structuring-your-code.html),
  configuração tipada (https://docs.spring.io/spring-boot/reference/features/external-config.html)
  (fonte: `.adoc` em `spring-projects/spring-boot`).
- Artefatos em https://repo1.maven.org/maven2/: `com/puppycrawl/tools/checkstyle/14.3.0`,
  `net/sourceforge/pmd/pmd-java/7.28.0`, `com/github/spotbugs/spotbugs/4.10.5`,
  `com/google/errorprone/error_prone_core/2.51.0`, `com/tngtech/archunit/archunit/1.5.1`,
  `com/uber/nullaway/nullaway/0.14.3`, `com/google/googlejavaformat/google-java-format/1.37.0`,
  `com/diffplug/spotless/spotless-maven-plugin/3.10.4`, `org/jspecify/jspecify/1.0.0`,
  `com/h3xstream/findsecbugs/findsecbugs-plugin/1.14.0`,
  `org/springframework/modulith/spring-modulith-core/2.1.2`; plugins Gradle em
  https://plugins.gradle.org/m2/ (`com.github.spotbugs` 6.5.13, `net.ltgt.errorprone` 5.1.1,
  `com.diffplug.spotless` 8.10.4).
- LICENSE/README nos repositórios: `checkstyle/checkstyle`, `pmd/pmd`, `spotbugs/spotbugs`,
  `google/error-prone` (COPYING), `TNG/ArchUnit`, `uber/NullAway`, `google/google-java-format`,
  `diffplug/spotless`, `find-sec-bugs/find-sec-bugs`, `SonarSource/sonarqube`,
  `SonarSource/sonar-java`.
- npm: https://registry.npmjs.org/jscpd/latest (5.4.2, MIT), https://registry.npmjs.org/@ast-grep/cli/latest
  (0.51.0, MIT); linguagens do ast-grep: `website/reference/languages.md` em
  `ast-grep/ast-grep.github.io` (Java: `java`).
- Pesquisa anterior: `desenvolvimento/research/2026-10-23-ferramentas-do-validador.md`.
