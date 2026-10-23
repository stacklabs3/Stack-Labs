# Validador de regras de código

Confere por máquina o que as regras de código decidiram (ADR 0010 do método). O motor é do método e
fica protegido em `.claude/validador/`; o projeto decide **o que** validar em
`docs/regras-de-codigo.json`. Princípio de julgamento fica com o revisor.

```bash
node .claude/validador/validar.mjs                 # tudo; sai 0 aprovado, 1 violação, 2 configuração inválida
node .claude/validador/validar.mjs --so-cobertura  # só a tabela dos 22 contra o .json (fecha o mapa)
node .claude/validador/validar.mjs --base origin/main --json
```

A última linha é sempre `VALIDADOR: APROVADO` ou `VALIDADOR: REPROVADO`.

## `docs/regras-de-codigo.json`

Nasce do modelo `.claude/validador/modelos/regras-de-codigo.json` no ticket que escreve
`docs/regras-de-codigo.md`. Cada regra do `.md` tem uma entrada, com o mesmo `id`:

```json
{
  "versao": 1,
  "projeto": { "codigo": ["src/**"], "ignorar": ["dist/**"] },
  "regras": [
    { "id": "R6", "principio": "D", "texto": "O domínio não importa infraestrutura",
      "verificacao": { "tipo": "opengrep", "regras": "docs/regras-de-codigo/opengrep/dominio-sem-infra.yml" } }
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
| `opengrep` | `regras` (arquivo ou lista), `alvos`, `binario` | Padrões de código em 30+ linguagens |
| `comando` | `executar` | Qualquer ferramenta: saída diferente de 0 é violação |
| `revisor` | | Nada: marca a regra como julgamento do revisor |

Todos aceitam `arquivos` e `ignorar` (globs) quando leem código; o padrão vem de `projeto`.

**Receitas de `comando`** (ferramentas MIT, sem custo; pesquisa em `desenvolvimento/research/` do
Stack-Labs): duplicação (E7) com `npx jscpd --min-lines 8 --threshold 0 --exitCode 1 src`; imports
entre pastas em JS/TS (P4, D) com `npx depcruise --config .dependency-cruiser.cjs src`; lint e
typecheck do projeto pelo comando deles.

## Refinar para o projeto

1. **Copie o modelo** do `.json` e, para cada regra, escolha o `tipo` que a linha *No validador* do
   princípio indica ([principios-de-codigo.md](principios-de-codigo.md)).
2. **Preencha os parâmetros** com o que o grilling decidiu: pastas, pacotes, regex do commit.
3. **Regra de padrão** (`opengrep`): copie o modelo de `.claude/validador/modelos/opengrep/` para
   `docs/regras-de-codigo/opengrep/`, troque os marcadores `<...>` e acrescente a linguagem do projeto
   quando o modelo não a tiver. Cada regra nova do Opengrep ganha um exemplo que deve pegar e um que
   deve passar, rodados uma vez com `opengrep scan --config <arquivo> <pasta-de-exemplo>`.
4. **Nada no catálogo serve?** Use `comando` com a ferramenta da stack. Checagem que valeria para
   outros projetos vira proposta de PR no Stack-Labs, para entrar no motor.

Pronto quando `node .claude/validador/validar.mjs` termina em `VALIDADOR: APROVADO` no código atual,
ou as violações listadas viraram tickets.

## Onde roda

- **CI do projeto**, configurado na primeira fatia da primeira fase, junto com o lint:
  `node .claude/validador/validar.mjs --base origin/${{ github.base_ref }}`, com o Opengrep instalado
  no passo anterior quando o `.json` usa `opengrep`.
- **Implementador**, antes do commit; **revisor** da orquestração, como violação dura.
- **Fechamento do mapa** de Regras de código, com `--so-cobertura`.
