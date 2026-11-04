# Memória e tempo das validações: trava da máquina, NODE_OPTIONS e CI local

Pesquisa de 2026-11-04. Pergunta: numa máquina de desenvolvimento com 8 GB de RAM e (provavelmente)
sem swap, onde vários produtos da casa (Next.js/TypeScript, Dart/Flutter, Rust, Java) rodam com
agentes do Claude Code, só um produto anda por vez e as validações são lentas. O que, com apoio em
fonte primária, **reduz o uso de memória** e **reduz o tempo das validações**, e em que arquivo do
método cada mudança mora?

O que o método faz hoje (lido no código e nos documentos deste repositório):

1. **Trava global** (ADR 0016; skill `orquestrar-issues`, seção *A trava da máquina*; papel
   `implementador`): todo comando pesado de qualquer projeto roda dentro de
   `$HOME/.stacklabs/pesado.lock`, um por vez na máquina. É um `mkdir` atômico com o PID `$$` do
   shell, espera de 15 s em laço, limpeza da trava de dono morto por `kill -0`, e `trap` que apaga a
   trava quando o shell sai. O bloco é colado no prompt e roda no shell transitório de cada chamada
   de Bash do agente. `next dev` segura a trava enquanto está de pé.
2. **`NODE_OPTIONS=--max-old-space-size=4096`** em todo comando pesado.
3. **Sem swap**: o incidente de 2026-09-13 (dois `next build` ao mesmo tempo derrubaram dev servers,
   build e o shell do agente) é o motivo da trava.
4. **CI local no pre-push** (ADR 0015; `.claude/watchdogs/src/ci.rs`, `validador.rs`,
   `revisao_por_modelo.rs`, `pre_push.rs`): (a) as regras `comando` de `docs/regras-de-codigo.json`
   rodam **em sequência**, cada uma num `sh -c` com `.output()` (`validador.rs`, `rodar_no_shell`);
   nos modelos estão `npx jscpd@5.4.2 ... {arquivos}`, `cargo clippy`, `cargo install --locked
   --quiet cargo-machete@0.9.2 && cargo machete`, `cargo install --locked --quiet cargo-deny@0.20.2
   && cargo deny check`, `cargo fmt --check`, `dart analyze`, `dart run dependency_validator`,
   `mvn ... compile ...checkstyle...pmd`, `mvn ... test -Dtest='*ArquiteturaTest'`; (b) o ast-grep,
   um processo por arquivo de regra; (c) a revisão por modelo, em lotes de **4 `claude -p`
   simultâneos** (`CHAMADAS_SIMULTANEAS = 4`), prazo de 120 s cada, com cache em
   `.claude/state/revisao-de-testes`; (d) `.claude/watchdogs/rodar` compila os watchdogs com
   `cargo build --release --locked` (`lto = "thin"`, `opt-level = 3`) **em cada projeto**, na
   primeira vez e sempre que o fonte muda, cada um com o seu `target/`.
5. O mesmo commit passa por validação mais de uma vez: o `implementador` roda `rodar validar --base`
   (que já inclui as regras `comando`), o `revisor` roda `rodar ci --base <integração>`, e o
   `git push` dispara o `pre-push`, que roda `rodar ci` de novo contra `origin/main`
   (`implementador.md`, `revisor.md`, `pre_push.rs`).

Legenda: **[medido]** = número tirado neste container com o comando dado na seção 9;
**[fonte]** = dito pela fonte primária citada; **[inferido]** = conclusão minha a partir das duas,
sem medição na máquina real; **[confirmar]** = não achei a fonte primária nesta passada.

## Resultado

