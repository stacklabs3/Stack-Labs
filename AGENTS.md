# AGENTS.md: repositório Stack-Labs

Os processos da StackLabs. A área `desenvolvimento/` guarda o método de desenvolvimento com IA que
todo projeto da casa segue.

## Levar o método a um projeto

Quando o pedido é ler, aplicar ou configurar o método de desenvolvimento num projeto (começar um
projeto, instalar os watchdogs, trazer as skills, atualizar um projeto que já segue o método), o
caminho é a skill **`/setup-stacklabs`**:

1. **Instale a skill na máquina**, uma vez:
   `cp -R desenvolvimento/agentes/skills/setup-stacklabs ~/.claude/skills/`
   (com o Stack-Labs clonado; sem clone, `git clone https://github.com/stacklabs1/Stack-Labs` antes).
2. **Diga ao humano para rodar `/setup-stacklabs`** dentro do projeto. Ela é invocada só pelo humano:
   o agente instala e avisa, quem dispara é a pessoa.

A skill copia o `modelo-de-projeto`, mescla o que o projeto já tem, instala as skills da casa e as
mais novas do Matt Pocock, ativa o pre-push, cria as labels e abre a PR do setup. Ela é a única
forma de instalar: assim todo projeto fica com a mesma versão e com a origem registrada em
`.claude/skills/ORIGEM.json`.

## Mapa do repositório

- `desenvolvimento/humanos/`: o guia para pessoas.
- `desenvolvimento/agentes/`: constituição, watchdogs e skills que a `/setup-stacklabs` instala. As
  skills da casa ficam em `desenvolvimento/agentes/skills/`: `setup-stacklabs`, `orquestrar-issues`,
  `auditar-metodo` e as recomendadas (família `*-prototipo`, `nova-sessao`, `voltar-a-principal` e outras).
- `desenvolvimento/GLOSSARY.md` e `desenvolvimento/adr/`: linguagem e decisões do método.
