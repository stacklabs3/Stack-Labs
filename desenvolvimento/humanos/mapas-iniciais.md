# Os quatro mapas iniciais

Todo projeto abre estes quatro mapas no primeiro dia ([ADR 0002](../adr/0002-quatro-mapas-iniciais.md)).
Cada um é uma issue no GitHub, com tickets filhos que são perguntas. O corpo exato de cada mapa, que
o agente usa, fica em [docs/agents/mapas-iniciais.md](../agentes/modelo-de-projeto/docs/agents/mapas-iniciais.md).

Em todos: **grilling** em cada decisão, e **nenhuma recomendação sem pesquisa**.

```
Protótipo ──────► Fases
     │              ▲
     ▼              │
Arquitetura ────────┤
     │              │
     └──────► Infra (fecha até o primeiro deploy)
```

## 1. Protótipo

**Decide:** como o produto parece e se comporta, rotina por rotina, até cada uma estar aprovada.

**Quem participa:** quem fala com o cliente, quem desenha, e o cliente nas aprovações.

**Perguntas que você vai ouvir:** em que aparelho cada pessoa usa o produto? Quais rotinas existem?
Qual o caminho mais curto para fazer cada coisa? Como fica vazio, carregando, com erro, sem internet?

**Como anda:** `/novo-prototipo` cria o protótipo único uma vez; cada rotina entra com
`/integrar-ao-prototipo`; a auditoria (`/auditar-prototipo`) e as correções (`/melhorar-prototipo`)
vêm antes de você aprovar; só o aprovado vira código (`/portar-prototipo`).

**Fecha quando:** não há mais rotina a prototipar. Pode seguir aberto durante a implementação.

## 2. Arquitetura

**Decide:** a linguagem, o framework, o banco, os módulos principais e as integrações externas.

**Quem participa:** os sócios técnicos e quem vai manter o projeto.

**Perguntas que você vai ouvir:** quem mantém isso daqui a dois anos? Qual o orçamento? Que
integrações são obrigatórias? Entre as opções que a pesquisa comparou, qual?

**Como anda:** cada escolha técnica tem antes um ticket de pesquisa comparando as opções (versão
atual, licença, maturidade, custo). O grilling usa essa pesquisa. Cada escolha difícil de reverter
vira uma ADR.

**Fecha quando:** a stack e os módulos estão decididos. **Bloqueia a implementação.**

## 3. Fases

**Decide:** a ordem de construir o front, o backend e a integração, o que entra em cada fase e quando
cada uma está pronta.

**Quem participa:** os sócios e quem vai implementar.

**Padrão da casa** (o grilling pode mudar, com motivo registrado):

1. **Front** a partir do protótipo aprovado, com dados falsos atrás de uma interface combinada.
2. **Backend** atrás da mesma interface.
3. **Integração**: troca os dados falsos pelos reais, sem mexer no front.

**Fecha quando:** cada fase tem entrega e critério de pronto. **Bloqueia a implementação.** Cada
fase vira depois uma spec e fatias verticais.

## 4. Infra

**Decide:** onde o projeto roda, os ambientes, o deploy, os segredos e o custo mensal.

**Quem participa:** os sócios técnicos e quem paga a conta.

**Perguntas que você vai ouvir:** qual provedor? Quantos ambientes? Quanto custa por mês no começo e
com o dobro de uso?

**Como anda:** preço e limite de provedor só entram com fonte e data da consulta. O que só você pode
fazer (criar conta, cartão, DNS) vira um ticket de tarefa com checklist.

**Fecha quando:** está tudo decidido para o primeiro deploy.
