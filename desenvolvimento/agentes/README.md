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
│   │   ├── mapas-iniciais.md          os 5 mapas e o corpo de cada um
│   │   ├── pesquisa-antes-de-recomendar.md
│   │   ├── roteiro-de-prototipo.md    família *-prototipo
│   │   ├── watchdogs.md
│   │   ├── caminho-ate-ready-for-agent.md   rotas e definição de pronto
│   │   ├── orquestracao.md            a orquestração de issues
│   │   ├── adocao.md                  as etapas de adoção num projeto que já existia
│   │   ├── issue-tracker.md · triage-labels.md · domain.md   (setup das skills do Matt)
│   ├── .claude/                hooks do Claude Code (watchdogs) e os papéis da orquestração (agents/)
│   ├── .github/workflows/      vigia-do-metodo (watchdog de CI)
│   └── .gitignore
└── skills/
    ├── setup-stacklabs/        instala o método num projeto novo, ou atualiza
    ├── adotar-metodo-stack-labs/  adota o método por etapas num projeto que já existe
    ├── orquestrar-issues/      orquestra a fila de issues ready-for-agent
    ├── auditar-metodo/         auditoria do projeto inteiro, sob demanda
    └── recomendada/            cópia das skills recomendadas (matheus3006/my-claude-skills): a família
                                *-prototipo, /nova-sessao, /voltar-a-principal, human-architect-mindset
                                e outras. Difere da origem em dois pontos: as skills do Matt seguem
                                mattpocock/skills 5beb35b (v1.3.1) e o tirar-prints.sh virou só .mjs
```

## Instalar num projeto novo

Uma vez por máquina, instale as skills de entrada:

```bash
git clone https://github.com/stacklabs1/Stack-Labs
cp -R Stack-Labs/desenvolvimento/agentes/skills/{setup-stacklabs,adotar-metodo-stack-labs,auditar-metodo} ~/.claude/skills/
```

Depois, numa sessão do Claude Code dentro do projeto:

- **Projeto novo:** **`/setup-stacklabs`**. Ela instala o `modelo-de-projeto`, as labels, o pre-push
  e, em `.claude/skills/` do projeto, as skills da casa e as skills publicadas mais novas do Matt
  Pocock. Termina numa PR `chore/setup-metodo`.
- **Projeto que já está em execução** e não segue o método, ou segue em partes:
  **`/adotar-metodo-stack-labs`**. Ela diagnostica, instala com os watchdogs em observação e leva o
  projeto às etapas seguintes ([docs/agents/adocao.md](modelo-de-projeto/docs/agents/adocao.md)).

**Atualizar** um projeto é rodar `/setup-stacklabs` de novo. Watchdog desatualizado num projeto que
já tem watchdogs só um humano atualiza:
`node ~/.claude/skills/setup-stacklabs/instalar.mjs --atualizar-watchdogs`.

Não instale também o plugin `mattpocock-skills`: as skills dele já ficam no projeto e apareceriam
duplicadas.

Requisitos na máquina: `node` 18+, `gh` autenticado e `git`.

## Mudar o método

O `modelo-de-projeto` é a fonte única. Mude aqui primeiro, por PR no Stack-Labs, e propague para os
projetos. Mudança só num projeto é exceção dele e fica na seção `## Regras deste projeto`.
