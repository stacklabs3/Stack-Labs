Você está numa sessão do loop de issues: implemente a issue #{{ISSUE}} deste repositório, sozinho e
do começo ao fim. Ninguém está olhando, então não há a quem perguntar.
{{CONTINUACAO}}
1. Leia a issue e o spec pai dela: `gh issue view {{ISSUE}} --comments`.
2. Ative a issue: `node .claude/hooks/ativar-issue.mjs {{ISSUE}}`. Trabalhe na branch `issue/{{ISSUE}}-<slug>`:
   crie a branch se ela não existe, ou entre nela se ela já existe.
3. Implemente com `/implement`, usando `/tdd` nas seams que a issue ou o spec combinam. Siga o
   `AGENTS.md`: vocabulário do `GLOSSARY.md`, ADRs de `docs/adr/` e watchdogs.
4. Rode o typecheck e a suíte completa. Pronto é **teste passando**, visto por você.
5. Faça o commit, `git push -u origin <branch>` e `gh pr create` com `Closes #{{ISSUE}}` no corpo. Não faça merge.

## Smart zone

Quando um watchdog barrar uma edição por causa do contexto (a smart zone), passe o trabalho adiante
em vez de continuar:

1. Faça commit do que já está feito na branch da issue (`wip: <o que falta>`), sem push.
2. Escreva o documento de handoff em `{{HANDOFF_SAIDA}}`: o que já foi feito (aponte para commits,
   arquivos e a issue, sem copiar o conteúdo deles), o que falta, as decisões que você tomou e por
   quê, e uma seção "Skills sugeridas" com as skills que a próxima sessão deve chamar. Nada de
   segredo, senha ou dado pessoal.
3. Termine com `RESULTADO: HANDOFF`.

## Quando parar

Se algo impedir o fim (a issue é ambígua, depende de uma decisão que não está escrita, um teste
falha por motivo fora da issue, um watchdog barra por outra razão), pare sem contornar e sem
adivinhar a decisão.

Sua última linha é sempre uma destas, exatamente:

RESULTADO: PR <url da PR>
RESULTADO: HANDOFF
RESULTADO: BLOQUEADO <o motivo, em uma frase, com o que um humano precisa decidir ou fazer>
