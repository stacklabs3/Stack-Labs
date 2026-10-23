# Práticas de TypeScript

Parte dos [princípios de código](../principios-de-codigo.md). Como cada princípio aparece em
TypeScript (Node e React), a regra de máquina que o confere e as práticas **só desta linguagem**
(TS1 a TS5). Fontes primárias, testes e o porquê de cada linha: pesquisa
`desenvolvimento/research/2026-10-23-praticas-typescript.md` do Stack-Labs.

No grilling, cada linha é a recomendação da pergunta do princípio; o projeto decide. No diff, a
regra ligada é violação dura; a desligada é julgamento.

## Ferramentas

Todas MIT, ISC ou Apache-2.0, sem plano pago. Fixe a versão no `package.json`.

| Ferramenta | Versão | Papel |
|---|---|---|
| `typescript` | **6.0.x** | Compilador e typecheck (`tsc --noEmit`) |
| `eslint` + `typescript-eslint` | 10.13.0 + 8.72.0 | Lint com tipos; preset `strictTypeChecked` |
| `eslint-plugin-react-hooks`, `eslint-plugin-react`, `eslint-plugin-jsx-a11y` | 7.1.1, 7.37.5, 6.10.2 | Frontend React |
| `@eslint-community/eslint-plugin-eslint-comments` | 4.8.1 | `eslint-disable` só com motivo |
| `knip` | 6.42.1 | Export, arquivo e dependência sem uso |
| `@ast-grep/cli`, `jscpd`, `dependency-cruiser` | 0.51.0, 5.4.2, 18.7.0 | Validador |

**TypeScript 7:** o `typescript-eslint` 8.72.0 aceita só `typescript <6.1.0`, e sem ele as regras com
tipo (`no-floating-promises`, `switch-exhaustiveness-check`, `no-unsafe-*`) param. O modelo fixa o
TS 6.0. Projeto que precisa do TS 7 troca o ESLint pelo `oxlint --type-aware` com `oxlint-tsgolint`
(MIT), que roda sobre o TS 7 e tem 60 das 62 regras com tipo; falta `naming-convention` (E1). A
decisão volta quando o `typescript-eslint` aceitar o TS 7.

## Por princípio

| Princípio | Em TypeScript | Confere |
|---|---|---|
| E1 (obrigatório) | Nenhum nome de uma letra, nem em laço, lambda, `catch` ou genérico; só `_` | `id-length` com `{ min: 2, exceptions: ["_"], properties: "always" }`; ast-grep `parametro-de-tipo-sem-nome.yml` (o `id-length` não vê o genérico) |
| E1 | Caixa por tipo de identificador; booleano com prefixo do glossário (`esta`, `tem`, `pode`) | `@typescript-eslint/naming-convention` (fora dos presets); `termo-evitado` |
| E2 | Até 3 níveis e 3 parâmetros; `switch` sem cair no próximo `case`; `else` depois de `return` some | `max-depth`, `max-params`, `complexity`, `no-else-return`, `no-nested-ternary`; `noFallthroughCasesInSwitch` |
| E3 | `strict` e acesso por índice que pode faltar; nada de `any`, `x!` nem `{...} as T`; `x as unknown as T` só na conversão da borda | tsconfig `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`; `no-explicit-any`, `no-unsafe-*`, `no-non-null-assertion`, `consistent-type-assertions`; ast-grep `afirmacao-dupla.yml`; `tsc --noEmit` |
| E4 | Promessa sempre esperada ou tratada; só se lança `Error`; erro do domínio com classe própria; `.catch(() => {})` é `catch` vazio | `no-floating-promises`, `no-misused-promises`, `only-throw-error`, `return-await`; ast-grep `catch-vazio.yml`, `promessa-engolida.yml`, `erro-generico-no-dominio.yml` |
| E4 (verificador) | `@ts-ignore` e `@ts-nocheck` nunca; `@ts-expect-error` e `eslint-disable` só com o porquê | `@typescript-eslint/ban-ts-comment`, `@eslint-community/eslint-comments/require-description` |
| E5 | `const`; membro privado não reatribuído é `readonly`; parâmetro não se muta; nada de `export let` | `prefer-const`, `no-param-reassign` (`props: true`), `@typescript-eslint/prefer-readonly` (fora dos presets), `no-restricted-syntax` |
| E6 | Código morto e ramo impossível saem | `noUnusedLocals`, `noUnusedParameters`, `allowUnreachableCode: false`, `no-unnecessary-condition`; `knip`; `todo-sem-issue` |
| E7 | Bloco repetido | `npx jscpd@5.4.2 --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` |
| E8 | Número com significado vira constante; conjunto fechado vira união de literais | `@typescript-eslint/no-magic-numbers` |
| P1 | Função exportada declara entrada e retorno | `explicit-module-boundary-types` (opcional); profundidade é do revisor |
| P3, D | Domínio sem import de infraestrutura nem efeito global (`fetch`, `localStorage`, `Date.now()`, `Math.random()`) | ast-grep `dominio-sem-infra.yml`, `efeito-global-no-dominio.yml`, `ambiente-so-na-montagem.yml`; ou `no-restricted-imports` |
| P4 | Sem ciclo nem import cruzado entre pastas | `npx -p dependency-cruiser@18.7.0 depcruise --config .dependency-cruiser.cjs src` |
| P8 | `import type`; nada de API `@deprecated`; um formatador | `verbatimModuleSyntax`, `no-deprecated`, Prettier ou Biome no `comando` |
| O | `switch` sobre união cobre todo membro, sem `default` que esconda o novo | `@typescript-eslint/switch-exhaustiveness-check` (fora dos presets) |
| L | Método que sobrescreve declara `override`; adapter passa no contrato da port | `noImplicitOverride`; `teste-de-contrato-por-port` |

