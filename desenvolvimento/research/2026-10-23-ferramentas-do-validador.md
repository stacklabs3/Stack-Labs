# Ferramentas para o validador de regras de código: custo e licença

Pesquisa de 2026-10-23 para a ADR 0010. Pergunta: as ferramentas que cobririam o catálogo do
validador cobram pelo uso? Critério da decisão, dado pelo humano: **sem cobrança, usamos; com
cobrança, escrevemos a nossa**.

## Resultado

| Ferramenta | Cobre | Licença | Cobra? | Decisão |
|---|---|---|---|---|
| dependency-cruiser 18.7.0 | imports entre pastas, ciclos (JS/TS) | MIT | Não | Usar, pelo tipo `comando` |
| eslint-plugin-boundaries 7.3.0 | imports entre camadas, dentro do ESLint (JS/TS) | MIT | Não | Usar, pelo lint do projeto |
| jscpd 5.4.2 | duplicação de código, várias linguagens | MIT | Não | Usar, pelo tipo `comando` |
| Semgrep CE (motor) | padrões de código, 30+ linguagens | LGPL-2.1 | O motor não; a empresa vende planos pagos e as regras prontas têm licença própria | Não usar |
| Opengrep (fork do Semgrep) | o mesmo do Semgrep CE, com as mesmas regras | LGPL-2.1 | Não | Não usar: 25 vezes mais lento que o ast-grep (medição abaixo) |
| ast-grep 0.51.0 (Rust) | padrões de código por árvore sintática, várias linguagens | MIT | Não | **Usar**, pelo tipo `ast-grep` |

## Detalhes e fontes

- **dependency-cruiser, eslint-plugin-boundaries, jscpd**: o campo `license` do pacote no registro do
  npm é `MIT` para os três (registry.npmjs.org/dependency-cruiser/latest,
  registry.npmjs.org/eslint-plugin-boundaries/latest, registry.npmjs.org/jscpd/latest, consultados em
  2026-10-23). Nenhum tem edição paga. Os dois primeiros só analisam JS/TS.
- **Semgrep**: o motor está sob LGPL-2.1 (arquivo LICENSE de github.com/semgrep/semgrep, último commit
  em 2026-10-21). As regras mantidas pela empresa ficaram sob a *Semgrep Rules License v1.0*, que
  restringe o uso a fins internos e proíbe uso em produto ou serviço concorrente
  ([anúncio de dez/2024](https://semgrep.dev/blog/2024/important-updates-to-semgrep-oss),
  [licenciamento](https://docs.semgrep.dev/licensing); LICENSE de github.com/semgrep/semgrep-rules).
  Há planos pagos por contribuidor (Team e Enterprise); o valor varia entre as fontes e não foi
  confirmado na página oficial, que está bloqueada na rede desta pesquisa **[confirmar]**.
  Não precisamos das regras deles: as nossas são escritas por nós.
- **Opengrep**: fork do Semgrep sob LGPL-2.1, criado quando o Semgrep moveu recursos para licença
  comercial; mantido por um consórcio (Aikido, Endor Labs, Orca Security e outros), compatível com as
  regras do Semgrep, binário autocontido com releases assinados (README e LICENSE de
  github.com/opengrep/opengrep, último commit em 2026-10-23). Sem plano pago.
- **ast-grep**: campo `license` `MIT` em registry.npmjs.org/@ast-grep/cli/latest (0.51.0, consultado
  em 2026-10-23), repositório github.com/ast-grep/ast-grep; escrito em Rust sobre o tree-sitter,
  distribuído pelo npm com binário por plataforma. Sem plano pago. Não tem a análise de fluxo de dados
  (taint) do Opengrep, que nenhuma regra do catálogo usa.
- **LGPL-2.1 no nosso uso**: rodamos o Opengrep como programa no CI, sem embutir nem modificar o
  código dele; a LGPL não impõe obrigação a quem só executa.

## Medição: tempo e memória

Repositório sintético de 2.000 arquivos TypeScript (80 mil linhas, 8 MB) com 20 `catch` vazios; os
dois acharam os 20. Pico de memória medido por `getrusage` do processo filho, no ambiente desta
pesquisa (Linux x86_64).

| O que roda | Tempo | Pico de memória |
|---|---|---|
| Opengrep 1.32.0, repositório inteiro | 8,37 s | 401 MB |
| ast-grep 0.51.0, repositório inteiro | 0,33 s | 14 MB |
| Opengrep, só 10 arquivos alterados | 2,97 s | 267 MB |
| ast-grep, só 10 arquivos alterados | 0,01 s | 14 MB |
| Motor do validador em Node (termo-evitado e todo-sem-issue), repositório inteiro | 0,32 s | 59 MB |
| O mesmo motor reescrito em Rust (ADR 0011), mesma saída | 0,14 s | 11 MB |

O custo do Opengrep é quase todo de arranque: mesmo com 10 arquivos ele leva 3 s. Com o ast-grep e o
modo incremental (ler só o que mudou desde a base), o validador de uma PR típica cabe em décimos de
segundo.

## O que continua sendo nosso

Três checagens que nenhuma ferramenta faz: `cobertura` (os 22 princípios decididos), `termo-evitado`
(os termos *Evitar* do glossário) e `teste-de-contrato-por-port` (princípio L). Também `lint-cobre`,
`todo-sem-issue` e `mensagem-de-commit`, que são poucas linhas e valem para qualquer linguagem.
