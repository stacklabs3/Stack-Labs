# Watchdogs, validador e scripts do método em Rust, compilados em cada máquina

Os watchdogs rodam a cada comando e a cada edição do agente, e o validador roda em todo CI e em toda
revisão de issue: o tempo e a memória deles se pagam muitas vezes por dia em todo projeto. Em Node,
cada chamada de hook custava o arranque do Node (cerca de 35 ms e 40 MB), e o validador lia o
repositório com 59 MB. Havia três caminhos: manter Node, reescrever em Rust e distribuir binários
prontos (por release ou commitados), ou reescrever em Rust e **compilar em cada máquina**. O humano
escolheu a terceira, com escopo total: hooks, `pre-push`, validador, instalador do `/setup-stacklabs`
e os scripts das skills recomendadas (`tirar-prints`, `gerar-folha`).

**Consequências:** cada pasta com código é um crate com `Cargo.toml`, `Cargo.lock` commitado e um
lançador `rodar` em `sh` que compila com `cargo build --release --locked` na primeira vez e quando o
fonte muda; `target/` fica fora do git. O requisito da máquina passa de Node a Rust (`cargo`), mais o
`gh` e o `git` de antes; a primeira compilação dos watchdogs leva uns 20 s e precisa de rede para o
crates.io, e por isso o `SessionStart` tem limite de 300 s. Sem `cargo`, os watchdogs falham fechados.
Os watchdogs viraram um binário só em `.claude/watchdogs/` (hooks, `ativar-issue`, `pre-push`,
`validar`), protegido como antes; `.claude/hooks/` e `.claude/validador/` deixaram de existir, e o
instalador os remove de um projeto antigo quando um humano aplica os watchdogs. A troca foi verificada
por equivalência: 73 casos de hook rodados nas duas versões com a mesma entrada e um `gh` falso deram
a mesma saída, o mesmo código e o mesmo estado; o validador deu o mesmo relatório. Medido depois: um
hook passou de 38 ms para 7 ms, e o validador num repositório de 2.000 arquivos de 0,30 s e 59 MB para
0,14 s e 11 MB. Mudar um watchdog agora pede `cargo test` antes da PR.
