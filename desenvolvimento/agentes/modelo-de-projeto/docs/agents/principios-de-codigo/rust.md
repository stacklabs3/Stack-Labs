# Práticas de Rust

Parte dos [princípios de código](../principios-de-codigo.md). Como cada princípio aparece em Rust,
a regra de máquina que o confere e as práticas **só desta linguagem** (RS1 a RS6). Fontes primárias
(Rust Book, Rust API Guidelines, Reference, Clippy), testes e o porquê de cada linha: pesquisa
`desenvolvimento/research/2026-10-23-praticas-rust.md` do Stack-Labs.

No grilling, cada linha é a recomendação da pergunta do princípio; o projeto decide. No diff, a
regra ligada é violação dura; a desligada é julgamento.

## Ferramentas

Todas MIT ou Apache-2.0, sem plano pago.

| Ferramenta | Versão | Papel |
|---|---|---|
| Clippy, lints do rustc, rustfmt | do toolchain (1.97) | Lint e formatador; nível na tabela `[lints]` do `Cargo.toml` |
| `cargo-deny` | 0.20.2 | Vulnerabilidade, licença, origem, duplicata e crate proibido |
| `cargo-machete` | 0.9.2 | Dependência sem uso (o `cargo-udeps` exige nightly) |
| `thiserror` / `anyhow` | 2.0.21 / 1.0.104 | Erro com tipo no domínio / erro de aplicação na borda |
| `ast-grep`, `jscpd` | 0.51.0, 5.4.2 | Validador |

**Nível no `Cargo.toml`, prova no `comando`.** A regra adotada entra em `[lints.clippy]` ou
`[lints.rust]` com `"deny"`, que o `lint-cobre` confere. O código a cumpre quando
`cargo clippy --all-targets --locked -- -D warnings` passa. A tabela `[lints.cargo]` ainda é
ignorada no cargo estável.

**Lint por pasta.** Um lint vale só no domínio com `#![deny(clippy::<lint>)]` no `mod.rs` dele; a
exceção pontual é `#[expect(clippy::<lint>, reason = "...")]`, que avisa quando deixa de ser preciso.

## Por princípio

| Princípio | Em Rust | Confere |
|---|---|---|
| E1 (obrigatório) | Nenhum nome de uma letra, nem em laço ou closure; só `_` | `min_ident_chars` com `allowed-idents-below-min-chars = []` no `clippy.toml` (o padrão libera `i`, `j`, `x`, `y`, `z`, `w`, `n`); ast-grep `parametro-de-tipo-sem-nome.yml` para o genérico |
| E1 | Caixa da RFC 430; `as_`, `to_`, `into_` dizem o custo; getter sem `get_` | `nonstandard_style`, `wrong_self_convention`; `termo-evitado` |
| E2 | Função curta; poucos parâmetros; pouco aninhamento; saída cedo com `let ... else` | `too_many_lines`, `too_many_arguments`, `excessive_nesting` (só age com `excessive-nesting-threshold`), `manual_let_else`; limites no `clippy.toml` |
| E3 | Newtype para conceito do domínio, com campos privados e construtor que valida; nada de `bool` solto nem `serde_json::Value` no domínio | `fn_params_excessive_bools`, `struct_excessive_bools`; `disallowed_types` com `#![deny]` no domínio; o newtype é do revisor |
| E4 | Erro esperado volta em `Result` com tipo próprio (`thiserror`), nunca `()`, `String` ou `anyhow` no domínio; `Result` nunca descartado; causa nunca perdida; ramo `Err` vazio é `catch` vazio | `let_underscore_must_use`, `unused_result_ok`, `map_err_ignore`, `result_unit_err`, `unused_must_use`; ast-grep `catch-vazio.yml`, `erro-sem-tipo.yml` |
| E4 (verificador) | `#[allow]` só com `reason`; prefira `#[expect]` | `allow_attributes_without_reason` |
| E5 | `let` sem `mut` por padrão; consulta recebe `&self` | `unused_mut`; `&mut self` desnecessário é do revisor |
| E6 | `todo!()` e `dbg!()` não entram; `TODO` com a issue | `todo`, `unimplemented`, `dbg_macro`; `todo-sem-issue` |
| E7 | Bloco repetido | `npx jscpd@5.4.2 --min-lines 8 --threshold 0 --exit-code 1 {arquivos}` |
| E8 | Número com significado vira `const` | Revisor (o Clippy não tem lint de número mágico) |
| P1, I | Privado por padrão, `pub(crate)` para o resto, `pub` só a interface; quem chama pede só o *bound* que usa | `unreachable_pub` |
| P3, D | Domínio sem crate de infraestrutura; ambiente, relógio, acaso, `exit` e terminal só na montagem | ast-grep `dominio-sem-infra.yml`, `relogio-e-acaso-entram.yml`; `disallowed_methods` (`std::env::var`, `SystemTime::now`) com `#[expect]` no `main`; `exit`, `print_stdout`, `print_stderr` |
| P3, D (mais forte) | Domínio num crate próprio do workspace, sem a dependência de infraestrutura no `Cargo.toml` dele | O compilador recusa; `cargo deny check bans` com `wrappers` |
| P5 | Teste de integração em `tests/` usa só a API pública; mock só no que não é nosso | ast-grep `dominio-sem-infra.yml` apontado para `tests/**` com `mockall` |
| P7 | Dependência sem uso sai; código morto sai | `cargo machete`; `dead_code` com `-D warnings` |
| P8 | Um formatador; uma versão de cada dependência; crates que o projeto exclui | `cargo fmt --all -- --check`; `cargo deny check bans` |
| O | Enum público que vai crescer leva `#[non_exhaustive]`; variante nova é `impl` novo | Revisor |
| L | Adapter cumpre o trait inteiro, sem `unimplemented!()` | `teste-de-contrato-por-port` (`contrato: "tests/contrato_{nome}.rs"`); `unimplemented` |

