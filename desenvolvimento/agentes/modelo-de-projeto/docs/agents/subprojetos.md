# Subprojeto com `CLAUDE.md` próprio

Num monorepo, o Claude Code carrega o `CLAUDE.md` de uma subpasta só quando a ferramenta **Read**
abre um arquivo dela. `cat`, `sed`, `grep` e o resto do Bash não o carregam, e o agente que lê por
Bash trabalha no subprojeto sem as regras dele.

## Como se escreve

1. **A raiz aponta.** Em `## Regras deste projeto` → `### Acrescentadas` do `AGENTS.md`, uma regra
   `RP<n>` lista cada subprojeto que tem `CLAUDE.md` e manda lê-lo com Read antes de mexer nele:
   `- **RP3** Antes de ler ou editar \`backend/\`, abra \`backend/CLAUDE.md\` com Read.`
2. **O `CLAUDE.md` do subprojeto é índice.** O que a pasta é, em duas linhas; o que vale em toda
   edição nela (o que roda antes do commit, o que nunca se faz); e um ponteiro com gatilho para cada
   assunto: `- Mexendo na autenticação: leia [docs/agents/backend-auth.md](../docs/agents/backend-auth.md).`
   Comando que o `package.json` ou o `Makefile` já diz fica lá: o índice aponta, não copia.
3. **O detalhe mora em `docs/agents/<subprojeto>-<assunto>.md`**, um assunto por arquivo, escrito
   com o `/writing-for-agents`. `docs/` é caminho livre (regra 3): mudar o detalhe não pede issue ativa.

Pronto quando todo subprojeto com `CLAUDE.md` está na regra da raiz, cada `CLAUDE.md` de subprojeto
cabe numa tela, e cada assunto que só alguns pedidos usam está atrás de um ponteiro que diz quando
ler.