| Proposta | Fontes que sustentam | Fontes que contestam ou limitam | Conclusão |
|---|---|---|---|
| 1. Pôr swap (swapfile + zswap, ou zram) na máquina Linux | systemd-oomd: "highly recommended for the system to have swap enabled"; sem swap o sistema entra em *livelock* mais depressa; kernel: zswap é para "desktop/laptop users with limited RAM"; zram serve "as swap disks" | Swap não cria memória: troca morte por lentidão; zram gasta CPU e RAM própria; no macOS o compressor com swap já vem ligado (XNU), então a conclusão é só para Linux | **Sustentada** (Linux). Primeira ação, fora do método: configuração da máquina |
| 2. Teto de memória por comando (cgroup v2 via `systemd-run --user --scope -p MemoryHigh= -p MemoryMax=`) | Kernel: `memory.high` freia e nunca chama o OOM killer; `memory.max` chama o OOM killer **dentro** do cgroup; `memory.oom.group` mata o grupo inteiro; systemd entrega o controlador `memory` ao usuário (`Delegate=pids memory cpu` em `user@.service`) | Só Linux com systemd e cgroup v2; `memory.high` "may be breached"; não há equivalente simples no macOS | **Sustentada** como rede de proteção: o OOM mata o build, não o shell do agente |
| 3. Tirar o `--max-old-space-size=4096` global e limitar workers | Node: a flag limita só a "old memory section" do V8; o Next repassa `NODE_OPTIONS` (inclusive a flag) a cada worker, que por padrão são `CPUs − 1`; [medido] processo com `--max-old-space-size=256` chegou a 1,18 GB de RSS com `Buffer` | Abaixar demais faz o build morrer por *heap out of memory*; o valor certo depende do projeto | **Sustentada**: o teto de 4 GB por worker numa máquina de 8 GB não protege nada; trocar por limite de workers e cgroup |
| 4. Trocar o mutex global por vagas com `flock` + admissão por `MemAvailable`, num subcomando Rust | `flock(2)`: a trava acompanha o descritor, herdado por `fork`, e cai quando o último fecha; [medido] `flock` segura enquanto o filho vive e solta quando ele morre; `MemAvailable` é a estimativa do kernel para "starting new applications, without swapping"; `File::lock` estável no Rust 1.89; jobserver do GNU make é o precedente de vagas por máquina | `MemAvailable` é estimativa; admissão não prevê o pico de um build; [medido] a trava atual por `mkdir`+`$$` some quando o shell sai e o processo em segundo plano continua | **Sustentada**: a trava atual não protege o caso que mais pesa (dev server em segundo plano) |
| 5. Revisão por modelo numa chamada só (ou menos simultâneas) | [medido] um `claude -p` com as flags do `revisao_por_modelo.rs` = 219 MiB de pico; quatro juntos = 872 MiB; docs do Claude Code: requisito de 4 GB+ de RAM | Lote maior junta testes num prompt só (risco de *lost in the middle*, pesquisa de 2026-10-25); o cache por teste precisa continuar por teste | **Sustentada**: ~0,9 GB para uma etapa consultiva é caro numa máquina de 8 GB |
| 6. Binário dos watchdogs compilado uma vez por máquina | [medido] build limpo: 21–29 s e 870–930 MiB de pico com `-j 4`; 73 s e 331 MiB com `-j 1`; `CARGO_TARGET_DIR`/`--target-dir` (man `cargo-build`) | ADR 0011 e 0020 pedem a mesma versão por projeto; o binário compartilhado precisa ser chaveado pelo fonte | **Sustentada**, custo baixo: chave = hash do fonte |
| 7. Cache do resultado do `ci` por árvore git | O método já faz isso na revisão por modelo (ADR 0015); [inferido] o mesmo commit é validado 2–3 vezes no fluxo | Regra que lê algo fora do git (rede, `osv-scanner`, `cargo deny` com banco de avisos) muda sem o código mudar | **Sustentada** para as regras puras de código; regras com rede ficam fora do cache ou com validade curta |
| 8. Instalar as ferramentas uma vez, fora do `ci` | `cargo install` não reinstala versão já instalada (man `cargo-install`); [medido] 2.ª vez 0,1–0,2 s, 1.ª vez 41,7 s (machete) e ~150 s (deny, inferido); [medido] `npx jscpd` gasta ~0,8 s, o jscpd 5 trabalha em 17–21 ms (motor em Rust) | Ganho só na primeira execução e a cada troca de versão | **Sustentada**, ganho pequeno no dia a dia e grande no primeiro push |
| 9. Regras `comando` em paralelo, dentro do orçamento | `std::thread::scope` já é usado na revisão; as regras são independentes | Paralelo soma picos de memória: só com a admissão da proposta 4 | **Sustentada com a 4**; antes dela, não |
| 10. Caches e "só o que mudou" das próprias ferramentas | ESLint `--cache`; `tsc --incremental`/`.tsbuildinfo` e referências de projeto (wiki do TypeScript); Jest `--changedSince`/`--findRelatedTests`; Vitest `--changed`; TypeScript 7 (nativo) já é o `latest` no npm | Seleção por mudança erra com dependência dinâmica (Jest `--onlyChanged` exige grafo estático); o Vitest avisa dos arquivos que não rastreia | **Sustentada**, por projeto, no `regras-de-codigo.json` |
| 11. Linters mais rápidos (oxlint, Biome) | Benchmark do próprio oxc: "50x - 100x faster than ESLint" | Alegação de fornecedor; a cobertura das regras usadas (`lint-cobre`: `id-length`, `max-depth`, `react-hooks/*`) não foi conferida | **Não adotar agora**: medir antes, só se o ESLint aparecer como gargalo |
| 12. Mover o pesado para GitHub Actions | Runner padrão: 2 vCPU/8 GB em repositório privado (minutos cobrados), 4 vCPU/16 GB em público (grátis) | A ADR 0015 escolheu CI local de propósito; o runner privado tem a mesma RAM da máquina | **Decisão do humano**: só a suíte longa e o e2e, e exige reabrir a ADR 0015 |

## 1. Por que 8 GB sem swap derruba tudo

- **Sem swap não há degradação, há morte.** O manual do `systemd-oomd` diz que é "highly
  recommended for the system to have swap enabled", porque com swap "the system spends enough time
  swapping pages to let systemd-oomd react. Without swap, the system enters a livelocked state much
  more quickly and may prevent systemd-oomd from responding in a reasonable amount of time" [fonte:
  `man/systemd-oomd.service.xml`]. O `earlyoom` existe porque o OOM killer do kernel só age quando
  "it has absolutely no other choice" e não dá para configurá-lo para agir antes [fonte: README do
  earlyoom]. O resultado é o incidente de 2026-09-13: o kernel escolhe a vítima pelo `oom_score`, e
  ela pode ser o shell do agente.
- **zswap**: "a lightweight compressed cache for swap pages", que "trades CPU cycles for potentially
  reduced swap I/O"; benefício listado: "Desktop/laptop users with limited RAM capacities can
  mitigate the performance impact of swapping" [fonte: `Documentation/admin-guide/mm/zswap.rst`].
  Precisa de um dispositivo de swap atrás (swapfile).
- **zram**: dispositivo de bloco em RAM, comprimido, com uso "as swap disks" entre os casos [fonte:
  `Documentation/admin-guide/blockdev/zram.rst`]. Não precisa de disco, mas a RAM comprimida
  disputa com os processos.
