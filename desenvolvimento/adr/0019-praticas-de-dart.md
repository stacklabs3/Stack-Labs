# Práticas de Dart e Flutter, e o validador lendo o `analysis_options.yaml`

Dart entra como linguagem da casa no formato da ADR 0012, quase sempre com Flutter (o app do
Excurfly). A pesquisa `research/2026-10-25-praticas-dart.md`, em fonte primária, ligou cada prática a
um dos 22 princípios e mostrou três coisas que mudam o método: lint do Dart é `info` e `info` não
reprova o `dart analyze`; o linter não tem `id-length` nem número mágico; e o `lint-cobre` lia o
`analysis_options.yaml` como texto solto, contando `regra: false` e linha comentada como ligadas. O
humano decidiu no grilling: preset `very_good_analysis` com `public_member_api_docs` desligado em app;
`dart analyze --fatal-infos` como prova, com as regras duras também em `error`; o motor lendo o YAML
de verdade; E1 por ast-grep (o ast-grep 0.51 lê Dart); mutação fora do CI de PR; golden só em tela
que é contrato visual, com `alchemist`; e seis práticas só de Dart, DT1 a DT6 (`BuildContext` depois
de `await`, `const`, liberar recurso, lockfile e OSV, widget como classe, acessibilidade testada).

**Consequências:** o guia `docs/agents/principios-de-codigo/dart.md` e os modelos
`regras-de-codigo.dart.json`, `lint/analysis_options.yaml` e os documentos `Dart` do ast-grep, com o
`nome-de-uma-letra.yml` novo. O `lint-cobre` conta a regra só se ela estiver em `linter: rules:` ou
subir para `error`/`warning` em `analyzer: errors:`; `false` e `ignore` desligam. Regra que vem só do
`include:` não conta: o projeto a escreve no arquivo, como no ESLint. O `riscos.rs` reconhece `test`,
`testWidgets`, `goldenTest` e `patrolTest` em `.dart`, e lê o nome na linha de baixo quando o
`dart format` quebra a chamada. O `termo-evitado` lê `final`, `late`, a declaração tipada, `mixin`,
`extension` e `typedef`. `DT<n>` passa a ser ID de prática. O Excurfly, adotando, tem 57 nomes de uma
letra, 47 `catch` sem `on` e 19 testes com `skip:` a resolver.
