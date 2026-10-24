# Validador de regras de código

Confere por máquina o que as regras de código decidiram (ADR 0010 do método). O motor é do método e
fica protegido em `.claude/watchdogs/` (o mesmo binário Rust dos watchdogs); o projeto decide **o que** validar em
`docs/regras-de-codigo.json`. Princípio de julgamento fica com o revisor.

```bash
sh .claude/watchdogs/rodar validar --base origin/main  # só o que mudou desde a base (CI, implementador, revisor)
sh .claude/watchdogs/rodar validar                     # o repositório inteiro (auditoria)
sh .claude/watchdogs/rodar validar --so-cobertura      # só a tabela dos 22 contra o .json (fecha o mapa)
sh .claude/watchdogs/rodar ci --base origin/main       # o CI local: comandos, validador e revisão por modelo
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
regra do `.json` sem linha na tabela (fora a `revisao-por-modelo`, consultiva). O E1 e o P5 nunca são `afastado` (regras 20 e 15 da
constituição), e a linha do P5 cita uma regra com a checagem `teste-cobre-risco`. Regra que nasce de prática da linguagem leva o ID dela em
`principio` (`TS<n>`, `JV<n>`, `RS<n>`) e é citada na tabela *Práticas da linguagem* do `.md`, fora
dos 22.

## Catálogo

| `tipo` | Parâmetros | Confere |
|---|---|---|
| `termo-evitado` | `glossario` (padrão `GLOSSARY.md`), `ignorarTermos` | Nome declarado ou nome de arquivo com um termo *Evitar* do glossário |
| `todo-sem-issue` | `marcadores`, `issue` (regex) | `TODO`/`FIXME` sem a issue |
| `teste-cobre-risco` | `riscos` (padrão `docs/riscos.md`), `testes` (globs), `declaracao` (regex), `ignorar` | Teste que não cita `RK<n>` ou cita um que não está no registro; no registro, risco sem origem, `pendente` sem issue e risco não pendente sem teste ([riscos.md](riscos.md)). Obrigatória no P5 |
| `teste-de-contrato-por-port` | `ports` (glob), `contrato` (`{dir}`, `{dir_de_teste}`, `{nome}`), `ignorar` | Port sem o teste de contrato; `{dir_de_teste}` é o `{dir}` com `src/main/` trocado por `src/test/` (Maven e Gradle) |
| `lint-cobre` | `config` (arquivo ou lista), `regras` | A configuração do lint tem as regras delegadas a ele |
| `mensagem-de-commit` | `padrao` (regex), `base` | Título dos commits desde a base (padrão `origin/main`) |
| `ast-grep` | `regras` (arquivo ou lista), `alvos`, `binario` | Padrões de código (TypeScript, JavaScript, Python, Go, Java, Rust e outras) |
| `comando` | `executar` (aceita `{arquivos}`) | Qualquer ferramenta: saída diferente de 0 é violação |
| `revisor` | | Nada: marca a regra como julgamento do revisor |
| `revisao-por-modelo` | `consultivo` (obrigatório, `true`), `modelo` (padrão `claude-haiku-5-5`), `testes`, `declaracao`, `riscos`, `ignorar`, `tetoUsd` (padrão 0.05), `limiteSegundos` (padrão 120) | Nada no validador. O `ci` pergunta ao modelo, teste a teste, se cada teste alterado desde a base exercita o risco que cita, e lista os suspeitos com o motivo. Consultiva, fora da `cobertura` e não substitui o `teste-cobre-risco` (ADR 0015) |

**`revisao-por-modelo` é opcional.** Para ligá-la, acrescente ao `.json` uma regra do P5 que a tabela
não precisa citar:

```json
{ "id": "R15", "principio": "P5", "texto": "O modelo lista o teste que cita o risco e não o exercita",
  "verificacao": { "tipo": "revisao-por-modelo", "modelo": "claude-haiku-5-5", "consultivo": true } }
