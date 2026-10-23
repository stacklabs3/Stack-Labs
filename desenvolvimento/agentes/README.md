# Método StackLabs: versão para os agentes

O que o agente de IA lê e o que vigia o agente. Nada aqui se lê "para entender": tudo se **copia**
para o projeto, onde vira a constituição e os watchdogs dele. O guia para pessoas está em
[../humanos/](../humanos/README.md).

```
agentes/
├── modelo-de-projeto/          ← copie o conteúdo para a raiz de todo projeto novo
│   ├── AGENTS.md               constituição: as regras duras
│   ├── CLAUDE.md               @AGENTS.md
│   ├── docs/agents/            o que a constituição aponta
│   │   ├── mapas-iniciais.md          os 4 mapas e o corpo de cada um
│   │   ├── pesquisa-antes-de-recomendar.md
│   │   ├── roteiro-de-prototipo.md    família *-prototipo
│   │   ├── watchdogs.md
│   │   ├── issue-tracker.md · triage-labels.md · domain.md   (setup das skills do Matt)
│   ├── .claude/                hooks do Claude Code (watchdogs)
│   ├── .github/workflows/      vigia-do-metodo (watchdog de CI)
│   └── .gitignore
└── skills/
    ├── auditar-metodo/         auditoria do projeto inteiro, sob demanda
    └── recomendada/            cópia fiel das skills recomendadas (matheus3006/my-claude-skills):
                                as do Matt Pocock, a família *-prototipo, human-architect-mindset e outras
```

## Instalar num projeto novo

```bash
# 1. no clone do projeto novo, com o Stack-Labs clonado ao lado
cp -r ../Stack-Labs/desenvolvimento/agentes/modelo-de-projeto/. .
mkdir -p docs/adr docs/research

# 2. o pre-push que protege a main (uma vez por clone, por pessoa)
git config core.hooksPath .claude/githooks

# 3. as labels do método
#    (comando em docs/agents/issue-tracker.md, seção "Labels a criar no repositório novo")

# 4. as skills: as recomendadas (Matt Pocock, família *-prototipo e outras) e a auditoria
#    Não instale também o plugin mattpocock-skills: as skills dele já estão na cópia e ficariam duplicadas.
cp -R ../Stack-Labs/desenvolvimento/agentes/skills/recomendada/* ~/.claude/skills/
cp -R ../Stack-Labs/desenvolvimento/agentes/skills/auditar-metodo ~/.claude/skills/
```

Depois, numa sessão nova do Claude Code, o `sessao-inicio` mostra o que falta. O primeiro trabalho
é abrir os quatro mapas iniciais (`docs/agents/mapas-iniciais.md`).

Requisitos na máquina: `node` 18+, `gh` autenticado e `git`.

## Projeto que já existe

Copie igual e confira o `AGENTS.md` do projeto: o que ele tinha de próprio entra numa seção
`## Regras deste projeto` no fim, sem contradizer as regras do modelo. O que já foi decidido antes
do método vira ADR em `docs/adr/`. Os mapas iniciais que já estão resolvidos na prática se abrem e
fecham na mesma sessão, com o ponteiro para a ADR ou o documento que registrou a decisão.

## Mudar o método

O `modelo-de-projeto` é a fonte única. Mude aqui primeiro, por PR no Stack-Labs, e propague para os
projetos. Mudança só num projeto é exceção dele e fica na seção `## Regras deste projeto`.
