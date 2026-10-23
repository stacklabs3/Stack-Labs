# Práticas de Java

Parte dos [princípios de código](../principios-de-codigo.md). Como cada princípio aparece em Java
(21 LTS ou mais, Spring Boot), a regra de máquina que o confere e as práticas **só desta linguagem**
(JV1 a JV6). Fontes primárias (Effective Java, Google Java Style, Oracle Secure Coding, JEPs,
Spring), testes em Maven e Gradle e o porquê de cada linha: pesquisa
`desenvolvimento/research/2026-10-23-praticas-java.md` do Stack-Labs.

No grilling, cada linha é a recomendação da pergunta do princípio; o projeto decide. No diff, a
regra ligada é violação dura; a desligada é julgamento.

## Ferramentas

Nenhuma cobra. Fixe a versão no `pom.xml` ou no `build.gradle`.

| Ferramenta | Versão | Papel |
|---|---|---|
| Error Prone + NullAway (+ JSpecify) | 2.51.0 + 0.14.3 (+ 1.0.0) | No compilador: bugs e nulidade |
| PMD (inclui CPD) | 7.28.0 | Desenho, erro, código morto |
| Checkstyle | 14.3.0 | Nome, tamanho, número mágico, imports |
| ArchUnit | 1.5.1 | Teste de arquitetura: camadas, ciclos, injeção |
| Spotless + google-java-format | 3.10.4 (Maven) / 8.10.4 (Gradle) | Formatador único |
| SpotBugs (+ Find Security Bugs) | 4.10.5 (+ 1.14.0) | Opcional: pesa uns 14 s; entra com a segurança (JV4) |
| `@ast-grep/cli`, `jscpd` | 0.51.0, 5.4.2 | Validador |

**Conjunto mínimo:** Error Prone com NullAway, PMD, Spotless, ArchUnit e os modelos de ast-grep.
Checkstyle quando o projeto quer as regras de nome e tamanho que o PMD não tem.

**Warning do Error Prone não reprova.** Toda regra que o projeto adotar entra no `pom.xml` como
`-Xep:<Regra>:ERROR`: isso quebra o build e deixa a regra visível para o `lint-cobre`. No JDK 21, o
Error Prone exige `-XDaddTypeAnnotationsToSymbol=true`.

## Por princípio

| Princípio | Em Java | Confere |
|---|---|---|
| E1 | `UpperCamelCase`, `lowerCamelCase`, `UPPER_SNAKE_CASE`; sigla como palavra | Checkstyle `TypeName`, `MethodName`, `MemberName`, `ConstantName`, `AbbreviationAsWordInName`; `termo-evitado` |
| E2 | Até 3 parâmetros; aninhamento raso; variável perto do uso | Checkstyle `ParameterNumber`, `NestedIfDepth`, `MethodLength`, `VariableDeclarationUsageDistance`; PMD `CognitiveComplexity` |
| E3 | Valor do domínio é `record` validado no construtor compacto; nulidade no tipo (`@NullMarked`, `@Nullable` onde falta) | `-Xep:NullAway:ERROR -XepOpt:NullAway:OnlyNullMarked=true`; o tipo próprio é do revisor |
| E4 | Exceção nunca ignorada; nada de `catch (Exception)` fora da borda; causa encadeada; ausência é `Optional`, e `get()` só depois de tratar; retorno não se ignora | ast-grep `catch-vazio.yml`, `optional-get-direto.yml`; PMD `EmptyCatchBlock`, `AvoidCatchingGenericException` (categoria `errorprone`), `PreserveStackTrace`; Error Prone `CatchAndPrintStackTrace`, `ReturnValueIgnored`, `NullOptional` |
| E4 (verificador) | `@SuppressWarnings` só com o porquê ao lado | Revisor |
| E5 | Campo `final`; classe imutável; cópia defensiva; parâmetro não se reatribui | PMD `ImmutableField`, `AvoidReassigningParameters`, `MutableStaticState`; Error Prone `MutablePublicArray` |
| E6 | Código morto sai; `TODO` com a issue | Error Prone `UnusedVariable`, `UnusedMethod`; PMD `UnusedPrivateField`; `todo-sem-issue` |
| E7 | Bloco repetido | `npx jscpd@5.4.2 --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` ou `mvn pmd:cpd-check` |
| E8 | Número com significado vira constante; conjunto fechado vira `enum` | Checkstyle `MagicNumber` |
| P1, I | Classe de implementação package-private; só a interface é pública | ArchUnit `classes().that().resideInAPackage("..adapters..").should().notBePublic()` |
| P3, D | Injeção por construtor, campo `final`; domínio sem Spring, JPA, SQL ou SDK; configuração tipada (`@ConfigurationProperties`) lida na montagem | ast-grep `injecao-por-campo.yml`, `dominio-sem-infra.yml`, `ambiente-so-na-montagem.yml`; ArchUnit (abaixo) |
| P3 | Saída no terminal e log por uma biblioteca só | ast-grep `saida-padrao.yml` |
| P4 | Pacote por conceito do domínio, sem ciclo entre fatias | ArchUnit `slices().matching("<base>.(*)..").should().beFreeOfCycles()` ou Spring Modulith `verify()` |
| P5 | Mock só no que não é nosso | ast-grep `dominio-sem-infra.yml` apontado para `src/test/java/**/dominio/**` com `org\.mockito` |
| P8 | Um formatador; sem import com curinga; uma biblioteca de log e de datas | `mvn spotless:check`; Checkstyle `AvoidStarImport`; ArchUnit `GeneralCodingRules` |
| O | Variante fechada é `sealed`, e o `switch` sobre ela não tem `default` | `javac` (exaustividade); Error Prone `MissingCasesInEnumSwitch`; nunca Checkstyle `MissingSwitchDefault` junto |
| L | `@Override` sempre; adapter passa no contrato da port | Error Prone `MissingOverride`; `teste-de-contrato-por-port` com `{dir_de_teste}` |