## Ressalvas

- **Teste pela interface (P5) vence o hábito do `mod tests` em função privada.** O Rust Book permite
  testar função privada; aqui, o teste mora na interface do módulo, e o teste de função privada só
  entra numa seam interna que o próprio módulo usa. O lint `tests_outside_test_module` fica fora.
- **Limite de tamanho (E2) mede a função, não desenha o módulo.** Quebre em funções privadas; o
  `pub` continua pequeno (P1).
- **`pedantic` inteiro é caro.** Nos watchdogs do método deu 84 achados, 31 só de
  `must_use_candidate`; o modelo desliga esse e `module_name_repetitions`. Adote por etapas.
- **`missing_docs` só em crate publicado como biblioteca.** Em binário, choca com E6.

## Só desta linguagem

Cada uma é decidida no ticket *Lint e formatador* do mapa de Regras de código e, adotada, vira regra
com `"principio": "RS<n>"` no `.json` e uma linha na tabela *Práticas da linguagem* do
`docs/regras-de-codigo.md`.

| ID | Prática | Confere |
|---|---|---|
| RS1 | **Sem `unsafe`.** Quando inevitável: num módulo só, com `// SAFETY:` em cada bloco | `[lints.rust] unsafe_code = "forbid"`; `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block` |
| RS2 | **Sem pânico por atalho** no código de produção: `unwrap`, `expect`, `panic!`, `v[i]`, fatia de `str`. O pânico derruba a thread e não aparece na assinatura | `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `string_slice`, `unwrap_in_result`, `panic_in_result_fn`; liberados nos testes pelo `clippy.toml` |
| RS3 | **Árvore de dependências auditada:** vulnerabilidade (RustSec), licença, origem, duplicata | `cargo deny check` |
| RS4 | **Build reprodutível:** `Cargo.lock` no git e todo build com `--locked` | `cargo build --locked` |
| RS5 | **Conversão numérica sem perda silenciosa** (`as` trunca e troca sinal) | `cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`, `cast_precision_loss` |
| RS6 | **Tipo público implementa `Debug`** | `missing_debug_implementations` |

## Modelos

Em `.claude/watchdogs/modelos/`: `regras-de-codigo.rust.json`, `lint/Cargo.lints.toml` (o bloco
`[lints]` para colar no `Cargo.toml`), `lint/clippy.toml` e, em `ast-grep/`, os documentos `Rust`.
Copie só o que o grilling decidiu.
