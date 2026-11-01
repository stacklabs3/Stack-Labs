# Dart e Flutter: princípios de bom desenvolvimento e como o validador os confere

Pesquisa de 2026-10-25. Pergunta: quais práticas de Dart e Flutter, em fonte primária, concretizam os
22 princípios da casa (E1–E8, P1–P9, S O L I D), e como cada uma se confere por máquina com
ferramenta que não cobra.

Desfecho: decidido no grilling de 2026-10-25 (ADR 0019). O guia é
`docs/agents/principios-de-codigo/dart.md`. Das candidatas abaixo, a DT5 (keys) ficou no preset e não
virou prática; as adotadas foram renumeradas: DT1 `BuildContext`, DT2 `const`, DT3 liberar recurso,
DT4 lockfile e OSV, DT5 widget como classe, DT6 acessibilidade. As regex `TESTE_DART` e
`DECLARACAO_DART` e as regras ast-grep de Dart foram testadas no motor; a de nome de uma letra deu 57
achados no Excurfly fora do código gerado.

## Como foi feito, e o limite das fontes

- Fontes lidas na página publicada (`dart.dev`, `docs.flutter.dev`, `api.flutter.dev`, `pub.dev`,
  `ast-grep.github.io`, `google.github.io/osv-scanner`, `dcm.dev`) e no GitHub
  (`VeryGoodOpenSource/very_good_analysis`, `invertase/dart_custom_lint`, `ast-grep/ast-grep`,
  `Workiva/dependency_validator`, `bancolombia/dart-code-linter`, `domohuhn/mutation-test`), em
  2026-10-25.
- Versão e licença dos pacotes: API do pub.dev (`https://pub.dev/api/packages/<pacote>` e
  `/score`, tags `license:*`), em 2026-10-25. Versão do SDK:
  `storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION` e o
  `releases_macos.json` do Flutter.
- Toolchain local: **Dart 3.12.2, Flutter 3.44.9** (a estável publicada é **Dart 3.13.7**, de
  2026-10-22, e **Flutter 3.47.8**). Todo comportamento marcado "testado" foi rodado aqui, em pastas
  de teste no scratchpad, fora de qualquer repositório.
- Conteúdo dos presets: lido do pub-cache (`lints-6.1.0/lib/core.yaml`, `recommended.yaml`,
  `flutter_lints-6.0.0/lib/flutter.yaml`) e do GitHub (`very_good_analysis/lib/analysis_options.11.0.0.yaml`).

Atalhos de link usados na tabela:
[lint] = `https://dart.dev/tools/linter-rules/<regra>` ·
[analysis] = <https://dart.dev/tools/analysis> ·
[ED-style] = <https://dart.dev/effective-dart/style> · [ED-doc] = <https://dart.dev/effective-dart/documentation> ·
[ED-usage] = <https://dart.dev/effective-dart/usage> · [ED-design] = <https://dart.dev/effective-dart/design> ·
[arch] = <https://docs.flutter.dev/app-architecture/recommendations>.

## 0. Achados que mudam o método (ler primeiro)

