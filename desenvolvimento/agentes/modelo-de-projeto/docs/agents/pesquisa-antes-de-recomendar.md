# Pesquisa antes de recomendar

Regra 2 do `AGENTS.md` (ADR 0003 do método). O modelo recomenda com a mesma confiança o que sabe e o
que imagina; a pesquisa é o que separa os dois.

## Quando pesquisar

Antes de recomendar qualquer coisa que dependa de um fato do mundo: versão, preço, limite, licença,
compatibilidade, suporte, lei, comportamento de uma API, prática do mercado. O que o próprio
repositório responde (código, `package.json`, ADRs, glossário) você olha direto, sem `/research`.

## Como

1. Rode `/research` num subagente em segundo plano. Ele lê **fontes primárias**: documentação
   oficial, código-fonte, especificação, página de preço do fornecedor, texto da lei.
2. O resultado vai para `docs/research/<assunto>.md`, com a data da consulta e a fonte de cada
   afirmação. O que não tiver fonte primária fica marcado **[confirmar]**.
3. No grilling, a pergunta que espera a pesquisa vai para a rodada seguinte; as demais seguem agora.

## Como recomendar

Toda recomendação traz a fonte na mesma linha:

```
❓ **Q2** - **Banco de dados**: Postgres ou MongoDB para o histórico de pedidos?

➡️ Postgres: os pedidos são relacionais e o provedor escolhido oferece Postgres gerenciado no plano
atual ([docs/research/banco-de-dados.md](../research/banco-de-dados.md), seção 2).
```

Sem fonte, diga que ainda não há recomendação e qual pesquisa vai trazê-la.

## Como fechar o ticket

O comentário de resolução de todo ticket de grilling, tarefa ou pesquisa termina com:

```markdown
## Fontes

- [docs/research/banco-de-dados.md](link-para-o-arquivo-na-branch)
- https://www.postgresql.org/docs/current/...
```

O `guarda-bash` barra o `gh issue close` de ticket sem essa seção, com ao menos um link `http` ou
`docs/research/`. Ticket de protótipo é a exceção: a evidência dele é o protótipo aprovado.
