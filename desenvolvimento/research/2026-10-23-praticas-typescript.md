# Princípios de bom desenvolvimento em TypeScript (Node e React) e como verificá-los

Pesquisa de 2026-10-23 para as validações automáticas do método. Pergunta: que práticas de
TypeScript as fontes canônicas recomendam, a qual dos 22 princípios da casa cada uma se liga, e com
que checagem do catálogo (`lint-cobre`, `comando`, `ast-grep`, `revisor`) ela se confere com
ferramenta gratuita.

Desfecho: [ADR 0012](../adr/0012-praticas-da-linguagem.md). As linhas desta pesquisa viraram
`docs/agents/principios-de-codigo/typescript.md` e os modelos de `.claude/watchdogs/modelos/` do
`modelo-de-projeto`; as práticas fora dos 22 viraram TS1 a TS5. Os achados sobre o `lint-cobre` e o
jscpd foram corrigidos no motor e nas receitas na mesma mudança.

## Como as fontes foram lidas

Os sites de documentação (typescriptlang.org, typescript-eslint.io, eslint.org, react.dev,
google.github.io, biomejs.dev, oxc.rs) estavam bloqueados pela rede desta pesquisa. Em todos os
casos lemos o **arquivo-fonte da própria página no repositório oficial**, que é o que o site
publica. Os dois links aparecem abaixo: a página pública (para quem lê) e o arquivo lido.

| Fonte | Página pública | Arquivo lido (raw.githubusercontent.com) |
|---|---|---|
| tsconfig reference | `https://www.typescriptlang.org/tsconfig/#<flag>` | `microsoft/TypeScript-Website/v2/packages/tsconfig-reference/copy/en/options/<flag>.md`; família `strict` em `.../scripts/tsconfigRules.ts` (linhas 113-134) |
| Notas do TypeScript 6.0 | `https://www.typescriptlang.org/docs/handbook/release-notes/typescript-6-0.html` | `microsoft/TypeScript-Website/v2/packages/documentation/copy/en/release-notes/TypeScript 6.0.md` |
| typescript-eslint, regras | `https://typescript-eslint.io/rules/<regra>` | `typescript-eslint/typescript-eslint/main/packages/eslint-plugin/docs/rules/<regra>.mdx` |
| typescript-eslint, presets | `https://typescript-eslint.io/users/configs` | `.../packages/eslint-plugin/src/configs/flat/{strict,stylistic,recommended}-type-checked.ts` |
| ESLint core | `https://eslint.org/docs/latest/rules/<regra>` | `eslint/eslint/main/docs/src/rules/<regra>.md` |
| eslint-plugin-react-hooks | `https://react.dev/reference/eslint-plugin-react-hooks` | `facebook/react/main/packages/eslint-plugin-react-hooks/README.md` |
| eslint-plugin-react | `https://github.com/jsx-eslint/eslint-plugin-react` | `jsx-eslint/eslint-plugin-react/master/docs/rules/<regra>.md` |
| Google TypeScript Style Guide | `https://google.github.io/styleguide/tsguide.html#<âncora>` | `google/styleguide/gh-pages/tsguide.html` |
| tsgolint (oxlint com tipos) | `https://github.com/oxc-project/tsgolint` | `oxc-project/tsgolint/main/README.md` |
| Biome, lista de regras | `https://biomejs.dev/linter/rules/` | `configuration_schema.json` do pacote `@biomejs/biome@2.6.2` (npm) |
| oxlint, lista de regras | `https://oxc.rs/docs/guide/usage/linter/rules.html` | `configuration_schema.json` do pacote `oxlint@1.89.0` (npm) |

## Achado que muda a recomendação: TypeScript 7 e o lint com tipos

- O `latest` do TypeScript no npm é **7.0.2** (publicado em 2026-07-08; registry.npmjs.org/typescript),
  a versão nativa (Go). O 6.0 foi a versão de transição.
- **typescript-eslint 8.72.0 declara `peerDependencies.typescript: ">=4.8.4 <6.1.0"`**
  (registry.npmjs.org/typescript-eslint/latest). Ou seja: as regras com tipo (`no-floating-promises`,
  `no-misused-promises`, `switch-exhaustiveness-check`, `no-unsafe-*`) **ainda não rodam oficialmente
  com o TypeScript 7**. Projeto que quer essas regras pelo ESLint fixa `typescript@~6.0`.
- A alternativa é o **oxlint `--type-aware`** com `oxlint-tsgolint` (MIT, 7.0.2005), que roda sobre o
  typescript-go, "targets TypeScript 7", se declara **estável** e implementa 60 das 62 regras com tipo
  do typescript-eslint, entre elas todas as citadas nesta pesquisa (faltam só `naming-convention` e
  `prefer-destructuring`) (README de oxc-project/tsgolint, seções *Versioning* e *Implemented Rules*).
- No TypeScript 6.0, **`strict` passou a ser `true` por padrão**, e também
  `noUncheckedSideEffectImports` (notas do 6.0, seção *Simple Default Changes*). Mesmo assim,
  recomendamos escrever `"strict": true` no tsconfig: deixa a decisão visível e o `lint-cobre`
  consegue achá-la.
- Se o typescript-eslint tem data para aceitar TS 7 **[confirmar]** (não achamos anúncio no repositório).

## Achado sobre o `lint-cobre`

O motor (`validador.rs`, linhas 458-471) confere **se o nome da regra aparece no texto** do arquivo de
configuração. Duas consequências:

