# Comandos pesados: trava por projeto, vagas com limite de memória e push sem CI repetido

Emenda as ADRs 0016, 0015 e 0011. A 0016 pôs todo comando pesado numa trava única,
`$HOME/.stacklabs/pesado.lock`, valendo para todos os projetos, com
`NODE_OPTIONS=--max-old-space-size=4096`. Na prática, um projeto segurava o outro: com o dev server,
a suíte ou o `git push` (que roda o CI local) do Excurfly de pé, o Organiza-ai esperava parado, e uma
tarefa de 15 minutos levava horas. A pesquisa
[2026-11-04-memoria-e-tempo-das-validacoes](../research/2026-11-04-memoria-e-tempo-das-validacoes.md)
achou as causas: o teto de 4 GB vale só para o heap do V8, não para a memória total do processo, e é
herdado por cada worker do `next build`, do Jest e do Vitest; a trava guardava o PID de um shell que
morre a cada chamada do agente, enquanto o dev server em segundo plano seguia vivo; sem swap, o
kernel mata quem estiver no caminho, inclusive o shell do agente. No push, a revisão por modelo
(consultiva) era a parte mais lenta, e o mesmo conteúdo passava pelo CI duas ou três vezes.

O humano decidiu (2026-11-04) que dois produtos rodam ao mesmo tempo na mesma máquina e que o push
não repete validação. Todo comando pesado roda por `sh .claude/watchdogs/rodar pesado "<comando>"`:

- **Trava do projeto** (`stacklabs-<projeto>.lock`, o nome da raiz do git): um comando pesado por
  vez em cada projeto, e cada worktree conta como projeto.
- **Vagas da máquina** (`vaga-<n>.lock`): projetos diferentes rodam juntos até o número de vagas, que
  sai da memória total (2 GB de reserva e no mínimo 2,5 GB por vaga: 2 vagas em 8 GB, 5 em 16 GB).
  Antes de entrar, a vaga espera a memória livre chegar a metade dela, por até 2 minutos.
- **Limite de memória de verdade**, no Linux com systemd de usuário: o comando roda num scope com
  `MemoryHigh` e `MemoryMax` do tamanho da vaga, e quem passa dele morre sozinho, não a máquina. Uma
  sonda confere no `memory.max` do próprio scope que o kernel aplicou o limite; sem isso, avisa e
  segue só com a trava.
- **Grupo de processos próprio**: SIGINT, SIGTERM e SIGHUP no `rodar pesado` chegam a todos os
  processos do comando, e o que sobrar quando ele terminar é encerrado.
- As travas usam `flock`, que o sistema solta quando o processo morre. O que roda dentro de um
  comando pesado não espera de novo.
- O heap do Node deixa de ser forçado. O consumo se controla pelo número de workers, na configuração
  de cada projeto (`experimental.cpus` do Next, `maxWorkers` do Jest e do Vitest, `workers` do
  Playwright), e o teto de heap só entra quando a máquina pede (`STACKLABS_HEAP_MB`).

No CI local (emenda a 0015): o `rodar ci` e o pre-push rodam como comando pesado; o CI aprovado com
a árvore limpa grava a chave do conteúdo (árvore do HEAD, commit da base, regras e versão dos
watchdogs), e o pre-push que encontra a chave não roda de novo. O pre-push roda sem a revisão por
modelo, que fica no `rodar ci` à mão, com uma chamada `claude -p` por vez (~220 MB cada, medido).

Nos watchdogs (emenda a 0011): o binário de cada versão do fonte fica guardado em
`~/.stacklabs/watchdogs/<hash>/`; o próximo projeto com o mesmo fonte copia em vez de compilar, e a
compilação usa 2 jobs.

**Consequências:** ninguém mais cola bloco de shell nos prompts; os papéis só mandam passar o
comando pelo `rodar pesado`. Os watchdogs pedem Rust 1.89 (`File::lock`) e ganham a dependência
`libc` (só em Unix). Swap continua necessário no Linux: o guia humano mostra como criar. Limites
conhecidos: SIGKILL no `rodar pesado` não se intercepta, então o comando segue vivo depois que a
trava solta (o scope ainda o limita); sem systemd de usuário (macOS, WSL sem systemd, container), não
há limite por cgroup, só a trava, as vagas e a espera por memória livre; um comando que o limite
matou aparece como SIGKILL, com o aviso de reduzir os workers ou ajustar
`STACKLABS_MEMORIA_POR_VAGA_MB`. O cache do CI só vale para a mesma base: depois do `git fetch` que
move `origin/main`, o push roda o CI de novo. A primeira compilação dos watchdogs ficou mais lenta
(~40 s contra ~29 s com todos os núcleos), mas agora acontece uma vez por versão na máquina, não uma
vez por projeto. Ajustes da máquina, sem mudar o método: `STACKLABS_VAGAS`,
`STACKLABS_MEMORIA_POR_VAGA_MB`, `STACKLABS_HEAP_MB`, `STACKLABS_LIMITE_DE_MEMORIA=desligado`.