**ArchUnit e ast-grep em D e P3:** os dois, com papéis diferentes. O ast-grep fica no validador
porque lê o fonte, só os arquivos alterados, em décimos de segundo. O ArchUnit entra como teste do
projeto, pelo `comando` (`mvn -B test -Dtest='*ArquiteturaTest'`), porque lê o bytecode e pega o que
o fonte esconde: chamada estática qualificada, herança, ciclo entre pacotes.

## Ressalvas

- **Interface com uma implementação só é seam hipotético** (P2). O par `PedidoService` e
  `PedidoServiceImpl` vira uma classe `PedidoService`. A interface nasce quando o segundo adapter
  existe (em geral, o de teste). O item 64 do Effective Java ("refira-se pela interface") fala de
  interfaces que já existem, como `List`.
- **Limite de tamanho (E2) mede o método, não desenha a classe.** Quebre o método longo em métodos
  privados; a interface pública continua pequena (P1).
- **Javadoc em toda API exposta** (Google, Effective Java item 56) choca com E6. Em código de
  aplicação, o padrão é afastar; em biblioteca publicada, adotar.

## Só desta linguagem

Cada uma é decidida no ticket *Lint e formatador* do mapa de Regras de código e, adotada, vira regra
com `"principio": "JV<n>"` no `.json` e uma linha na tabela *Práticas da linguagem* do
`docs/regras-de-codigo.md`.

| ID | Prática | Confere |
|---|---|---|
| JV1 | **Recurso se fecha em todo caminho**, com `try-with-resources` | PMD `CloseResource`, `UseTryWithResources`; SpotBugs `OBL_UNSATISFIED_OBLIGATION` |
| JV2 | **Contratos de `Object`:** `equals` com `hashCode`; `compareTo` coerente com `equals` (o `record` gera os dois) | Error Prone `EqualsHashCode`; PMD `OverrideBothEqualsAndHashcode` |
| JV3 | **Concorrência:** estado mutável compartilhado só com sincronização; executor em vez de `Thread`; `Future` não ignorado | Error Prone `FutureReturnValueIgnored`; PMD categoria `multithreading` |
| JV4 | **Injeção e desserialização:** SQL nunca por concatenação; desserialização de dado não confiável só com filtro | SpotBugs `SQL_NONCONSTANT_STRING_PASSED_TO_EXECUTE`; Find Security Bugs |
| JV5 | **Construtor não chama método sobrescrevível** | PMD `ConstructorCallsOverridableMethod` |
| JV6 | **Objeto se compara com `equals`;** primitivo antes de boxed | Error Prone `ReferenceEquality`; PMD `CompareObjectsWithEquals` |

## Modelos

Em `.claude/watchdogs/modelos/`: `regras-de-codigo.java.json`, `lint/checkstyle.xml`,
`lint/pmd.xml`, `lint/error-prone.pom.xml` (com o `.mvn/jvm.config` em `lint/error-prone.jvm.config`) e, em `ast-grep/`, os documentos `Java`. Copie só o que o
grilling decidiu.