1. Regra ligada só pelo preset (`...tseslint.configs.strictTypeChecked`) **não aparece no texto**.
   Ou o projeto lista as regras delegadas explicitamente no `eslint.config.js`, ou o `.json` usa o
   nome do preset como `regras` (`"strictTypeChecked"`), que é texto que aparece.
2. `"max-depth": "off"` também passa, porque o nome está lá. Proposta para o motor (PR no
   Stack-Labs): ler `npx eslint --print-config <arquivo>` em vez do texto **[confirmar viabilidade]**.
3. Flags do compilador cabem no `lint-cobre` com `config: ["tsconfig.json"]` e
   `regras: ["noUncheckedIndexedAccess", ...]`, com o mesmo limite (presença, não valor). A prova de
   que o código obedece às flags é o `comando` do typecheck: `npx tsc --noEmit -p tsconfig.json`.

## 1. Tabela: prática, fonte, princípio, verificação

Legenda de verificação: **LC** = `lint-cobre` (nome exato da regra/flag a listar), **CMD** =
`comando`, **AG** = `ast-grep`, **REV** = `revisor`. "Preset STC" = já vem no
`strict-type-checked`; "fora dos presets" = precisa ser ligada à mão.

### 1.1 Compilador (tsconfig)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Ligar `strict` (família: `alwaysStrict`, `strictNullChecks`, `strictBindCallApply`, `strictBuiltinIteratorReturn`, `strictFunctionTypes`, `strictPropertyInitialization`, `noImplicitAny`, `noImplicitThis`, `useUnknownInCatchVariables`). Por quê: `null`/`undefined` e `any` implícito viram erro de compilação. | [tsconfig#strict](https://www.typescriptlang.org/tsconfig/#strict); tsconfigRules.ts l.113-134 | E3 | LC `config: tsconfig.json`, `regras: ["\"strict\""]` + CMD `npx tsc --noEmit -p tsconfig.json` |
| `useUnknownInCatchVariables` (dentro de `strict`): o erro do `catch` é `unknown`, não `any`. Por quê: obriga a inspecionar o erro antes de usar. | [tsconfig#useUnknownInCatchVariables](https://www.typescriptlang.org/tsconfig/#useUnknownInCatchVariables) | E4 | Coberto por `strict`; CMD typecheck |
| `noUncheckedIndexedAccess`: acesso por índice ganha `| undefined`. Por quê: `lista[0]` e `mapa[chave]` podem faltar e o tipo passa a dizer isso. Fora de `strict`. | [tsconfig#noUncheckedIndexedAccess](https://www.typescriptlang.org/tsconfig/#noUncheckedIndexedAccess) | E3 | LC `regras: ["noUncheckedIndexedAccess"]` |
| `exactOptionalPropertyTypes`: `prop?: T` não aceita `undefined` explícito. Por quê: "ausente" e "presente com `undefined`" são estados diferentes do domínio. Fora de `strict`. | [tsconfig#exactOptionalPropertyTypes](https://www.typescriptlang.org/tsconfig/#exactOptionalPropertyTypes) | E3 | LC `regras: ["exactOptionalPropertyTypes"]` |
| `noImplicitOverride`: método que sobrescreve a base exige `override`. Por quê: renomear na base quebra a subclasse em silêncio. | [tsconfig#noImplicitOverride](https://www.typescriptlang.org/tsconfig/#noImplicitOverride) | L (a subclasse acompanha o contrato da base) | LC `regras: ["noImplicitOverride"]` |
| `noPropertyAccessFromIndexSignature`: chave de assinatura de índice só com `obj["k"]`. Por quê: o ponto passa a significar "campo declarado". | [tsconfig#noPropertyAccessFromIndexSignature](https://www.typescriptlang.org/tsconfig/#noPropertyAccessFromIndexSignature) | E3 | LC `regras: ["noPropertyAccessFromIndexSignature"]` |
| `noImplicitReturns`: todo caminho de função com retorno devolve valor. | [tsconfig#noImplicitReturns](https://www.typescriptlang.org/tsconfig/#noImplicitReturns) | E4 (caminho esquecido vira `undefined` silencioso) | LC `regras: ["noImplicitReturns"]` |
| `noFallthroughCasesInSwitch`: `case` não vazio termina em `break`/`return`/`throw`. | [tsconfig#noFallthroughCasesInSwitch](https://www.typescriptlang.org/tsconfig/#noFallthroughCasesInSwitch) | E2 | LC `regras: ["noFallthroughCasesInSwitch"]` |
| `noUnusedLocals` e `noUnusedParameters`: variável e parâmetro sem uso são erro. | [tsconfig#noUnusedLocals](https://www.typescriptlang.org/tsconfig/#noUnusedLocals), [#noUnusedParameters](https://www.typescriptlang.org/tsconfig/#noUnusedParameters) | E6 (código morto se apaga), P7 | LC `regras: ["noUnusedLocals", "noUnusedParameters"]` (ou `@typescript-eslint/no-unused-vars`, preset STC) |
| `verbatimModuleSyntax`: import só de tipo precisa de `import type`; nada é apagado por inferência. | [tsconfig#verbatimModuleSyntax](https://www.typescriptlang.org/tsconfig/#verbatimModuleSyntax) | P8 | LC `regras: ["verbatimModuleSyntax"]` |
| `erasableSyntaxOnly`: proíbe `enum`, `namespace` com código e *parameter properties*, para o arquivo rodar com a remoção de tipos do Node (≥ 23.6). | [tsconfig#erasableSyntaxOnly](https://www.typescriptlang.org/tsconfig/#erasableSyntaxOnly) | **Nova** (ver seção 2) | LC `regras: ["erasableSyntaxOnly"]` |
| `isolatedModules`: cada arquivo transpila sozinho (Babel, esbuild, SWC). | [tsconfig#isolatedModules](https://www.typescriptlang.org/tsconfig/#isolatedModules) | **Nova** (ferramental), ver seção 2 | LC `regras: ["isolatedModules"]` |
| `allowUnreachableCode: false`: código inalcançável é erro (o padrão só avisa no editor). | [tsconfig#allowUnreachableCode](https://www.typescriptlang.org/tsconfig/#allowUnreachableCode) | E6 | LC `regras: ["allowUnreachableCode"]` |

### 1.2 typescript-eslint (8.72.0)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Usar o preset `strict-type-checked` (+ `stylistic-type-checked`). Por quê: é o conjunto mantido pelo projeto para código com tipos; cobre `any`, promessas, `throw`, condições inúteis. | [users/configs](https://typescript-eslint.io/users/configs); `configs/flat/strict-type-checked.ts` | E3, E4 (base) | LC `regras: ["strictTypeChecked"]` (texto do preset; ver achado do `lint-cobre`) |
| `no-floating-promises`: promessa criada precisa de `await`, `return`, `.catch`, `.then(ok, erro)` ou `void`. Por quê: rejeição solta some. Preset STC. | [rules/no-floating-promises](https://typescript-eslint.io/rules/no-floating-promises) | E4 | LC `@typescript-eslint/no-floating-promises` |
| `no-misused-promises`: promessa onde o lugar não a espera (`if (promessa)`, callback `void` de `forEach`, `onClick` async). Preset STC. | [rules/no-misused-promises](https://typescript-eslint.io/rules/no-misused-promises) | E4 | LC `@typescript-eslint/no-misused-promises` |
| `no-explicit-any`. Preset STC. | [rules/no-explicit-any](https://typescript-eslint.io/rules/no-explicit-any) | E3 (já citado no princípio) | LC `@typescript-eslint/no-explicit-any` |
| `no-unsafe-assignment`, `-argument`, `-member-access`, `-call`, `-return`: valor `any` (ex.: `JSON.parse`, `res.json()`) não atravessa sem conversão. Preset STC. | [rules/no-unsafe-assignment](https://typescript-eslint.io/rules/no-unsafe-assignment) e irmãs | E3 (validar na borda) | LC as cinco `@typescript-eslint/no-unsafe-*` |
| `switch-exhaustiveness-check`: `switch` sobre união cobre todos os membros. Opções: `considerDefaultExhaustiveForUnions: false` (o `default` não esconde o caso novo), `requireDefaultForNonUnion: true`. **Fora dos presets.** | [rules/switch-exhaustiveness-check](https://typescript-eslint.io/rules/switch-exhaustiveness-check) | O (variante nova acusa cada `switch` que precisa dela; *Repeated Switches*), E3 | LC `@typescript-eslint/switch-exhaustiveness-check` |
| `prefer-readonly`: membro privado nunca reatribuído fora do construtor vira `readonly`. **Fora dos presets.** | [rules/prefer-readonly](https://typescript-eslint.io/rules/prefer-readonly); Google [#use-readonly](https://google.github.io/styleguide/tsguide.html#use-readonly) | E5 | LC `@typescript-eslint/prefer-readonly` |
| `prefer-readonly-parameter-types`: parâmetro tipado como `readonly`. Custoso e ruidoso com tipos de biblioteca. **Fora dos presets.** | [rules/prefer-readonly-parameter-types](https://typescript-eslint.io/rules/prefer-readonly-parameter-types) | E5 | LC opcional; padrão da casa: REV (`no-param-reassign` com `props: true` cobre o grosso) |
| `no-non-null-assertion` (`x!`). Preset STC. Por quê: só cala o compilador, sem checagem em execução. | [rules/no-non-null-assertion](https://typescript-eslint.io/rules/no-non-null-assertion); Google [#type-and-non-nullability-assertions](https://google.github.io/styleguide/tsguide.html#type-and-non-nullability-assertions) | E3 | LC `@typescript-eslint/no-non-null-assertion` |
| `consistent-type-assertions` com `{ assertionStyle: "as", objectLiteralTypeAssertions: "never" }`: proíbe `{...} as Pedido`. Preset stylistic (com outra opção). | [rules/consistent-type-assertions](https://typescript-eslint.io/rules/consistent-type-assertions); Google [#type-assertions-and-object-literals](https://google.github.io/styleguide/tsguide.html#type-assertions-and-object-literals) | E3 | LC `@typescript-eslint/consistent-type-assertions` |
| Afirmação dupla `x as unknown as T`: nenhuma regra dos presets pega. Google exige razão explícita. | Google [#double-assertions](https://google.github.io/styleguide/tsguide.html#double-assertions) | E3 | **AG** `afirmacao-dupla.yml` (seção 4) |
| `only-throw-error`, `prefer-promise-reject-errors`, `use-unknown-in-catch-callback-variable`: só se lança e rejeita `Error`; o erro do `.catch` é `unknown`. Preset STC. | [rules/only-throw-error](https://typescript-eslint.io/rules/only-throw-error); Google [#only-throw-errors](https://google.github.io/styleguide/tsguide.html#only-throw-errors) | E4 | LC as três |
| `no-unnecessary-condition`: condição sempre verdadeira ou falsa pelo tipo. Preset STC. | [rules/no-unnecessary-condition](https://typescript-eslint.io/rules/no-unnecessary-condition) | E6 (ramo morto), E2 | LC `@typescript-eslint/no-unnecessary-condition` |
| `strict-boolean-expressions`: só `boolean` em condição (sem `if (texto)`). **Fora dos presets.** | [rules/strict-boolean-expressions](https://typescript-eslint.io/rules/strict-boolean-expressions) | E1 (booleano lê como pergunta), E3 | LC opcional; decisão do grilling |
| `naming-convention`: caixa por tipo de identificador; `prefix` exige prefixo para booleanos (em PT: `["esta", "tem", "pode", "deve"]`, com o resto em PascalCase, como a própria doc explica). **Fora dos presets.** Não roda no oxlint. | [rules/naming-convention](https://typescript-eslint.io/rules/naming-convention) (seção `prefix / suffix`) | E1 | LC `@typescript-eslint/naming-convention` |
| `consistent-type-imports`: `import type` para o que é só tipo. **Fora dos presets.** Redundante com `verbatimModuleSyntax`. | [rules/consistent-type-imports](https://typescript-eslint.io/rules/consistent-type-imports); Google [#import-type](https://google.github.io/styleguide/tsguide.html#import-type) | P8 | LC um dos dois |
| `explicit-module-boundary-types`: função exportada declara tipos de entrada e retorno. **Fora dos presets.** Google deixa o retorno a critério do autor. | [rules/explicit-module-boundary-types](https://typescript-eslint.io/rules/explicit-module-boundary-types); Google [#return-types](https://google.github.io/styleguide/tsguide.html#return-types) | P1 (a interface diz o que quem chama precisa saber) | LC opcional; padrão: REV |
| `ban-ts-comment`: proíbe `@ts-ignore`/`@ts-nocheck`; `@ts-expect-error` só com descrição. Preset STC. Google proíbe os três. | [rules/ban-ts-comment](https://typescript-eslint.io/rules/ban-ts-comment); Google [#ts-ignore](https://google.github.io/styleguide/tsguide.html#ts-ignore) | **Nova** (não calar o verificador) | LC `@typescript-eslint/ban-ts-comment` |
| `eslint-disable` sem motivo: `@eslint-community/eslint-comments/require-description` (MIT, 4.8.1). | [eslint-plugin-eslint-comments](https://github.com/eslint-community/eslint-plugin-eslint-comments) **[confirmar texto da regra]** | **Nova** (idem) | LC `@eslint-community/eslint-comments/require-description` |
| `no-deprecated`: não usar API marcada `@deprecated`. Preset STC. | [rules/no-deprecated](https://typescript-eslint.io/rules/no-deprecated) | P8 | LC `@typescript-eslint/no-deprecated` |
| `no-magic-numbers` (extensão TS: `ignoreEnums`, `ignoreReadonlyClassProperties`, `ignoreNumericLiteralTypes`). | [rules/no-magic-numbers](https://typescript-eslint.io/rules/no-magic-numbers) | E8 | LC `@typescript-eslint/no-magic-numbers` (em vez da do core) |
| `return-await` (preset STC) e `promise-function-async` (fora): `return await` dentro de `try`; função que devolve promessa é `async`. | [rules/return-await](https://typescript-eslint.io/rules/return-await), [rules/promise-function-async](https://typescript-eslint.io/rules/promise-function-async) | E4 (o `catch` só pega a rejeição se houver `await`) | LC |

### 1.3 ESLint core (10.13.0)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| `max-depth` (padrão 4; a casa fala em 3 níveis → `["error", 3]`). | [rules/max-depth](https://eslint.org/docs/latest/rules/max-depth) | E2 | LC `max-depth` |
| `max-params` (padrão 3; já tem `countThis` para TS, então a extensão do typescript-eslint é dispensável). | [rules/max-params](https://eslint.org/docs/latest/rules/max-params) | E2 (Data Clumps) | LC `max-params` |
| `complexity` (padrão 20, variantes `classic`/`modified`). | [rules/complexity](https://eslint.org/docs/latest/rules/complexity) | E2 | LC `complexity` |
| `max-lines-per-function`. | [rules/max-lines-per-function](https://eslint.org/docs/latest/rules/max-lines-per-function) | E2 | LC |
| `no-else-return` e `no-nested-ternary`: saída cedo, sem ternário aninhado. | [rules/no-else-return](https://eslint.org/docs/latest/rules/no-else-return), [rules/no-nested-ternary](https://eslint.org/docs/latest/rules/no-nested-ternary) | E2 | LC |
| `prefer-const`; `no-param-reassign` com `{ props: true }` (o padrão `false` deixa mutar campos do parâmetro). | [rules/prefer-const](https://eslint.org/docs/latest/rules/prefer-const), [rules/no-param-reassign](https://eslint.org/docs/latest/rules/no-param-reassign) | E5 | LC (e registrar a opção `props: true` no `.md`) |
| `export let` proibido (exportação mutável): `no-restricted-syntax` com o seletor `ExportNamedDeclaration > VariableDeclaration[kind='let']`. | Google [#mutable-exports](https://google.github.io/styleguide/tsguide.html#mutable-exports); [rules/no-restricted-syntax](https://eslint.org/docs/latest/rules/no-restricted-syntax) | E5 | LC `no-restricted-syntax` (ou AG `pattern: export let $$$A`) |
| Só exportação nomeada (sem `export default`): `no-restricted-exports` com `restrictDefaultExports`. | Google [#exports](https://google.github.io/styleguide/tsguide.html#exports); [rules/no-restricted-exports](https://eslint.org/docs/latest/rules/no-restricted-exports) | P8 | LC `no-restricted-exports` (decisão do grilling: Next.js e React.lazy exigem default em certos arquivos) |
| `no-restricted-imports` com `patterns`: alternativa no lint ao `dominio-sem-infra.yml`. | [rules/no-restricted-imports](https://eslint.org/docs/latest/rules/no-restricted-imports) | D, P3, P8 | LC (se o projeto preferir lint a AG) |
| `no-console` no código de produção. | [rules/no-console](https://eslint.org/docs/latest/rules/no-console) | E4 ("catch que só loga"), P3 (log entra por port) | LC `no-console` |
| `eqeqeq`; `no-implicit-coercion`. | [rules/eqeqeq](https://eslint.org/docs/latest/rules/eqeqeq); Google [#implicit-coercion](https://google.github.io/styleguide/tsguide.html#implicit-coercion) | **Nova** (correção da linguagem) | LC |
| `no-warning-comments` (TODO/FIXME). | [rules/no-warning-comments](https://eslint.org/docs/latest/rules/no-warning-comments) | E6 | Já coberto pelo `todo-sem-issue`; não duplicar |

### 1.4 React (frontend)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| `react-hooks/rules-of-hooks`: hook só no topo de componente ou hook. | README de eslint-plugin-react-hooks 7.1.1; [react.dev/reference/rules/rules-of-hooks](https://react.dev/reference/rules/rules-of-hooks) | **Nova** (regra da biblioteca) | LC `react-hooks/rules-of-hooks` (ou preset `recommended`) |
| `react-hooks/exhaustive-deps`: o array de dependências lista tudo que o efeito lê. | idem | **Nova**; toca E5 | LC `react-hooks/exhaustive-deps` |
| `react-hooks/purity`, `immutability`, `set-state-in-render`, `set-state-in-effect`, `refs`, `globals`: render é cálculo puro; estado e props não se mutam. (Regras do React Compiler, no preset `recommended` da 7.x.) | README de eslint-plugin-react-hooks (*Custom Configuration*) | E5 (render é consulta, efeito é comando) | LC cada uma, ou `regras: ["recommended"]` |
| `react-hooks/static-components` / `react/no-unstable-nested-components`: não declarar componente dentro de componente. | README react-hooks; `eslint-plugin-react/docs/rules/no-unstable-nested-components.md` | **Nova** | LC |
| `react/jsx-key`, `react/no-array-index-key`: item de lista com `key` estável. | `eslint-plugin-react/docs/rules/jsx-key.md`, `no-array-index-key.md` | **Nova** | LC |
| Componente não chama `fetch` nem `localStorage` direto: busca por hook/adaptador. | Princípio P3 da casa; ver regra AG proposta | P3, D | **AG** `efeito-global-no-dominio.yml` apontado para `src/componentes/**` se o projeto decidir |
| Acessibilidade de JSX (`eslint-plugin-jsx-a11y` 6.10.2, MIT). | [jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y) **[confirmar regras recomendadas]** | **Nova** | LC preset `recommended` |

### 1.5 Projeto inteiro (comando)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Typecheck sem emitir. | [tsconfig#noEmit](https://www.typescriptlang.org/tsconfig/#noEmit) **[confirmar página]** | E3 (o tipo só garante se compila) | CMD `npx tsc --noEmit -p tsconfig.json` |
| Exportação, arquivo e dependência sem uso (código morto entre módulos, que `noUnusedLocals` não vê). | README de webpro-nl/knip ("finds and fixes unused dependencies, exports and files") | E6, P7 | CMD `npx knip` |
| Duplicação; imports entre pastas. | Pesquisa anterior (2026-10-23) | E7; P4, D | CMD jscpd; CMD depcruise (já no catálogo) |

## 2. Práticas que não cabem nos 22 princípios

Candidatas a um bloco de **princípios específicos de TypeScript** (ou a "Garantido pelo lint" sem ID
da casa). Nenhuma delas é julgamento: todas têm regra de máquina.

1. **Não calar o verificador.** `@ts-ignore`, `@ts-nocheck`, `eslint-disable` sem motivo, `as unknown
   as`. O mais próximo é P8, mas o dano é outro: o agente que não consegue passar no lint tende a
   silenciá-lo. Fontes: Google [#ts-ignore](https://google.github.io/styleguide/tsguide.html#ts-ignore);
   [ban-ts-comment](https://typescript-eslint.io/rules/ban-ts-comment). Esta é a mais forte das
   candidatas, e vale para qualquer linguagem (`# type: ignore`, `#[allow]`, `//nolint`).
2. **Sintaxe apagável / sem recurso de execução exclusivo do TS** (`enum`, `const enum`, `namespace`,
   *parameter properties*). Fontes: [erasableSyntaxOnly](https://www.typescriptlang.org/tsconfig/#erasableSyntaxOnly);
   Google [#enums](https://google.github.io/styleguide/tsguide.html#enums) (proíbe `const enum`).
   Conflito a decidir: E8 sugere "tipo enumerado"; com `erasableSyntaxOnly`, isso vira união de
   literais (`type Status = "pago" | "pendente"`) ou objeto `as const`.
3. **Arquivo transpilável sozinho** (`isolatedModules`, `verbatimModuleSyntax`). Fonte:
   [isolatedModules](https://www.typescriptlang.org/tsconfig/#isolatedModules). É exigência do
   ferramental (esbuild, SWC, Vite), não de design.
4. **Regras da biblioteca de UI**: regras dos hooks, `key` estável, componente não aninhado. Fonte:
   README de eslint-plugin-react-hooks; eslint-plugin-react. São contratos do React, não da casa.
5. **Acessibilidade** no frontend (jsx-a11y). Nenhum dos 22 fala de quem usa a tela.
6. **Igualdade e coerção explícitas** (`eqeqeq`, `no-implicit-coercion`). Fonte: Google
   [#equality-checks](https://google.github.io/styleguide/tsguide.html#equality-checks),
   [#implicit-coercion](https://google.github.io/styleguide/tsguide.html#implicit-coercion).
   Correção da linguagem, sem princípio de design por trás.

**Tensões com as fontes, para o grilling** (não são lacunas, são escolhas):

- **E4 vs. Google**: E4 aceita erro esperado como *resultado*; o Google prefere exceção
  ("Prefer throwing exceptions over ad-hoc error-handling approaches",
  [#exception-handling](https://google.github.io/styleguide/tsguide.html#exception-handling)).
  As duas formas cabem em E4; o projeto escolhe uma (P8).
- **`catch-vazio.yml` vs. Google**: o Google aceita `catch` vazio **com comentário que justifica**
  ([#empty-catch-blocks](https://google.github.io/styleguide/tsguide.html#empty-catch-blocks)); o
  modelo da casa reprova sempre. Se o projeto quiser a exceção, a regra AG ganha
  `not: { has: { kind: comment } }` dentro do bloco.
- **`switch` com `default` (Google, [#switch-statements](https://google.github.io/styleguide/tsguide.html#switch-statements))
  vs. exaustividade**: com `default`, o `switch-exhaustiveness-check` não acusa o membro novo, a
  menos que `considerDefaultExhaustiveForUnions` fique `false` (padrão) e
  `allowDefaultCaseForExhaustiveSwitch` também `false`. Recomendação: sem `default` em `switch` sobre
  união; o compilador e a regra garantem a cobertura (O).
- **E2 em limites**: ESLint usa 4 para `max-depth` e 3 para `max-params`; o texto de E2 fala em
  "três ou mais níveis". O `.md` do projeto precisa do número.

## 3. Ferramentas

Versões do `latest` em registry.npmjs.org/<pacote>/latest, consultado em 2026-10-23.

| Ferramenta | Versão | Licença | Cobra? | Link |
|---|---|---|---|---|
| typescript | 7.0.2 (6.0.x para typescript-eslint) | Apache-2.0 | Não | github.com/microsoft/TypeScript |
| typescript-eslint (+ `@typescript-eslint/eslint-plugin`) | 8.72.0 | MIT | Não | github.com/typescript-eslint/typescript-eslint |
| eslint | 10.13.0 | MIT | Não | github.com/eslint/eslint |
| eslint-plugin-react-hooks | 7.1.1 | MIT | Não | github.com/facebook/react |
| eslint-plugin-react | 7.37.5 | MIT | Não | github.com/jsx-eslint/eslint-plugin-react |
| eslint-plugin-jsx-a11y | 6.10.2 | MIT | Não | github.com/jsx-eslint/eslint-plugin-jsx-a11y |
| @eslint-community/eslint-plugin-eslint-comments | 4.8.1 | MIT | Não | github.com/eslint-community/eslint-plugin-eslint-comments |
| oxlint | 1.89.0 | MIT | Não. Dependências opcionais `oxlint-tsgolint` e `vite-plus` (MIT, 1.4.0) | github.com/oxc-project/oxc |
| oxlint-tsgolint (lint com tipos sobre TS 7) | 7.0.2005 | MIT | Não | github.com/oxc-project/tsgolint |
| @biomejs/biome | 2.6.2 | MIT OR Apache-2.0 | Não. Vende **suporte** empresarial (README: "Biome offers enterprise support"); a ferramenta é a mesma | github.com/biomejs/biome |
| knip | 6.42.1 | ISC | Não | github.com/webpro-nl/knip |
| @ast-grep/cli | 0.51.0 | MIT | Não | github.com/ast-grep/ast-grep |
| eslint-plugin-sonarjs | 4.2.2 | **LGPL-3.0-only** | O plugin não; a SonarSource vende SonarQube/SonarCloud | github.com/SonarSource/SonarJS |

**Biome e oxlint como alternativa ao ESLint:**

- **oxlint** tem `max-depth`, `max-params`, `complexity`, `max-lines-per-function`, `no-magic-numbers`,
  `react/rules-of-hooks` e `react/exhaustive-deps` (schema do pacote 1.89.0) e, com `--type-aware`,
  quase todas as regras com tipo do typescript-eslint. Falta `naming-convention` (E1). Não tem as
  regras do React Compiler (`purity`, `set-state-in-effect`...) **[confirmar]**.
- **Biome** 2.6.2 tem `noExplicitAny`, `noNonNullAssertion`, `useImportType`, `noMagicNumbers`,
  `useMaxParams`, `noExcessiveCognitiveComplexity`, `noTsIgnore`, `noConstEnum`, `noDefaultExport`,
  `useReadonlyClassProperties`, `useHookAtTopLevel`, `useExhaustiveDependencies`; mas
  **`noFloatingPromises`, `noMisusedPromises` e `useExhaustiveSwitchCases` estão no grupo `nursery`**
  (instáveis), e o Biome usa inferência de tipo própria, não o compilador **[confirmar cobertura]**.
  Não tem equivalente direto de `max-depth` (os mais próximos são `noExcessiveCognitiveComplexity`,
  `noExcessiveLinesPerFunction`, `noNestedTernary` e `noExcessiveNestedCallbacks`).
- **Recomendação:** ESLint + typescript-eslint continua o padrão (cobertura e documentação), com
  `typescript@~6.0` enquanto o peer não aceitar 7. Projeto que já está em TS 7 ou quer velocidade:
  oxlint `--type-aware`. Biome só quando o projeto aceitar as regras de promessa em `nursery`.
- **eslint-plugin-sonarjs** fica fora por precaução: LGPL é usável (como no Opengrep), mas é o
  funil de um produto pago e nada aqui depende dele.

## 4. Regras de ast-grep propostas como modelo

Quatro regras, cada uma com documento `TypeScript` e `Tsx` (o padrão dos modelos atuais). Todas
cobrem algo que o lint dos presets **não** pega. Os arquivos estão em `.claude/watchdogs/modelos/ast-grep/` do `modelo-de-projeto`.

**Teste:** ast-grep 0.51.0 numa pasta de teste; marcadores trocados por `src/dominio`;
`ast-grep scan --rule <arquivo> .` numa pasta de teste
(arquivos `src/dominio/pega.ts`, `src/dominio/passa.ts`, `src/adapters/http.ts`, `src/ui/Tela.tsx`,
`src/ui/TelaOk.tsx`).

| Regra | Pegou (esperado) | Falso positivo em `passa.ts`, `TelaOk.tsx`, `adapters/` |
|---|---|---|
| `efeito-global-no-dominio` | 6 de 6 em `pega.ts`; `adapters/` ignorado por `files` | 0 (inclusive `new Date(agora)`) |
| `promessa-engolida` | 8 de 8 em `pega.ts` + 1 em `Tela.tsx` | 0 |
| `erro-generico-no-dominio` | 3 de 3 em `pega.ts` | 0 (classe própria passa; `Error` em `adapters/` passa) |
| `afirmacao-dupla` | 3 de 3 em `pega.ts` + 1 em `Tela.tsx` | 0 (`as Dinheiro` simples passa) |

Na primeira rodada, duas regras falharam e foram corrigidas: `new Date` sem parênteses também casava
com `new Date(x)` (trocado por `kind: new_expression` sem argumento) e `e => {}` sem parênteses não
casava com `($E) => {}` (padrão próprio acrescentado). Uma pasta à parte confirmou `new Date()` e
`new Date` pegos, `new Date(x)`, `new Date(2020, 1)` e `new Date("2020-01-01")` não.

### 4.1 `efeito-global-no-dominio.yml` (P3, D, P5)

Por quê: `dominio-sem-infra.yml` olha **imports**. `fetch`, `localStorage`, `Date.now()` e
`Math.random()` são globais, não têm import e escapam. P5 diz que relógio e aleatório são seam.

```yaml
# Princípios P3, D e P5: o domínio não chama rede, armazenamento, relógio nem aleatório direto. Esses
# efeitos são globais (não têm import), então o dominio-sem-infra.yml não os vê. Copie para
# docs/regras-de-codigo/ast-grep/ e troque <PASTA_DO_DOMINIO> (ex.: src/dominio). Relógio e aleatório
# entram como port (P5: mock só no que não é nosso).
id: efeito-global-no-dominio
language: TypeScript
message: "P3/P5: o domínio usa rede, armazenamento, relógio ou aleatório direto. Receba pela port."
severity: error
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  any:
    - pattern: fetch($$$)
    - pattern: window.fetch($$$)
    - pattern: globalThis.fetch($$$)
    - pattern: new XMLHttpRequest($$$)
    - pattern: new WebSocket($$$)
    - pattern: localStorage.$M($$$)
    - pattern: sessionStorage.$M($$$)
    - pattern: Date.now()
    - kind: new_expression          # new Date() ou new Date, sem argumento: lê o relógio
      has: { field: constructor, regex: ^Date$ }
      not: { has: { field: arguments, has: { regex: "^[^()]" } } }
    - pattern: Math.random()
    - pattern: crypto.randomUUID()
    - pattern: performance.now()

---
# documento igual com id: efeito-global-no-dominio-tsx e language: Tsx
```

Pega:

```ts
// src/dominio/conta.ts
export async function fecharConta(pedido: Pedido) {
  const resposta = await fetch("https://api/cobrar");
  const fechadaEm = new Date();
}
```

Passa:

```ts
// src/dominio/conta.ts
export async function fecharConta(pedido: Pedido, cobranca: Cobranca, relogio: Relogio) {
  const fechadaEm = new Date(relogio.agora());
  return cobranca.cobrar(pedido);
}
```

### 4.2 `promessa-engolida.yml` (E4)

Por quê: `catch-vazio.yml` só vê `try/catch`. `.catch(() => {})` é a forma mais comum de engolir erro
em código assíncrono, e `no-floating-promises` **aceita** a promessa assim (ela está "tratada").

```yaml
# Princípio E4: erro nunca é engolido, também em promessa. O catch-vazio.yml pega try/catch; este pega
# o .catch() que descarta a rejeição ou só loga e segue. Copie para docs/regras-de-codigo/ast-grep/.
id: promessa-engolida
language: TypeScript
message: "E4: a rejeição da promessa é engolida. Trate o caso esperado ou deixe o erro subir até a borda."
severity: error
rule:
  any:
    - pattern: $P.catch(() => {})
    - pattern: $P.catch(($E) => {})
    - pattern: $P.catch($E => {})
    - pattern: $P.catch(() => undefined)
    - pattern: $P.catch(() => null)
    - pattern: $P.catch(console.$LOG)
    - pattern: $P.catch(($E) => console.$LOG($$$))
    - pattern: $P.catch($E => console.$LOG($$$))
    - pattern: $P.catch(($E) => { console.$LOG($$$) })
    - pattern: $P.then($OK, () => {})

---
# documento igual com id: promessa-engolida-tsx e language: Tsx
```

Pega: `enviarPedido(pedido).catch(() => {});` e `useEffect(() => { carregar().catch(console.error); }, []);`

Passa: `enviarPedido(pedido).catch((erro) => { registro.erro("falha ao enviar", erro); throw erro; });`
e `carregar().catch(mostrarErro);`

### 4.3 `erro-generico-no-dominio.yml` (E4)

Por quê: E4 diz que erro esperado tem tipo e nome próprios e lista `throw new Error("erro")` no
*No diff*. Nenhum lint pega isso (`only-throw-error` aceita `Error`). Fica restrito ao domínio: na
borda e nos adapters, `Error` cru é aceitável.

```yaml
# Princípio E4: erro esperado é parte da interface, com tipo e nome próprios. No domínio, `Error` cru
# esconde qual caso aconteceu e obriga quem chama a comparar texto. Copie para
# docs/regras-de-codigo/ast-grep/ e troque <PASTA_DO_DOMINIO>. Se o projeto decidiu erro esperado como
# resultado (não exceção), a regra continua valendo para o que sobra como exceção.
id: erro-generico-no-dominio
language: TypeScript
message: "E4: erro do domínio sem tipo próprio. Crie a classe do caso (class CupomVencido extends Error) ou devolva um resultado com tipo."
severity: error
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  any:
    - pattern: throw new Error($$$)
    - pattern: throw Error($$$)
    - pattern: Promise.reject(new Error($$$))

---
# documento igual com id: erro-generico-no-dominio-tsx e language: Tsx
```

Pega: `if (pedido.itens.length === 0) throw new Error("pedido vazio");`

Passa: `export class PedidoVazio extends Error {}` … `throw new PedidoVazio(pedido.id);`

### 4.4 `afirmacao-dupla.yml` (E3)

Por quê: E3 manda construir o tipo válido na borda. `x as unknown as T` fabrica o tipo sem validar e
nenhuma regra dos presets pega a forma dupla (`no-explicit-any` só pega `as any as T` pela metade).
O Google exige razão explícita para ela ([#double-assertions](https://google.github.io/styleguide/tsguide.html#double-assertions)).

```yaml
# Princípio E3: o tipo do domínio se constrói válido, na borda. `x as unknown as T` desliga o
# compilador e fabrica o tipo sem validar; nenhuma regra dos configs do typescript-eslint pega a forma
# dupla. Copie para docs/regras-de-codigo/ast-grep/. Use `ignores` para testes, se o projeto aceitar lá.
id: afirmacao-dupla
language: TypeScript
message: "E3: afirmação de tipo dupla fabrica o tipo sem validar. Converta na borda com a função que valida."
severity: error
rule:
  any:
    - pattern: $X as unknown as $T
    - pattern: $X as any as $T
    - pattern: <$T><unknown>$X

---
id: afirmacao-dupla-tsx
language: Tsx
message: "E3: afirmação de tipo dupla fabrica o tipo sem validar. Converta na borda com a função que valida."
severity: error
rule:
  any:
    - pattern: $X as unknown as $T
    - pattern: $X as any as $T
```

(O documento `Tsx` não tem a forma `<T>x`, que em TSX é JSX.)

Pega: `const valor = corpo.valor as unknown as Dinheiro;`

Passa: `const valor = Dinheiro.deCentavos(corpo.valorEmCentavos);`

**Fora das quatro, considerada e descartada:** "status comparado por texto" (E8). Com união de
literais (`status: "pago" | "pendente"`), `pedido.status === "pago"` é verificado pelo compilador e é
o jeito idiomático; a regra daria mais falso positivo que acerto.

## 5. O que ficou por confirmar

- **[confirmar]** Quando o typescript-eslint passa a aceitar TypeScript 7 (hoje o peer é `<6.1.0`).
- **[confirmar]** Texto da regra `require-description` do `@eslint-community/eslint-plugin-eslint-comments`
  (só confirmamos pacote, versão e licença no npm).
- **[confirmar]** Regras do preset `recommended` do `eslint-plugin-jsx-a11y`.
- **[confirmar]** Se o oxlint tem as regras do React Compiler (`purity`, `set-state-in-effect`).
- **[confirmar]** Cobertura real das regras com tipo do Biome (inferência própria; regras de promessa
  em `nursery`).
- **[confirmar]** Página `tsconfig#noEmit` (não baixamos o arquivo dela; o uso `tsc --noEmit` é padrão).
- **[confirmar viabilidade]** `lint-cobre` ler `eslint --print-config` em vez do texto do arquivo
  (proposta para o motor).