- **macOS**: o compressor de memória com swap é o padrão (`vm_compressor_mode =
  VM_PAGER_COMPRESSOR_WITH_SWAP` quando `XNU_TARGET_OS_OSX`, em `osfmk/vm/vm_compressor.c` do XNU),
  e o guia de memória da Apple diz que "OS X supports a backing store" [fonte]. Num Mac, "sem swap"
  não é o estado normal; o incidente teria outra causa (pressão de memória com compressor e swap
  cheios). **As propostas 1 e 2 são de Linux.** As propostas 3 a 12 valem nos dois sistemas.
  **[confirmar]** qual é o sistema da máquina do desenvolvedor (`uname -a`; `swapon --show` ou
  `sysctl vm.swapusage`).

## 2. Teto por cgroup: o OOM mata o build, não o agente

- `memory.high`: "Memory usage throttle limit. If a cgroup's usage goes over the high boundary, the
  processes of the cgroup are throttled and put under heavy reclaim pressure. Going over the high
  limit never invokes the OOM killer and under extreme conditions the limit may be breached"
  [fonte: `Documentation/admin-guide/cgroup-v2.rst`].
- `memory.max`: "Memory usage hard limit. ... If a cgroup's memory usage reaches this limit and
  can't be reduced, the OOM killer is invoked in the cgroup" [fonte: idem]. É isso que protege o
  shell do agente: a vítima sai de dentro do cgroup do build.
- `memory.oom.group`: "all tasks belonging to the cgroup ... are killed together or not at all"
  [fonte: idem]. Evita um build meio morto com workers órfãos.
- systemd: `MemoryHigh=` "is the main mechanism to control memory usage of a unit"; `MemoryMax=`
  "out-of-memory killer is invoked inside the unit. It is recommended to use MemoryHigh= as the main
  control mechanism and use MemoryMax= as the last line of defense"; aceitam porcentagem da memória
  física [fonte: `man/systemd.resource-control.xml`]. `systemd-run --scope` cria uma unidade
  `.scope` transitória e `-p` define as propriedades dela [fonte: `man/systemd-run.xml`]. Para o
  usuário comum, o `user@.service` do systemd tem `Delegate=pids memory cpu` [fonte:
  `units/user@.service.in`], então `systemd-run --user --scope -p MemoryHigh=3G -p MemoryMax=4G --
  <comando>` funciona sem root **[inferido; não testado: este container não roda systemd como
  init]**.
- **Leitura para o método**: a trava de hoje evita o OOM por exclusão (um por vez); o cgroup evita
  que o OOM, quando vier, mate o que não devia. Os dois se somam. Fora do Linux com systemd, o
  subcomando da proposta 4 roda o comando sem o cgroup e avisa.

## 3. `--max-old-space-size` e os workers

- **O que a flag limita**: "Sets the max memory size of V8's old memory section. As memory
  consumption approaches the limit, V8 will spend more time on garbage collection" [fonte: Node,
  `doc/api/cli.md`]. Não é teto de RSS: `Buffer`, código nativo, Turbopack (Rust) e o próprio
  binário ficam fora. [medido] `node --max-old-space-size=256` alocando 12 `Buffer` de 100 MB chegou
  a **1,18 GB de RSS** e terminou sem erro.
- **O padrão já é alto**: [medido] neste container (16 GB), o `heap_size_limit` padrão do Node 22 é
  2096 MiB; com `--max-old-space-size=4096`, 4144 MiB. O padrão depende da memória física
  [inferido]; na máquina de 8 GB, medir com `node -e "console.log(require('v8').getHeapStatistics().heap_size_limit/2**20)"`.
  O Node também tem `--max-old-space-size-percentage`, relativo à memória disponível [fonte: idem].
- **Herança**: `NODE_OPTIONS` é lido por todo processo Node, e `child_process` passa
  `process.env` aos filhos por padrão ("**Default:** `process.env`") [fonte: `doc/api/child_process.md`].
  O Next.js monta o ambiente de cada worker com `...process.env` e `NODE_OPTIONS` reformatado, e
  **só remove `max-old-space-size` quando `isolatedMemory` é verdadeiro** [fonte:
  `packages/next/src/lib/worker.ts`]. O número de workers padrão é `os.availableParallelism() − 1`
  (`cpus` em `config-shared.ts`) [fonte]. Numa máquina de 8 núcleos, são até 7 workers, cada um com
  teto de heap de 4 GB [inferido: o teto vale por processo; o total possível passa muito da RAM].
