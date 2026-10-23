# Validador de regras de código

Confere por máquina o que as regras de código decidiram (ADR 0010 do método). O motor é do método e
fica protegido em `.claude/validador/`; o projeto decide **o que** validar em
`docs/regras-de-codigo.json`. Princípio de julgamento fica com o revisor.

```bash
node .claude/validador/validar.mjs --base origin/main  # só o que mudou desde a base (CI, implementador, revisor)
node .claude/validador/validar.mjs                     # o repositório inteiro (auditoria)
node .claude/validador/validar.mjs --so-cobertura      # só a tabela dos 22 contra o .json (fecha o mapa)
```

Sai com 0 aprovado, 1 violação, 2 configuração inválida; a última linha é sempre
`VALIDADOR: APROVADO` ou `VALIDADOR: REPROVADO`. Com `--base`, cada checagem lê só os arquivos
criados ou mudados desde a base, commitados ou não; `cobertura`, `lint-cobre` e `mensagem-de-commit`
rodam inteiras. `--completo` força o repositório inteiro.

## `docs/regras-de-codigo.json`

Nasce do modelo `.claude/validador/modelos/regras-de-codigo.json` no ticket que escreve
`docs/regras-de-codigo.md`. Cada regra do `.md` tem uma entrada, com o mesmo `id`:

```json
{
  "versao": 1,
  "projeto": { "codigo": ["src/**"], "ignorar": ["dist/**"] },
  "regras": [
    { "id": "R6", "principio": "D", "texto": "O domínio não importa infraestrutura",
      "verificacao": { "tipo": "ast-grep", "regras": "docs/regras-de-codigo/ast-grep/dominio-sem-infra.yml" } }
  ]
}
```

A tabela de cobertura do `.md` é a dona das decisões; o `.json` diz como cada regra se verifica. A
checagem `cobertura` roda sempre e confere os dois juntos: os 22 princípios com decisão válida, cada
princípio adotado ou adaptado citando um `R<n>` que existe no `.json` com o mesmo princípio, e nenhuma
regra do `.json` sem linha na tabela.

## Catálogo

| `tipo` | Parâmetros | Confere |
|---|---|---|
| `termo-evitado` | `glossario` (padrão `GLOSSARY.md`), `ignorarTermos` | Nome declarado ou nome de arquivo com um termo *Evitar* do glossário |
| `todo-sem-issue` | `marcadores`, `issue` (regex) | `TODO`/`FIXME` sem a issue |
| `teste-de-contrato-por-port` | `ports` (glob), `contrato` (`{dir}`, `{nome}`), `ignorar` | Port sem o teste de contrato |
| `lint-cobre` | `config` (arquivo ou lista), `regras` | A configuração do lint tem as regras delegadas a ele |
| `mensagem-de-commit` | `padrao` (regex), `base` | Título dos commits desde a base (padrão `origin/main`) |
| `ast-grep` | `regras` (arquivo ou lista), `alvos`, `binario` | Padrões de código (TypeScript, JavaScript, Python, Go, Java, Rust e outras) |
| `comando` | `executar` (aceita `{arquivos}`) | Qualquer ferramenta: saída diferente de 0 é violação |
| `revisor` | | Nada: marca a regra como julgamento do revisor |

Todos aceitam `arquivos` e `ignorar` (globs) quando leem código; o padrão vem de `projeto`. Em
`comando`, `{arquivos}` vira a lista dos arquivos lidos (só os alterados, com `--base`); sem ele, a
ferramenta roda no projeto inteiro.

**Receitas de `comando`** (ferramentas MIT, sem custo; pesquisa em `desenvolvimento/research/` do
Stack-Labs): duplicação (E7) com `npx jscpd --min-lines 8 --threshold 0 --exitCode 1 {arquivos}`; imports
entre pastas em JS/TS (P4, D) com `npx depcruise --config .dependency-cruiser.cjs src`; lint e
typecheck do projeto pelo comando deles.

## Refinar para o projeto

1. **Copie o modelo** do `.json` e, para cada regra, escolha o `tipo` que a linha *No validador* do
   princípio indica ([principios-de-codigo.md](principios-de-codigo.md)).
2. **Preencha os parâmetros** com o que o grilling decidiu: pastas, pacotes, regex do commit.
3. **Regra de padrão** (`ast-grep`): copie o modelo de `.claude/validador/modelos/ast-grep/` para
   `docs/regras-de-codigo/ast-grep/`, troque os marcadores `<...>` e apague os documentos das
   linguagens que o projeto não usa. Linguagem que falta: copie um documento e troque `language` e o
   `kind` (`ast-grep run --debug-query=cst -p '<trecho>' -l <linguagem>` mostra os nomes). Cada regra
   nova ganha um exemplo que deve pegar e um que deve passar, rodados uma vez com
   `ast-grep scan --rule <arquivo> <pasta-de-exemplo>`.
4. **Nada no catálogo serve?** Use `comando` com a ferramenta da stack. Checagem que valeria para
   outros projetos vira proposta de PR no Stack-Labs, para entrar no motor.

Pronto quando `node .claude/validador/validar.mjs` termina em `VALIDADOR: APROVADO` no código atual,
ou as violações listadas viraram tickets.

## Onde roda

- **CI do projeto**, configurado na primeira fatia da primeira fase, junto com o lint:
  `node .claude/validador/validar.mjs --base origin/${{ github.base_ref }}`, com o ast-grep instalado
  no passo anterior quando o `.json` usa `ast-grep` (`npm i -g @ast-grep/cli`).
- **Implementador**, antes do commit; **revisor** da orquestração, como violação dura.
- **Fechamento do mapa** de Regras de código, com `--so-cobertura`.
