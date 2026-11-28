# A sessão roda as skills do projeto, não cópias pessoais de mesmo nome

No Claude Code, uma skill em `~/.claude/skills/<nome>` vence a de mesmo nome em `.claude/skills/<nome>`
do projeto ([tabela de precedência](https://code.claude.com/docs/en/skills), "personal over project").
O `/setup-stacklabs` instala as skills do método no projeto, mas uma pasta pessoal antiga faz a sessão
rodar outra versão sem ninguém ver. Em 2026-11-27, numa máquina da casa, 54 skills dos projetos da
casa tinham homônima pessoal; no Excurfly, 23 diferiam, entre elas `tdd`, `to-tickets`,
`code-review`, `grilling`, `implement` e `triage`
([pesquisa](../research/2026-11-28-home-repo-do-matt.md), seção "O diretório do usuário nesta
máquina"). O humano decidiu que a sessão segue as skills do projeto; o grilling de 2026-11-28 fechou
como.

**Decisão:**

- **Skill encoberta** é a skill do projeto que tem homônima em `$CLAUDE_CONFIG_DIR/skills/` (sem a
  variável, `~/.claude/skills/`) com **conteúdo diferente**, comparado arquivo a arquivo, fora
  `target/` e `.DS_Store`. A homônima idêntica roda o mesmo texto e não conta.
- As três **skills de máquina** (`setup-stacklabs`, `adotar-metodo-stack-labs`, `auditar-metodo`)
  nunca contam: moram na pasta pessoal por design e o `--conferir-skill` as mantém em dia.
- **Watchdogs:** o `sessao-inicio` lista as encobertas e diz como resolver. O `guarda-skill`
  (PreToolUse `Skill`, novo) barra o agente de chamar uma encoberta, e o `vigia-prompt` barra o
  `/nome` digitado, porque o `SessionStart` não barra a sessão e o comando digitado não passa pelo
  Skill tool. Só a skill encoberta é barrada; as outras seguem livres. Na etapa `observacao`, os dois
  só avisam.
- **Instalador:** o relatório do `/setup-stacklabs` ganha `HOMÔNIMA <nome>`. O
  `--mover-homonimas` move as encobertas para `<config>/skills-encobertas/<data>/` e nunca apaga. Se
  a pasta pessoal estiver num repositório git, ele recusa e só lista, porque mover ali deixaria
  exclusões surpresa num repositório alheio ao método: a pessoa resolve nesse repositório. O agente
  roda o `--mover-homonimas` no passo 4 da skill e reporta o que moveu (ADR 0024: a atualização não
  pede passo humano).
- **Sem escape por projeto:** não há ajuste em `.claude/metodo.json` para liberar uma homônima. O
  escape viraria o caminho padrão para não resolver.

**Consequências:** a versão do método sobe para 5.0.0, porque um watchdog passa a bloquear algo
novo. Numa máquina com a pasta pessoal num repositório git, a skill encoberta fica barrada até a
pessoa resolver lá. A homônima idêntica continua rodando da pasta pessoal: com o mesmo conteúdo, o
efeito é o mesmo, mas uma mudança só na pessoal a torna encoberta na sessão seguinte. Skill de
plugin (`plugin:skill`) tem nome próprio e fica fora. Descartados: só avisar, porque o aviso some no
contexto e as 23 divergentes passaram assim; mover sempre, sem flag e sem checar o git; e barrar
toda skill enquanto houver uma encoberta.