1. **Lint é `info` por padrão, e `info` não reprova** (testado). `dart analyze` devolve **0** com só
   infos, **2** com warning, **3** com erro; `--fatal-infos` faz info devolver **1**;
   `--no-fatal-warnings` faz warning devolver 0. A doc: "the analyzer reports failure when it finds
   any errors or warnings, but not when it finds info-level issues"
   ([dart analyze](https://dart.dev/tools/dart-analyze)). Logo a receita segura é
   `dart analyze --fatal-infos` (ou `flutter analyze --fatal-infos`), o equivalente ao
   `-D warnings` do Rust. Alternativa por regra: `analyzer: errors: <regra>: error` sobe só aquela
   ([analysis, "Changing the severity of rules"][analysis]; testado: `avoid_print: error` → exit 3).
2. **`lint-cobre` não lê `analysis_options.yaml`** (lido no fonte). `cobertura_de_lint` em
   `.claude/watchdogs/src/validador.rs` só tem leitura própria para `toml`, `xml` e `json`; YAML cai
   em `cobertura_generica` (substring), o mesmo falso aprovado do achado 1 da pesquisa de Rust:
   `prefer_final_locals: false` e `# - avoid_print` contariam como cobertos. **Proposta para o motor:**
   `cobertura_yaml` que lê o YAML e exige a regra em `linter: rules:` (lista, ou mapa com `true`) e
   não `ignore` em `analyzer: errors:`; e, se o CI não roda `--fatal-infos`, exige
   `analyzer: errors: <regra>: error|warning`. Regra que só vem de `include:` (preset) não é vista
   sem resolver o pacote; aceitar uma lista "o preset X liga Y" embutida ou exigir a regra explícita.
   Até lá: `comando` com `dart analyze --fatal-infos`.
3. **`termo-evitado` não vê declarações típicas de Dart** (lido no fonte). A regex `DECLARACAO`
   pega `class`, `const`, `var`, `enum`, mas não `final usuario = ...`, `late final`, declaração
   tipada (`String usuario`, `Usuario usuario`), `mixin`, `extension`, `extension type`, `typedef`.
   Correção sugerida: `DECLARACAO_DART` com
   `\b(?:class|mixin|enum|typedef|extension(?:\s+type)?|final|const|var|late(?:\s+final)?)\s+(?:[A-Za-z_][\w<>?,\s]*\s+)?([A-Za-zÀ-ÿ_]\w*)`
   **[confirmar]** por teste no motor; declaração só tipada (`Usuario usuario;`) fica para o
   `revisor` ou para um lint de plugin.
4. **`riscos.rs` não reconhece teste Dart** (lido no fonte). `declaracao_padrao` não mapeia `.dart`.
   `TESTE_JS` (`^\s*(?:it|test)\b[\w.]*\s*\(`) pegaria `test(` mas não `testWidgets(`. Regex
   sugerida: `TESTE_DART = ^\s*(?:test|testWidgets|testGoldens|goldenTest|patrolTest)\s*\(`,
   ligada a `"dart"`. No Excurfly ela casa **1.220** declarações (686 `test(` + 534 `testWidgets(`),
   139 delas com o nome na linha de baixo (`test(` sozinho), o que o leitor de "riscos até o nome"
   precisa aceitar. `group(` não é teste.
5. **Não existe lint de número mágico nem de nome curto no linter do Dart** (testado:
   `no_magic_numbers` → `undefined_lint` no Dart 3.12.2; a lista em <https://dart.dev/tools/linter-rules>
   não tem `id-length`). E1 (nome de uma letra) e E8 ficam para `ast-grep` (abaixo) ou `revisor`.
6. **ast-grep 0.51 tem Dart embutido** (testado; `Dart` está em `impl_lang!` de
   [crates/language/src/lib.rs](https://github.com/ast-grep/ast-grep/blob/main/crates/language/src/lib.rs),
   extensão `dart`). A regra `catch-vazio-dart` da seção 4 pegou 4 de 4 e passou 0 de 3. **jscpd
   5.4.2 lê Dart** (testado: `jscpd --list` inclui `dart`).
7. **`custom_lint` (invertase) não é mais desenvolvido**: o README diz "This package is no longer
   under active development" e aponta o oficial `analysis_server_plugin`
   ([README](https://github.com/invertase/dart_custom_lint)). O sistema nativo existe desde o Dart 3.10
   ("Support for analyzer plugins was added in Dart 3.10", [Analyzer plugins](https://dart.dev/tools/analyzer-plugins)),
   ligado na chave de primeiro nível `plugins:` do `analysis_options.yaml`, e os diagnósticos saem
   no próprio `dart analyze`. Lint próprio da casa em Dart, se um dia houver, nasce nele.

## 1. Tabela: prática → fonte → princípio da casa → verificação

"Analyzer (CI)" abaixo significa sempre: a regra declarada no `analysis_options.yaml`, **mais** o
`comando` `dart analyze --fatal-infos` (achado 1). Onde a regra já vem num preset, ele é dito entre
parênteses: **core**, **rec** (recommended, inclui core), **flutter** (flutter_lints, inclui rec),
**vga** (very_good_analysis 11.0.0).

### Escrita (E1–E8)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Caixa: `UpperCamelCase` para tipos e extensões, `lowerCamelCase` para membros, variáveis e constantes, `lowercase_with_underscores` para arquivos, pacotes e prefixos de import | [ED-style] | E1 (caixa) | Analyzer (CI): `camel_case_types`, `non_constant_identifier_names`, `file_names`, `library_prefixes` (core); `constant_identifier_names` (rec) |
| Nome na língua do domínio, sem termo *Evitar* | Glossário do projeto | E1 | `termo-evitado` (com a correção do achado 3) |
| Nome sem abreviação, "PREFER using terms consistently", "AVOID abbreviations" | [ED-design](https://dart.dev/effective-dart/design#avoid-abbreviations) | E1 | **Não há lint** (achado 5). `ast-grep`: `nome-de-uma-letra-dart` (seção 4, a calibrar) ou `revisor` |
| Função curta, pouco aninhamento, poucos parâmetros | Nenhum lint do SDK | E2 | `dart_code_linter` (MIT, livre) com `metrics: cyclomatic-complexity`, `number-of-parameters`, `maximum-nesting-level` ([pub.dev](https://pub.dev/packages/dart_code_linter)); ou `revisor`. DCM faz isso, mas é pago acima de 50k LOC (seção 3) |
| Tipos estritos: sem cast implícito de `dynamic`, sem inferência para `dynamic`, sem tipo genérico cru | [analysis, "Enabling stricter type checks"][analysis] | E3 | `lint-cobre` em `analyzer: language: strict-casts`, `strict-inference`, `strict-raw-types: true` (vga já liga os três) |
| Nada de `dynamic` solto; chamada dinâmica é proibida | [ED-design, "AVOID using dynamic unless you want to disable static checking"](https://dart.dev/effective-dart/design) | E3 | Analyzer (CI): `avoid_dynamic_calls` (vga) |
| Conceito do domínio em tipo próprio (`extension type`, classe com construtor que valida) | [Extension types](https://dart.dev/language/extension-types) | E3 | `revisor` |
| Bool posicional não diz o que é | [ED-design, "AVOID positional boolean parameters"](https://dart.dev/effective-dart/design) | E3 | Analyzer (CI): `avoid_positional_boolean_parameters` (vga) |
| Null safety sem atalho: `!` só onde o tipo não sabe o que o programador sabe | [Understanding null safety](https://dart.dev/null-safety/understanding-null-safety) | E3, E4 | Analyzer (CI): `unnecessary_null_checks`, `cast_nullable_to_non_nullable`, `null_check_on_nullable_type_parameter` (vga) |
| Erro esperado se captura pelo tipo (`on X catch`), nunca `catch` genérico | [ED-usage, "AVOID catches without on clauses"](https://dart.dev/effective-dart/usage#avoid-catches-without-on-clauses); "DON'T explicitly catch Error or types that implement it" | E4 | Analyzer (CI): `avoid_catches_without_on_clauses` (vga), `avoid_catching_errors` (vga) |
| `catch` vazio é proibido | [lint] `empty_catches` | E4 | Analyzer (CI): `empty_catches` (core). Só pega o bloco vazio: o `catch` que só imprime vai para `ast-grep` `catch-vazio-dart` (seção 4) |
| Lança só `Exception`/`Error`, não string | [ED-usage, "DO throw objects that implement Error only for programmatic errors"](https://dart.dev/effective-dart/usage) | E4 | Analyzer (CI): `only_throw_errors` (vga) |
| `rethrow` para repassar o erro capturado | [ED-usage, "DO use rethrow to rethrow a caught exception"](https://dart.dev/effective-dart/usage) | E4 | Analyzer (CI): `use_rethrow_when_possible` (rec) |
| `Future` nunca descartado | [lint] `unawaited_futures`, `discarded_futures` | E4 | Analyzer (CI): `unawaited_futures`, `discarded_futures` (vga); `unawaited(...)` de `dart:async` marca o descarte intencional |
| Erro esperado na interface: `sealed class` de resultado + `switch` exaustivo | [Class modifiers, `sealed`](https://dart.dev/language/class-modifiers#sealed); [Patterns, exhaustiveness](https://dart.dev/language/branches#exhaustiveness-checking) | E4, O | Compilador (exaustividade é erro); Analyzer (CI): `exhaustive_cases` (rec), `no_default_cases` (vga) para o `default` não esconder caso novo |
| Imutável por padrão: `final` local e campo, construtor `const` | [ED-usage, "PREFER final"](https://dart.dev/effective-dart/usage) | E5 | Analyzer (CI): `prefer_final_locals`, `prefer_final_in_for_each` (vga), `prefer_final_fields` (rec) |
| Comando ou consulta: getter não tem efeito colateral | [ED-design, "DO use getters for operations that conceptually access properties"](https://dart.dev/effective-dart/design) | E5 | `revisor` |
| Comentário de doc com `///`, começando por uma frase de resumo | [ED-doc] | E6, P1 | Analyzer (CI): `slash_for_doc_comments` (rec); `public_member_api_docs` (vga; ver custo na seção 5) |
| `// ignore:` sempre com o porquê | [lint] `document_ignores` ("DO document all ignored diagnostic reports", Dart 3.5) | E6 | Analyzer (CI): `document_ignores` (vga). Testado: pega `// ignore: avoid_print` sem comentário em cima |
| `TODO` com issue | — | E6 | `todo-sem-issue` (lê `.dart` igual); o lint `flutter_style_todos` exige `TODO(usuario): ...`, outra forma |
| `print` só na borda | [lint] `avoid_print` | E6, P3 | Analyzer (CI): `avoid_print` (flutter, vga) |
| Duplicação | — | E7 | `comando`: `npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` (achado 6) |
| Número com significado tem nome | **Não há lint** (achado 5) | E8 | `revisor` |
| Formatação única | [`dart format`](https://dart.dev/tools/dart-format) | P8 | `comando`: `dart format --output=none --set-exit-if-changed .` (testado: exit 1 com arquivo fora do formato, nada é escrito) |
| Correção automática do que o analyzer aponta | [`dart fix`](https://dart.dev/tools/dart-fix) | — | Uso local (`dart fix --apply`); no CI só `--dry-run`, que não muda o exit code de forma útil **[confirmar]** |

### Padrões (P1–P9)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Interface pequena: privado com `_`, `export` só o que é API, implementação em `lib/src/` | [Package layout](https://dart.dev/tools/pub/package-layout#implementation-files) | P1, I | `revisor`; Analyzer (CI): `implementation_imports` (rec) impede importar `src/` de outro pacote |
| Item público documentado | [ED-doc] | P1 | Analyzer (CI): `public_member_api_docs` (vga) — só para pacote publicado (1.042 achados no Excurfly, seção 5) |
| Classe abstrata/`interface class` (port) só onde há duas implementações reais | [Class modifiers](https://dart.dev/language/class-modifiers) | P2, O | `revisor` |
| Dependência entra; domínio sem infraestrutura (`http`, `shared_preferences`, plugins) | [arch]: "Use clearly defined data and UI layers" e repositories/services (Strongly recommend) | P3, D | `ast-grep`: `dominio-sem-infra-dart` (seção 4); mais forte: domínio num pacote próprio (workspace do pub) sem a dependência no `pubspec.yaml` dele |
| Import relativo só dentro de `lib/`, nunca `../lib` | [lint] `avoid_relative_lib_imports` | P4 | Analyzer (CI): `avoid_relative_lib_imports` (core) |
| Todo pacote importado está no `pubspec.yaml` | [lint] `depend_on_referenced_packages` | P7, P8 | Analyzer (CI): `depend_on_referenced_packages` (core) |
| Dependência sem uso sai | [dependency_validator](https://pub.dev/packages/dependency_validator) ("Checks for missing, under-promoted, over-promoted, and unused dependencies") | P7 | `comando`: `dart run dependency_validator` |
| Código morto sai | Analyzer `unused_element`, `unused_import`, `unused_local_variable` (warning por padrão) | P7 | Analyzer (CI): já reprova (warning = exit 2) |
| Teste pela interface: `package:test` para lógica, `flutter_test` (`testWidgets`) para widget, `integration_test` ponta a ponta | [Testing Flutter apps](https://docs.flutter.dev/testing/overview) | P5 | `revisor`; `riscos.rs` com `TESTE_DART` (achado 4) |
| Mock só na fronteira: `mocktail` (sem geração) ou `mockito` (`build_runner`) para repositório/serviço, nunca para classe do domínio | [Mock dependencies](https://docs.flutter.dev/cookbook/testing/unit/mocking); [arch] (fakes para testar a camada) | P5 | `ast-grep` `dominio-sem-infra-dart` apontado para `test/**` com `mocktail|mockito` e o import do domínio, a calibrar |
| Teste pulado tem motivo: `skip: 'motivo'` (aceita `bool` ou `String`) | [package:test, `skip`](https://pub.dev/packages/test#skipping-tests) | P5 | `ast-grep` (`skip: true` proibido) **[confirmar]** a forma no grilling |
| Port com teste de contrato | — | L | `teste-de-contrato-por-port` com `ports: ["lib/dominio/portas/*.dart"]`, `contrato: "test/contrato_{nome}_test.dart"` |
| Vermelho, verde, refatorar | — | P6 | `revisor` |
| Override sempre marcado | [lint] `annotate_overrides` | L, P8 | Analyzer (CI): `annotate_overrides` (rec) |
| Retorno sempre declarado | [lint] `always_declare_return_types` | P8 | Analyzer (CI): `always_declare_return_types` (vga) |
| Commit conta uma mudança | — | P9 | `mensagem-de-commit` |

### SOLID

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Um motivo para mudar: View só desenha, ViewModel guarda o estado da tela, Repository é a fonte da verdade | [arch]: "Use ViewModels and Views in the UI layer (MVVM)" (Strongly recommend), "Use the repository pattern" | S | `revisor` |
| Variante nova = nova subclasse de `sealed`; o compilador aponta todo `switch` que falta | [Class modifiers, sealed](https://dart.dev/language/class-modifiers#sealed) | O | Compilador + `no_default_cases` (vga) |
| Adapter cumpre a interface inteira (sem `throw UnimplementedError()`) | — | L | `teste-de-contrato-por-port`; `ast-grep` simples sobre `UnimplementedError` em `lib/` |
| Quem chama pede só o que usa (`implements` de interface pequena) | [ED-design] | I | `revisor` |
| O domínio define a interface; a infra implementa | [arch] | D | `ast-grep` `dominio-sem-infra-dart` |

## 2. Práticas que não cabem nos 22 princípios

Candidatas a princípio específico de Dart/Flutter (DT1…DT7):

| Prática | Fonte | Verificação |
|---|---|---|
| **DT1: `BuildContext` não atravessa `await`** sem conferir `mounted` (`context.mounted` ou `State.mounted`) | [lint] `use_build_context_synchronously` | Analyzer (CI): `use_build_context_synchronously` (flutter) |
| **DT2: pedaço de UI reutilizável é classe de widget, não função que devolve widget** | [StatelessWidget, "Performance considerations"](https://api.flutter.dev/flutter/widgets/StatelessWidget-class.html): "When trying to create a reusable piece of UI, prefer using a widget rather than a helper method" | `ast-grep` (método/função privada com retorno `Widget` fora de `build`) **[confirmar]** ruído; ou `revisor` |
| **DT3: construtor `const` onde der** (o Flutter pula o rebuild de subárvore `const`) | Mesma página do StatelessWidget; [lint] `prefer_const_constructors`, `prefer_const_constructors_in_immutables` | Analyzer (CI): `prefer_const_constructors` (vga), `prefer_const_constructors_in_immutables` (flutter) |
| **DT4: recurso com ciclo de vida é liberado**: `dispose()` de controller, `cancel()` de `StreamSubscription`, `close()` de `StreamController`/`Sink` | [lint] `cancel_subscriptions`, `close_sinks`; [State.dispose](https://api.flutter.dev/flutter/widgets/State/dispose.html) | Analyzer (CI): `cancel_subscriptions` (vga). `close_sinks` o vga **desliga** (`close_sinks: ignore`, falso positivo); controller sem `dispose` fica com o `revisor` |
| **DT5: widget com `key` no construtor** (`super.key`) e `Key` em lista que muda de ordem | [lint] `use_key_in_widget_constructors`; [Key](https://api.flutter.dev/flutter/foundation/Key-class.html) | Analyzer (CI): `use_key_in_widget_constructors` (flutter) |
| **DT6: acessibilidade testada**: cada tela tem teste com `meetsGuideline` (`androidTapTargetGuideline`, `iOSTapTargetGuideline`, `textContrastGuideline`, `labeledTapTargetGuideline`) | [meetsGuideline](https://api.flutter.dev/flutter/flutter_test/meetsGuideline.html) | `revisor` / `comando` grep por `meetsGuideline` em `test/` (0 no Excurfly) |
| **DT7: build reprodutível e dependências auditadas**: `pubspec.lock` commitado em app (não em pacote), vulnerabilidade pelo OSV | [What not to commit](https://dart.dev/tools/pub/private-files): "For regular packages, don't commit the pubspec.lock file [...] For application packages, we recommend that you commit the pubspec.lock file"; [OSV-Scanner, lockfiles](https://google.github.io/osv-scanner/supported-languages-and-lockfiles/) (Dart: `pubspec.lock`) | `comando`: `osv-scanner scan source --lockfile pubspec.lock` sintaxe da versão atual; `dart pub get --enforce-lockfile` no CI ([dart pub get](https://dart.dev/tools/pub/cmd/pub-get)); `dart pub outdated` só informativo |

## 3. Ferramentas

| Ferramenta | Versão (2026-10-25) | Licença | Cobra? | Link | Comando para o CI |
|---|---|---|---|---|---|
| Dart SDK (analyzer, `dart format`, `dart fix`, linter) | 3.13.7 (local 3.12.2) | BSD-3-Clause | Não | <https://dart.dev/tools/dart-analyze> | `dart analyze --fatal-infos`; `dart format --output=none --set-exit-if-changed .` |
| Flutter SDK (`flutter analyze`, `flutter test`) | 3.47.8 (local 3.44.9) | BSD-3-Clause | Não | <https://docs.flutter.dev> | `flutter analyze --fatal-infos`; `flutter test` |
| `lints` (core, recommended) | 6.1.0 (2026-01-30); core 35 regras, recommended +58 | BSD-3-Clause | Não | <https://pub.dev/packages/lints> | `include: package:lints/recommended.yaml` |
| `flutter_lints` | 6.0.0 (2025-05-27); recommended + 10 de Flutter | BSD-3-Clause | Não | <https://pub.dev/packages/flutter_lints> | `include: package:flutter_lints/flutter.yaml` |
| `very_good_analysis` | 11.0.0 (2026-09-03); 216 regras, os três `strict-*`, `close_sinks: ignore`, `formatter: trailing_commas: automate` | MIT | Não | <https://pub.dev/packages/very_good_analysis> | `include: package:very_good_analysis/analysis_options.yaml` (o `main` já aponta 11.1.0, não publicado) |
| `analysis_server_plugin` (plugins nativos) | 0.3.25 (2026-10-20) | BSD-3-Clause | Não | <https://dart.dev/tools/analyzer-plugins> | `plugins:` no `analysis_options.yaml` |
| `custom_lint` | 0.8.1 (2025-09-09), **sem desenvolvimento ativo** | Apache-2.0 | Não | <https://github.com/invertase/dart_custom_lint> | não adotar |
| DCM (Dart Code Metrics) | 1.40.0; o pacote `dart_code_metrics` 5.7.6 está **descontinuado** | comercial; plano Free: 1 seat, até 50k LOC, 100 regras ([pricing](https://dcm.dev/pricing)); Pro US$ 16/mês | **Sim** | <https://dcm.dev> | não adotar como padrão |
| `dart_code_linter` (fork livre do DCM) | 4.4.2 (2026-10-07) | MIT | Não | <https://github.com/bancolombia/dart-code-linter> | plugin com `metrics:`; CLI `dart run dart_code_linter:metrics analyze lib` **[confirmar]** |
| `test` | 1.33.0 (2026-10-08) | BSD-3-Clause | Não | <https://pub.dev/packages/test> | `dart test` |
| `mocktail` | 1.0.5 (2026-04-10) | MIT | Não | <https://pub.dev/packages/mocktail> | (biblioteca) |
| `mockito` | 5.8.1 (2026-08-06) | Apache-2.0 | Não | <https://pub.dev/packages/mockito> | (biblioteca; exige `build_runner`) |
| `alchemist` (golden) | 0.14.1 (2026-10-08) | MIT | Não | <https://pub.dev/packages/alchemist> | `flutter test --update-goldens` local; `flutter test` no CI |
| `golden_toolkit` | 0.15.0, **descontinuado** | BSD-3-Clause | Não | <https://pub.dev/packages/golden_toolkit> | não adotar |
| `mutation_test` | 1.8.1 (2026-09-12) | BSD-3-Clause | Não | <https://github.com/domohuhn/mutation-test> | `dart run mutation_test <arquivos>`, fora do CI de PR |
| `dependency_validator` | 5.1.0 (2026-09-15) | Apache-2.0 (LICENSE do repo; pub.dev mostra "unknown") | Não | <https://github.com/Workiva/dependency_validator> | `dart run dependency_validator` |
| `patrol` (integração nativa) | 4.10.0 | Apache-2.0 | Não | <https://pub.dev/packages/patrol> | opcional |
| OSV-Scanner | **[confirmar]** versão | Apache-2.0 **[confirmar]** | Não | <https://google.github.io/osv-scanner/> | `osv-scanner scan source --lockfile pubspec.lock` |
| ast-grep | 0.51.0 (Dart embutido) | MIT | Não | <https://ast-grep.github.io/reference/languages.html> | o validador roda |
| jscpd | 5.4.2 (lê `dart`) | MIT | Não | <https://github.com/kucherenko/jscpd> | `npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` |

`integration_test` no pub.dev (1.0.2+3) é o pacote antigo, descontinuado; o atual vem do SDK
(`integration_test: sdk: flutter`), como o Excurfly já faz.

### Como o `analysis_options.yaml` se lê (para o validador)

Fonte: [analysis].

- `include:` importa outro arquivo (string ou lista, ex.: `package:lints/recommended.yaml`). As
  regras do preset **não aparecem no arquivo do projeto**.
- `linter: rules:` aceita **lista** (`- regra`, só liga) **ou mapa** (`regra: true|false`); para
  desligar regra de um `include`, só o mapa ("requires the use of a map rather than a list").
  As duas formas não se misturam no mesmo bloco.
- `analyzer: errors: <regra>: error|warning|info|ignore` muda a severidade (`ignore` desliga).
- `analyzer: language: strict-casts|strict-inference|strict-raw-types: true`.
- `analyzer: exclude: [glob]` tira arquivo da análise (útil para `*.g.dart`).
- `plugins:` (primeiro nível, Dart 3.10+) liga plugins nativos; o diagnóstico de plugin se ignora
  com `// ignore: plugin/regra`.
- Supressão: `// ignore: regra` (linha seguinte ou a mesma) e `// ignore_for_file: regra`; não
  vale para erro de compilação. O único lint que exige motivo é `document_ignores` (Dart 3.5).

Regra coberta, para o motor: está ligada (lista ou `true`) em `linter: rules:` **ou** vem de um
preset conhecido, **e** não está com `false` no mapa nem `ignore` em `analyzer: errors:`; e, sem
`--fatal-infos` no CI, está com `error` ou `warning` em `analyzer: errors:`.

### Bloco sugerido para o `analysis_options.yaml` de um app Flutter da casa

Ponto de partida do grilling, não regra decidida. Os nomes foram aceitos pelo analyzer 3.12.2 (testado
no scratch, sem `undefined_lint`).

```yaml
include: package:very_good_analysis/analysis_options.yaml

analyzer:
  exclude:
    - "**/*.g.dart"
    - "lib/l10n/**"
  errors:
    # E4: erro engolido reprova mesmo sem --fatal-infos
    empty_catches: error
    avoid_catches_without_on_clauses: error
    unawaited_futures: error
    discarded_futures: error
    # DT1
    use_build_context_synchronously: error

linter:
  rules:
    public_member_api_docs: false   # app, não pacote publicado (1.042 achados no Excurfly)
    lines_longer_than_80_chars: false  # o dart format decide a largura
    close_sinks: true               # o vga desliga; religar só se o grilling quiser DT4 por lint
```

## 4. Regras de ast-grep para Dart

Testadas com `npx -y -p @ast-grep/cli ast-grep scan --rule <regra>.yml <arquivo>` (ast-grep 0.51.0).
Na árvore do tree-sitter de Dart o bloco do `catch` é **irmão** do `catch_clause`, filho direto de
`try_statement` (visto com `--debug-query=cst`), por isso a regra usa `follows`.

### `catch-vazio-dart.yml` (E4) — testado: pega 4 de 4, passa 0 de 3

```yaml
# E4 em Dart: o erro é engolido. O lint empty_catches pega só o bloco vazio; esta regra pega também
# o bloco que só imprime ou loga e segue.
id: catch-vazio-dart
language: Dart
severity: error
message: "E4: o erro é engolido. Trate o caso esperado ou deixe subir."
rule:
  kind: block
  follows:
    any: [{ kind: catch_clause }, { kind: type }]
  inside: { kind: try_statement }
  any:
    - regex: '^\{\s*\}$'
    - regex: '^\{\s*(print|debugPrint|log|developer\.log|logger\.\w+)\([^;]*\);\s*\}$'
```

Deve pegar:

```dart
void a() { try { g(); } catch (e) {} }
void b() { try { g(); } on FormatException {} }
void c() { try { g(); } on Exception catch (e) { print(e); } }
void d() { try { g(); } catch (_) { debugPrint('x'); } }
```

Deve passar:

```dart
void a() { try { g(); } on FormatException catch (e) { throw ErroDeLeitura(e); } }
void b() { try { g(); } catch (e, s) { relatar(e, s); rethrow; } }
void c() { try { g(); } finally {} }
```

### `dominio-sem-infra-dart.yml` (D/P3) — **não testado**, esboço

```yaml
id: dominio-sem-infra-dart
language: Dart
severity: error
message: "D/P3: o domínio importa infraestrutura. Defina a interface no domínio; o adapter vive fora."
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  kind: import_or_export
  regex: "['\"](package:(<PACOTES_DE_INFRA>)/|dart:(io|html|ui))"
```

**[confirmar]** o nome do nó de import na gramática (`import_or_export` / `library_import`) com
`--debug-query=cst` antes de publicar. O Excurfly já faz essa regra como teste
(`test/disciplina_test.dart`: "só o módulo de HTTP fala HTTP"), lendo os arquivos por texto.

### `nome-de-uma-letra-dart.yml` (E1) — ideia, a calibrar

Declaração local ou parâmetro com nome de uma letra, exceto `i`/`j` em `for` numérico e `_`. Não
escrita nem testada nesta pesquisa: a gramática de Dart junta tipo e nome em vários nós
(`initialized_identifier`, `formal_parameter`), e a regra precisa ser calibrada contra o Excurfly.

## 5. O que o analyzer acharia hoje no Excurfly (custo de adotar)

Projeto: `excurfly/passenger-app` (Flutter, `environment: sdk: ^3.12.2`), lido sem editar.

- `analysis_options.yaml`: o gerado pelo `flutter create`, só `include: package:flutter_lints/flutter.yaml`,
  `rules:` vazio (com os comentários de exemplo). Dev deps: `flutter_lints ^6.0.0`, `flutter_test`,
  `integration_test` (SDK). Sem mock de pacote (sem `mocktail`/`mockito`): o projeto usa fakes à mão
  (`gateways/mundo_em_memoria.dart`, importado em 43 testes) e um relógio injetado (`app/relogio.dart`).
- `lib/`: 37.180 linhas. Testes: **95 arquivos `*_test.dart`** (`test/` e `integration_test/`),
  **686 `test(`, 534 `testWidgets(`, 304 `group(`**; 1 `matchesGoldenFile`; **0 `meetsGuideline`**;
  `skip:` aparece em `test/prints/` (testes de print de tela). Há testes de disciplina de arquitetura
  (`test/disciplina_test.dart`: "nenhum arquivo fora do tema escreve uma cor", "só o módulo de HTTP
  fala HTTP").
- `dart analyze` no projeto (só leitura; `pubspec.lock` e `.dart_tool/package_config.json` com o
  mesmo SHA antes e depois): **404 issues, exit 3**, todas causadas pelo pacote gerado
  `contrato_gerado` ausente (`path_does_not_exist`; 250 `undefined_getter`, 56 `undefined_method`, …)
  e 28 `unused_element` em `lib/gateways/contrato.g.dart`. **Zero infos de lint** com o
  `flutter_lints`: o preset atual não pega nada.
- Com uma cópia no scratch e as regras desta pesquisa ligadas (mais os três `strict-*`, excluindo
  `contrato.g.dart`):

| Regra | Achados |
|---|---|
| `public_member_api_docs` | 1.042 |
| `avoid_catches_without_on_clauses` | 47 |
| `unnecessary_null_checks` | 31 |
| `discarded_futures` | 24 |
| `prefer_const_constructors` | 19 |
| `avoid_dynamic_calls` | 16 |
| `inference_failure_on_untyped_parameter` (strict-inference) | 12 |
| `cast_nullable_to_non_nullable` | 8 |
| `unawaited_futures` | 2 |
| `only_throw_errors`, `avoid_positional_boolean_parameters` | 1 cada |
| `prefer_final_locals`, `empty_catches`, `avoid_print`, `use_build_context_synchronously`, `document_ignores`, `exhaustive_cases`, `no_default_cases`, `cancel_subscriptions`, `close_sinks` | 0 |

Leitura: o código já é disciplinado (zero `final` faltando, zero `catch` vazio, zero `context` depois
de `await`). O custo real é `public_member_api_docs` (desligar em app) e os 47 `catch` sem `on`,
que são exatamente o E4. Os demais (~115) são correções de uma linha, várias com `dart fix`.

## 6. O que ficou sem confirmação em fonte primária

- **[confirmar]** Comando e versão do OSV-Scanner para `pubspec.lock` (a página lista o lockfile;
  a sintaxe da CLI não foi aberta).
- **[confirmar]** CLI de métricas do `dart_code_linter` (a configuração `metrics:` foi lida no README
  do pub.dev; o comando não foi rodado).
- **[confirmar]** `dominio-sem-infra-dart` e `nome-de-uma-letra-dart`: não testados.
- **[confirmar]** Regex `DECLARACAO_DART` e `TESTE_DART` no motor: propostas, não rodadas no binário.
- Links das páginas do Effective Dart com âncora: o conteúdo foi lido nas páginas, mas nem toda âncora
  foi aberta.

## Recomendações para o grilling

1. **Preset base: `very_good_analysis`** (MIT, 216 regras, já liga os `strict-*` e quase todo o E4).
   `flutter_lints` não pegou nada no Excurfly. Desligar `public_member_api_docs` em app.
   Fonte: seção 3, seção 5.
2. **`--fatal-infos` no CI: sim.** Sem ele lint é info e não reprova (achado 1). Para as regras que
   a casa considera dura, também `analyzer: errors: <regra>: error`, para reprovar no editor e para o
   `lint-cobre` ter o que ler. Fonte: [dart analyze](https://dart.dev/tools/dart-analyze).
3. **`lint-cobre` com YAML de verdade** (achado 2): PR no motor com `cobertura_yaml`. Até lá,
   `comando` `dart analyze --fatal-infos`.
4. **E1 sem `id-length`:** `termo-evitado` com `DECLARACAO_DART` (achado 3) + `revisor`; ast-grep de
   nome de uma letra só se a calibração no Excurfly der pouco ruído. Fonte: achado 5.
5. **E2 (tamanho/complexidade):** `revisor` por padrão; `dart_code_linter` (MIT) como opção se o
   grilling quiser número. DCM não (pago acima de 50k LOC; o Excurfly tem 37k em `lib/`, mas cresce).
   Fonte: [dcm.dev/pricing](https://dcm.dev/pricing).
6. **DT1 a DT7: adotar DT1, DT3, DT4, DT7 por lint/comando**; DT2 (widget como classe) e DT6
   (`meetsGuideline`) como `revisor` + regra do projeto; DT5 já vem no preset. Fonte: seção 2.
7. **`riscos.rs`: `TESTE_DART`** = `^\s*(?:test|testWidgets|testGoldens|goldenTest|patrolTest)\s*\(`
   ligada a `.dart` (achado 4).
8. **Mutação: não no CI de PR.** `mutation_test` (BSD-3, 1.8.1, mantido) é opcional, rodado à mão em
   módulo de domínio. Fonte: [repo](https://github.com/domohuhn/mutation-test).
9. **Golden: só onde a tela é contrato visual**, com `alchemist` (MIT, mantido) e não
   `golden_toolkit` (descontinuado); golden depende de fonte e plataforma, então CI num SO fixo.
   Fonte: seção 3.
10. **Mock: fake à mão por padrão** (como o Excurfly e o [arch] fazem), `mocktail` quando precisar,
    sem `mockito` (exige `build_runner`).
11. **Plugins de lint: `analysis_server_plugin` nativo**, nunca `custom_lint` (achado 7).
12. **Dependências:** `pubspec.lock` commitado em app, `dart pub get --enforce-lockfile` no CI,
    `osv-scanner` para vulnerabilidade, `dependency_validator` para sem uso. Fonte: DT7.
13. **Formatação:** `dart format --output=none --set-exit-if-changed .` no pre-push/CI.
