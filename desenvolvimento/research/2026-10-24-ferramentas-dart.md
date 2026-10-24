# Ferramentas de verificação para Dart e Flutter

Pesquisa de 2026-10-24 para o ticket [Ferramentas de verificação para Dart e
Flutter](https://github.com/stacklabs1/Stack-Labs/issues/20), filho do mapa [Dart e Flutter no CI
local](https://github.com/stacklabs1/Stack-Labs/issues/18). Pergunta: que ferramenta, com versão fixa,
confere código Dart/Flutter em cada tipo de checagem do validador, e de que formas um teste Dart é
declarado e afirma. Ambiente dos testes locais: macOS arm64, Dart SDK 3.12.2, Flutter 3.44.9,
`@ast-grep/cli` 0.51.0, `jscpd` 5.4.2.

## Resultado

| Checagem | Ferramenta e versão | Licença / custo | Receita | Situação |
|---|---|---|---|---|
| Padrões de código (tipo `ast-grep`) | ast-grep 0.51.0, **Dart nativo** (`language: Dart`) | MIT, grátis | `npx -y -p @ast-grep/cli@0.51.0 ast-grep scan --rule <arquivo> <pasta>` | **Serve.** Testado: as regras `teste-desligado` e `teste-sem-afirmacao` em Dart acharam os 6 casos esperados entre 14 testes, sem falso positivo |
| Lint e typecheck, projeto Dart puro | `dart analyze` do SDK fixado | BSD-3 (SDK), grátis | `dart analyze --fatal-infos` | Sem `--fatal-infos` o lint (nível *info*) **não reprova**: testado, sai 0 com um `avoid_print` |
| Lint e typecheck, projeto Flutter | `flutter analyze` do SDK fixado | BSD-3, grátis | `flutter analyze` | `--fatal-infos` e `--fatal-warnings` já vêm ligados |
| Conjunto de regras de lint | `lints` 6.1.0, `flutter_lints` 6.0.0, `very_good_analysis` 11.0.0 | BSD-3 / MIT, grátis | `include:` no `analysis_options.yaml` | Mudar a versão pode reprovar código que passava |
| Regras próprias com tipos | plugin do analisador (Dart 3.10+) | grátis | `plugins:` no `analysis_options.yaml`; roda dentro do `dart analyze` | Alternativa ao ast-grep para regra que precisa do tipo |
| Regras próprias (legado) | `custom_lint` 0.8.1 | Apache-2.0 | `dart run custom_lint` | Comando separado do `dart analyze`; última versão em 2025-09-09 |
| Regras prontas e métricas | DCM 1.40.2 | **Comercial**; chave de CI só no plano Teams (US$ 80/mês, 5 assentos) | — | Pago: pela ADR 0010, não usar |
| Formatador | `dart format` do SDK fixado | BSD-3, grátis | `dart format --output=none --set-exit-if-changed {arquivos}` | Testado: sai 1 com arquivo fora do formato |
| Duplicação (E7) | jscpd 5.4.2 | MIT, grátis | a receita atual, com `--min-tokens` (abaixo) | **Lê `.dart`** (formato `dart` no `--list`); testado: achou o clone e saiu 1 |
| Dependências sem uso, faltando ou no lugar errado | `dependency_validator` 5.1.0 | Apache-2.0, grátis | `dart pub global activate dependency_validator 5.1.0 && dart pub global run dependency_validator` | Sai 1 ao achar infração |
| Dependências auditadas (vulnerabilidade) | OSV-Scanner v2.7.0 | Apache-2.0, grátis | `osv-scanner scan source --lockfile pubspec.lock` | Sai 1 com vulnerabilidade, 128 sem pacote lido. O `dart pub get` só avisa |
| Mutação (P5) | `mutation_test` 1.8.1 | BSD-3, grátis | `dart run mutation_test <arquivos .dart alterados de lib/>` com `<threshold failure="80">` no XML | Por arquivo, não por linha do diff; sai -1 (255) abaixo do limite |

## Padrões de código: o ast-grep lê Dart

- **Dart é linguagem nativa do ast-grep desde a 0.42.1.** O CHANGELOG registra "remove builtin dart
  support" na 0.30.0 e "Support Dart again" ([PR 2534](https://github.com/ast-grep/ast-grep/pull/2534))
  na 0.42.1 ([CHANGELOG.md](https://github.com/ast-grep/ast-grep/blob/main/CHANGELOG.md)). O
  `crates/language/Cargo.toml` depende de `tree-sitter-dart = "0.2.0"` e o `src/lib.rs` liga a
  extensão `dart` à linguagem `Dart`
  ([crates/language](https://github.com/ast-grep/ast-grep/tree/main/crates/language), consultado em
  2026-10-24). A gramática é o crate tree-sitter-dart 0.2.0 (MIT, publicado em 2026-04-26,
  repositório [nielsenko/tree-sitter-dart](https://github.com/nielsenko/tree-sitter-dart),
  [crates.io](https://crates.io/crates/tree-sitter-dart)).
- **A página de linguagens está desatualizada:** a [tabela de linguagens
  nativas](https://ast-grep.github.io/reference/languages.html) não lista Dart, mas o binário 0.51.0
  aceita `-l dart` e recusa uma linguagem inexistente (`klingon is not supported!`). O catálogo tem
  uma [seção Dart](https://ast-grep.github.io/catalog/dart/) que usa `-l dart`. Não é preciso
  `customLanguages` nem compilar gramática.
- **Cuidado com padrão solto:** o Dart não aceita expressão no nível do arquivo, então `-p 'f($$$A)'`
  vira um nó `ERROR`. O catálogo envolve o padrão numa função e usa `--selector call_expression`
  ([catálogo Dart](https://ast-grep.github.io/catalog/dart/)). Regra em YAML com `kind:` não tem o
  problema.
- **Nós que as regras de teste usam** (lidos com `--debug-query=cst`): a chamada é `call_expression`
  com `function: identifier` e `arguments: arguments`; o corpo do teste é `function_expression`; o
  argumento nomeado é `named_argument` com um `label` (`skip:`); a anotação de arquivo é `annotation`.
- **Teste local.** Duas regras escritas para Dart, rodadas com `ast-grep scan` sobre dois arquivos de
  teste com 14 testes. Acharam exatamente: o `test` sem afirmação, o `testWidgets` sem afirmação, o
  `test` com `skip: 'motivo'`, o `test` com `solo: true`, o `group` com `skip: true` e o `@Skip(...)`
  do arquivo. Não acusaram `skip: false`, `blocTest` com `expect:` nem com `skip: 1`, `verify(...)` do mocktail nem
  `expectLater(..., matchesGoldenFile(...))`. Núcleo das regras:

  ```yaml
  # teste-desligado-dart
  rule:
    any:
      - kind: call_expression
        all:
          - has: { field: function, regex: '^(test|testWidgets|group|patrolTest|patrolWidgetTest|goldenTest)$' }
          - has:
              field: arguments
              has:
                kind: named_argument
                all:
                  - has: { kind: label, regex: '^(skip|solo)\s*:' }
                  - not: { has: { regex: '^false$' } }
      - { kind: annotation, regex: '^@Skip\b' }
  # teste-sem-afirmacao-dart
  rule:
    kind: call_expression
    all:
      - has: { field: function, regex: '^(test|testWidgets|patrolTest|patrolWidgetTest)$' }
      - has: { field: arguments, has: { kind: function_expression } }
      - not: { has: { stopBy: end, kind: identifier, regex: '^(expect|verify|fail|conferir)' } }
  ```

  O `blocTest` fica de fora do `teste-desligado` de propósito: o `skip` dele é outra coisa (abaixo).
- **Quando a regra precisa do tipo** (ex.: "o domínio não importa `package:flutter`" cabe no
  ast-grep; "este valor é um `Future` não aguardado" não cabe), o caminho grátis é um plugin do
  analisador. O suporte entrou no Dart 3.10, ativa-se na seção `plugins:` do `analysis_options.yaml` e
  os diagnósticos saem no `dart analyze` e no `flutter analyze`; regra de lint de plugin vem desligada
  até ser ligada em `diagnostics:` ([Analyzer plugins](https://dart.dev/tools/analyzer-plugins)). O
  sistema antigo (`analyzer: plugins:`, usado pelo `custom_lint`) aceita um plugin por arquivo e "will
  be deprecated in a future release" ([Customizing static analysis](https://dart.dev/tools/analysis)).

## Lint, typecheck e formatador

- **`dart analyze` não reprova lint.** "Normally the analyzer reports failure when it finds any
  errors or warnings, but not when it finds info-level issues"; `--fatal-infos` muda isso
  ([dart analyze](https://dart.dev/tools/dart-analyze)). As regras de lint saem como *info*. Testado
  no SDK 3.12.2: `avoid_print` ligado, `dart analyze` sai 0 e `dart analyze --fatal-infos` sai 1.
  Aceita arquivo ou pasta como argumento, então `{arquivos}` funciona.
- **`flutter analyze` reprova por padrão:** `fatal-infos` e `fatal-warnings` têm `defaultsTo: true`
  em `packages/flutter_tools/lib/src/commands/analyze.dart` do Flutter 3.44.9 (lido no SDK local).
- **Severidade e rigor no `analysis_options.yaml`:** `analyzer: errors: <regra>: error` sobe uma
  regra para erro; `strict-casts`, `strict-inference` e `strict-raw-types` vêm `false`; `lints` traz
  os conjuntos *core* e *recommended*, `flutter_lints` é um superconjunto para Flutter, e uma versão
  nova de `lints` pode reprovar código que passava
  ([Customizing static analysis](https://dart.dev/tools/analysis)). Versões em pub.dev, consultadas em
  2026-10-24: [lints 6.1.0](https://pub.dev/packages/lints),
  [flutter_lints 6.0.0](https://pub.dev/packages/flutter_lints),
  [very_good_analysis 11.0.0](https://pub.dev/packages/very_good_analysis).
- **`lint-cobre` e o `analysis_options.yaml`:** é YAML, e o validador hoje só tem leitura própria para
  `.toml`, `.xml` e `.json`; cai na regra "Outras" (o nome fora de comentário, sem `false` logo
  depois), que serve para `- avoid_print` e `avoid_print: true` (catálogo em
  `docs/agents/validador.md` do modelo de projeto).
- **Formatador:** com `--set-exit-if-changed`, "If changes occur, the `dart format` command returns
  an exit code of `1`"; `-o none` não grava os arquivos; a largura vai em `formatter: page_width` do
  `analysis_options.yaml` e as opções configuráveis exigem *language version* 3.7
  ([dart format](https://dart.dev/tools/dart-format)). Testado: sai 1 com arquivo fora do formato. O
  formatador vem no SDK, então a versão fixa é a do SDK do projeto.
- **DCM:** produto comercial da CQLabs. Plano grátis: 1 assento, até 50 mil linhas, 100 regras; Pro
  US$ 16/mês; a "CI/CD key" só aparece no Teams, US$ 80/mês por 5 assentos
  ([preços](https://dcm.dev/pricing), consultado em 2026-10-24). Última versão: 1.40.2
  ([CQLabs/homebrew-dcm](https://github.com/CQLabs/homebrew-dcm/releases)). Pela ADR 0010 (com
  cobrança, escrevemos a nossa), fica de fora. O `dart_code_linter` 4.4.2 (fork comunitário do antigo
  `dart_code_metrics`, [pub.dev](https://pub.dev/packages/dart_code_linter)) não foi avaliado aqui.
- **`custom_lint` 0.8.1** (Apache-2.0, publicado em 2025-09-09): roda como plugin do sistema antigo e
  o próprio README avisa que o `dart analyze` não pega as regras dele; precisa de `dart run
  custom_lint` à parte ([pub.dev](https://pub.dev/packages/custom_lint)).

## Duplicação

- O jscpd 5.4.2 (MIT, [registro do npm](https://registry.npmjs.org/jscpd/latest)) lista `dart` entre
  os formatos (`jscpd --list`). Testado com dois arquivos `.dart` de 10 linhas iguais:
  `--min-lines 8 --min-tokens 10 --threshold 0 --exit-code 1` achou 1 clone e saiu 1.
- **Achado que vale para toda linguagem:** com o `--min-tokens` padrão (50), arquivos pequenos foram
  descartados e o jscpd saiu **0** com "jscpd analyzed no files". A receita atual do `validador.md`
  (`--min-lines 8 --threshold 0 --exit-code 1`) passa em silêncio nesse caso. O `--help` da 5.4.2 tem
  `--fail-on-empty` ("Exit 1 when the scan analyzes no files [...] or every file was below
  --min-tokens"); ligá-la na receita é decisão do motor, fora desta pesquisa.

## Dependências

- **Sem uso, faltando, promovida demais ou de menos:** `dependency_validator` 5.1.0, da Workiva,
  Apache-2.0 ([pub.dev](https://pub.dev/packages/dependency_validator),
  [LICENSE](https://github.com/Workiva/dependency_validator/blob/master/LICENSE)). Configura-se em
  `dependency_validator.yaml` (`ignore`, `exclude`, `allow_pins`). O código põe `exitCode = 1` ao
  achar infração (`lib/src/dependency_validator.dart`, commit
  [7728cb7](https://github.com/Workiva/dependency_validator/blob/7728cb7f27de46d3fefc515018f54b9aee3a2587/lib/src/dependency_validator.dart)).
- **Vulnerabilidade:** o pub mostra os avisos do GitHub Advisory Database na resolução e aceita
  `ignored_advisories` no `pubspec.yaml`; a página não traz comando que saia com erro, e o exemplo
  termina em "Got dependencies!" ([Security advisories](https://dart.dev/tools/pub/security-advisories)).
  Para reprovar, o OSV-Scanner lê `pubspec.lock`
  ([lockfiles suportados](https://google.github.io/osv-scanner/supported-languages-and-lockfiles/)),
  sai 1 com vulnerabilidade e 128 quando não acha pacote
  ([saída](https://google.github.io/osv-scanner/output/)). Última versão: v2.7.0, de 2026-10-12
  ([releases](https://github.com/google/osv-scanner/releases)).

## Mutação (P5)

- **`mutation_test` 1.8.1** (BSD-3, publicado em 2026-09-12,
  [pub.dev](https://pub.dev/packages/mutation_test)): troca texto por regex ou literal, roda
  `dart test` (o comando é configurável no XML, então `flutter test` cabe), aceita `--coverage
  lcov.info` para pular código sem cobertura e define o limite em `<threshold failure="80">`. Não lê o
  git: o README mostra a lista de arquivos alterados vinda de `git diff --name-only` passada como
  argumento; restringir a linhas exige `<lines>` no XML.
- **Código de saída:** `bin/mutation_test.dart` termina com `exit(-1)` quando nem todos os mutantes
  foram mortos, `exit(1)` em erro de processamento e `exit(0)` no sucesso (commit
  [fc1fd86](https://github.com/domohuhn/mutation-test/blob/fc1fd86218abca7336cb21c5548060df94be6448/bin/mutation_test.dart)).
  É menos maduro que `cargo-mutants` e PIT: o mutante é textual, não sintático, e o README avisa que
  pode levar horas em base grande.

## Formas de declarar um teste e de afirmar

| Pacote (versão) | Declara | Desliga ou foca | Afirma | Fonte |
|---|---|---|---|---|
| `test` 1.33.0 | `test(descricao, corpo)`, `group(descricao, corpo)`; `setUp`, `tearDown`, `setUpAll`, `tearDownAll` | `skip:` (`true` ou texto com o motivo) em `test` e `group`; `solo: true` (marcado `@doNotSubmit`) roda só os marcados no arquivo; `@Skip('motivo')` antes de `library;` desliga o arquivo inteiro e não pode ser desfeito por `skip:`; `onPlatform: {'windows': Skip(...)}` desliga por plataforma | `expect(real, matcher)`, `expectLater(...)` (assíncrono, devolve `Future`), `fail(...)` | `pkgs/test_core/lib/src/scaffolding.dart`, commit [da21f85](https://github.com/dart-lang/test/blob/da21f85a842c0a8fc79bfba66bffd6623190a7fb/pkgs/test_core/lib/src/scaffolding.dart) |
| `flutter_test` (Flutter 3.44.9) | `testWidgets(descricao, (WidgetTester tester) async {...})`; `test` e `group` próprios | `skip:` (`bool?`) em `testWidgets`, `test` e `group`; **sem `solo`** | `expect`, `expectSync`, `expectLater`; matchers `findsOneWidget`, `findsNothing` etc. | `packages/flutter_test/lib/src/widget_tester.dart` (linhas 148, 466, 485, 497) e `test_compat.dart` (linhas 175, 208) do SDK local |
| Golden (flutter_test) | dentro de `testWidgets` | o do `testWidgets` | `await expectLater(finder, matchesGoldenFile('x.png'))` | `matchers.dart`, linha 588, do SDK local |
| `alchemist` 0.14.1 | `goldenTest(descricao, fileName:, builder:)` | `skip: true` (`bool`, padrão `false`) | **implícita**: o próprio `goldenTest` compara a imagem; não há `expect` no corpo | `lib/src/golden_test.dart`, commit [a462226](https://github.com/Betterment/alchemist/blob/a462226801e7d77dd0c71ab5293265a1c869e2a3/lib/src/golden_test.dart) |
| `golden_toolkit` 0.15.0 | `testGoldens(...)` | — | `screenMatchesGolden` | **Descontinuado** no pub.dev, última versão em 2023-02-21 ([pub.dev](https://pub.dev/packages/golden_toolkit)) |
| `bloc_test` 10.0.0 | `blocTest<Bloc, Estado>(descricao, build:, act:, expect:, verify:, errors:)`, todos nomeados | **não tem desligar**: `skip` é `int` (quantos estados ignorar antes de comparar, padrão 0) | **nos argumentos nomeados** `expect:`, `verify:` e `errors:`; não há corpo com `expect(...)` | `packages/bloc_test/lib/src/bloc_test.dart`, linhas 27 e 139-152 ([felangel/bloc](https://github.com/felangel/bloc/blob/master/packages/bloc_test/lib/src/bloc_test.dart)) |
| `mocktail` 1.0.5 | dentro de `test`/`testWidgets` | — | `verify(() => mock.m()).called(n)`, `verifyNever`, `verifyInOrder`, `verifyNoMoreInteractions`, `verifyZeroInteractions` | `packages/mocktail/lib/src/mocktail.dart`, linhas 335-458 ([felangel/mocktail](https://github.com/felangel/mocktail/blob/main/packages/mocktail/lib/src/mocktail.dart)) |
| `mockito` 5.8.1 | dentro de `test` | — | `verify`, `verifyNever`, `verifyInOrder` (mesmos nomes) | [pub.dev](https://pub.dev/packages/mockito) |
| `patrol` 4.10.0 | `patrolTest(descricao, ($) async {...})`; `patrolWidgetTest` (em `patrol_finders`) | `skip:` (`bool?`) | `expect` do flutter_test | `packages/patrol/lib/src/common.dart` linha 93 e `packages/patrol_finders/lib/src/common.dart` linha 27, commit [c5defac](https://github.com/leancodepl/patrol/tree/c5defac580a619822a502836234c800697b153ff/packages) |

Versões consultadas na API do pub.dev em 2026-10-24.

**O que isso muda no motor (insumo para a spec, não decisão):**

- **Arquivos de teste:** `test/**/*_test.dart` e `integration_test/**` já caem em `**/*_test.*` e
  `**/test/**` do `TESTES_PADRAO` (`src/riscos.rs`). Falta a declaração: `declaracao_padrao` não tem
  braço para `dart` e devolve `None`. Uma linha que cobre os pacotes acima:
  `^\s*(?:test|testWidgets|blocTest|patrolTest|patrolWidgetTest|goldenTest)\b[^(]*\(`.
- **`blocTest` quebra a leitura por linha:** com os argumentos nomeados, o `dart format` costuma pôr a
  descrição (onde mora o `RK<n>`) na linha **seguinte** à declaração, como nos exemplos do
  [README do bloc_test](https://github.com/felangel/bloc/blob/master/packages/bloc_test/README.md). O `testes_do_arquivo` só lê a linha da declaração, o cabeçalho acima e
  as linhas que a contêm, então não veria o risco. O mesmo vale para `test(` quando a descrição longa
  quebra a linha.
- **`trecho_do_teste` da revisão por modelo** conta chaves a partir da declaração: num `test(...,
  () {...}, skip: ...)` corta o `skip:` final; num `blocTest` com `act: (bloc) {...}` para no fim do
  `act` e perde o `expect:`.
- **Afirmação:** além de `expect`/`verify`/`fail`, contam `expectLater`, os `verify*` do mocktail e
  do mockito, os argumentos `expect:`/`verify:`/`errors:` do `blocTest` e o `goldenTest` inteiro.
- **Desligado:** `skip:` diferente de `false` e `solo:` em `test`/`group`/`testWidgets`/`patrolTest`/
  `goldenTest`, e `@Skip` no arquivo; **nunca** o `skip:` do `blocTest`.

## Fontes

- ast-grep: [CHANGELOG](https://github.com/ast-grep/ast-grep/blob/main/CHANGELOG.md),
  [crates/language](https://github.com/ast-grep/ast-grep/tree/main/crates/language),
  [linguagens](https://ast-grep.github.io/reference/languages.html),
  [catálogo Dart](https://ast-grep.github.io/catalog/dart/),
  [linguagem customizada](https://ast-grep.github.io/advanced/custom-language.html),
  [registro do npm](https://registry.npmjs.org/@ast-grep/cli/latest) (0.51.0, MIT),
  [tree-sitter-dart no crates.io](https://crates.io/crates/tree-sitter-dart).
- Dart: [dart analyze](https://dart.dev/tools/dart-analyze),
  [Customizing static analysis](https://dart.dev/tools/analysis),
  [Analyzer plugins](https://dart.dev/tools/analyzer-plugins),
  [dart format](https://dart.dev/tools/dart-format),
  [Security advisories](https://dart.dev/tools/pub/security-advisories).
- Flutter 3.44.9 (SDK local): `packages/flutter_tools/lib/src/commands/analyze.dart`,
  `packages/flutter_test/lib/src/widget_tester.dart`, `test_compat.dart`, `matchers.dart`.
- Pacotes: [test scaffolding.dart](https://github.com/dart-lang/test/blob/da21f85a842c0a8fc79bfba66bffd6623190a7fb/pkgs/test_core/lib/src/scaffolding.dart),
  [bloc_test.dart](https://github.com/felangel/bloc/blob/master/packages/bloc_test/lib/src/bloc_test.dart),
  [mocktail.dart](https://github.com/felangel/mocktail/blob/main/packages/mocktail/lib/src/mocktail.dart),
  [alchemist golden_test.dart](https://github.com/Betterment/alchemist/blob/a462226801e7d77dd0c71ab5293265a1c869e2a3/lib/src/golden_test.dart),
  [patrol](https://github.com/leancodepl/patrol/tree/c5defac580a619822a502836234c800697b153ff/packages),
  [golden_toolkit](https://pub.dev/packages/golden_toolkit), [mockito](https://pub.dev/packages/mockito),
  [custom_lint](https://pub.dev/packages/custom_lint),
  [dependency_validator](https://pub.dev/packages/dependency_validator),
  [mutation_test](https://pub.dev/packages/mutation_test) e
  [bin/mutation_test.dart](https://github.com/domohuhn/mutation-test/blob/fc1fd86218abca7336cb21c5548060df94be6448/bin/mutation_test.dart),
  [lints](https://pub.dev/packages/lints), [flutter_lints](https://pub.dev/packages/flutter_lints),
  [very_good_analysis](https://pub.dev/packages/very_good_analysis).
- jscpd: [registro do npm](https://registry.npmjs.org/jscpd/latest) (5.4.2, MIT) e o `--list`/`--help` do binário.
- OSV-Scanner: [lockfiles](https://google.github.io/osv-scanner/supported-languages-and-lockfiles/),
  [saída](https://google.github.io/osv-scanner/output/),
  [releases](https://github.com/google/osv-scanner/releases).
- DCM: [preços](https://dcm.dev/pricing), [releases](https://github.com/CQLabs/homebrew-dcm/releases).
