# Os quatro mapas iniciais

Todo projeto abre estes quatro mapas no primeiro dia ([ADR 0002](../adr/0002-quatro-mapas-iniciais.md)).
Cada um é uma issue no GitHub, com tickets filhos que são perguntas. O corpo exato de cada mapa, que
o agente usa, fica em [docs/agents/mapas-iniciais.md](../agentes/modelo-de-projeto/docs/agents/mapas-iniciais.md).

Em todos: **grilling** em cada decisão, e **nenhuma recomendação sem pesquisa**. A implementação só
começa com **Protótipo**, **Arquitetura** e **Fases** fechados.

```
Protótipo ──────► Fases
     │              ▲
     ▼              │
Arquitetura ────────┤
     │              │
     └──────► Infra (informativo, não trava)
```

## 1. Protótipo

**Decide:** o desenho inicial do produto: como ele parece e se comporta nas rotinas principais.

**Quem participa:** quem fala com o cliente, quem desenha, e o cliente nas aprovações.

**Perguntas que você vai ouvir:** em que aparelho cada pessoa usa o produto? Quais rotinas existem?
Qual o caminho mais curto para fazer cada coisa? Como fica vazio, carregando, com erro, sem internet?

**Como anda:** `/novo-prototipo` cria o protótipo único uma vez; cada rotina entra com
`/integrar-ao-prototipo`; a auditoria (`/auditar-prototipo`) e as correções (`/melhorar-prototipo`)
vêm antes de você aprovar; só o aprovado vira código (`/portar-prototipo`).

**Fecha quando:** o desenho inicial está aprovado. **Bloqueia a implementação.** Protótipos que
surgirem depois entram como tickets nos mapas de cada esforço.

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

**Decide:** como o desenvolvimento se divide em fases, o que entra em cada uma e quando cada uma
está pronta.

**Quem decide:** o desenvolvedor que vai conduzir o projeto. Não há ordem padrão da casa. Dividir em
front, backend e integração é um exemplo possível, não uma regra; o agente pergunta ao desenvolvedor
como ele quer fazer.

**Fecha quando:** cada fase tem entrega e critério de pronto. **Bloqueia a implementação.** Cada
fase vira depois uma spec e fatias verticais.

## 4. Infra

**Decide:** nada que trave o projeto. É um retrato para **conhecimento de causa**: que infra vamos
usar, quanto custa, quais tecnologias e soluções estarão disponíveis e o que teremos que construir
em volta delas.

**Quem participa:** os sócios técnicos e quem paga a conta.

**Como anda:** é o menor dos quatro mapas, com poucos tickets, quase todos de pesquisa. Preço e
limite de provedor só entram com fonte e data da consulta.

**Fecha quando:** não precisa fechar. Detalhes de ambientes e deploy viram um mapa próprio quando a
implementação chegar perto.
