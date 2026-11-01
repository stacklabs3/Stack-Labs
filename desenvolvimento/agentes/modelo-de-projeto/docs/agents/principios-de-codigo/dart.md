# Práticas de Dart e Flutter

Parte dos [princípios de código](../principios-de-codigo.md). Como cada princípio aparece em Dart
(quase sempre com Flutter), a regra de máquina que o confere e as práticas **só desta linguagem**
(DT1 a DT6). Fontes primárias (Effective Dart, dart.dev, docs.flutter.dev, pub.dev), testes e o
porquê de cada linha: pesquisa `desenvolvimento/research/2026-10-25-praticas-dart.md` do Stack-Labs.

No grilling, cada linha é a recomendação da pergunta do princípio; o projeto decide. No diff, a
regra ligada é violação dura; a desligada é julgamento.

## Ferramentas

Todas BSD-3-Clause, MIT ou Apache-2.0, sem plano pago. Fixe a versão no `pubspec.yaml`.

| Ferramenta | Versão | Papel |
|---|---|---|
| Dart SDK (analyzer, linter, `dart format`) | 3.13 (3.12 com o preset 10.3.0) | Lint e formatador; regras no `analysis_options.yaml` |
| Flutter SDK (`flutter analyze`, `flutter test`) | 3.47 | O mesmo analyzer, mais `flutter_test` |
| `very_good_analysis` | 11.0.0 com Dart 3.13 ou mais novo; 10.3.0 com Dart 3.12 | Preset base: 216 regras (na 11.0.0) e os três `strict-*` |
| `mocktail` | 1.0.5 | Mock sem geração de código, quando o fake à mão não basta |
| `alchemist` | 0.14.1 | Golden test |
| `dependency_validator` | 5.1.0 | Dependência sem uso ou fora do `pubspec.yaml` |
| OSV-Scanner | v2.7.0 | Vulnerabilidade no `pubspec.lock` |
| `mutation_test` | 1.8.1 | Mutação, fora do CI de PR |
| `ast-grep`, `jscpd` | 0.51.0, 5.4.2 | Validador; os dois leem Dart |

**Requisito para o `rodar ci` passar.** O motor usa o `PATH` e não procura SDK: ferramenta ausente
sai com 127 e reprova (ADR 0021).

- Na máquina e no CI: SDK do Dart ou Flutter, Node (`npx jscpd`) e OSV-Scanner v2.7.0.
- No `pubspec.yaml`: `environment.sdk` preenchido; `very_good_analysis` e `dependency_validator` em
  `dev_dependencies`.
- No git: `pubspec.lock`, em app.

**Lint é `info` e `info` não reprova.** `dart analyze` sai com 0 quando só há infos. Toda receita usa
`dart analyze --fatal-infos` (ou `flutter analyze --fatal-infos`), o `-D warnings` do Dart. As regras
duras (E4, DT1) sobem também para `error` em `analyzer: errors:`, para o editor mostrar em vermelho.

**Nível no `analysis_options.yaml`, prova no `comando`.** A regra adotada entra escrita em
`linter: rules:`, mesmo que o preset já a ligue: o `include:` não aparece no arquivo, e o
`lint-cobre` só conta o que está nele. O código a cumpre quando `dart analyze --fatal-infos` passa.

**Exceção pontual:** `// ignore: <regra>` com o porquê no comentário de cima; o lint
`document_ignores` exige o porquê.

**Lint próprio da casa** nasce como plugin do `analysis_server_plugin` (chave `plugins:`, Dart
3.10+). O `custom_lint` não é mais desenvolvido, e o DCM é pago acima de 50 mil linhas.

## Por princípio

