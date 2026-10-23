# Método StackLabs: versão para os agentes

O que o agente de IA lê e o que vigia o agente. Nada aqui se lê "para entender": tudo se **copia**
para o projeto, onde vira a constituição e os watchdogs dele. O guia para pessoas está em
[../humanos/](../humanos/README.md).

```
agentes/
├── modelo-de-projeto/          ← o que o /setup-stacklabs instala na raiz de cada projeto
│   ├── AGENTS.md               constituição: as regras duras
│   ├── CLAUDE.md               @AGENTS.md
│   ├── docs/agents/            o que a constituição aponta
│   │   ├── mapas-iniciais.md          os 4 mapas e o corpo de cada um
│   │   ├── pesquisa-antes-de-recomendar.md
│   │   ├── roteiro-de-prototipo.md    família *-prototipo
│   │   ├── watchdogs.md
│   │   ├── loop-de-issues.md          o loop de issues em sequência
│   │   ├── issue-tracker.md · triage-labels.md · domain.md   (setup das skills do Matt)
│   ├── .claude/                hooks do Claude Code (watchdogs) e o loop de issues
│   ├── .github/workflows/      vigia-do-metodo (watchdog de CI)
│   └── .gitignore
└── skills/
    ├── setup-stacklabs/        instala e atualiza o método num projeto
    ├── loop-de-issues/         prepara e lê rodadas do loop de issues (.claude/loop/)
    ├── auditar-metodo/         auditoria do projeto inteiro, sob demanda
    └── recomendada/            cópia das skills recomendadas (matheus3006/my-claude-skills): a família
                                *-prototipo, /nova-sessao, /voltar-a-principal, human-architect-mindset
                                e outras. Difere da origem em dois pontos: as skills do Matt seguem
                                mattpocock/skills 5beb35b (v1.3.1) e o tirar-prints.sh virou só .mjs
```

## Instalar num projeto novo

Uma vez por máquina, instale a skill de setup:

```bash
git clone https://github.com/stacklabs1/Stack-Labs
cp -R Stack-Labs/desenvolvimento/agentes/skills/setup-stacklabs ~/.claude/skills/
```

Depois, no projeto (novo ou existente), numa sessão do Claude Code: **`/setup-stacklabs`**. Ela
instala o `modelo-de-projeto`, as labels, o pre-push e, em `.claude/skills/` do projeto, as skills da
casa e as skills publicadas mais novas do Matt Pocock. Num projeto existente, mescla o `AGENTS.md`,
o `CLAUDE.md` e o `settings.json` em vez de sobrescrever. Termina numa PR `chore/setup-metodo`.

**Atualizar** um projeto é rodar `/setup-stacklabs` de novo. Watchdog desatualizado num projeto que
já tem watchdogs só um humano atualiza:
`node ~/.claude/skills/setup-stacklabs/instalar.mjs --atualizar-watchdogs`.

Não instale também o plugin `mattpocock-skills`: as skills dele já ficam no projeto e apareceriam
duplicadas.

Requisitos na máquina: `node` 18+, `gh` autenticado e `git`.

## Mudar o método

O `modelo-de-projeto` é a fonte única. Mude aqui primeiro, por PR no Stack-Labs, e propague para os
projetos. Mudança só num projeto é exceção dele e fica na seção `## Regras deste projeto`.
