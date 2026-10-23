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
│   │   ├── mapas-iniciais.md          os 5 mapas; o corpo de cada um em mapas-iniciais/
│   │   ├── pesquisa-antes-de-recomendar.md
│   │   ├── roteiro-de-prototipo.md    família *-prototipo
│   │   ├── watchdogs.md
│   │   ├── caminho-ate-ready-for-agent.md   rotas e definição de pronto
│   │   ├── orquestracao.md            a orquestração de issues
│   │   ├── adocao.md                  as etapas de adoção num projeto que já existia
│   │   ├── principios-de-codigo.md    os 22 princípios (ADR 0008); grupos, grilling e práticas de
│   │   │                              TypeScript, Java e Rust (ADR 0012) em principios-de-codigo/
│   │   ├── validador.md               o formato de regras-de-codigo.json, o catálogo e como refinar
│   │   ├── riscos.md                  o registro de riscos e o que faz um teste valer (ADR 0014)
│   │   ├── issue-tracker.md · triage-labels.md · domain.md   (setup das skills do Matt)
│   ├── .claude/                watchdogs/: os hooks do Claude Code, o pre-push e o validador de regras
│   │                           de código, num crate Rust compilado em cada máquina (ADRs 0010 e 0011),
│   │                           e os modelos por linguagem em watchdogs/modelos/;
│   │                           agents/: os papéis da orquestração; githooks/: o pre-push
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
                                mattpocock/skills 5beb35b (v1.3.1) e os scripts da casa
                                (gerar-folha, tirar-prints) são Rust
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
`sh ~/.claude/skills/setup-stacklabs/rodar --atualizar-watchdogs`.

Não instale também o plugin `mattpocock-skills`: as skills dele já ficam no projeto e apareceriam
duplicadas.

Requisitos na máquina: `cargo` (Rust, https://rustup.rs), `gh` autenticado e `git`. O instalador e
os watchdogs são compilados na máquina pelo `cargo` na primeira vez que rodam.

## Mudar o método

O `modelo-de-projeto` é a fonte única. Mude aqui primeiro, por PR no Stack-Labs, e propague para os
projetos. Mudança só num projeto é exceção dele e fica na seção `## Regras deste projeto`.

Todo documento do método se escreve com o `/writing-for-agents` (ADR 0009). Cada significado tem um
dono; os outros lugares apontam para ele:

| Significado | Dono |
|---|---|
| Definição de um termo | [GLOSSARY.md](../GLOSSARY.md) |
| O porquê de uma decisão do método | [adr/](../adr/) |
| Regra dura do agente | `modelo-de-projeto/AGENTS.md`, uma linha por regra e o ponteiro |
| Mecânica do agente: passos, critério de pronto, comandos | `modelo-de-projeto/docs/agents/` |
| O que a pessoa faz, decide e aprova | [humanos/](../humanos/README.md) |
| Procedimento de uma skill | o `SKILL.md` dela |
| Fato que se descobre olhando (constante, caminho, comando) | o código ou a configuração |

O que só um ramo usa vai para um arquivo irmão, com um ponteiro que diz quando ler. Caminho que um
hook ou outro doc cita não muda.
