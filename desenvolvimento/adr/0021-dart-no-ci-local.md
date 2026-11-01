# Dart no CI local: SDK do PATH, Flutter e analysis_options por pasta

Emenda a ADR 0019. Ao levar o Dart ao `rodar ci` no `passenger-app/` do Excurfly, três pontos ficaram
em aberto. O SDK: o motor podia procurá-lo ou usar o que está no `PATH`. O projeto Flutter: `dart
analyze` ou `flutter analyze`. E o `analysis_options.yaml` dentro de uma subpasta: testado com Dart
3.12.2, um `lib/sub/analysis_options.yaml` com `avoid_print: false` fez o `dart analyze --fatal-infos`
deixar de acusar o `print` daquela pasta, e o `lint-cobre` não lia esse arquivo. O humano decidiu no
grilling: o motor não procura SDK, usa o `PATH`, e ferramenta ausente reprova (saída 127, falha
fechada); projeto Flutter usa `flutter analyze --fatal-infos` e põe o passo de geração no próprio
comando; e o `lint-cobre` reprova `analysis_options.yaml` versionado dentro da pasta de um `config`
declarado e fora da lista de `config`.

**Consequências:** quem roda o CI precisa do SDK na máquina, e o CI do projeto o instala antes do
validador; o requisito mora no `dart.md`, não nos requisitos de instalação do método, que valem para
todo projeto. Projeto que atualiza os watchdogs pode ver reprovado um `analysis_options.yaml` por
pasta que antes passava. Arquivo ignorado pelo git não dispara, como o de código gerado. Regras Dart
em subpasta usam prefixos por regra e `cd <pasta> && ...`, sem campo novo no motor; `{arquivos}` vem
relativo à raiz e não combina com `cd`.