| Princípio | Em Dart | Confere |
|---|---|---|
| E1 (obrigatório) | Nenhum nome de uma letra, nem em laço, lambda, `catch` ou genérico; só `_` | ast-grep `nome-de-uma-letra.yml` (o linter do Dart não tem `id-length`) |
| E1 | Caixa do Effective Dart: `UpperCamelCase` em tipo, `lowerCamelCase` em membro e constante, `snake_case` em arquivo | `camel_case_types`, `non_constant_identifier_names`, `constant_identifier_names`, `file_names`; `termo-evitado` |
| E2 | Função curta, poucos parâmetros, pouco aninhamento; `bool` posicional não diz o que é | `avoid_positional_boolean_parameters`; tamanho e aninhamento são do revisor (o SDK não mede) |
| E3 | Tipos estritos; nada de `dynamic` solto; `!` só onde o tipo não sabe o que o programador sabe; conceito do domínio em `extension type` ou classe que valida | `strict-casts`, `strict-inference`, `strict-raw-types` em `analyzer: language:`; `avoid_dynamic_calls`, `unnecessary_null_checks`, `cast_nullable_to_non_nullable` |
| E4 | Erro esperado capturado pelo tipo (`on X catch`); só se lança `Exception` ou `Error`; `Future` nunca descartado (`unawaited(...)` marca o descarte de propósito); `catch` que só imprime é `catch` vazio | `avoid_catches_without_on_clauses`, `avoid_catching_errors`, `empty_catches`, `only_throw_errors`, `use_rethrow_when_possible`, `unawaited_futures`, `discarded_futures`; ast-grep `catch-vazio.yml` |
| E4 (verificador) | `// ignore:` só com o porquê | `document_ignores` |
| E5 | `final` em local e campo; construtor `const`; getter sem efeito colateral | `prefer_final_locals`, `prefer_final_in_for_each`, `prefer_final_fields`; o getter é do revisor |
| E6 | Código morto sai; `print` só na borda; `TODO` com a issue | `unused_element`, `unused_import` (warning, já reprova); `avoid_print`; `todo-sem-issue` |
| E7 | Bloco repetido | `npx jscpd@5.4.2 --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` |
| E8 | Número com significado vira `const` com nome | Revisor (o linter do Dart não tem lint de número mágico) |
| P1, I | Privado com `_`; implementação em `lib/src/`, `export` só a interface | `implementation_imports`; o resto é do revisor |
| P3, D | Domínio sem `http`, `shared_preferences`, plugin nem `dart:io`; relógio e acaso entram por parâmetro. A View só desenha, o ViewModel guarda o estado da tela, o Repository é a fonte da verdade | ast-grep `dominio-sem-infra.yml` (documento `Dart`); mais forte: domínio num pacote do workspace do pub sem a dependência no `pubspec.yaml` dele |
| P4 | Import relativo nunca sai de `lib/` | `avoid_relative_lib_imports` |
| P5 (obrigatório) | Todo `test`, `testWidgets`, `goldenTest` e `patrolTest` cita o `RK<n>` (no nome ou no `group`); afirma com `expect`; fake à mão por padrão, `mocktail` só no que não é nosso | `teste-cobre-risco`; ast-grep `teste-sem-afirmacao.yml`, `teste-desligado.yml` |
| P5 (golden) | Golden só na tela que é contrato visual, com `alchemist`, num SO fixo no CI (a fonte muda entre macOS e Linux) | Revisor |
| P7 | Pacote importado está no `pubspec.yaml`; dependência sem uso sai | `depend_on_referenced_packages`; `dart run dependency_validator` |
| P8 | Um formatador; retorno sempre declarado; `@override` sempre escrito | `dart format --output=none --set-exit-if-changed .`; `always_declare_return_types`, `annotate_overrides` |
| O | Variante nova é subclasse nova de `sealed class`; `switch` sem `default` que esconda o caso novo | O compilador recusa `switch` incompleto sobre `sealed`; `exhaustive_cases`, `no_default_cases` |
| L | Adapter cumpre a interface inteira, sem `throw UnimplementedError()` | `teste-de-contrato-por-port` (`contrato: "test/contrato_{nome}_test.dart"`) |

## Ressalvas

- **`public_member_api_docs` só em pacote publicado.** Em app, desligue (`false`): no Excurfly deu
  1.042 achados e choca com E6.
- **Keys** (`use_key_in_widget_constructors`) já vêm no preset; `Key` em lista que muda de ordem é do
  revisor.
- **`close_sinks` o preset desliga** por falso positivo; DT3 fica com `cancel_subscriptions` e o revisor.
- **Código gerado** (`*.g.dart`, `*.freezed.dart`, `*.mocks.dart`) sai em `analyzer: exclude:` e no `ignorar` do `.json`.
- **Mutação fora do CI de PR.** `dart run mutation_test <arquivos>` é lento; rode à mão no domínio.

## Só desta linguagem

Cada uma é decidida no ticket *Lint e formatador* do mapa de Regras de código e, adotada, vira regra
com `"principio": "DT<n>"` no `.json` e uma linha na tabela *Práticas da linguagem* do
`docs/regras-de-codigo.md`.

| ID | Prática | Confere |
|---|---|---|
| DT1 | **`BuildContext` não atravessa `await`** sem conferir `context.mounted` | `use_build_context_synchronously` |
| DT2 | **Construtor `const` onde der:** o Flutter pula o rebuild da subárvore `const` | `prefer_const_constructors`, `prefer_const_constructors_in_immutables` |
| DT3 | **Recurso com ciclo de vida é liberado:** `dispose()` do controller, `cancel()` da `StreamSubscription` | `cancel_subscriptions`; o `dispose` é do revisor |
| DT4 | **Build reprodutível e dependências auditadas:** `pubspec.lock` no git em app (não em pacote), `--enforce-lockfile` no CI, vulnerabilidade pelo OSV | `dart pub get --enforce-lockfile`; `osv-scanner scan source --lockfile pubspec.lock` |
| DT5 | **Pedaço de UI reutilizável é classe de widget**, não método que devolve `Widget` | Revisor |
| DT6 | **Acessibilidade testada:** cada tela tem teste com `meetsGuideline` (alvo de toque, contraste, rótulo) | Revisor |

## Modelos

Em `.claude/watchdogs/modelos/`: `regras-de-codigo.dart.json`, `lint/analysis_options.yaml` e, em
`ast-grep/`, os documentos `Dart` e o `nome-de-uma-letra.yml`. Copie só o que o grilling decidiu.