## Ressalvas

- **Limite de tamanho (E2) mede a função, não desenha o módulo.** Quebre a função longa em passos
  privados do mesmo módulo; a interface continua pequena (P1).
- **Preset não aparece no texto da configuração.** Para o `lint-cobre`, liste no `eslint.config.js`
  as regras delegadas a ele. A prova de que o código cumpre é o `comando` com o próprio lint
  ([validador.md](../validador.md)).
- **Google prefere `default` em todo `switch`;** a casa fica com a exaustividade (O). O Google aceita
  `catch` vazio com comentário; o modelo `catch-vazio.yml` reprova sempre, e o projeto que quiser a
  exceção decide no grilling.

## Só desta linguagem

Cada uma é decidida no ticket *Lint e formatador* do mapa de Regras de código e, adotada, vira regra
com `"principio": "TS<n>"` no `.json` e uma linha na tabela *Práticas da linguagem* do
`docs/regras-de-codigo.md`.

| ID | Prática | Confere |
|---|---|---|
| TS1 | **Sintaxe apagável:** sem `enum`, `namespace` com código nem *parameter properties*, para o arquivo rodar com a remoção de tipos do Node. Conjunto fechado vira união de literais (E8) | tsconfig `erasableSyntaxOnly` |
| TS2 | **Arquivo transpilável sozinho** (esbuild, SWC, Vite) | tsconfig `isolatedModules`, `verbatimModuleSyntax` |
| TS3 | **Contratos do React:** hook só no topo; dependências completas; render puro; componente nunca declarado dentro de outro; `key` estável | `react-hooks/rules-of-hooks`, `exhaustive-deps`, preset `recommended` da 7.x; `react/no-unstable-nested-components`, `jsx-key`, `no-array-index-key` |
| TS4 | **Acessibilidade** da tela | `eslint-plugin-jsx-a11y`, preset `recommended` |
| TS5 | **Igualdade e coerção explícitas** | `eqeqeq`, `no-implicit-coercion` |

## Modelos

Em `.claude/watchdogs/modelos/`: `regras-de-codigo.json` (o `.json` de TypeScript),
`lint/eslint.config.js`, `lint/tsconfig.json` e, em `ast-grep/`, os documentos `TypeScript` e `Tsx`.
Copie só o que o grilling decidiu.
