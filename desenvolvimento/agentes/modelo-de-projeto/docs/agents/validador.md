# Validador de regras de código

Confere por máquina o que as regras de código decidiram (ADR 0010 do método). O motor é do método e
fica protegido em `.claude/watchdogs/` (o mesmo binário Rust dos watchdogs); o projeto decide **o que** validar em
`docs/regras-de-codigo.json`. Princípio de julgamento fica com o revisor.

```bash
sh .claude/watchdogs/rodar validar --base origin/main  # só o que mudou desde a base (CI, implementador, revisor)
sh .claude/watchdogs/rodar validar                     # o repositório inteiro (auditoria)
sh .claude/watchdogs/rodar validar --so-cobertura      # só a tabela dos 22 contra o .json (fecha o mapa)
```

Sai com 0 aprovado, 1 violação, 2 configuração inválida; a última linha é sempre
`VALIDADOR: APROVADO` ou `VALIDADOR: REPROVADO`. Com `--base`, cada checagem lê só os arquivos
criados ou mudados desde a base, commitados ou não; `cobertura`, `lint-cobre` e `mensagem-de-commit`
rodam inteiras. `--completo` força o repositório inteiro.

## `docs/regras-de-codigo.json`

Nasce do modelo `.claude/watchdogs/modelos/regras-de-codigo.json` no ticket que escreve
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
regra do `.json` sem linha na tabela. Regra que nasce de prática da linguagem leva o ID dela em
`principio` (`TS<n>`, `JV<n>`, `RS<n>`) e é citada na tabela *Práticas da linguagem* do `.md`, fora
dos 22.

## Catálogo

| `tipo` | Parâmetros | Confere |
|---|---|---|
| `termo-evitado` | `glossario` (padrão `GLOSSARY.md`), `ignorarTermos` | Nome declarado ou nome de arquivo com um termo *Evitar* do glossário |
| `todo-sem-issue` | `marcadores`, `issue` (regex) | `TODO`/`FIXME` sem a issue |
| `teste-de-contrato-por-port` | `ports` (glob), `contrato` (`{dir}`, `{dir_de_teste}`, `{nome}`), `ignorar` | Port sem o teste de contrato; `{dir_de_teste}` é o `{dir}` com `src/main/` trocado por `src/test/` (Maven e Gradle) |
| `lint-cobre` | `config` (arquivo ou lista), `regras` | A configuração do lint tem as regras delegadas a ele |
| `mensagem-de-commit` | `padrao` (regex), `base` | Título dos commits desde a base (padrão `origin/main`) |
| `ast-grep` | `regras` (arquivo ou lista), `alvos`, `binario` | Padrões de código (TypeScript, JavaScript, Python, Go, Java, Rust e outras) |
| `comando` | `executar` (aceita `{arquivos}`) | Qualquer ferramenta: saída diferente de 0 é violação |
| `revisor` | | Nada: marca a regra como julgamento do revisor |

Todos aceitam `arquivos` e `ignorar` (globs) quando leem código; o padrão vem de `projeto`. Em
`comando`, `{arquivos}` vira a lista dos arquivos lidos (só os alterados, com `--base`); sem ele, a
ferramenta roda no projeto inteiro.

**Receitas de `comando`**, sempre com a versão fixa: ferramenta que muda uma flag sem avisar faz a
regra reprovar tudo (pesquisas em `desenvolvimento/research/` do Stack-Labs):

| Para | Comando |
|---|---|
| Duplicação (E7), qualquer linguagem | `npx jscpd@5.4.2 --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` |
| Imports entre pastas em JS/TS (P4, D) | `npx -p dependency-cruiser@18.7.0 depcruise --config .dependency-cruiser.cjs src` |
| Lint e typecheck em TypeScript | `npx eslint --max-warnings 0 {arquivos}` e `npx tsc --noEmit -p tsconfig.json` |
| Lint em Java | o build com Error Prone (`mvn -B compile`), `mvn -B pmd:check`, `mvn -B checkstyle:check` |
| Arquitetura em Java (P3, P4, D) | `mvn -B test -Dtest='*ArquiteturaTest'` (ArchUnit) |
| Lint e formatador em Rust | `cargo clippy --all-targets --locked -- -D warnings` e `cargo fmt --all -- --check` |
| Dependências em Rust (P7, P8, RS3) | `cargo machete` e `cargo deny check` |

As regras de cada linguagem e o porquê estão no arquivo dela, em
[principios-de-codigo.md](principios-de-codigo.md).

## Refinar para o projeto

1. **Copie o modelo da stack** (`regras-de-codigo.json` para TypeScript, `regras-de-codigo.java.json`,
   `regras-de-codigo.rust.json`) e, para cada regra, escolha o `tipo` que a linha do princípio no
   arquivo da linguagem indica ([principios-de-codigo.md](principios-de-codigo.md)). A configuração
   de lint de referência está em `.claude/watchdogs/modelos/lint/`.
2. **Preencha os parâmetros** com o que o grilling decidiu: pastas, pacotes, regex do commit.
3. **Regra de padrão** (`ast-grep`): copie o modelo de `.claude/watchdogs/modelos/ast-grep/` para
   `docs/regras-de-codigo/ast-grep/`, troque os marcadores `<...>` e apague os documentos das
   linguagens que o projeto não usa. Linguagem que falta: copie um documento e troque `language` e o
   `kind` (`ast-grep run --debug-query=cst -p '<trecho>' -l <linguagem>` mostra os nomes). Cada regra
   nova ganha um exemplo que deve pegar e um que deve passar, rodados uma vez com
   `npx -y -p @ast-grep/cli@0.51.0 ast-grep scan --rule <arquivo> <pasta-de-exemplo>`.
4. **Nada no catálogo serve?** Use `comando` com a ferramenta da stack. Checagem que valeria para
   outros projetos vira proposta de PR no Stack-Labs, para entrar no motor.

Pronto quando `sh .claude/watchdogs/rodar validar` termina em `VALIDADOR: APROVADO` no código atual,
ou as violações listadas viraram tickets.

## Onde roda

- **CI do projeto**, configurado na primeira fatia da primeira fase, junto com o lint:
  `sh .claude/watchdogs/rodar validar --base origin/${{ github.base_ref }}`, com o ast-grep instalado
  no passo anterior quando o `.json` usa `ast-grep` (`npm i -g @ast-grep/cli@0.51.0`).
- **Implementador**, antes do commit; **revisor** da orquestração, como violação dura.
- **Fechamento do mapa** de Regras de código, com `--so-cobertura`.
