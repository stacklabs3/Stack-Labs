# Dart e Flutter: os 22 princípios e o que só a linguagem tem

Pesquisa de 2026-10-24, ticket [#19](https://github.com/stacklabs1/Stack-Labs/issues/19) do mapa
[#18](https://github.com/stacklabs1/Stack-Labs/issues/18). Pergunta: qual é a prática de Dart e
Flutter, em fonte primária, para cada um dos 22 princípios da casa (E1–E8, P1–P9, S O L I D), e
quais práticas só Dart/Flutter têm (candidatas a `DT<n>`, ADR 0012). Mesmo formato de
[2026-10-23-praticas-rust.md](2026-10-23-praticas-rust.md). Dart e Flutter num arquivo só, como o
React dentro do `typescript.md`.

Fora daqui: ferramentas além do SDK (jscpd, ast-grep para Dart, mutação, auditoria de dependência)
são da pesquisa irmã [#20](https://github.com/stacklabs1/Stack-Labs/issues/20). Nesta, a coluna
*Verificação* cita só o que o SDK do Dart/Flutter e os pacotes oficiais de lint já trazem, e diz
"revisor" ou "#20" onde não há regra.

## Como foi feito, e o limite das fontes

- Fontes lidas na página publicada, em 2026-10-24: Effective Dart
  ([style], [usage], [design], [documentation]), [Customizing static analysis][analysis],
  [Linter rules](https://dart.dev/tools/linter-rules) e a página de cada regra
  (`https://dart.dev/lints/<regra>`), guias de linguagem do dart.dev, o
  [guia de arquitetura do Flutter][arq-rec], a API do Flutter (api.flutter.dev) e os pacotes oficiais
  no pub.dev.
- Lista de regras de cada conjunto: lida nos arquivos dos pacotes, não na página. `core.yaml` e
  `recommended.yaml` em [dart-lang/core/pkgs/lints](https://github.com/dart-lang/core/tree/main/pkgs/lints/lib),
  `flutter.yaml` em [flutter/packages/flutter_lints](https://github.com/flutter/packages/tree/main/packages/flutter_lints/lib).
  O `main` de `lints` já é `7.0.0-wip` (acrescenta `switch_on_type` ao core e
  `simple_directive_paths`, `simplify_variable_pattern`, `unnecessary_unawaited` ao recommended,
  segundo o [CHANGELOG](https://github.com/dart-lang/core/blob/main/pkgs/lints/CHANGELOG.md)); a
  tabela abaixo segue a versão **publicada** 6.1.0, conferida no `pubspec.lock` do projeto de teste.
- Estado e versão de cada regra: o arquivo de máquina do linter,
  [`pkg/linter/tool/machine/rules.json`](https://github.com/dart-lang/sdk/blob/main/pkg/linter/tool/machine/rules.json)
  (264 regras, com `state` e `sinceDartSdk`).
- Versões atuais (2026-10-24): **Dart 3.13.7** (estável, 2026-10-22, pelo
  [arquivo de versões do Dart](https://storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION)),
  **Flutter 3.47.8** (estável, 2026-10-23, com Dart 3.13.7, pelo
  [manifesto de releases](https://storage.googleapis.com/flutter_infra_release/releases/releases_macos.json)),
  **`lints` 6.1.0** (2026-01-30, [pub.dev](https://pub.dev/packages/lints)), **`flutter_lints` 6.0.0**
  (2025-05-27, [pub.dev](https://pub.dev/packages/flutter_lints)), **`test` 1.33.0** (2026-10-08,
  [pub.dev](https://pub.dev/packages/test)), **`leak_tracker_flutter_testing` 3.0.10**
  ([pub.dev](https://pub.dev/packages/leak_tracker_flutter_testing)). Os dois pacotes de lint exigem
  Dart `^3.8.0`.
- Toolchain local dos testes: **Flutter 3.44.9 / Dart 3.12.2** (um trimestre atrás do estável). Todo
  comportamento marcado "testado" rodou num `flutter create --empty` em pasta fora do repositório,
  com `flutter_lints` 6.0.0 e `lints` 6.1.0.

Atalhos de link usados nas tabelas:
[style] = <https://dart.dev/effective-dart/style> ·
[usage] = <https://dart.dev/effective-dart/usage> ·
[design] = <https://dart.dev/effective-dart/design> ·
[documentation] = <https://dart.dev/effective-dart/documentation> ·
[analysis] = <https://dart.dev/tools/analysis> ·
[arq-rec] = <https://docs.flutter.dev/app-architecture/recommendations> ·
regra `x` = `https://dart.dev/lints/x`. Entre parênteses, o conjunto onde a regra já vem ligada:
**core**, **rec** (recommended, inclui core), **flutter** (`flutter_lints`, inclui rec) ou
**fora** (precisa ligar).

## 0. Achados que mudam o método (ler primeiro)

1. **`dart analyze` aprova com lint violado** (testado). Toda regra de lint tem severidade `info`, e
   `dart analyze` só falha com `info` se receber `--fatal-infos`: com 10 achados de lint, saiu com
   código 0; com `--fatal-infos`, código 1. O `flutter analyze` faz o contrário: `--fatal-infos` e
   `--fatal-warnings` são ligados por padrão (`flutter analyze --help`), e o mesmo projeto saiu com
   1. A [página de análise][analysis] diz: info é "An informational message that doesn't cause
   analysis to fail". **Receita para o `comando`:** `flutter analyze` em projeto Flutter, e
   `dart analyze --fatal-infos` em pacote Dart puro. Para uma regra crítica, subir a severidade no
   próprio arquivo (`analyzer: errors: use_build_context_synchronously: error`, testado: vira
   `error` e o código de saída passa a 3).
2. **`lint-cobre` dá falso aprovado com `analysis_options.yaml`** (testado numa cópia dos watchdogs,
   com um teste a mais em `tests/validador.rs`). O YAML cai na leitura genérica de
   `cobertura_de_lint` (`src/validador.rs`), que entende `regra: false` como desligada e `# regra`
   como comentada, mas:
   - `analyzer: errors: avoid_print: ignore` (regra **calada**) → aprovado, porque `ignore` não está
     na lista de valores desligados (`off`, `allow`, `false`, `0`);
   - regra ligada só pelo `include: package:flutter_lints/flutter.yaml` → "não está configurada"
     (falha fechado, como a ADR 0012 decidiu para preset; o projeto precisa repetir a regra no
     arquivo ou o motor precisa resolver o `include`).
   **Proposta para o motor** (decidir na spec do mapa): ler o YAML, exigir a regra em `linter: rules:`
   (lista ou mapa com `true`) e tratar `analyzer: errors: <regra>: ignore` como desligada. O `comando`
   do achado 1 continua sendo o que prova que o código cumpre.
3. **Lint por pasta existe sem ferramenta extra** (testado). O analisador usa o
   `analysis_options.yaml` mais próximo do arquivo; um `lib/dominio/analysis_options.yaml` com
   `include: ../../analysis_options.yaml` e regras a mais (`avoid_catches_without_on_clauses`,
   `avoid_positional_boolean_parameters`, `avoid_dynamic_calls`, `only_throw_errors`) pegou as quatro
   só em `lib/dominio/`, e a cópia do mesmo arquivo em `lib/` não. A ordem de mescla do `include`
   está na [página de análise][analysis] ("options defined directly in the current
   analysis_options.yaml file override any included options").
4. **O linter do Dart não tem regra de tamanho, complexidade, aninhamento, número mágico nem de
   import proibido** (procurado nas 264 regras do `rules.json`). E2, E8 e o P3 por import ficam para
   revisor, para regra própria (os [analyzer plugins](https://dart.dev/tools/analyzer-plugins), desde o
   Dart 3.10, deixam o projeto escrever lints que rodam no `dart analyze`) ou para a ferramenta que a
   #20 achar.
5. **`dispose` esquecido não tem lint; tem teste** (testado). `cancel_subscriptions` e `close_sinks`
   só olham `StreamSubscription` e `Sink`; um `TextEditingController` sem `dispose` passou no
   analisador. O `leak_tracker_flutter_testing`, ligado em `test/flutter_test_config.dart` com
   `LeakTesting.enable()` ([DETECT.md](https://github.com/dart-lang/leak_tracker/blob/main/doc/leak_tracking/DETECT.md)),
   reprovou o `testWidgets` com `notDisposed: TextEditingController` e passou depois do `dispose`.
6. **`prefer_const_constructors` saiu do `flutter_lints`** na 5.0.0, junto com
   `prefer_const_declarations` e `prefer_const_literals_to_create_immutables`
   ([CHANGELOG do flutter_lints](https://github.com/flutter/packages/blob/main/packages/flutter_lints/CHANGELOG.md));
   no teste, `EdgeInsets.all(8)` sem `const` passou com o conjunto padrão e foi pego ao ligar a regra.
   A API do Flutter continua recomendando: "Use const widgets where possible"
   ([StatelessWidget](https://api.flutter.dev/flutter/widgets/StatelessWidget-class.html)). É decisão
   do projeto (DT4).

## 1. Tabela: prática → fonte → princípio da casa → verificação

"Analisador (CI)" abaixo significa: a regra no `analysis_options.yaml` (`lint-cobre`, com a correção
do achado 2) **mais** o `comando` `flutter analyze` ou `dart analyze --fatal-infos` (achado 1).

### Escrita (E1–E8)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| `UpperCamelCase` para tipos e extensões; `lowerCamelCase` para o resto, inclusive constantes; `lowercase_with_underscores` para arquivo, pasta, pacote e prefixo de import | [style](https://dart.dev/effective-dart/style#do-name-types-using-uppercamelcase), [#lowercamelcase](https://dart.dev/effective-dart/style#do-name-other-identifiers-using-lowercamelcase), [#constantes](https://dart.dev/effective-dart/style#prefer-using-lowercamelcase-for-constant-names) | E1 (caixa) | Analisador (CI): `camel_case_types`, `camel_case_extensions`, `non_constant_identifier_names`, `file_names` (core); `constant_identifier_names`, `library_prefixes`, `package_names` (rec) |
| Sem prefixo de letra (`kPadrao`, `mValor`) e sem `_` em nome que não é privado | [style #prefix-letters](https://dart.dev/effective-dart/style#dont-use-prefix-letters), [#leading-underscore](https://dart.dev/effective-dart/style#dont-use-a-leading-underscore-for-identifiers-that-arent-private) | E1 | `no_leading_underscores_for_local_identifiers` (rec); prefixo de letra é revisor |
| Sem abreviação; termos usados de forma consistente | [design #avoid-abbreviations](https://dart.dev/effective-dart/design#avoid-abbreviations), [#terms-consistently](https://dart.dev/effective-dart/design#do-use-terms-consistently) | E1 | `termo-evitado` (precisa reconhecer as declarações de Dart: `final`, `var`, `late`, tipo antes do nome; a conferir na spec) |
| Nome de membro: substantivo para valor, verbo imperativo para efeito; booleano positivo e não imperativo; sem `get` no começo; `toX()` copia, `asX()` é vista | [design #side-effect](https://dart.dev/effective-dart/design#prefer-an-imperative-verb-phrase-for-a-function-or-method-whose-main-purpose-is-a-side-effect), [#get](https://dart.dev/effective-dart/design#avoid-starting-a-function-or-method-name-with-get), [#to-as](https://dart.dev/effective-dart/design#prefer-naming-a-method-to___-if-it-copies-the-objects-state-to-a-new-object) | E1, E5 | `use_to_and_as_if_applicable` (fora); o resto é revisor |
| Parâmetro de callback sem uso é `_` (wildcard, Dart 3.7) | [style #wildcards](https://dart.dev/effective-dart/style#prefer-using-wildcards-for-unused-callback-parameters) | E1 | `no_wildcard_variable_uses` (core), `unnecessary_underscores` (rec) |
| Função curta, pouco aninhamento, saída cedo | — | E2 | **Sem regra no linter** (achado 4): revisor ou #20 |
| Chaves em todo `if`/`for`/`while` | [style #curly-braces](https://dart.dev/effective-dart/style#do-use-curly-braces-for-all-flow-control-statements) | E2 | `curly_braces_in_flow_control_structures` (core) |
| Sem `bool` posicional; parâmetro opcional nomeado quando há mais de um | [design #positional-boolean](https://dart.dev/effective-dart/design#avoid-positional-boolean-parameters), [#optional-positional](https://dart.dev/effective-dart/design#avoid-optional-positional-parameters-if-the-user-may-want-to-omit-earlier-parameters) | E3, E2 | `avoid_positional_boolean_parameters` (fora; testado) |
| Tipo próprio para conceito do domínio, de custo zero: *extension type* sobre o primitivo | [Extension types](https://dart.dev/language/extension-types) | E3 | revisor |
| Nada de `dynamic` no domínio; inferência que falha não vira `dynamic` calado | [design #dynamic](https://dart.dev/effective-dart/design#avoid-using-dynamic-unless-you-want-to-disable-static-checking); modos estritos na [página de análise](https://dart.dev/tools/analysis#enabling-additional-type-checks) | E3 | `analyzer: language: strict-casts / strict-inference / strict-raw-types: true` (testado: `final int quantidade = mapa['q']` vindo de `jsonDecode` vira erro `invalid_assignment`; `List` sem tipo vira `strict_raw_type`); `avoid_dynamic_calls` (fora; testado) |
| Assinatura pública com tipo de retorno e de parâmetro | [design #return-types](https://dart.dev/effective-dart/design#do-annotate-return-types-on-function-declarations) | E3, P1 | `strict_top_level_inference` (core, Dart 3.7); `type_annotate_public_apis` (fora) |
| Erro do programador é `Error` e não se captura; falha esperada é `Exception` | [usage #error](https://dart.dev/effective-dart/usage#do-throw-objects-that-implement-error-only-for-programmatic-errors), [#catch-error](https://dart.dev/effective-dart/usage#dont-explicitly-catch-error-or-types-that-implement-it); [Error handling](https://dart.dev/language/error-handling) | E4 | `avoid_catching_errors`, `only_throw_errors` (fora; `throw 'texto'` pego no teste) |
| `catch` sem `on` só para relançar ou na borda; `catch` vazio não | [usage #on-clauses](https://dart.dev/effective-dart/usage#avoid-catches-without-on-clauses), [#discard](https://dart.dev/effective-dart/usage#dont-discard-errors-from-catches-without-on-clauses) | E4 | `empty_catches` (core); `avoid_catches_without_on_clauses` (fora). `catch (e) { print(e); }` só é pego por `avoid_print` (testado): o "imprime e segue" é revisor ou ast-grep (#20) |
| Relançar com `rethrow`, que guarda o *stack trace* | [usage #rethrow](https://dart.dev/effective-dart/usage#do-use-rethrow-to-rethrow-a-caught-exception) | E4 | `use_rethrow_when_possible` (rec) |
| Erro esperado na interface: `Result` selado (`Ok`/`Error`) devolvido por repositório e serviço, casado com `switch` | [Flutter, Result pattern](https://docs.flutter.dev/app-architecture/design-patterns/result): "Result classes force the calling method to check for errors" | E4 | revisor (decisão do grilling: `Result` próprio ou exceção tipada) |
| `Future` nunca solto: `await` ou `unawaited(...)` explícito; função síncrona não dispara `Future` | regra [`unawaited_futures`](https://dart.dev/lints/unawaited_futures), [`discarded_futures`](https://dart.dev/lints/discarded_futures); [usage #async-await](https://dart.dev/effective-dart/usage#prefer-asyncawait-over-using-raw-futures) | E4 | `unawaited_futures`, `discarded_futures` (fora; os dois testados). Equivale ao `unused_must_use` do Rust e à promessa solta do TS |
| `ignore` sempre com o porquê; `ignore` que não cala nada sai | regra [`document_ignores`](https://dart.dev/lints/document_ignores), [`unnecessary_ignore`](https://dart.dev/lints/unnecessary_ignore) | E4 (verificador) | `document_ignores` (Dart 3.5), `unnecessary_ignore` (Dart 3.8) (fora; os dois testados) |
| `final` por padrão em campo, local e laço; campo só leitura é `final`, não getter | [design #final](https://dart.dev/effective-dart/design#prefer-making-fields-and-top-level-variables-final), [usage #final-field](https://dart.dev/effective-dart/usage#prefer-using-a-final-field-to-make-a-read-only-property) | E5 | `prefer_final_fields` (rec); `prefer_final_locals`, `prefer_final_in_for_each` (fora; `prefer_final_locals` testado) |
| Modelo de dados imutável; fluxo de dados num sentido só | [arq-rec, "Use immutable data models"](https://docs.flutter.dev/app-architecture/recommendations) (Strongly recommend); [`@immutable`](https://api.flutter.dev/flutter/meta/immutable-constant.html) | E5 | revisor; `avoid_equals_and_hash_code_on_mutable_classes` (fora) cobre a igualdade (DT6) |
| Getter para consulta, setter para mudança, método com verbo para comando | [design #getters](https://dart.dev/effective-dart/design#do-use-getters-for-operations-that-conceptually-access-properties), [#setters](https://dart.dev/effective-dart/design#do-use-setters-for-operations-that-conceptually-change-properties) | E5 | `use_setters_to_change_properties`, `avoid_setters_without_getters` (fora) |
| Comentário de doc com `///`, primeira frase sozinha; `TODO` com dono e issue | [documentation](https://dart.dev/effective-dart/documentation#do-use-doc-comments-to-document-members-and-types); regra [`flutter_style_todos`](https://dart.dev/lints/flutter_style_todos) (`// TODO(username): message, https://URL-to-issue`) | E6 | `slash_for_doc_comments` (rec); `flutter_style_todos` (fora; testado); `todo-sem-issue` lê `.dart` igual |
| Doc de API pública | [documentation #public-apis](https://dart.dev/effective-dart/documentation#prefer-writing-doc-comments-for-public-apis) | P1, E6 | `public_member_api_docs` (fora) — só em pacote publicado; em app choca com E6, como o `missing_docs` do Rust |
| Duplicação | — | E7 | #20 (jscpd lê Dart?) |
| Número com significado tem nome | — | E8 | **Sem regra no linter** (achado 4): revisor. `use_named_constants` (fora) só cobre constante que a própria classe já tem (`Duration.zero`) |
| Formatação única | [style #dart-format](https://dart.dev/effective-dart/style#do-format-your-code-using-dart-format); [dart format](https://dart.dev/tools/dart-format) | P8 | `comando`: `dart format --output=none --set-exit-if-changed .` (testado: código 1 com arquivo fora do formato). `page_width` e `trailing_commas: preserve` (Dart 3.8) vão na seção `formatter:` do `analysis_options.yaml` |

### Padrões (P1–P9)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Privado por padrão (`_`); tipo privado não vaza na API pública | [design #private](https://dart.dev/effective-dart/design#prefer-making-declarations-private) | P1, I | `library_private_types_in_public_api` (rec); o resto é revisor |
| Implementação em `lib/src/`; ninguém importa o `src` de outro pacote | [usage #src](https://dart.dev/effective-dart/usage#dont-import-libraries-that-are-inside-the-src-directory-of-another-package) | P1 | `implementation_imports` (rec) |
| Membro público só para teste é marcado `@visibleForTesting` | [`visibleForTesting`](https://api.flutter.dev/flutter/meta/visibleForTesting-constant.html) | P1, P5 | o analisador avisa o uso fora de teste ([`invalid_use_of_visible_for_testing_member`](https://dart.dev/tools/diagnostics/invalid_use_of_visible_for_testing_member)) |
| Sem classe só de membros estáticos; sem classe abstrata de um membro quando uma função serve | [design #static-only](https://dart.dev/effective-dart/design#avoid-defining-a-class-that-contains-only-static-members), [#one-member](https://dart.dev/effective-dart/design#avoid-defining-a-one-member-abstract-class-when-a-simple-function-will-do) | P1, P2 | `avoid_classes_with_only_static_members` (fora); `one_member_abstracts` está *deprecated* no `rules.json` |
| Repositório abstrato com implementação real e *fake* | [arq-rec, "Use abstract repository classes"](https://docs.flutter.dev/app-architecture/recommendations) (Strongly recommend) | P2, O, D | revisor. O guia justifica com ambientes ("development" e "staging") e com o *fake* do teste: duas implementações reais, o que fecha com o P2 |
| Camadas de dados e de UI separadas; domínio (use cases) só quando a lógica transborda do ViewModel | [arq-rec, "Use clearly defined data and UI layers"](https://docs.flutter.dev/app-architecture/recommendations); "Use a domain layer" é *Conditional* | P3, P4, S | revisor; lint por pasta (achado 3) para o que o domínio não pode usar |
| Dependência entra por injeção, sem objeto global | [arq-rec, "Use dependency injection"](https://docs.flutter.dev/app-architecture/recommendations): "prevents your app from having globally accessible objects"; [estudo de caso](https://docs.flutter.dev/app-architecture/case-study/dependency-injection) | P3, D | revisor; import de infraestrutura no domínio é #20 (sem regra no linter, achado 4) |
| Ambiente e relógio só na montagem (`Platform.environment`, `DateTime.now()`, `Random()`) | — | P3 | Sem regra no linter (testado: nada pegou `Platform.environment` nem `DateTime.now()` no domínio); `do_not_use_environment` (fora) só cobre `fromEnvironment`. ast-grep ou plugin: #20 |
| Saída no terminal só na borda | regra [`avoid_print`](https://dart.dev/lints/avoid_print) | P3, E6 | `avoid_print` (flutter; testado) |
| Widget sem lógica: só `if` de exibir, animação, layout e rota; a lógica mora no ViewModel | [arq-rec, "Do not put logic in widgets"](https://docs.flutter.dev/app-architecture/recommendations) (Strongly recommend); [UI layer](https://docs.flutter.dev/app-architecture/case-study/ui-layer) | S, P4 | revisor; `no_logic_in_create_state` (flutter) cobre só o `createState` |
| O que muda junto fica junto: uma pasta por funcionalidade dentro de `ui/` e `data/`; nome pela peça (`HomeViewModel`, `UserRepository`) | [arq-rec, "Use standardized naming conventions"](https://docs.flutter.dev/app-architecture/recommendations) | P4, P8 | revisor |
| Import relativo dentro de `lib/`, nunca atravessando a fronteira de `lib/` | [usage #relative](https://dart.dev/effective-dart/usage#prefer-relative-import-paths), [#lib](https://dart.dev/effective-dart/usage#dont-allow-an-import-path-to-reach-into-or-out-of-lib) | P8 | `avoid_relative_lib_imports` (core); `prefer_relative_imports` **ou** `always_use_package_imports` (fora, são incompatíveis: escolher um) |
| Teste de cada serviço, repositório e ViewModel; teste de widget para a tela | [arq-rec, "Test architectural components separately, and together"](https://docs.flutter.dev/app-architecture/recommendations); [Testing overview](https://docs.flutter.dev/testing/overview) | P5 | `teste-cobre-risco` (precisa ler `test('RK2 ...')` e `group`, ver #18) |
| *Fake* em vez de mock: o teste olha entrada e saída | [arq-rec, "Make fakes for testing"](https://docs.flutter.dev/app-architecture/recommendations); [estudo de caso, testing](https://docs.flutter.dev/app-architecture/case-study/testing) | P5 | revisor; proibir `mockito`/`mocktail` em `test/` do domínio é #20. O [cookbook](https://docs.flutter.dev/cookbook/testing/unit/mocking) ainda ensina Mockito; o guia de arquitetura prefere *fake* |
| Teste desligado tem motivo e é exceção | [package:test, *Skipping Tests*](https://pub.dev/packages/test#skipping-tests) | P5 | `skip:` em `test`/`group`/`testWidgets` é o `teste-desligado` (ast-grep, #20) |
| Port com teste de contrato | — | L | `teste-de-contrato-por-port` com `ports: ["lib/data/repositories/*.dart"]` (o motor só confere arquivo) |
| Vermelho, verde, refatorar | — | P6 | revisor |
| Nada para requisito que não existe; dependência e código sem uso saem | [`depend_on_referenced_packages`](https://dart.dev/lints/depend_on_referenced_packages); diagnósticos [`unused_import`](https://dart.dev/tools/diagnostics/unused_import), [`unused_element`](https://dart.dev/tools/diagnostics/unused_element), [`dead_code`](https://dart.dev/tools/diagnostics/dead_code) | P7 | já são avisos do analisador (`warning` falha o `dart analyze` por padrão); dependência declarada sem uso é #20 |
| Consistência: um conjunto de lint, um formatador | [`lints` README](https://pub.dev/packages/lints): "The recommended lints include all the core lints"; `flutter_lints` estende o recommended | P8 | `include: package:flutter_lints/flutter.yaml` (ou `package:lints/recommended.yaml`) |
| Commit conta uma mudança | — | P9 | `mensagem-de-commit` |

### SOLID

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Uma classe por papel: View, ViewModel, Repository, Service | [arq-rec](https://docs.flutter.dev/app-architecture/recommendations); [guia](https://docs.flutter.dev/app-architecture/guide) | S | revisor |
| Variante nova é subtipo novo; hierarquia fechada é `sealed` e o `switch` é exaustivo | [Class modifiers, `sealed`](https://dart.dev/language/class-modifiers): "a switch over those subtypes that is statically ensured to be exhaustive"; [Branches](https://dart.dev/language/branches) | O | o compilador recusa `switch` incompleto sobre `sealed`; `exhaustive_cases` (rec) para enum-like; `switch_on_type` entra no core 7.0 |
| A classe diz se pode ser estendida ou implementada (`final`, `base`, `interface`) | [design #extended](https://dart.dev/effective-dart/design#do-use-class-modifiers-to-control-if-your-class-can-be-extended), [#interface](https://dart.dev/effective-dart/design#do-use-class-modifiers-to-control-if-your-class-can-be-an-interface); [Class modifiers for API maintainers](https://dart.dev/language/class-modifiers-for-apis) | O, L | compilador; escolha do modificador é revisor |
| Adapter cumpre a interface inteira (sem `throw UnimplementedError()`) | — | L | `teste-de-contrato-por-port`; ast-grep (#20) |
| Quem chama recebe o tipo mínimo; função como parâmetro em vez de interface de um método | [design #one-member](https://dart.dev/effective-dart/design#avoid-defining-a-one-member-abstract-class-when-a-simple-function-will-do) | I | revisor |
| O domínio/ViewModel define o repositório abstrato; a implementação fica em `data/` | [arq-rec, "Use abstract repository classes"](https://docs.flutter.dev/app-architecture/recommendations) | D | revisor; lint por pasta (achado 3) |

## 2. Práticas que não cabem nos 22 princípios (candidatas a DT\<n\>)

| ID | Prática | Fonte | Verificação |
|---|---|---|---|
| DT1 | **`BuildContext` não atravessa `await`** sem conferir `mounted` (`context.mounted` para o contexto recebido, `mounted` do `State` para o dele) | regra [`use_build_context_synchronously`](https://dart.dev/lints/use_build_context_synchronously): "Storing BuildContext for later usage can easily lead to difficult-to-diagnose crashes"; [`BuildContext.mounted`](https://api.flutter.dev/flutter/widgets/BuildContext/mounted.html) | já vem no `flutter_lints` (testado); sugestão: subir para `error` no `analyzer: errors:` |
| DT2 | **Quem cria, descarta**: todo controller, `AnimationController`, `FocusNode`, `ChangeNotifier` criado no `State` tem `dispose()`; `StreamSubscription` é cancelada; `Sink`/`StreamController` é fechado | [`State.dispose`](https://api.flutter.dev/flutter/widgets/State/dispose.html) ("release any resources retained by this object"); [`TextEditingController`](https://api.flutter.dev/flutter/widgets/TextEditingController-class.html) ("Remember to dispose of the TextEditingController"); [`ChangeNotifier.dispose`](https://api.flutter.dev/flutter/foundation/ChangeNotifier/dispose.html) | `cancel_subscriptions`, `close_sinks` (fora; testados); para widget, `leak_tracker_flutter_testing` com `LeakTesting.enable()` no `flutter_test_config.dart` (achado 5, testado) |
| DT3 | **Null safety sem atalho**: `!` só onde o invariante está provado ao lado; preferir promoção, `if (x case final valor?)` e `??`; `late` só com inicialização garantida, nunca para "ver se já iniciou" | [Understanding null safety, *Not-null assertion operator*](https://dart.dev/null-safety/understanding-null-safety) (o `!` é um *cast* que falha em tempo de execução); [Variables, *Late variables*](https://dart.dev/language/variables) ("If you fail to initialize a late variable, a runtime error occurs"); [usage #late](https://dart.dev/effective-dart/usage#avoid-late-variables-if-you-need-to-check-whether-they-are-initialized), [design #late-final](https://dart.dev/effective-dart/design#avoid-public-late-final-fields-without-initializers) | `unnecessary_late` (rec), `null_check_on_nullable_type_parameter` (core), `cast_nullable_to_non_nullable`, `unnecessary_null_checks` (fora). **Nenhuma regra proíbe o `!`** (testado: `apelido!.length` passou): ast-grep ou revisor (#20). É o par do RS2 (pânico por atalho) |
| DT4 | **`const` onde der**: widget e literal constante são `const`; widget sem estado tem construtor `const` | [`StatelessWidget`](https://api.flutter.dev/flutter/widgets/StatelessWidget-class.html): "Use const widgets where possible, and provide a const constructor"; [design #const](https://dart.dev/effective-dart/design#consider-making-your-constructor-const-if-the-class-supports-it) | `prefer_const_constructors_in_immutables`, `use_key_in_widget_constructors` (flutter); `prefer_const_constructors`, `prefer_const_declarations`, `prefer_const_literals_to_create_immutables` (fora desde o `flutter_lints` 5, achado 6) |
| DT5 | **Build reprodutível de app**: `pubspec.lock` no git (só em app; pacote publicado não versiona) e CI com `--enforce-lockfile` | [What not to commit, *pubspec.lock*](https://dart.dev/tools/pub/private-files): "For application packages, we recommend that you commit the pubspec.lock file"; [`dart pub get --enforce-lockfile`](https://dart.dev/tools/pub/cmd/pub-get) ("Useful for CI or deploying to production") | `comando`: `flutter pub get --enforce-lockfile` (ou `dart pub get --enforce-lockfile`). Já é regra da casa para Rust (RS4, ADR 0011) |
| DT6 | **Igualdade só em classe imutável**: quem sobrescreve `==` sobrescreve `hashCode`, testa o tipo do argumento e marca a classe `@immutable`; widget nunca sobrescreve `==` | [design #hashcode](https://dart.dev/effective-dart/design#do-override-hashcode-if-you-override), [#mutable](https://dart.dev/effective-dart/design#avoid-defining-custom-equality-for-mutable-classes); [Performance best practices](https://docs.flutter.dev/perf/best-practices) ("Avoid overriding operator == on Widget objects") | `hash_and_equals` (core); `avoid_equals_and_hash_code_on_mutable_classes`, `test_types_in_equals` (fora). É o par do `equals`/`hashCode` do Java (JV) |

Ficaram de fora como DT, porque cabem num princípio: `Future` solto (E4, como em Rust e TS),
`dynamic` e modos estritos (E3, como o `any` do TS), widget sem lógica (S/P4), `sealed` (O).

## 3. Ferramentas do SDK (o resto é da #20)

Todas vêm com o SDK ou são pacotes oficiais, BSD-3-Clause, sem plano pago.

| Ferramenta | Versão (2026-10-24) | Comando para o CI |
|---|---|---|
| Analisador + linter | do SDK (Dart 3.13.7 / Flutter 3.47.8) | `flutter analyze` (falha com info) ou `dart analyze --fatal-infos` (achado 1) |
| `lints` / `flutter_lints` | 6.1.0 / 6.0.0 | `include:` no `analysis_options.yaml` |
| `dart format` | do SDK | `dart format --output=none --set-exit-if-changed .` |
| `dart fix` | do SDK | `dart fix --dry-run` mostra o que o lint corrige sozinho ([dart fix](https://dart.dev/tools/dart-fix)) |
| `package:test` / `flutter_test` | 1.33.0 / do SDK | `dart test` / `flutter test` |
| `leak_tracker_flutter_testing` | 3.0.10 (versão vem do SDK: o guia manda `any`) | roda dentro do `flutter test` (DT2) |
| Analyzer plugins | Dart ≥ 3.10 | `plugins:` no `analysis_options.yaml`; caminho para regra própria (achado 4) |

### Bloco sugerido para o `analysis_options.yaml` de um app Flutter da casa

Testado (Flutter 3.44.9, `flutter_lints` 6.0.0): todas as chaves aceitas e cada regra disparou no
exemplo que devia pegar. Ponto de partida do grilling, não regra decidida.

```yaml
include: package:flutter_lints/flutter.yaml

analyzer:
  language:
    strict-casts: true
    strict-inference: true
    strict-raw-types: true
  errors:
    use_build_context_synchronously: error   # DT1

linter:
  rules:
    # E4
    - unawaited_futures
    - discarded_futures
    - only_throw_errors
    - avoid_catching_errors
    - document_ignores
    - unnecessary_ignore
    # E3
    - avoid_dynamic_calls
    - avoid_positional_boolean_parameters
    # E5
    - prefer_final_locals
    - prefer_final_in_for_each
    # E6
    - flutter_style_todos
    # DT2, DT4, DT6
    - cancel_subscriptions
    - close_sinks
    - prefer_const_constructors
    - prefer_const_declarations
    - avoid_equals_and_hash_code_on_mutable_classes
```

```yaml
# lib/dominio/analysis_options.yaml (ou lib/data/), achado 3
include: ../../analysis_options.yaml

linter:
  rules:
    - avoid_catches_without_on_clauses
```

## 4. O que ficou sem confirmação em fonte primária

- **[confirmar]** Os testes rodaram no Flutter 3.44.9 / Dart 3.12.2, não no estável atual (3.47.8 /
  3.13.7). Nenhuma das regras usadas é mais nova que o Dart 3.8, mas o comportamento de
  `flutter analyze` e do `leak_tracker` não foi refeito no 3.47.
- **[confirmar]** O custo de adotar o bloco num app real (quantos achados no `passenger-app/` do
  excurfly) não foi medido: o projeto não está neste repositório. É o critério de aceite do mapa.
- **[confirmar]** O link da discussão que tirou `prefer_const_constructors` do conjunto
  (`dart-lang/lints#205`, citado no CHANGELOG do `flutter_lints`) devolve 404: o repositório foi
  movido para `dart-lang/core`.
- Ausência de regra de tamanho, complexidade, número mágico e import proibido: confirmada nas 264
  regras do `rules.json` do `main` do SDK em 2026-10-24.