- **Guia oficial de memória do Next.js** (`docs/01-app/02-guides/memory-usage.mdx`) [fonte]:
  `experimental.webpackMemoryOptimizations: true` desde a v15.0.0 ("reduces max memory usage but may
  increase compilation times by a slight amount", "low-risk"); `next build
  --experimental-debug-memory-usage` desde a 14.2.0 (imprime heap e GC durante o build, e tira
  *heap snapshots* perto do limite); worker de build do Webpack ligado por padrão desde a v14.1.0;
  desligar o cache do Webpack; `typescript.ignoreBuildErrors` quando o OOM é no passo "Running
  TypeScript" (com o aviso de que exige checagem de tipos em outro lugar); desligar source maps
  (`productionBrowserSourceMaps`, `experimental.serverSourceMaps`, `enablePrerenderSourceMaps`);
  `experimental.preloadEntriesOnStart: false` para o servidor. A página **não cita**
  `experimental.cpus`; ele existe em `config-shared.ts`.
- **Turbopack é o padrão**: na CLI do Next 16 (`latest` no npm: 16.4.5), `--turbopack` está
  "enabled by default" em `next dev` e `next build`, e `--webpack` volta ao Webpack [fonte:
  `docs/.../06-cli/next.mdx`]. As opções `webpack*` acima só valem com `--webpack`. Turbopack é
  Rust: a memória dele não obedece à flag do V8 [inferido].
- **Testes** [fonte: docs de cada um]: Jest `maxWorkers` padrão "number of the cores ... minus one"
  e `workerIdleMemoryLimit` recicla o worker que passou do limite depois de cada arquivo; Vitest
  `maxWorkers` padrão "all available parallelism" fora do watch, `pool` padrão `forks`;
  Playwright `--workers`/`workers` (o exemplo oficial usa 2 no CI e `workers: 1` desliga o
  paralelismo).
- **TypeScript**: `--incremental` grava o estado em `.tsbuildinfo`; referências de projeto dividem o
  programa; `--extendedDiagnostics` imprime "Memory used" [fonte: wiki *Performance* do TypeScript].
  O TypeScript 7 (o port nativo) é o `latest` no npm (7.0.2, 2026-07-08), com binários por
  plataforma em `optionalDependencies`; o repositório `typescript-go` diz que o port "is now
  completed" e lista *incremental build* e *project references* como "done" [fonte: registry npm;
  README do `microsoft/typescript-go`]. Sendo binário nativo, não lê `--max-old-space-size`
  [inferido]. O ganho de velocidade anunciado pela Microsoft (~10x) **[confirmar]**: o post do
  anúncio não pôde ser aberto nesta sessão.
- **Maven**: `MAVEN_OPTS` ou `.mvn/jvm.config` (por projeto, versionado) passam opções da JVM, como
  `-Xmx` [fonte: `maven-site`, `configure.md`]. **Gradle** (`org.gradle.jvmargs`, daemon) e o
  consumo do `dart analyze` **[confirmar]**: as páginas oficiais não abriram nesta sessão.

## 4. A trava atual e uma trava que acompanha o processo

**Defeitos do `mkdir` + `$$`** (lidos no bloco da skill, conferidos por medição):

- O dono gravado é o PID do shell transitório da chamada de Bash, e o `trap ... EXIT` apaga a trava
  quando esse shell sai. [medido] Um `sleep 30 &` lançado dentro da trava continuou vivo depois que
  o shell saiu, e a trava já tinha sumido. Isso é o `next dev` em segundo plano: ocupa memória sem
  segurar a trava.
- Polling de 15 s: até 15 s ociosos por espera [inferido].
- `kill -0` num PID reutilizado dá trava "viva" falsa [inferido].
- Um por vez **na máquina inteira**: um `cargo fmt --check` de um projeto espera o `next build` de
  outro, embora caibam os dois.

**`flock`**: "the lock is released either by an explicit LOCK_UN operation on any of these duplicate
file descriptors, or when all such file descriptors have been closed"; descritores duplicados por
`fork(2)` compartilham a trava [fonte: man-pages `flock.2`]. O `flock(1)` envolve um comando e
espera por padrão; `-o/--close` evita que filhos herdem a trava [fonte: util-linux `flock.1.adoc`].
[medido] `flock -n arquivo sleep 37 &` segurou a trava depois que o shell que o lançou saiu, e o
kernel a soltou quando o processo morreu por sinal. Não há dono morto a limpar nem PID a conferir.
O `flock(1)` não vem no macOS [inferido: é do util-linux]; por isso a trava fica no binário Rust:
`File::lock`, `File::try_lock` e as versões `_shared` foram estabilizadas no **Rust 1.89.0**
(2025-08-07) [fonte: `RELEASES.md` do rust-lang/rust]. Crates prontos, se preciso: `fs4` 1.1.0 e
`fd-lock` 4.0.4 (MIT OR Apache-2.0, `cargo info`).

**Vagas em vez de mutex**: o GNU make limita o total de jobs "even across recursive invocations"
por um *jobserver* passado no ambiente (`MAKEFLAGS`, `--jobserver-auth=`), e quem participa devolve
as vagas "even under error conditions" [fonte: manual do GNU make, *Job Slots*]. O cargo herda o
jobserver do ambiente (`RELEASES.md`: "Inherit jobserver from env"). Lição: vaga em pipe se perde
quando o dono morre sem devolver; N arquivos travados por `flock` não têm esse defeito, porque o
kernel devolve [inferido a partir das duas fontes].

**Admissão por memória**: `MemAvailable` é "an estimate of how much memory is available for starting
new applications, without swapping", calculada de MemFree, SReclaimable, LRU de arquivos e marcas
d'água [fonte: `Documentation/filesystems/proc.rst`]. É o número certo para decidir se um comando
entra; não prevê o pico dele [inferido]. Desenho proposto:

```
sh .claude/watchdogs/rodar pesado [--orcamento 3G] -- <comando>
  1. trava do projeto: ~/.stacklabs/projetos/<hash da raiz>.lock (flock exclusivo) —
     um build por projeto, como hoje;
  2. vaga da máquina: tenta ~/.stacklabs/vagas/0..N-1.lock (flock); N padrão = 2 com 8 GB;
  3. admissão: entra só se MemAvailable − orçamento ≥ reserva (1 GB); senão espera
     (re-tenta ao soltar uma vaga, sem polling fixo de 15 s);
  4. Linux com systemd: roda dentro de systemd-run --user --scope -p MemoryHigh=<orç.>
     -p MemoryMax=<orç.+25%>; senão roda direto e avisa;
  5. o rodar fica como pai e espera o filho: a trava vive exatamente o tempo do comando,
     inclusive `next dev`; no fim (ou na morte) o kernel solta as duas.
```

No macOS não há `MemAvailable`; a admissão usa `vm_stat`/`sysctl` ou fica desligada **[confirmar]**.

## 5. Revisão por modelo: memória dos `claude -p`

- [medido] Um `claude -p` com as mesmas flags de `perguntar_ao_modelo` (Haiku, `--tools ""`,
  `--json-schema`, `--no-session-persistence`): **219 MiB de pico, 6,3 s**. Quatro simultâneos (o
  lote de `CHAMADAS_SIMULTANEAS`): **872 MiB de pico, 11,3 s**. O `claude` 2.1.318 instalado aqui é
  um executável nativo de 338 MB (instalação nativa, "Recommended" nos docs) [fonte: docs de
  *setup* do Claude Code, que pedem "4 GB+ RAM"].
- **Uma chamada para N testes**: o esquema passaria a ser uma lista de `{teste, exercita, motivo}`;
  o cache continua por teste (gravado item a item da resposta). Memória: ~0,2 GB em vez de ~0,9 GB;
  tempo: uma partida do CLI em vez de N [inferido]. Risco: prompt longo dilui a atenção (Liu et
  al., *Lost in the Middle*, citado na pesquisa de 2026-10-25); limitar o lote (ex.: 8 testes) e
  medir a concordância contra o modo atual antes de trocar.
- **Alternativa mínima**: `CHAMADAS_SIMULTANEAS = 1` ou configurável (`parametros`), pois a etapa é
  consultiva e falha aberta.
- `--bare` deixa a partida mais rápida, mas a ADR 0015 o recusou por exigir `ANTHROPIC_API_KEY`;
  continua valendo.

## 6. Watchdogs: compilar uma vez por máquina

- [medido] Build `--release` limpo, `lto = "thin"`: 21–29 s de parede e 870–930 MiB de pico somado
  com `-j 4`; com `-j 1`, 73 s e 331 MiB. Sem LTO (`CARGO_PROFILE_RELEASE_LTO=false`): 17,5 s,
  943 MiB, binário 3,37 MB contra 3,39 MB. Depois de tocar um arquivo de `src/`: 15 s. O `target/`
  ocupa 79 MB **por projeto**.
- O `-j`/`build.jobs` padrão é "the number of logical CPUs"; `--target-dir`/`CARGO_TARGET_DIR` muda
  o diretório dos artefatos [fonte: man `cargo-build`, do toolchain 1.97].
- **Proposta**: `rodar` calcula o SHA-256 de `src/`, `Cargo.toml` e `Cargo.lock` e procura
  `~/.stacklabs/watchdogs/<hash>/watchdogs`; se não existe, compila com
  `CARGO_TARGET_DIR=~/.stacklabs/watchdogs/target -j 2` (dentro da `rodar pesado`) e copia. Projetos
  na mesma versão do método (ADR 0020) dividem o binário; versão diferente gera outro hash. O LTO
  não pesa na memória aqui (o pico vem do paralelismo); pode ficar.

## 7. Tempo do CI local

**Onde o tempo está** [medido, crate dos watchdogs como projeto Rust de teste, seção 9]: o primeiro
`rodar ci` levou **162 s** e 1,06 GB de pico; o segundo, **2,9 s**. A diferença é quase toda o
`cargo install` do `cargo-deny` na primeira vez (≈150 s, inferido por subtração; o do
`cargo-machete` medido sozinho: 41,7 s e 1,03 GB). O `cargo clippy` frio: 6,5 s; sem mudança:
0,2 s; com um arquivo tocado: 0,8 s (incremental do perfil dev). O ast-grep com as 17 regras em 23
arquivos: 0,5 s. Num projeto TS/Next real, o peso deve estar em `tsc`, ESLint e testes, que este
container não tem [inferido; medir no Excurfly].

1. **Ferramentas fora do `ci`**: "If the package is already installed, Cargo will reinstall it if
   the installed version does not appear to be up-to-date" (versão, binários, features, perfil,
   alvo) [fonte: man `cargo-install`]. [medido] Já instalado: 0,1–0,2 s. O custo é na primeira vez e
   a cada troca de versão, e cai dentro do pre-push. Instalar no `/setup-stacklabs` (ou num
   `rodar instalar`) e deixar no `comando` só `cargo deny check`/`cargo machete`, com mensagem clara
   quando faltar.
2. **`npx` → binário fixo**: o jscpd 5 é "a Rust engine that ships as a self-contained binary"
   [fonte: README do jscpd]. [medido] A detecção leva 17–21 ms; o `npx jscpd@5.4.2` inteiro, 0,8–0,9
   s com cache e 2,0 s frio. Instalar como devDependency fixa ou binário do crates.io tira a
   resolução do `npx` [inferido].
3. **Cache por árvore git** (proposta 7): chave = SHA-256 da linha de comando + `git rev-parse
   HEAD^{tree}` + diff do working tree + base; resultado aprovado grava em
   `.claude/state/ci/`. Com isso, o `pre-push` depois do `revisor` não repete o que já passou. Regras
   que consultam rede (`cargo deny check` com banco de avisos, `osv-scanner`) ficam fora ou com
   validade de um dia.
4. **`comando` em paralelo**: hoje um `for` sequencial em `validador.rs`. Com a admissão da seção 4,
   rodar em paralelo as regras declaradas leves (fmt, jscpd, machete) e em série as pesadas
   (clippy, `mvn`, `tsc`). Ganho = soma das leves [inferido].
5. **Caches das ferramentas** (no `regras-de-codigo.json` de cada projeto): ESLint `--cache` ("can
   dramatically improve ESLint's run time", grava `.eslintcache`; `--cache-strategy content` para
   não depender de mtime) [fonte: docs do ESLint]; `tsc --incremental` [fonte: wiki do TypeScript];
   testes só dos afetados: Jest `--changedSince <base>`/`--findRelatedTests` [fonte: CLI do Jest],
   Vitest `--changed <base>` (roda tudo quando mudam `setupFiles`, config ou `package.json`)
   [fonte: `docs/config/changed.md`]. A suíte completa continua no revisor ou no GitHub.
6. **oxlint/Biome**: o benchmark do próprio projeto oxc diz "Oxlint is 50x - 100x faster than ESLint
   depending on the number of CPU cores" e "~2x faster than Biome" [fonte de fornecedor:
   `oxc-project/bench-linter`]. Não conferi se cobrem `id-length`, `max-depth`, `react-hooks/*`,
   que o `lint-cobre` exige. Só vale se a medição mostrar o ESLint como gargalo.
7. **GitHub Actions**: runner padrão para repositório privado: Linux, 2 CPUs, 8 GB, minutos da
   conta e depois cobrados; para público: 4 CPUs, 16 GB, grátis [fonte: `github/docs`,
   `supported-github-runners.md`]. Em repositório privado, o runner tem a mesma RAM da máquina. A
   ADR 0015 escolheu o CI local; mover suíte longa e e2e para o Actions alivia a máquina, mas exige
   outra ADR.

## 8. O que não resolve, ou resolve pouco

- Subir `--max-old-space-size` para "dar folga": aumenta o teto por processo, não a RAM.
- Só trocar a trava por outra global mais esperta: o ganho maior é permitir dois comandos que
  cabem, e isso precisa de orçamento por comando.
- earlyoom/systemd-oomd sem swap: o próprio oomd avisa que sem swap pode não reagir a tempo
  (seção 1). Com o cgroup da seção 2, o kernel já mata dentro do escopo certo.

## 9. Medições neste container

**Não é a máquina do usuário**: Linux 6.18, 4 vCPU, 16 GB, **sem swap**, cargo/rustc 1.97.0, Node
22.23.4, Claude Code 2.1.318. Os números servem para ordem de grandeza e comparação entre opções.
"Pico somado" = maior soma de `VmRSS` da árvore de processos, amostrada a cada 50 ms em `/proc` por
um script de 50 linhas (`medir.py`, no scratchpad da sessão, fora do repositório); "maior processo"
= `ru_maxrss` dos filhos. Cópias do crate em pasta temporária; nada foi escrito no repositório.

| Comando | Parede | Pico somado | Maior processo |
|---|---|---|---|
| `cargo build --release --locked --quiet -j 4` (watchdogs, limpo) | 28,7 s / 21,4 s | 930 / 869 MiB | 313 MiB |
| idem `-j 1` | 72,7 s | 331 MiB | 299 MiB |
| idem `-j 4` com `CARGO_PROFILE_RELEASE_LTO=false` | 17,5 s | 943 MiB | 304 MiB |
| rebuild depois de `touch src/ci.rs` | 15,1 s | 344 MiB | 314 MiB |
| `cargo clippy --all-targets --locked -- -D warnings` frio / sem mudança / 1 arquivo | 6,5 / 0,2 / 0,8 s | 694 / 58 / 439 MiB | 251 MiB |
| `cargo install --locked --quiet cargo-machete@0.9.2` 1.ª / 2.ª vez | 41,7 / 0,1 s | 1034 / 4 MiB | 341 MiB |
| `npx --yes jscpd@5.4.2 --min-lines 8 --threshold 0 --exit-code 1 <3 arquivos>` frio / com cache | 2,0 / 0,9 s | 223 / 147 MiB | 163 MiB |
| `node -e "setTimeout(()=>{},1500)"` (Node ocioso) | 1,6 s | 40 MiB | 40 MiB |
| `node --max-old-space-size=256 -e "<12 Buffer.alloc(1e8)>"` | 1,6 s | 1184 MiB | 1188 MiB |
| `claude -p` com as flags de `revisao_por_modelo.rs`, 1 chamada | 6,3 s | 219 MiB | 218 MiB |
| idem, 4 simultâneas | 11,3 s | 872 MiB | 218 MiB |
| `watchdogs ci --base origin/main` (modelo Rust, 2 arquivos alterados) 1.ª / 2.ª vez | 162,4 / 2,9 s | 1055 / 170 MiB | 684 MiB |
| 17 × `ast-grep scan --rule <regra> --json=stream src tests` | 0,5 s | — | — |

Projeto de teste do `ci`: o próprio crate dos watchdogs com `docs/regras-de-codigo.json` =
`modelos/regras-de-codigo.rust.json`, regras ast-grep copiadas de `modelos/ast-grep`, `ast-grep`
0.51.0 por `AST_GREP`, `CARGO_INSTALL_ROOT` temporário. O `ci` reprovou (formatação, licença no
`cargo deny`, tabela de cobertura incompleta); o tempo das regras não depende do veredito.
Trava: `sh -c 'mkdir T; echo $$ > T/pid; trap "rm -rf T" EXIT; sleep 30 &'` → trava ausente com o
`sleep` vivo; `sh -c "flock -n F sleep 37 &"` → `flock -n F true` falha enquanto o `sleep` vive e
passa depois de `pkill -x sleep`.

## Recomendação

Em ordem de prioridade. "Arquivo" é onde a mudança mora no método.

### Memória

1. **Swap na máquina Linux: swapfile de 8 GB com zswap ligado** (ou zram, se não houver disco
   sobrando). Efeito: o pico vira lentidão em vez de processo morto; o oomd passa a ter tempo de
   agir. Custo: 10 minutos, uma vez, por máquina. Arquivo: guia humano
   (`desenvolvimento/humanos/`) e uma linha de pré-requisito na ADR 0016. Antes, **confirmar o
   sistema operacional**; num Mac, pular.
2. **Subcomando `rodar pesado -- <comando>` em Rust**, no lugar do bloco de shell: trava por projeto
   + N vagas por máquina com `File::lock`, admissão por `MemAvailable` e orçamento, o `rodar` como
   pai do comando (cobre `next dev`), e `systemd-run --user --scope -p MemoryHigh/MemoryMax` quando
   houver. Efeito: dois projetos andam juntos quando cabem; trava solta na morte; o OOM mata o build,
   não o agente. Custo: médio (~200 linhas de Rust, testes, uma ADR emendando a 0016). Arquivos:
   ADR 0016 (nova ADR), skill `orquestrar-issues` (seção *A trava da máquina* vira uma linha),
   `implementador.md`, `.claude/watchdogs/src/` (novo `pesado.rs`), `docs/agents/orquestracao.md`.
3. **Tirar o `NODE_OPTIONS=--max-old-space-size=4096` do bloco** e pôr o limite onde ele age: nº de
   workers por projeto (`experimental.cpus` do Next, `--maxWorkers`/`maxWorkers` do Jest e do
   Vitest, `workers` do Playwright, todos em 2 numa máquina de 8 GB) e o orçamento do `rodar
   pesado`. Manter um teto de heap só por comando que precise, menor que o orçamento. Efeito: o pior
   caso cai de "N workers × 4 GB" para "2 workers × padrão". Custo: baixo. Arquivos: ADR 0016,
   skill `orquestrar-issues`, `implementador.md`, e o `next.config`/config de testes de cada
   projeto (fora do método).
4. **Revisão por modelo com menos processos**: já, `CHAMADAS_SIMULTANEAS` = 1 (ou parâmetro); depois,
   piloto de uma chamada por lote de até 8 testes, comparado com o modo atual. Efeito: −650 MiB de
   pico [medido: 872 → 219 MiB]. Custo: baixo / médio. Arquivo: `revisao_por_modelo.rs`; ADR 0015
   (registrar a mudança).
5. **Binário dos watchdogs por hash do fonte, em `~/.stacklabs/watchdogs/`, compilado com `-j 2`**.
   Efeito: some o build de 21–29 s e ~0,9 GB a cada projeto novo ou atualizado; −79 MB de disco por
   projeto. Custo: baixo (o script `rodar`). Arquivo: `.claude/watchdogs/rodar`; nota na ADR 0011.

### Tempo

1. **Cache do resultado do `ci` por árvore git** (regras puras de código; as de rede fora). Efeito:
   o `pre-push` depois do `revisor` vira segundos; o mesmo commit deixa de passar 2–3 vezes pelas
   mesmas regras. Custo: médio (~100 linhas, mesmo padrão do cache da revisão). Arquivos:
   `.claude/watchdogs/src/ci.rs`, `validador.rs`; ADR 0015 (consequência nova).
2. **Ferramentas instaladas uma vez**: `cargo install` de `cargo-deny`/`cargo-machete` e o
   `ast-grep` no `/setup-stacklabs`; nos modelos, só `cargo deny check`/`cargo machete`; jscpd como
   binário fixo em vez de `npx`. Efeito: −40 s a −150 s no primeiro push de cada máquina/versão,
   −0,8 s por execução do jscpd. Custo: baixo. Arquivos: `modelos/regras-de-codigo*.json`, skill
   `setup-stacklabs`.
3. **"Só o que mudou" nas ferramentas de cada projeto**: ESLint `--cache --cache-strategy content`,
   `tsc --incremental` (ou TypeScript 7, medido antes), Jest `--changedSince`/Vitest `--changed`
   contra a base no CI local, suíte completa no revisor. Efeito: proporcional ao tamanho do
   projeto; medir no Excurfly antes e depois. Custo: baixo por projeto. Arquivo: o
   `regras-de-codigo.json` do projeto e o modelo TS (`modelos/regras-de-codigo.json`, `lint/`).
4. **Regras `comando` em paralelo dentro do orçamento**, só depois da recomendação 2 de memória.
   Efeito: o tempo do CI vira o da regra mais lenta, não a soma. Custo: médio. Arquivo:
   `validador.rs` (campo `"pesado": true` na regra).
5. **Decisão do humano: suíte longa e e2e no GitHub Actions**, pre-push só com o rápido. Efeito:
   tira da máquina o comando mais pesado. Custo: médio, mais minutos cobrados em repositório
   privado (runner de 2 vCPU/8 GB). Arquivo: nova ADR reabrindo a 0015; workflow no
   `modelo-de-projeto/.github/workflows/`.
6. **Não adotar agora**: trocar ESLint por oxlint/Biome (só alegação de fornecedor, cobertura das
   regras não conferida).

**Medir antes de fechar a ADR**, na máquina real, com o mesmo `medir.py`: `next build` do Excurfly
com `cpus` padrão e com 2, com e sem a flag de 4 GB; `tsc --noEmit` com TS 5/6 e com TS 7; um `rodar
ci` frio e quente. É isso que fixa o N de vagas e os orçamentos.

## Fontes

- Linux, cgroup v2 (`memory.high`, `memory.max`, `memory.oom.group`, `memory.swap.max`):
  <https://docs.kernel.org/admin-guide/cgroup-v2.html> (lido em
  `torvalds/linux/Documentation/admin-guide/cgroup-v2.rst`)
- Linux, `/proc/meminfo` (`MemAvailable`): <https://docs.kernel.org/filesystems/proc.html>
  (`Documentation/filesystems/proc.rst`)
- Linux, zswap: <https://docs.kernel.org/admin-guide/mm/zswap.html>; zram:
  <https://docs.kernel.org/admin-guide/blockdev/zram.html>; sysctl vm (`overcommit_memory`):
  <https://docs.kernel.org/admin-guide/sysctl/vm.html>
- systemd, `systemd.resource-control(5)`, `systemd-run(1)`, `systemd-oomd.service(8)`:
  <https://www.freedesktop.org/software/systemd/man/latest/> (lidos em `systemd/systemd/man/*.xml`);
  `units/user@.service.in` (`Delegate=pids memory cpu`): <https://github.com/systemd/systemd/blob/main/units/user@.service.in>
- earlyoom, README: <https://github.com/rfjakob/earlyoom>
- `flock(2)`: <https://man7.org/linux/man-pages/man2/flock.2.html> (lido em `mkerrisk/man-pages`);
  `flock(1)`: util-linux, `sys-utils/flock.1.adoc` <https://github.com/util-linux/util-linux>
- GNU make, *Sharing Job Slots with GNU make*: <https://www.gnu.org/software/make/manual/html_node/Job-Slots.html>
  (lido em `doc/make.texi`, espelho `mirror/make`)
- Rust 1.89.0, `File::lock`: <https://github.com/rust-lang/rust/blob/main/RELEASES.md>; cargo, man
  `cargo-build(1)` e `cargo-install(1)` do toolchain 1.97 (<https://doc.rust-lang.org/cargo/commands/cargo-build.html>,
  <https://doc.rust-lang.org/cargo/commands/cargo-install.html>); crates `fs4`, `fd-lock` (`cargo info`)
- Apple, XNU `osfmk/vm/vm_compressor.c`: <https://github.com/apple-oss-distributions/xnu>; *About the
  Virtual Memory System*: <https://developer.apple.com/library/archive/documentation/Performance/Conceptual/ManagingMemory/Articles/AboutMemory.html>
- Node.js, `doc/api/cli.md` (`--max-old-space-size`, `--max-old-space-size-percentage`,
  `NODE_OPTIONS`), `doc/api/child_process.md`: <https://nodejs.org/api/cli.html>,
  <https://nodejs.org/api/child_process.html>
- Next.js, *How to optimize memory usage*: <https://nextjs.org/docs/app/guides/memory-usage>
  (`docs/01-app/02-guides/memory-usage.mdx`); CLI: <https://nextjs.org/docs/app/api-reference/cli/next>;
  código: `packages/next/src/lib/worker.ts`, `packages/next/src/server/config-shared.ts`
  (<https://github.com/vercel/next.js>, branch `canary`, lido em 2026-11-04); versão no npm: 16.4.5
- Jest, *Configuration* e *CLI*: <https://jestjs.io/docs/configuration>, <https://jestjs.io/docs/cli>
  (`jestjs/jest/docs`)
- Vitest, `pool`, `maxWorkers`, `changed`, *Improving Performance*, CLI: <https://vitest.dev/config/>
  (`vitest-dev/vitest/docs`)
- Playwright, *Parallelism*: <https://playwright.dev/docs/test-parallel> (`docs/src/test-parallel-js.md`)
- ESLint, *Command Line Interface* (`--cache`, `--cache-strategy`): <https://eslint.org/docs/latest/use/command-line-interface>
- TypeScript, wiki *Performance*: <https://github.com/microsoft/TypeScript/wiki/Performance>;
  `microsoft/typescript-go` README: <https://github.com/microsoft/typescript-go>; registry npm
  `typescript` 7.0.2 e `@typescript/native-preview`
- jscpd, README (motor Rust, v5): <https://github.com/kucherenko/jscpd>
- oxc, benchmark de linters (fornecedor): <https://github.com/oxc-project/bench-linter>
- Maven, *Configuring Apache Maven* (`MAVEN_OPTS`, `.mvn/jvm.config`): <https://maven.apache.org/configure.html>
  (`apache/maven-site`)
- GitHub, runners padrão: <https://docs.github.com/en/actions/reference/runners/github-hosted-runners>
  (`github/docs`, `data/reusables/actions/supported-github-runners.md`)
- Claude Code, *CLI reference* (`--bare`, `--json-schema`, `--tools`, `--max-budget-usd`) e *Setup*
  (requisitos): <https://code.claude.com/docs/en/cli-reference>, <https://code.claude.com/docs/en/setup>
- Neste repositório: ADR 0011, 0015, 0016, 0020; `desenvolvimento/agentes/skills/orquestrar-issues/SKILL.md`;
  `desenvolvimento/agentes/modelo-de-projeto/.claude/agents/{implementador,revisor}.md`;
  `.claude/watchdogs/{rodar,Cargo.toml,src/ci.rs,src/validador.rs,src/revisao_por_modelo.rs,src/pre_push.rs,modelos/regras-de-codigo*.json}`
- **[confirmar]** (não abriram nesta sessão): docs do Gradle (`org.gradle.jvmargs`, daemon), consumo
  de memória do `dart analyze`, post de anúncio do TypeScript nativo (números de velocidade).
