# Os cinco mapas iniciais

Todo projeto abre estes cinco mapas no primeiro dia ([ADR 0002](../adr/0002-quatro-mapas-iniciais.md)
e [ADR 0007](../adr/0007-mapa-de-regras-de-codigo.md)).
Cada um é uma issue no GitHub, com tickets filhos que são perguntas. O que o agente faz em cada
mapa está em [docs/agents/mapas-iniciais.md](../agentes/modelo-de-projeto/docs/agents/mapas-iniciais.md);
aqui fica o que você decide.

Num projeto que já está rodando, os mapas registram as decisões que ele já tomou e fecham na hora
quando nada está em aberto: veja [adotar-em-projeto-existente.md](adotar-em-projeto-existente.md).

Em todos: **grilling** em cada decisão, e **nenhuma recomendação sem pesquisa**. A implementação só
começa com **Protótipo**, **Arquitetura**, **Fases** e **Regras de código** fechados.

```
Protótipo ──────► Fases
     │              ▲
     ▼              │
Arquitetura ────────┤
     │
     ├──────► Regras de código
     │
     └──────► Infra (informativo, não trava)
```

## 1. Protótipo

**Decide:** o desenho inicial do produto: como ele parece e se comporta nas rotinas principais.

**Quem participa:** quem fala com o cliente, quem desenha, e o cliente nas aprovações.

**Perguntas que você vai ouvir:** em que aparelho cada pessoa usa o produto? Quais rotinas existem?
Qual o caminho mais curto para fazer cada coisa? Como fica vazio, carregando, com erro, sem internet?

**Como anda:** o agente monta e audita cada rotina antes de levar a você; só o que você aprovar por
escrito vira código.

**Fecha quando:** o desenho inicial está aprovado. **Bloqueia a implementação.** Protótipos que
surgirem depois entram como tickets nos mapas de cada esforço.

## 2. Arquitetura

**Decide:** a linguagem, o framework, o banco, os módulos principais e as integrações externas.

**Quem participa:** os sócios técnicos e quem vai manter o projeto.

**Perguntas que você vai ouvir:** quem mantém isso daqui a dois anos? Qual o orçamento? Que
integrações são obrigatórias? Entre as opções que a pesquisa comparou, qual?

**Como anda:** cada escolha técnica chega a você com a pesquisa que comparou as opções. Cada escolha
difícil de reverter vira uma ADR.

**Fecha quando:** a stack e os módulos estão decididos. **Bloqueia a implementação.**

## 3. Fases

**Decide:** como o desenvolvimento se divide em fases, o que entra em cada uma e quando cada uma
está pronta.

**Quem decide:** o desenvolvedor que vai conduzir o projeto. Não há ordem padrão da casa. Dividir em
front, backend e integração é um exemplo possível, não uma regra; o agente pergunta ao desenvolvedor
como ele quer fazer.

**Fecha quando:** cada fase tem entrega e critério de pronto. **Bloqueia a implementação.** Cada
fase vira depois uma spec e fatias verticais.

## 4. Regras de código

**Decide:** as regras que todo código do projeto segue, no documento `docs/regras-de-codigo.md`. É a
régua do agente que implementa e do agente que revisa.

**Quem participa:** os sócios técnicos e quem vai manter o projeto.

**Perguntas que você vai ouvir:** os nomes no código são em português ou em inglês? Um termo do
glossário vira o nome da classe? Quando vale criar uma interface? Onde se valida o que o usuário
digita? O que se testa e o que se mocka? Qual lint e qual formatador?

**Como anda:** depois da Arquitetura. O grilling roda em torno dos
[princípios de código](principios-de-codigo.md) da casa: primeiro as restrições, depois um ticket
por grupo de princípios, e cada princípio sai adotado, adaptado ou deixado de fora, com o porquê.
No ticket de lint, você decide também as práticas só da linguagem do projeto (TypeScript, Java ou
Rust).

**Fecha quando:** `docs/regras-de-codigo.md` está escrito, com os 22 princípios decididos na tabela
de cobertura, e você aprovou. **Bloqueia a implementação.** Regra nova ou mudada depois passa por
ticket.

## 5. Infra

**Decide:** nada que trave o projeto. É um retrato para **conhecimento de causa**: que infra vamos
usar, quanto custa, quais tecnologias e soluções estarão disponíveis e o que teremos que construir
em volta delas.

**Quem participa:** os sócios técnicos e quem paga a conta.

**Como anda:** é o menor dos cinco, quase todo de pesquisa. Preço e limite de provedor chegam com
fonte e data da consulta.

**Fecha quando:** não precisa fechar. Detalhes de ambientes e deploy viram um mapa próprio quando a
implementação chegar perto.
