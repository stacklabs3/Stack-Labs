---
name: adotar-metodo-stack-labs
description: Adota o método StackLabs, por etapas, num projeto que já está em execução.
disable-model-invocation: true
argument-hint: "(opcional) o que priorizar nesta sessão"
---

Conduz a **adoção** de um projeto que **já existe** e não segue o método, ou segue em partes (ADR
0006 do método). Projeto novo é o `/setup-stacklabs`. A adoção atravessa várias sessões e avança
pelas **etapas de adoção** `observacao`, `planejamento` e `completo`. O que cada etapa bloqueia está
em `docs/agents/adocao.md` (antes da instalação, no `modelo-de-projeto` do Stack-Labs).

## Onde a adoção está

Leia `.claude/metodo.json` e siga **um** ramo:

| Situação | Ramo |
|---|---|
| Não existe | [INICIO.md](INICIO.md): diagnóstico, plano e instalação em observação |
| `observacao` | [OBSERVACAO.md](OBSERVACAO.md): mapas retroativos, triage e passagem para planejamento |
| `planejamento` | [PLANEJAMENTO.md](PLANEJAMENTO.md): passagem para completo |
| `completo` | A adoção acabou. Diga ao humano que daqui em diante é o `/auditar-metodo` |

## Em todo ramo

- **O plano é a memória da adoção:** `.scratch/adocao/plano.md` diz o que já foi feito e o que vem.
  Leia antes de agir; atualize ao fim de cada passo.
- **Uma unidade por sessão:** um passo, um mapa ou um lote de triage. Ao fechá-la, rode o
  `/nova-sessao`: o kickoff abre a janela seguinte com `/adotar-metodo-stack-labs`.
- **O humano decide, o agente prepara:** grilling em toda decisão, com recomendação e fonte. Mudar
  de etapa é uma PR do humano, com a label `altera-watchdogs`: você mostra as condições cumpridas,
  ele grava.
