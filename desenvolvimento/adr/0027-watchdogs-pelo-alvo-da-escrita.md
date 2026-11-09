# Os watchdogs do Bash decidem pelo alvo da escrita

Emenda a ADR 0011 (watchdogs em Rust) no `guarda-bash` e no `vigia-codigo` e `vigia-teste`. A retro
de 2026-11-09 no Excurfly, sobre dez sessões de issue (43 subagentes), achou dois defeitos do mesmo
lado:

- o `guarda-bash` barrava todo trecho que citava um caminho protegido e tinha um verbo de escrita.
  Seis bloqueios em três sessões, nenhum alterava um protegido: `rodar ci > "$TMPDIR/log"` (lê os
  watchdogs, grava no `$TMPDIR`), um `cat > brief.md <<'EOF'` cujo texto cita `.claude/agents`, e o
  `python3` que grava o estado da orquestração em `.claude/orquestracao/`;
- o `vigia-codigo` e o `vigia-teste` rodavam só depois de Write e Edit, e 8 de 11 implementadores
  editaram código por `sed -i`, heredoc ou `python`. A regra quebrada só voltava no `rodar validar`.

**Decisão:** um leitor de shell pequeno (`escrita_por_bash.rs`) devolve o alvo de cada escrita do
comando: o destino de `>`, `>>`, `&>`, `tee`, `cp`, `mv`, `ln`, `rm`, `touch`, `chmod`, `truncate`,
`dd of=`, `sed -i`, `perl -i`, `git checkout|restore|rm|mv|stash|clean`, `find -delete`, o de dentro
de `sh -c`, `eval`, `$(...)` e `rodar pesado`, e os caminhos entre aspas nas linhas de um script de
`python`, `node`, `ruby` ou `perl` que chamam função que grava. O corpo do heredoc é texto, salvo
quando vai para um interpretador ou um shell. O `guarda-bash` barra quando um alvo é protegido ou é a
pasta que o contém. Onde o alvo só existe na execução (`xargs rm`, um script que monta o caminho
numa variável), continua barrando pelo texto: script que grava e cita um protegido, `xargs` ou
`find -exec` que gravam num comando que cita um. Os dois vigias passam a ouvir o Bash e conferem os
mesmos alvos que existem no projeto como arquivo.

**Consequências:** o agente lê, registra e copia dos protegidos sem bloqueio, e a edição por Bash
recebe a regra na hora. O leitor não é um shell completo: caminho montado em variável dentro de um
script que não cita o protegido escapa, como já escapava antes (a regra 21 e o `vigia-do-metodo` do
CI seguem valendo). O `guarda-edicao` (issue ativa, contrato da fatia) continua só no Write e no Edit,
por isso a constituição e o implementador mandam editar código com elas. Os projetos recebem a
mudança pelo `/setup-stacklabs`, que troca os watchdogs e o `.claude/settings.json`.
