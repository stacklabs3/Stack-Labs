# A trava dos comandos pesados é por projeto, com vagas de memória na máquina

Emenda a ADR 0016. Nela todo comando pesado passava por uma trava única,
`$HOME/.stacklabs/pesado.lock`, valendo para todos os projetos, com
`NODE_OPTIONS=--max-old-space-size=4096`. Na prática, um projeto segurava o outro: com o dev
server, a suíte ou o `git push` (que roda o CI local) do Excurfly de pé, o Organiza-ai esperava
parado, e uma tarefa de 15 minutos levava horas. O teto de 4 GB não protegia: o `NODE_OPTIONS` é
herdado por todo worker (do `next build`, do Jest, do Vitest), e cada um podia crescer até metade de
uma máquina de 8 GB. A trava era um bloco de shell colado nos prompts, com o PID do shell do agente,
que no Claude Code morre a cada chamada enquanto o dev server em segundo plano segue vivo.

O humano decidiu (2026-11-04) que dois produtos rodam ao mesmo tempo. Escolhemos duas camadas, as
duas com `flock`, que o sistema solta quando o processo morre:

- **Trava do projeto** (`stacklabs-<projeto>.lock`): um comando pesado por vez em cada projeto
  (a pasta da raiz do git, então cada worktree conta como projeto).
- **Vagas da máquina** (`vaga-<n>.lock`): projetos diferentes rodam juntos até o número de vagas,
  que sai da memória total (reserva de 2 GB para o sistema e 2,5 GB por vaga: 2 vagas em 8 GB,
  5 em 16 GB).

O comando roda com `--max-old-space-size=2048`. Tudo mora no subcomando
`sh .claude/watchdogs/rodar pesado <comando>`; o `rodar ci` e o pre-push pegam a trava sozinhos, e
o que roda dentro de outro comando pesado não espera de novo. A revisão por modelo do CI cai de 4
para 2 chamadas `claude -p` simultâneas.

**Consequências:** ninguém mais cola o bloco de shell; o prompt dos papéis só manda passar o
comando pelo `rodar pesado`. Os watchdogs passam a exigir Rust 1.89 (`File::lock`). Se o
`rodar pesado` for morto à força, o filho continua e a trava solta antes dele terminar; matar o
comando derruba os dois. Numa máquina sem swap, duas vagas ainda podem somar mais que a memória
livre, se um projeto ignorar o teto do heap: os ajustes de swap, workers e caches das ferramentas
ficam para a pesquisa `research/2026-11-04-memoria-e-tempo-das-validacoes.md`. `STACKLABS_VAGAS` e
`STACKLABS_HEAP_MB` ajustam a máquina sem mudar o método.
