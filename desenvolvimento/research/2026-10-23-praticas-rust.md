# Rust: princípios de bom desenvolvimento e como o validador os confere

Pesquisa de 2026-10-23. Pergunta: quais práticas de Rust, em fonte primária, concretizam os 22
princípios da casa (E1–E8, P1–P9, S O L I D), e como cada uma se confere por máquina com ferramenta
que não cobra.

Desfecho: [ADR 0012](../adr/0012-praticas-da-linguagem.md). As linhas desta pesquisa viraram
`docs/agents/principios-de-codigo/rust.md` e os modelos de `.claude/watchdogs/modelos/` do
`modelo-de-projeto`; as práticas fora dos 22 viraram RS1 a RS6. Os achados da seção 0 sobre o motor
(`lint-cobre`, `termo-evitado`, jscpd) foram corrigidos na mesma mudança; o lint
`tests_outside_test_module` ficou fora, pelo P5 lido com o `/codebase-design`.

## Como foi feito, e o limite das fontes

- A rede desta sessão bloqueia `doc.rust-lang.org`, `rust-lang.github.io` e `crates.io` (site). As
  fontes foram lidas **no repositório de origem de cada documento**, clonado do GitHub em
  2026-10-23: `rust-lang/book`, `rust-lang/api-guidelines`, `rust-lang/reference`,
  `rust-lang/cargo` (`src/doc`), `rust-lang/rust-clippy` (commit `4e2a51fa`, de 2026-10-23: `book/`,
  `CHANGELOG.md` e o grupo de cada lint lido do `declare_clippy_lint!` no fonte),
  `rust-lang/rustfmt`, `EmbarkStudios/cargo-deny`, `rustsec/rustsec`, `rustsec/advisory-db`,
  `bnjbvr/cargo-machete`, `est31/cargo-udeps`, `dtolnay/anyhow`. Os links abaixo apontam para a
  página publicada que esses fontes geram.
- Versão e licença das ferramentas: `cargo info <crate>` (lê o registro do crates.io) e
  `npm view`, em 2026-10-23.
- Toolchain local: **rustc/cargo 1.97.0, clippy 0.1.97, rustfmt 1.9.0**. Todo comportamento marcado
  "testado" foi rodado aqui, em pastas de teste fora do repositório.
- Nível padrão dos lints do rustc: `rustc -W help` (1.97). Texto dos lints do Clippy:
  `cargo clippy --explain <lint>`.

Atalhos de link usados na tabela:
[Clippy] = `https://rust-lang.github.io/rust-clippy/master/index.html#<lint>` ·
[rustc-allow] = <https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html> ·
[rustc-warn] = <https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html> ·
[API] = <https://rust-lang.github.io/api-guidelines/checklist.html> ·
[clippy.toml] = <https://doc.rust-lang.org/clippy/lint_configuration.html>.

## 0. Achados que mudam o método (ler primeiro)

1. **`lint-cobre` dá falso aprovado com `Cargo.toml`** (testado com o binário real dos watchdogs). Ele
   procura o nome da regra como **substring** no texto do arquivo. Três casos passaram sem estar
   cobertos:
   - `unwrap_used = "allow"` (regra desligada) → aprovado;
   - `# expect_used = "deny"` (regra comentada) → aprovado;
   - regra `panic` com só `panic_in_result_fn = "deny"` no arquivo → aprovado.
   E um falso reprovado: a regra escrita `clippy::unwrap_used` não é achada, porque na tabela
   `[lints.clippy]` a chave vai sem o prefixo (fonte: [Cargo, seção `[lints]`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section):
   "it is the part before `::` in the lint name"). Limites do `clippy.toml` (`too-many-lines-threshold = 60`)
   funcionam, porque são chaves literais.
   **Proposta para o motor (PR no Stack-Labs):** quando `config` for um `Cargo.toml`, ler o TOML e
   exigir a regra em `[lints.clippy]`, `[lints.rust]` ou `[workspace.lints.*]` com nível `deny` ou
   `forbid` (ou `warn`, se o CI roda `-D warnings`), aceitando a forma `{ level = "...", priority = n }`;
   lint ligado por grupo (`pedantic = { level = "deny", priority = -1 }`) conta como coberto se a regra
   pertence ao grupo. Até lá, a receita segura é a de baixo: `comando` com o próprio Clippy.
2. **Receita segura para tudo que é lint em Rust:** `comando` com
   `cargo clippy --all-targets --locked -- -D warnings`. O nível vem do `Cargo.toml` (`[lints]`, estável
   desde 1.74, "MSRV: Respected as of 1.74" na mesma página); `-D warnings` faz o que é `warn` reprovar.
   `lint-cobre` serve para provar que a regra **está declarada**; o `comando` prova que o código a
   **cumpre**. As duas juntas fecham a regra.
3. **`[lints.cargo]` não funciona no estável.** A doc do Cargo (master) já lista
   `[lints.cargo] unused_dependencies`, mas o cargo 1.97 responde
   `warning: unused manifest key 'lints.cargo' (may be supported in a future version)` (testado). Para
   dependência sem uso, use `cargo-machete` (abaixo).
4. **`termo-evitado` não vê três declarações de Rust** (testado). A regex `DECLARACAO` de
   `src/validador.rs` captura `mut` em `let mut usuario_novo` (o nome fica de fora) e não conhece
   `static` nem `mod`. Com o glossário mandando evitar "usuario", `let usuario_ativo` e `struct UsuarioX`
   foram pegos; `let mut usuario_novo`, `static USUARIO_PADRAO` e `mod usuario_repo` passaram.
   Correção sugerida: `\b(?:...|let(?:\s+mut)?|static(?:\s+mut)?|mod|union|...)\s+(...)`.
