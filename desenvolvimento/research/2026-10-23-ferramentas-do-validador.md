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
| Opengrep (fork do Semgrep) | o mesmo do Semgrep CE, com as mesmas regras | LGPL-2.1 | Não | **Usar**, pelo tipo `opengrep` |

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
- **LGPL-2.1 no nosso uso**: rodamos o Opengrep como programa no CI, sem embutir nem modificar o
  código dele; a LGPL não impõe obrigação a quem só executa.

## O que continua sendo nosso

Três checagens que nenhuma ferramenta faz: `cobertura` (os 22 princípios decididos), `termo-evitado`
(os termos *Evitar* do glossário) e `teste-de-contrato-por-port` (princípio L). Também `lint-cobre`,
`todo-sem-issue` e `mensagem-de-commit`, que são poucas linhas e valem para qualquer linguagem.
