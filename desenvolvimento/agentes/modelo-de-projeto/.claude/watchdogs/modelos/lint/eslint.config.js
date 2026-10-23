// Configuração de lint de REFERÊNCIA para TypeScript (Node e React). Adoção por etapas: o projeto
// copia para a raiz só as regras que o grilling de Regras de código decidiu, não o arquivo inteiro.
// Cada regra traz o princípio que concretiza (E1-E8, P1-P9, S O L I D, TS1-TS5).
//
// Versões fixas (package.json, devDependencies, sem ^):
//   typescript 6.0.3 · typescript-eslint 8.72.0 · eslint 10.13.0 · @eslint/js 10.0.1
//   @eslint-community/eslint-plugin-eslint-comments 4.8.1 · eslint-plugin-react-hooks 7.1.1
//
// Por que TypeScript 6.0 e não 7: o typescript-eslint 8.72.0 declara peer `typescript >=4.8.4 <6.1.0`.
// As regras com tipo (no-floating-promises, switch-exhaustiveness-check, no-unsafe-*) não rodam
// oficialmente no TypeScript 7 pelo ESLint. Projeto que já está no 7: oxlint 1.89.0 com
// `oxlint --type-aware` (oxlint-tsgolint 7.0.2005), que implementa 60 das 62 regras com tipo do
// typescript-eslint; faltam naming-convention e prefer-destructuring.
//
// O `lint-cobre` procura o NOME da regra no texto deste arquivo. Regra que só vem pelo preset não
// aparece no texto; por isso as que o docs/regras-de-codigo.json delega estão escritas abaixo,
// mesmo quando o preset já as liga.
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import comentarios from "@eslint-community/eslint-plugin-eslint-comments/configs";
import reactHooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  { ignores: ["dist/**", "build/**", "coverage/**"] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked, // E3, E4: any, promessas, throw, condição inútil
  ...tseslint.configs.stylisticTypeChecked,
  comentarios.recommended,
  {
    files: ["**/*.ts", "**/*.tsx"],
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      // E1: booleano lê como pergunta. Opcional: decisão do grilling (não roda no oxlint).
      // "@typescript-eslint/naming-convention": ["error",
      //   { selector: "variable", types: ["boolean"], format: ["PascalCase"], prefix: ["esta", "tem", "pode", "deve"] }],

      // E2: aninhamento raso, poucos parâmetros, função simples, saída cedo.
      "max-depth": ["error", 3], // E2
      "max-params": ["error", 3], // E2
      complexity: ["error", 10], // E2
      "no-else-return": ["error", { allowElseIf: false }], // E2
      "no-nested-ternary": "error", // E2
      // E2, opcional: não crie função pública só para cumprir o limite.
      // "max-lines-per-function": ["error", { max: 40, skipBlankLines: true, skipComments: true }],

      // E3: o tipo do domínio se constrói válido, na borda.
      "@typescript-eslint/no-explicit-any": "error", // E3
      "@typescript-eslint/no-unsafe-assignment": "error", // E3
      "@typescript-eslint/no-unsafe-argument": "error", // E3
      "@typescript-eslint/no-unsafe-member-access": "error", // E3
      "@typescript-eslint/no-unsafe-call": "error", // E3
      "@typescript-eslint/no-unsafe-return": "error", // E3
      "@typescript-eslint/no-non-null-assertion": "error", // E3
      "@typescript-eslint/consistent-type-assertions": ["error",
        { assertionStyle: "as", objectLiteralTypeAssertions: "never" }], // E3
      // E3, opcional: só boolean em condição (sem `if (texto)`).
      // "@typescript-eslint/strict-boolean-expressions": "error",

      // E4: erro nunca é engolido; promessa não fica solta; não calar o verificador.
      "@typescript-eslint/no-floating-promises": "error", // E4
      "@typescript-eslint/no-misused-promises": "error", // E4
      "@typescript-eslint/only-throw-error": "error", // E4
      "@typescript-eslint/prefer-promise-reject-errors": "error", // E4
      "@typescript-eslint/use-unknown-in-catch-callback-variable": "error", // E4
      "@typescript-eslint/return-await": ["error", "in-try-catch"], // E4
      "@typescript-eslint/promise-function-async": "error", // E4
      "@typescript-eslint/ban-ts-comment": ["error", { "ts-expect-error": "allow-with-description" }], // E4
      "@eslint-community/eslint-comments/require-description": "error", // E4
      "no-console": "error", // E4, P3: o registro entra por port

      // E5: dado não se muta por fora.
      "prefer-const": "error", // E5
      "no-param-reassign": ["error", { props: true }], // E5
      "@typescript-eslint/prefer-readonly": "error", // E5
      "no-restricted-syntax": ["error", {
        selector: "ExportNamedDeclaration > VariableDeclaration[kind='let']",
        message: "E5: exportação mutável. Exporte const ou uma função.",
      }], // E5

      // E6 e P7: ramo morto sai.
      "@typescript-eslint/no-unnecessary-condition": "error", // E6

      // E8: número com significado tem nome.
      "no-magic-numbers": "off",
      "@typescript-eslint/no-magic-numbers": ["error", {
        ignore: [-1, 0, 1], ignoreEnums: true, ignoreReadonlyClassProperties: true, ignoreNumericLiteralTypes: true,
      }], // E8

      // O: variante nova acusa cada switch que precisa dela (sem default em switch sobre união).
      "@typescript-eslint/switch-exhaustiveness-check": ["error", {
        considerDefaultExhaustiveForUnions: false, allowDefaultCaseForExhaustiveSwitch: false,
        requireDefaultForNonUnion: true,
      }], // O, E3

      // P8: um jeito de cada coisa.
      "@typescript-eslint/consistent-type-imports": "error", // P8
      "@typescript-eslint/no-deprecated": "error", // P8
      // P8, opcional: só exportação nomeada (Next.js e React.lazy exigem default em certos arquivos).
      // "no-restricted-exports": ["error", { restrictDefaultExports: { direct: true } }],

      // TS5: igualdade e coerção explícitas.
      eqeqeq: ["error", "always"], // TS5
      "no-implicit-coercion": "error", // TS5
    },
  },
  {
    // TS3: contratos do React. Só para frontend; projeto Node apaga este bloco e a dependência.
    files: ["**/*.tsx"],
    plugins: { "react-hooks": reactHooks },
    rules: {
      ...reactHooks.configs.recommended.rules, // TS3: regras do React Compiler (purity, refs...)
      "react-hooks/rules-of-hooks": "error", // TS3
      "react-hooks/exhaustive-deps": "error", // TS3
      "react-hooks/static-components": "error", // TS3: componente não se declara dentro de componente
    },
  },
  {
    // Arquivos .js de configuração (este inclusive) ficam fora do tsconfig: sem regras com tipo.
    files: ["**/*.js", "**/*.mjs", "**/*.cjs"],
    ...tseslint.configs.disableTypeChecked,
  },
  {
    // Testes: o desenho do teste fica com o revisor; afrouxe só o que o grilling decidiu.
    files: ["**/*.test.ts", "**/*.test.tsx"],
    rules: { "@typescript-eslint/no-magic-numbers": "off" },
  },
);