```

Ela roda só no `ci`, chamando o `claude -p` com o login de quem roda: uma chamada por teste alterado,
em paralelo, com o trecho do teste e a linha do risco em `docs/riscos.md`. O veredito fica em cache
em `.claude/state/revisao-de-testes/`; o mesmo teste com o mesmo risco não chama de novo. Falha
aberta: sem `claude` no PATH, sem rede ou sem login, o `ci` avisa e segue. Os hooks nunca a chamam.

**`lint-cobre` falha fechado:** a regra conta só se algum arquivo de `config` a liga com nível que
bloqueia; senão, a violação diz se ela falta, está comentada, está desligada ou se o arquivo é
inválido. O nome casa inteiro (`panic` não casa com `panic_in_result_fn`).

| Arquivo | Conta como ligada |
|---|---|
| `.toml` (`Cargo.toml`, `clippy.toml`) | Chave em `[lints.<grupo>]` ou `[workspace.lints.<grupo>]` com `"deny"` ou `"forbid"` (texto ou `{ level, priority }`); `clippy::x` só em `[lints.clippy]`; um grupo (`pedantic`) pela chave dele. No `clippy.toml`, o limite de primeiro nível que não é `false` |
| `.xml` | Checkstyle `<module name="X">` sem `severity` `ignore`; PMD `<rule ref=".../X">` sem `<exclude name="X"/>`; Error Prone só `-Xep:X:ERROR`. Comentário `<!-- -->` não conta |
| `.json`, `.jsonc` (`tsconfig.json`) | Chave em qualquer nível com valor que não é `false`, `"off"` nem `0` |
| Outras (`eslint.config.js`, `build.gradle`) | O nome fora de comentário, sem `"off"`, `"allow"`, `false` ou `0` logo depois. Para o ESLint, prefira o `comando` com o próprio lint |

Regra ligada só por preset, por grupo ou por padrão da ferramenta não aparece no arquivo: liste-a
explicitamente. O `lint-cobre` prova que a regra está declarada; o `comando` com o próprio lint prova
que o código a cumpre.

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
| Mutação no diff em TS/JS (P5) | `npx stryker run --mutate <fontes alterados, separados por vírgula> --incremental`, com `@stryker-mutator/core@10.0.0` e `thresholds.break` no `stryker.config.json` (não há flag para o limite) |
| Mutação no diff em Rust (P5) | `git diff origin/main...HEAD > alteracoes.diff && cargo mutants --in-diff alteracoes.diff`, com `cargo-mutants@27.1.0`; sai com 2 se um mutante sobrevive |
| Mutação no diff em Java (P5) | `mvn test-compile org.pitest:pitest-maven:1.30.0:mutationCoverage -DtargetClasses=<classes do diff> -DmutationThreshold=80 -DwithHistory` (o modo por git do PIT é pago; monte `targetClasses` do diff) |

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

- **CI local**, a cada `git push` pelo `pre-push` e à mão: `sh .claude/watchdogs/rodar ci [--base <ref>]`
  (base padrão `origin/main`; no `pre-push`, `<remoto>/main`). Roda, em ordem, as regras `comando`
  (lint e typecheck), o resto do validador com `--base` e a `revisao-por-modelo`; reprova só pelas
  duas primeiras. Sem `docs/regras-de-codigo.json`, passa. Termina em `CI: APROVADO` ou `CI: REPROVADO`.
- **CI do projeto**, configurado na primeira fatia da primeira fase, junto com o lint:
  `sh .claude/watchdogs/rodar validar --base origin/${{ github.base_ref }}`, com o ast-grep instalado
  no passo anterior quando o `.json` usa `ast-grep` (`npm i -g @ast-grep/cli@0.51.0`).
- **Implementador**, antes do commit; **revisor** da orquestração, como violação dura.
- **Fechamento do mapa** de Regras de código, com `--so-cobertura`.
