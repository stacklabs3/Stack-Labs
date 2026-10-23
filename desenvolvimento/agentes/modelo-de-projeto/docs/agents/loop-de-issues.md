# Loop de issues

O jeito da casa de implementar várias issues `ready-for-agent` sem ninguém olhando. É o padrão
"Ralph" do Matt Pocock: um laço que abre uma **sessão nova** do Claude Code (`claude -p`) para
cada issue, **uma de cada vez** (ADR 0004 do método). O motor é `.claude/loop/loop-de-issues.mjs`; a
skill `/loop-de-issues` prepara a rodada e lê o resultado.

## Como uma rodada anda

1. O script pega a **fronteira**: issues abertas com `ready-for-agent`, sem responsável, sem
   bloqueio aberto e sem label `wayfinder:*`, da mais antiga para a mais nova.
2. Reivindica a primeira, atualiza a `main` e abre uma sessão com `.claude/loop/prompt-da-issue.md`.
3. A sessão segue a constituição inteira, com os watchdogs ativos: ativa a issue, cria a branch,
   `/implement` com `/tdd`, testes passando, PR com `Closes #<n>`. **Merge nunca.**
4. A última linha da sessão decide o próximo passo:

   | Linha | O script |
   |---|---|
   | `RESULTADO: PR <url>` | Registra a PR e passa para a próxima issue |
   | `RESULTADO: HANDOFF` | Abre uma sessão nova para a **mesma** issue, começando pelo handoff (até 4 sessões por issue) |
   | `RESULTADO: BLOQUEADO <motivo>` ou nenhuma | Devolve a issue: comenta o motivo, tira o responsável, troca `ready-for-agent` por `needs-triage` |

5. Para quando: chegou a N issues, a fronteira esvaziou, **duas issues seguidas** saíram sem PR, ou
   uma sessão deixou mudanças sem commit (o trabalho nunca é descartado).

Issue que depende de outra só entra quando a outra fecha, o que acontece no merge da PR dela. Uma
rodada não implementa uma fatia em cima de uma PR ainda não revisada.

## Smart zone dentro do loop

Quando o `guarda-edicao` barra uma edição pelo tamanho do contexto, a sessão: faz commit `wip:` na
branch da issue, escreve o handoff no caminho que o prompt indica (fora do repositório, no tmp do
SO) e termina com `RESULTADO: HANDOFF`. A sessão seguinte lê o handoff e continua da branch.

## Travas

- Teto de gasto por sessão (`--orcamento`, padrão 5 USD), via `--max-budget-usd`.
- `--permission-mode auto --permission-prompts none`: o classificador revisa cada ação, e o que
  pediria aprovação é negado, nunca fica esperando.
- `.claude/loop/` é infraestrutura dos watchdogs: o agente não edita.
- Registros de cada sessão em `.claude/state/loop/<rodada>/` (fora do git).

## Se você é a sessão do loop

O prompt manda. Não pergunte nada (não há quem responda), não adivinhe decisão que não está escrita,
e termine sempre com uma das três linhas `RESULTADO:`.