5. **Receita do jscpd no `validador.md` está quebrada para o jscpd 5.4.2**: a flag é `--exit-code`, não
   `--exitCode` (testado: "unexpected argument '--exitCode' found; tip: a similar argument exists:
   '--exit-code'"). Com a flag certa o jscpd lê Rust (12 arquivos, 0 clones nos watchdogs).
6. **`npx -y @ast-grep/cli scan ...` falha** com o npm 10.9.4 ("could not determine executable to
   run"), porque o pacote tem dois binários (`ast-grep` e `sg`). Funciona com
   `npx -y -p @ast-grep/cli ast-grep scan ...`. O validador chama o binário `ast-grep` direto e o CI
   instala com `npm i -g @ast-grep/cli`, então só a instrução de teste manual muda.
7. **Lint por pasta existe em Rust sem ast-grep** (testado): o nível de um lint pode ser posto num
   módulo só, com atributo interno. `#![deny(clippy::disallowed_types)]` em `src/dominio/mod.rs` e
   `#![allow(clippy::disallowed_types, reason = "...")]` na raiz pegou `serde_json::Value` só no
   domínio. O livro do Clippy cita esse uso: "Running a lint only on a subset of code (e.g.
   `#[forbid(clippy::float_arithmetic)]` on a module)" ([Clippy book](https://doc.rust-lang.org/clippy/)).
   Com `clippy::disallowed_methods` / `disallowed_types` (lista no `clippy.toml`, com `reason`), isso
   cobre P3 resolvendo o caminho pelo compilador, inclusive através de `use std::env::var; var(..)`,
   o que o ast-grep não faz.

## 1. Tabela: prática → fonte → princípio da casa → verificação

Tipos do catálogo: `lint-cobre` (declarado no `Cargo.toml`/`clippy.toml`), `comando`, `ast-grep`,
`todo-sem-issue`, `termo-evitado`, `teste-de-contrato-por-port`, `mensagem-de-commit`, `revisor`.
"Clippy (CI)" abaixo significa sempre: `lint-cobre` com `config: ["Cargo.toml", "clippy.toml"]` e a
regra, **mais** o `comando` `cargo clippy --all-targets --locked -- -D warnings` (achado 2).
Grupo e versão de cada lint do Clippy estão entre parênteses (lidos do fonte).

### Escrita (E1–E8)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Caixa da RFC 430: `snake_case` para funções, variáveis e módulos; `UpperCamelCase` para tipos e traits; `SCREAMING_SNAKE_CASE` para `const`/`static` | [API C-CASE](https://rust-lang.github.io/api-guidelines/naming.html#c-case); rustc `nonstandard_style` = `non_snake_case`, `non_camel_case_types`, `non_upper_case_globals`, todos warn por padrão ([rustc-warn]) | E1 (caixa) | Clippy (CI): já é warn; o `comando` com `-D warnings` reprova. Em `lint-cobre`: `nonstandard_style` em `[lints.rust]` com `"deny"` |
| Nome na língua do domínio, sem termo *Evitar* | Glossário do projeto | E1 | `termo-evitado` (com a correção do achado 4) |
| Conversão nomeada `as_` (barata, empresta), `to_` (cara), `into_` (consome); getter sem `get_` | [API C-CONV, C-GETTER](https://rust-lang.github.io/api-guidelines/naming.html#c-conv) | E1 | Clippy (CI): `wrong_self_convention` (style, já ligado) |
| Nome do item não repete o módulo (`pedido::PedidoRepo` → `pedido::Repo`) | [Clippy `module_name_repetitions`] (restriction desde 1.84, era pedantic; CHANGELOG) | E1 | Clippy (CI): `module_name_repetitions`, opcional |
| Função curta | [Clippy `too_many_lines`] (pedantic; limite `too-many-lines-threshold`, padrão 100, [clippy.toml]) | E2 | Clippy (CI): `too_many_lines = "deny"` + `too-many-lines-threshold = <n>` no `clippy.toml` |
| Poucos parâmetros | [Clippy `too_many_arguments`] (complexity, já warn; `too-many-arguments-threshold`, padrão 7) | E2 | Clippy (CI); limite no `clippy.toml` |
| Pouco aninhamento | [Clippy `excessive_nesting`] (complexity; **só age com `excessive-nesting-threshold` definido**, padrão 0 = desligado, `--explain`) | E2 | Clippy (CI): `excessive-nesting-threshold = 3` (ou o que o grilling decidir) |
| Complexidade de leitura | [Clippy `cognitive_complexity`] (restriction desde 1.91, antes nursery; `cognitive-complexity-threshold`, padrão 25) | E2 | Clippy (CI), opcional; o `--explain` diz "We used to think it measured how hard a method is to understand [...] left in `restriction` so as to not mislead users"; prefira `too_many_lines` + `excessive_nesting` |
| Saída cedo com `let ... else` em vez de `match`/`if let` aninhado | [Clippy `manual_let_else`] (pedantic); `let else` na [Reference](https://doc.rust-lang.org/reference/statements.html#let-statements) | E2 | Clippy (CI): `manual_let_else` |
| Tipo próprio para conceito do domínio (newtype) | [API C-NEWTYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype); [Book cap. 20.3, newtype](https://doc.rust-lang.org/book/ch20-03-advanced-types.html) | E3 | `revisor` |
| Argumento diz o que é pelo tipo, não por `bool`/`Option` | [API C-CUSTOM-TYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-custom-type) | E3 | Clippy (CI): `fn_params_excessive_bools` (pedantic) com `max-fn-params-bools = 1` (testado: pega `fn dois(a: bool, b: bool)`, deixa `fn um(a: bool)`); `struct_excessive_bools` (pedantic) |
| Validar na borda, uma vez; depois o tipo garante | [API C-VALIDATE](https://rust-lang.github.io/api-guidelines/dependability.html#c-validate) ("Choose an argument type that rules out bad inputs"); [Book 9.3, "Custom Types for Validation"](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#custom-types-for-validation) | E3 | `revisor` |
| Nada de "saco genérico" no domínio (`serde_json::Value`, `HashMap<String, String>` como entidade) — o `any` do Rust | [API C-CUSTOM-TYPE] | E3 | Clippy (CI): `disallowed_types` com `serde_json::Value` no `clippy.toml` e `#![deny(clippy::disallowed_types)]` no módulo do domínio (achado 7, testado) |
| Erro esperado volta como `Result`; `panic!` só para estado impossível | [Book 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#guidelines-for-error-handling): "when failure is expected, it's more appropriate to return a `Result` than to make a `panic!` call"; "returning `Result` is a good default choice" | E4 | Clippy (CI): `unwrap_used`, `expect_used`, `panic`, `unwrap_in_result`, `panic_in_result_fn`, `indexing_slicing`, `string_slice` (todos restriction). Nos testes: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests = true` no `clippy.toml` (valem para "test functions or `#[cfg(test)]`", [clippy.toml]; com o bloco da seção 3, `tests/` dos watchdogs ficou com 0 achados, testado) |
| Resultado `Result` nunca descartado | `Result` é `#[must_use]`; rustc `unused_must_use` (warn) ([Reference, `must_use`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute)) | E4 | Clippy (CI): já é warn; `-D warnings` reprova `fs::write(..);` |
| `let _ = <Result>` descarta o erro | [Clippy `let_underscore_must_use`] (restriction) | E4 | Clippy (CI): `let_underscore_must_use` (testado) |
| `<Result>.ok();` descarta o erro | [Clippy `unused_result_ok`] (restriction, 1.82) | E4 | Clippy (CI): `unused_result_ok` (testado) |
| `.map_err(\|_\| ...)` perde a causa | [Clippy `map_err_ignore`] (restriction) | E4 | Clippy (CI): `map_err_ignore` (testado) |
| Ramo `Err` vazio ou que só imprime e segue | Nenhum lint cobre | E4 | `ast-grep`: `catch-vazio-rust` (seção 4) |
| Erro é tipo próprio, implementa `std::error::Error`, `Send + Sync`; nunca `()` | [API C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err) ("Never use `()` as an error type") | E4 | Clippy (CI): `result_unit_err` (style, já ligado) para `()`; `ast-grep`: `erro-sem-tipo` (seção 4, nova) para `String`, `&str`, `Box<dyn Error>`, `anyhow` no domínio |
| `thiserror` na biblioteca/domínio, `anyhow` na aplicação/borda | [README do anyhow, "Comparison to thiserror"](https://github.com/dtolnay/anyhow#comparison-to-thiserror): "Use Anyhow if you don't care what error type your functions return [...] This is common in application code. Use thiserror if you are a library [...]" | E4 | `ast-grep`: `erro-sem-tipo` com `files` no domínio |
| Doc de função pública diz quando dá erro e quando entra em pânico | [API C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure) | E4 | Clippy (CI): `missing_errors_doc`, `missing_panics_doc` (pedantic); `missing_safety_doc` (style, já ligado) |
| Contrato de "retorno precisa ser usado" no tipo/função | [Reference `must_use`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute) | E4, E5 | Clippy (CI): `must_use_candidate`, `return_self_not_must_use` (pedantic). `must_use_candidate` é o mais barulhento (31 achados nos watchdogs, seção 5) |
| Imutável por padrão (`let`), `mut` só onde muda | Rust é imutável por padrão ([Book 3.1](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)); rustc `unused_mut` (warn) | E5 | Clippy (CI): `unused_mut` já é warn; `needless_pass_by_ref_mut` (nursery) para `&mut` desnecessário |
| Comando ou consulta: função que só consulta recebe `&self`, não `&mut self`; não usa parâmetro de saída | [API C-NO-OUT](https://rust-lang.github.io/api-guidelines/predictability.html#c-no-out) | E5 | `revisor`; ajuda parcial: `needless_pass_by_ref_mut` (nursery), `needless_pass_by_value` (pedantic) |
| Comentário diz o porquê; `TODO` com issue | — | E6 | `todo-sem-issue` (lê `.rs` igual); a macro `todo!()`/`unimplemented!()` é outra coisa: Clippy (CI) `todo`, `unimplemented` (restriction) |
| Todo bloco `unsafe` tem `// SAFETY:` dizendo por que é seguro | [Clippy `undocumented_unsafe_blocks`] (restriction) | E6 (e "unsafe", seção 2) | Clippy (CI): `undocumented_unsafe_blocks` |
| `#[allow]` sempre com o porquê; preferir `#[expect]` (avisa quando o lint deixa de disparar) | [Reference, `reason` e `expect`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes); [Clippy `allow_attributes_without_reason`], [`allow_attributes`] (restriction) | E6 | Clippy (CI): `allow_attributes_without_reason`, `allow_attributes` |
| Código comentado | — | E6 | `revisor` |
| Duplicação | — | E7 | `comando`: `npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` (achado 5; jscpd lê Rust, testado) |
| Número com significado tem nome (`const`) | **Não há lint no Clippy** para número mágico (procurado no fonte `clippy_lints/src`, nenhum lint) | E8 | `revisor` |
| Formatação única | [rustfmt](https://github.com/rust-lang/rustfmt) | P8 | `comando`: `cargo fmt --all -- --check` |

### Padrões (P1–P9)

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Interface pequena: privado por padrão, `pub` só o necessário; `pub(crate)` para o resto | [Book 7.2–7.3](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html) (privado por padrão); rustc `unreachable_pub` (allow) | P1, I | Clippy (CI): `unreachable_pub` em `[lints.rust]` |
| Campos privados, construtor que valida | [API C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-struct-private), [C-NEWTYPE-HIDE](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-newtype-hide) | P1, E3 | `revisor`; ajuda: `partial_pub_fields` (restriction) |
| Item público documentado | [API C-CRATE-DOC, C-EXAMPLE](https://rust-lang.github.io/api-guidelines/documentation.html); rustc `missing_docs` (allow) | P1 | Clippy (CI): `missing_docs` em `[lints.rust]` — só para crate publicado como biblioteca (119 achados nos watchdogs, quase todos `pub` de um binário interno) |
| Trait (port) só onde há duas implementações reais | — | P2, O | `revisor` |
| Dependência entra; domínio não usa crate de infraestrutura | — | P3, D | `ast-grep`: `dominio-sem-infra-rust` (seção 4). Mais forte: domínio num crate próprio do workspace, sem a dependência no `Cargo.toml` dele ([Book 14.3, workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)); aí o compilador recusa. No workspace, `cargo-deny` `bans.deny` com `wrappers` limita um crate a ser usado só por crates nomeados ([cargo-deny bans](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html#the-deny-field-optional)) |
| Ambiente só na montagem | — | P3 | Preferido: Clippy (CI) `disallowed_methods` com `std::env::var`, `std::env::var_os`, `std::env::vars` no `clippy.toml` e `#[expect(clippy::disallowed_methods, reason = "montagem")]` no `main`/`config` (testado; pega `use std::env::var; var(..)`). Alternativa por pasta: `ast-grep` `ambiente-so-na-montagem-rust` |
| Relógio e acaso entram por parâmetro | — | P3 | `ast-grep`: `relogio-e-acaso-entram` (seção 4, nova); ou `disallowed_methods` (`std::time::SystemTime::now`, `std::time::Instant::now`) com `#![deny]` só no domínio |
| `std::process::exit` só no `main` | [Clippy `exit`] (restriction; `--explain`: "Detects calls to the `exit()` function, which terminates the program") | P3 | Clippy (CI): `exit` |
| Saída no terminal só na borda | [Clippy `print_stdout`, `print_stderr`] (restriction), [`dbg_macro`] (restriction) | P3, E6 | Clippy (CI): `print_stdout`, `print_stderr` com `#[expect]` no módulo da CLI; `dbg_macro` sempre |
| O que muda junto fica junto: um módulo por conceito, arquivos `mod.rs` ou `nome.rs` de forma consistente | [Book 7.5](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html); [Clippy `mod_module_files`, `self_named_module_files`] (restriction) | P4, P8 | `revisor` para a coesão; Clippy (CI) para a forma do arquivo (escolha um dos dois lints) |
| Teste pela interface: teste de integração em `tests/` usa só a API pública | [Book 11.3](https://doc.rust-lang.org/book/ch11-03-test-organization.html): "they use the library's public API to test the code" | P5 | `revisor`; `ast-grep` `dominio-sem-infra-rust` apontado para `tests/**` com `<PACOTES_DE_INFRA>` = `mockall` para proibir mock de módulo nosso |
| Teste unitário em `#[cfg(test)] mod tests` | [Book 11.3, "The Tests Module and `#[cfg(test)]`"](https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-module-and-cfgtest) | P5 | Clippy (CI): `tests_outside_test_module` (restriction) |
| Port com teste de contrato | — | L | `teste-de-contrato-por-port` com `ports: ["src/dominio/portas/*.rs"]`, `contrato: "tests/contrato_{nome}.rs"` (o motor só confere arquivo; serve igual) |
| Vermelho, verde, refatorar | — | P6 | `revisor` |
| Nada para requisito que não existe; dependência sem uso sai | [cargo-machete](https://github.com/bnjbvr/cargo-machete) | P7 | `comando`: `cargo machete` (estável, rápido, "imprecise" segundo o README; falso positivo resolve com `[package.metadata.cargo-machete] ignored = [...]`). `cargo-udeps` é preciso mas exige nightly ("it needs Rust nightly to actually run", README) |
| Código morto sai | rustc `dead_code` (warn) | P7 | Clippy (CI) com `-D warnings` |
| Consistência: bibliotecas que o projeto exclui | [cargo-deny bans](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html) | P8 | `comando`: `cargo deny check bans` com `deny = [...]` no `deny.toml` (resolve a árvore inteira, inclusive transitiva). O `ast-grep dominio-sem-infra` no código todo também serve, como em TS |
| Uma versão de cada dependência | cargo-deny `multiple-versions` | P8 | `comando`: `cargo deny check bans` |
| Commit conta uma mudança | — | P9 | `mensagem-de-commit` |

### SOLID

| Prática | Fonte | Princípio | Verificação |
|---|---|---|---|
| Um motivo para mudar por módulo | — | S | `revisor` |
| Variante nova = novo `impl Trait`; `#[non_exhaustive]` em enum/struct público que vai crescer | [Reference `non_exhaustive`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute): "indicates that a type or variant may have more fields or variants added in the future" | O | `revisor`; Clippy `exhaustive_enums`/`exhaustive_structs` (restriction) só para crate publicado |
| Adapter cumpre o trait inteiro (sem `unimplemented!()` dentro de `impl`) | — | L | `teste-de-contrato-por-port`; Clippy (CI) `unimplemented`, `todo` |
| Trait pequeno; quem chama pede só o bound que usa (`impl Read`, não o tipo concreto) | [API C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#c-generic), [C-RW-VALUE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value) | I | `revisor` |
| O domínio define o trait; a infra implementa | — | D | `ast-grep` `dominio-sem-infra-rust` (ou crate de domínio separado, acima) |

## 2. Práticas que não cabem nos 22 princípios

Candidatas a princípio específico de Rust (ou a uma regra fixa do perfil Rust da casa):

| Prática | Fonte | Verificação |
|---|---|---|
| **R-unsafe: sem `unsafe` no código da casa**; quando inevitável, isolado num módulo, com `// SAFETY:` por bloco e uma operação por bloco | [Reference, Unsafety](https://doc.rust-lang.org/reference/unsafety.html) ("Unsafe operations are those that can potentially violate the memory-safety guarantees"); rustc `unsafe_code` (allow, [rustc-allow]); Clippy `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block` (restriction) | Clippy (CI): `[lints.rust] unsafe_code = "forbid"` no crate; onde houver exceção, `deny` + `#[expect(unsafe_code, reason)]` no módulo. Na edição 2024, `unsafe_op_in_unsafe_fn` já é warn (testado no 1.97) |
| **R-pânico: código de produção não entra em pânico por atalho** (`unwrap`, `expect`, `panic!`, índice `v[i]`, fatia de `str`). Não é só E4: em Rust o pânico derruba a thread e não aparece na assinatura | [Book 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html); [API C-FAILURE] | Clippy (CI): `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `string_slice`, `unwrap_in_result`, `panic_in_result_fn` + liberações de teste no `clippy.toml` |
| **R-dependências: a árvore de crates é auditada** (vulnerabilidade RustSec, licença permitida, origem só crates.io, sem duplicata) | [API C-PERMISSIVE](https://rust-lang.github.io/api-guidelines/necessities.html#c-permissive); [cargo-deny checks](https://embarkstudios.github.io/cargo-deny/checks/index.html); [RustSec advisory-db](https://github.com/rustsec/advisory-db) | `comando`: `cargo deny check` (advisories, licenses, bans, sources). O `cargo audit` faz só a parte de vulnerabilidade, que o `cargo-deny` já cobre |
| **R-lockfile: build reprodutível**, `Cargo.lock` commitado e build com `--locked` | Já é regra da casa (ADR 0011); flag `--locked` na [doc do cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html) | `comando`: `cargo build --locked` |
| **R-cast: conversão numérica sem perda silenciosa** (`as` trunca e troca sinal) | Clippy `cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`, `cast_precision_loss` (pedantic); `as_conversions` (restriction) | Clippy (CI): os quatro `cast_*` (7 achados nos watchdogs) |
| **R-debug: tipo público implementa `Debug`, e não vazio** | [API C-DEBUG, C-DEBUG-NONEMPTY](https://rust-lang.github.io/api-guidelines/debuggability.html); rustc `missing_debug_implementations` (allow) | Clippy (CI): `missing_debug_implementations` |
| Edição 2024 | Estável desde o Rust 1.85 ([anúncio](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)) **[confirmar]** (página não acessível daqui; o comportamento da edição 2024 foi testado no 1.97). Os watchdogs estão em `edition = "2021"` | `revisor` / `lint-cobre` com `edition = "2024"` no `Cargo.toml` |

## 3. Ferramentas

Todas sem plano pago e sem edição comercial (nenhum README ou página lida menciona uma).

| Ferramenta | Versão (2026-10-23) | Licença | Cobra? | Link | Comando para o CI |
|---|---|---|---|---|---|
| Clippy | 0.1.97 (vem com o toolchain 1.97) | MIT OR Apache-2.0 | Não | <https://github.com/rust-lang/rust-clippy> | `cargo clippy --all-targets --locked -- -D warnings` |
| rustc lints (`[lints.rust]`) | 1.97.0 | MIT OR Apache-2.0 | Não | [rustc-allow] | (roda junto do clippy) |
| rustfmt | 1.9.0 (com o toolchain) | MIT OR Apache-2.0 (README) | Não | <https://github.com/rust-lang/rustfmt> | `cargo fmt --all -- --check` |
| cargo-deny | 0.20.2 (MSRV 1.88) | MIT OR Apache-2.0 | Não | <https://github.com/EmbarkStudios/cargo-deny> | `cargo deny check` (com `deny.toml`; `cargo deny init` gera um) |
| cargo-audit | 0.22.2 (MSRV 1.88) | Apache-2.0 OR MIT | Não | <https://github.com/rustsec/rustsec/tree/main/cargo-audit> | `cargo audit` — redundante com `cargo deny check advisories` |
| RustSec advisory-db | — | domínio público / CC0-1.0 (README) | Não | <https://github.com/rustsec/advisory-db> | (lida pelo cargo-deny/audit) |
| cargo-machete | 0.9.2 | MIT | Não | <https://github.com/bnjbvr/cargo-machete> | `cargo machete` |
| cargo-udeps | 0.1.61 | MIT OR Apache-2.0 | Não | <https://github.com/est31/cargo-udeps> | `cargo +nightly udeps` (exige nightly; não recomendado como padrão) |
| ast-grep | 0.51.0 (npm e crates.io; MSRV 1.90) | MIT | Não | <https://github.com/ast-grep/ast-grep> | `npm i -g @ast-grep/cli` ou `cargo install ast-grep --locked`; o validador roda |
| jscpd | 5.4.2 | MIT | Não | <https://github.com/kucherenko/jscpd> | `npx jscpd --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | Não | <https://github.com/dtolnay/thiserror> | (biblioteca) |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | Não | <https://github.com/dtolnay/anyhow> | (biblioteca) |
| cargo-semver-checks | 0.51.0 | Apache-2.0 OR MIT | Não | <https://github.com/obi1kenobi/cargo-semver-checks> | só para crate publicado; fora do escopo agora |

Instalação no CI das que não vêm com o toolchain: `cargo install --locked cargo-deny cargo-machete`
(compilam do fonte; leva minutos sem cache) **ou** a action/binário de release de cada projeto
**[confirmar]** qual forma o CI da casa prefere.

### Bloco sugerido para o `Cargo.toml` de um projeto Rust da casa

Testado em pastas de teste (nível e chaves aceitos pelo cargo 1.97). É ponto de partida do
grilling, não regra decidida.

```toml
[lints.rust]
unsafe_code = "forbid"
unreachable_pub = "warn"
missing_debug_implementations = "warn"

[lints.clippy]
# grupos, com prioridade baixa para que os lints abaixo os sobrescrevam
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"          # barulhento: 31 achados nos watchdogs
module_name_repetitions = "allow"
# E4 / R-pânico
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
indexing_slicing = "deny"
string_slice = "deny"
unwrap_in_result = "deny"
let_underscore_must_use = "deny"
unused_result_ok = "deny"
map_err_ignore = "deny"
# E6
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
allow_attributes_without_reason = "deny"
undocumented_unsafe_blocks = "deny"
# P3
exit = "deny"
print_stdout = "deny"
print_stderr = "deny"
disallowed_methods = "deny"
```

```toml
# clippy.toml
too-many-lines-threshold = 60
too-many-arguments-threshold = 5
excessive-nesting-threshold = 4
max-fn-params-bools = 1
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-print-in-tests = true
disallowed-methods = [
  { path = "std::env::var", reason = "P3: ambiente só na montagem" },
  { path = "std::env::var_os", reason = "P3: ambiente só na montagem" },
  { path = "std::env::vars", reason = "P3: ambiente só na montagem" },
]
```

Entrada correspondente no `docs/regras-de-codigo.json`:

```json
{ "id": "R4", "principio": "E4", "texto": "Erro não é engolido nem vira pânico",
  "verificacao": { "tipo": "lint-cobre", "config": ["Cargo.toml"],
    "regras": ["unwrap_used", "expect_used", "let_underscore_must_use", "unused_result_ok", "map_err_ignore"] } },
{ "id": "R5", "principio": "E4", "texto": "O código cumpre os lints declarados",
  "verificacao": { "tipo": "comando", "executar": "cargo clippy --all-targets --locked -- -D warnings" } }
```

## 4. Regras de ast-grep para Rust

Cinco regras, hoje em `.claude/watchdogs/modelos/ast-grep/` do `modelo-de-projeto`:
três equivalentes aos modelos de TS (para virarem um documento `language: Rust` dentro do modelo
existente) e duas novas. Cada uma foi testada com um exemplo que deve pegar e um que deve passar.

**Como foi testado:** os marcadores foram trocados (`<PASTA_DO_DOMINIO>` = `src/dominio`,
`<PACOTES_DE_INFRA>` = `sqlx|reqwest|crate::infra`, `<ARQUIVOS_DA_MONTAGEM>` = `src/main.rs`,
`src/config.rs`) e cada regra rodou com
`npx -y -p @ast-grep/cli ast-grep scan --rule <regra>.yml .` dentro de `pega/` e de `passa/`
(ast-grep 0.51.0).

| Regra | `pega/` | `passa/` |
|---|---|---|
| `catch-vazio-rust` | 5 de 5 | 0 |
| `dominio-sem-infra-rust` | 6 de 6 | 0 |
| `ambiente-so-na-montagem-rust` | 3 de 3 | 0 |
| `erro-sem-tipo` (nova) | 4 de 4 | 0 |
| `relogio-e-acaso-entram` (nova) | 5 de 5 | 0 |

Um caso de borda falhou no primeiro teste e foi corrigido: chamada com turbofish
(`rand::random::<u32>()`, `env::var::<..>`) vira um nó `generic_function`; as regex agora aceitam
`(::<.+>)?` no fim e o caso passou a ser pego.

Onde o Clippy já cobre, a regra **não** repete: `let _ = <Result>` (`let_underscore_must_use`),
`.ok();` (`unused_result_ok`), `map_err(|_| ..)` (`map_err_ignore`), `Result<_, ()>`
(`result_unit_err`). Para P3 o lint `disallowed_methods` é preferível quando o projeto aceita um
`#[expect]` na montagem; o ast-grep fica para quem quer a regra por pasta sem atributo no código.

**Limite conhecido do ast-grep em Rust:** ele vê texto e árvore, não resolve nomes. `use std::env::var;`
seguido de `var("X")` é pego pelo `use`, não pela chamada; um alias de crate
(`use sqlx as banco;`) é pego no `use`, mas `banco::query()` depois não. O `disallowed_methods` do
Clippy resolve os dois.

### `catch-vazio-rust.yml` (E4 — equivalente ao `catch-vazio.yml`)

```yaml
# E4 em Rust: o erro é engolido. Acrescente este documento ao catch-vazio.yml do projeto.
# O Clippy já cobre três formas, ligue-as no [lints.clippy] em vez de escrever padrão:
#   let _ = <Result>         -> clippy::let_underscore_must_use (restriction)
#   <Result>.ok();           -> clippy::unused_result_ok (restriction)
#   .map_err(|_| ...)        -> clippy::map_err_ignore (restriction)
#   <Result>; (descartado)   -> unused_must_use (rustc, warn por padrão; deny no CI com -D warnings)
# Esta regra pega o que nenhum lint pega: o ramo de Err vazio e o ramo de Err que só imprime e segue.
id: catch-vazio-rust
language: Rust
message: "E4: o erro é engolido. Trate o caso esperado ou devolva o erro com `?` até a borda."
severity: error
rule:
  any:
    - pattern: if let Err($$$) = $X { }
    - kind: match_arm
      all:
        - has: { field: pattern, regex: '^(Result::)?Err\(' }
        - has:
            field: value
            any:
              - { kind: block, regex: '^\{\s*\}$' }
              - { kind: unit_expression }
              - { kind: block, regex: '^\{\s*(std::)?(e?print(ln)?|log::\w+|tracing::\w+|warn|error|info|debug)!\([^;]*\);?\s*\}$' }
              - { kind: macro_invocation, regex: '^(std::)?(e?print(ln)?|log::\w+|tracing::\w+|warn|error|info|debug)!' }
```

Deve pegar (`src/lib.rs`):

```rust
pub fn a(r: Result<u8, String>) {
    if let Err(_) = r {}
}
pub fn c(r: Result<u8, String>) {
    match r {
        Ok(_) => {}
        Err(e) => {}
    }
}
pub fn d(r: Result<u8, String>) {
    match r {
        Ok(_) => (),
        Err(e) => eprintln!("falhou: {e}"),
    }
}
pub fn e(r: Result<u8, String>) {
    match r {
        Ok(_) => (),
        Err(e) => { log::warn!("falhou: {e}"); }
    }
}
pub fn f(r: Result<u8, String>) {
    match r { Ok(_) => (), Err(_) => () }
}
```

Deve passar (`src/lib.rs`):

```rust
pub fn a(r: Result<u8, String>) -> Result<u8, String> {
    let v = r?;
    Ok(v)
}
pub fn b(r: Result<u8, String>) -> Result<u8, Erro> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("falhou: {e}");
            Err(Erro::Leitura(e))
        }
    }
}
pub fn c(r: Result<u8, String>) {
    if let Err(e) = r {
        registrar_e_abortar(e);
    }
}
pub fn d(o: Option<u8>) {
    match o { Some(_) => (), None => () }
}
```

### `dominio-sem-infra-rust.yml` (D/P3 — equivalente ao `dominio-sem-infra.yml`)

```yaml
# D e P3 em Rust: o domínio não usa infraestrutura. Acrescente ao dominio-sem-infra.yml do projeto e
# troque <PASTA_DO_DOMINIO> (ex.: src/dominio) e <PACOTES_DE_INFRA> (regex com o nome do crate como
# aparece no código, hífen vira sublinhado; ex.: sqlx|reqwest|tokio|aws_sdk_s3|crate::infra).
# Pega `use`, `extern crate` e o caminho qualificado sem `use` (sqlx::query(...), x: sqlx::PgPool).
# Mais forte que esta regra: o domínio num crate próprio do workspace, sem as dependências de infra
# no Cargo.toml dele; aí o compilador recusa o import.
id: dominio-sem-infra-rust
language: Rust
message: "D/P3: o domínio usa infraestrutura. Defina o trait (port) no domínio; o adapter vive fora."
severity: error
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  any:
    - kind: use_declaration
      regex: '^(pub(\([^)]*\))?\s+)?use\s+(::)?(<PACOTES_DE_INFRA>)\b'
    - kind: extern_crate_declaration
      regex: '^extern\s+crate\s+(<PACOTES_DE_INFRA>)\b'
    - any: [{ kind: scoped_identifier }, { kind: scoped_type_identifier }]
      regex: '^(::)?(<PACOTES_DE_INFRA>)::'
      not:
        inside:
          stopBy: end
          any: [{ kind: use_declaration }, { kind: scoped_identifier }, { kind: scoped_type_identifier }]
```

Deve pegar (`src/dominio/pedido.rs`):

```rust
use sqlx::PgPool;
pub use reqwest::Client as Http;
use crate::infra::banco::Conexao;
extern crate sqlx;
pub struct Repo { pool: sqlx::postgres::PgPool }
pub async fn salvar(p: &Pedido) { sqlx::query("insert ...").execute(p).await; }
```

Deve passar (`src/dominio/pedido.rs`):

```rust
use std::collections::HashMap;
use crate::dominio::cliente::Cliente;
use serde::Serialize;
pub trait RepositorioDePedidos { fn salvar(&self, p: &Pedido) -> Result<(), ErroDoRepo>; }
pub struct Pedido { itens: HashMap<String, u32>, cliente: Cliente }
```

Deve passar (`src/infra/banco.rs`):

```rust
use sqlx::PgPool;
pub struct Conexao { pool: sqlx::postgres::PgPool }
```

### `ambiente-so-na-montagem-rust.yml` (P3 — equivalente ao `ambiente-so-na-montagem.yml`)

```yaml
# P3 em Rust: configuração por ambiente se lê num ponto só, a montagem. Acrescente ao
# ambiente-so-na-montagem.yml do projeto e troque <ARQUIVOS_DA_MONTAGEM> (ex.: src/main.rs,
# src/config.rs). Alternativa por lint: clippy::disallowed_methods com std::env::var no clippy.toml
# e #[expect(clippy::disallowed_methods, reason = "montagem")] no módulo da montagem.
# env!("CARGO_PKG_VERSION") e option_env! são de compilação e ficam fora.
id: ambiente-so-na-montagem-rust
language: Rust
message: "P3: leitura de ambiente fora da montagem. Receba o valor por parâmetro."
severity: error
ignores: ["<ARQUIVOS_DA_MONTAGEM>"]
rule:
  any:
    - kind: call_expression
      has: { field: function, regex: '^(::)?(std::)?env::(var|var_os|vars|vars_os|args|args_os)(::<.+>)?$' }
    - kind: use_declaration
      regex: '\bstd::env\b'
```

Deve pegar (`src/servico.rs`):

```rust
use std::env;
pub fn url() -> String { std::env::var("URL").unwrap_or_default() }
pub fn porta() -> Option<std::ffi::OsString> { env::var_os("PORTA") }
```

Deve passar (`src/main.rs`):

```rust
use std::env;
fn main() { let url = env::var("URL").expect("URL"); servico::rodar(&url); }
```

Deve passar (`src/servico.rs`):

```rust
pub fn rodar(url: &str) { let versao = env!("CARGO_PKG_VERSION"); conectar(url, versao); }
```

### `erro-sem-tipo.yml` (E4 — nova)

```yaml
# Nova. E4 ("erro esperado é parte da interface") e C-GOOD-ERR das Rust API Guidelines: função
# pública do domínio devolve um tipo de erro que quem chama consegue casar (enum com thiserror), não
# texto nem erro apagado. anyhow e Box<dyn Error> ficam na borda (main, CLI). Troque <PASTA_DO_DOMINIO>.
# Result<_, ()> o Clippy já pega: clippy::result_unit_err (style).
id: erro-sem-tipo
language: Rust
message: "E4: erro esperado sem tipo. Devolva um enum de erro do domínio (thiserror); anyhow e Box<dyn Error> só na borda."
severity: error
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  kind: function_item
  all:
    - has: { kind: visibility_modifier }
    - has:
        field: return_type
        regex: '^((std::)?result::)?Result<.+,\s*(String|&(''static\s+)?str|Box<dyn\s+(std::error::)?Error\b.*>|anyhow::Error)\s*>$|^anyhow::Result<'
```

Deve pegar (`src/dominio/conta.rs`):

```rust
pub fn sacar(saldo: u64, valor: u64) -> Result<u64, String> { todo!() }
pub fn abrir(nome: &str) -> anyhow::Result<Conta> { todo!() }
pub(crate) fn fechar(c: Conta) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { todo!() }
pub fn validar(n: &str) -> Result<Nome, &'static str> { todo!() }
```

Deve passar (`src/main.rs`):

```rust
pub fn main() -> anyhow::Result<()> { Ok(()) }
```

Deve passar (`src/dominio/conta.rs`):

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErroDeSaque { #[error("saldo insuficiente")] SaldoInsuficiente }
pub fn sacar(saldo: u64, valor: u64) -> Result<u64, ErroDeSaque> { todo!() }
fn auxiliar() -> Result<(), String> { Ok(()) }
```

### `relogio-e-acaso-entram.yml` (P3 — nova)

```yaml
# Nova. P3 ("dependência entra, não é criada"): o domínio não lê o relógio nem sorteia; recebe o
# instante ou o gerador por parâmetro (ou por um trait), e o teste fica determinístico.
# Troque <PASTA_DO_DOMINIO>. Alternativa por lint, no crate inteiro: clippy::disallowed_methods.
id: relogio-e-acaso-entram
language: Rust
message: "P3: o domínio cria a própria dependência (relógio ou acaso). Receba o instante ou o gerador por parâmetro."
severity: error
files: ["<PASTA_DO_DOMINIO>/**"]
rule:
  kind: call_expression
  has:
    field: function
    regex: '^(.*::)?((Instant|SystemTime|Utc|Local|OffsetDateTime|Timestamp|Zoned)::now(_utc|_local)?|rand::(random|thread_rng|rng)|Uuid::(new_v4|now_v7))(::<.+>)?$'
```

Deve pegar (`src/dominio/cobranca.rs`):

```rust
pub fn vencida(prazo: std::time::SystemTime) -> bool { std::time::SystemTime::now() > prazo }
pub fn criada() -> chrono::DateTime<chrono::Utc> { chrono::Utc::now() }
pub fn codigo() -> u32 { rand::random() }
pub fn id() -> uuid::Uuid { Uuid::new_v4() }
pub fn medir() { let t = Instant::now(); }
```

Deve passar (`src/adaptadores/relogio.rs`):

```rust
pub struct RelogioDoSistema;
impl Relogio for RelogioDoSistema { fn agora(&self) -> SystemTime { SystemTime::now() } }
```

Deve passar (`src/dominio/cobranca.rs`):

```rust
pub fn vencida(prazo: std::time::SystemTime, agora: std::time::SystemTime) -> bool { agora > prazo }
pub trait Relogio { fn agora(&self) -> std::time::SystemTime; }
```


## 5. O que o Clippy acharia hoje nos watchdogs (custo de adotar)

Rodado numa cópia de `.claude/watchdogs/` (2.134 linhas em `src/`, mais `tests/`), com o toolchain
1.97, contando cada local uma vez.

- `cargo clippy --all-targets` (padrão): **0 avisos**. `cargo fmt --check`: limpo.
- `cargo clippy --all-targets -- -W clippy::pedantic`: **84 achados, 17 lints**.

| Lint (pedantic) | Achados |
|---|---|
| `must_use_candidate` | 31 |
| `map_unwrap_or` | 10 |
| `missing_panics_doc` | 9 |
| `missing_errors_doc` | 7 |
| `wildcard_imports` | 6 |
| `format_push_string` | 3 |
| `needless_pass_by_value` | 3 |
| `too_many_lines` | 2 |
| `cast_possible_truncation` | 2 |
| `cast_sign_loss` | 2 |
| `cast_precision_loss` | 2 |
| `str_split_at_newline` | 2 |
| `cast_possible_wrap`, `unnested_or_patterns`, `case_sensitive_file_extension_comparisons`, `manual_let_else`, `bool_to_int_with_if` | 1 cada |

Sem `must_use_candidate` (a sugestão é desligá-lo) sobram 53, quase todos com correção automática
(`cargo clippy --fix`) ou de uma linha. Custo: baixo.

- `restriction` seletivo e lints do rustc (os da tabela 1), mesma cópia:

| Lint | Total | Em `tests/` |
|---|---|---|
| `missing_docs` (rustc) | 119 | 2 |
| `clippy::indexing_slicing` | 76 | 14 |
| `clippy::unwrap_used` | 40 | 40 |
| `clippy::expect_used` | 20 | 0 |
| `unused_results` (rustc) | 15 | 5 |
| `clippy::print_stderr` | 11 | 2 |
| `clippy::print_stdout` | 11 | 0 |
| `clippy::let_underscore_must_use` | 7 | 2 |
| `clippy::let_underscore_untyped` | 7 | 2 |
| `unreachable_pub` (rustc) | 6 | 0 |
| `clippy::shadow_unrelated` | 3 | 2 |
| `clippy::string_slice`, `unwrap_in_result`, `panic` | 2 cada | 0, 0, 2 |
| `clippy::exit`, `map_err_ignore`, `cognitive_complexity` | 1 cada | 0 |

Leitura: os 40 `unwrap_used` estão todos em `tests/` e somem com `allow-unwrap-in-tests = true`. `missing_docs` (119) só faz sentido em crate publicado: os watchdogs são um
binário interno. Os achados de E4 que importam são poucos e reais: 5 `let _ = <Result>` em `src/`
(`comum.rs:123, 201, 203, 262`, `pre_push.rs:15`, ex.: `let _ = fs::write(&caminho_estado, ...)`),
1 `map_err(|_| ..)` (`comum.rs:256`), 20 `expect` (quase todos `Regex::new(..).expect("regex válida")`,
que seriam `LazyLock` ou `#[expect]` com motivo). `cognitive_complexity` aponta
`hooks/guarda_bash.rs:55` (`rodar`). `exit` aponta `comum.rs:149` (`std::process::exit(2)` fora do
`main`).

As regras de ast-grep, com `src` inteiro como "domínio" (os watchdogs não têm camada de domínio,
então isto é só calibração de ruído): `catch-vazio-rust` 0; `ambiente-so-na-montagem-rust` 4
(`comum.rs`, `pre_push.rs`, `validador.rs`); `erro-sem-tipo` 4 (`comum.rs`, `Result<_, String>`);
`relogio-e-acaso-entram` 2 (`comum.rs`); `dominio-sem-infra-rust` com `std::process|std::fs|std::net`
como "infra": 9.

- **Custo do bloco sugerido inteiro** (seção 3 aplicada aos watchdogs, testado): o cargo e o Clippy
  aceitaram todas as chaves; **179 achados, 0 em `tests/`**. Os maiores: `indexing_slicing` 62,
  `expect_used` 20, `print_stdout` 10 + `print_stderr` 8 (são a saída legítima de uma CLI: pedem
  `#[expect]` no módulo de saída), `missing_panics_doc` 9, `missing_debug_implementations` 9,
  `map_unwrap_or` 7, `missing_errors_doc` 7, `wildcard_imports` 6, `unreachable_pub` 6,
  `let_underscore_must_use` 5, `disallowed_methods` 4 (`std::env::var` fora do `main`),
  `excessive_nesting` 3, `too_many_lines` 3 (com limite 60). Para os watchdogs, o sensato é adotar
  por etapas: primeiro E4 (`let_underscore_must_use`, `map_err_ignore`, `unwrap_in_result`: 8
  achados), depois o pedantic sem `must_use_candidate`, e só então `indexing_slicing`.

## 6. O que ficou sem confirmação em fonte primária

- **[confirmar]** Data e versão de estabilização da edição 2024 (Rust 1.85): o blog oficial não abriu
  daqui; o comportamento foi testado no 1.97.
- **[confirmar]** Forma de instalar `cargo-deny`/`cargo-machete` no CI (compilar com `cargo install`
  ou baixar binário de release/action): não verificado se os projetos publicam binário assinado.
- **[confirmar]** Os links das páginas publicadas (`doc.rust-lang.org`, `rust-lang.github.io`,
  `embarkstudios.github.io`) foram montados a partir dos fontes clonados; o conteúdo foi lido no
  fonte, mas as URLs não foram abertas (proxy bloqueia esses hosts nesta sessão).
- Ausência de lint de número mágico no Clippy: confirmada por busca no fonte de `clippy_lints/src`
  no commit `4e2a51fa`; não verifiquei se há proposta aberta no repositório.
