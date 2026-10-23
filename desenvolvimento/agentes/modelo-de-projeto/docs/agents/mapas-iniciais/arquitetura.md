# Mapa inicial de Arquitetura

Parte dos [mapas iniciais](../mapas-iniciais.md). Label `mapa-inicial:arquitetura`.

## Corpo do mapa

```markdown
## Destination

A linguagem, a stack e os módulos principais do projeto decididos, cada escolha difícil de reverter
registrada como ADR, com a pesquisa que a sustenta em docs/research/.

## Notes

- Vocabulário de arquitetura do `/codebase-design` (módulo, interface, profundidade, seam, adapter).
  Projeto com código existente: comece por `/improve-codebase-architecture`.
- A pesquisa de cada opção técnica compara versão atual, licença, maturidade, custo e quem mantém.
- Restrições do cliente (equipe que vai manter, orçamento, integrações obrigatórias) vêm antes da stack.

## Decisions so far

## Not yet specified

## Out of scope
```

## Tickets que costumam nascer

- *Restrições do projeto* (grilling): quem mantém, orçamento, prazos, integrações, LGPD.
- *Opções de linguagem e framework* (pesquisa) → *Linguagem e framework* (grilling, bloqueado pela pesquisa).
- *Opções de banco de dados* (pesquisa) → *Banco de dados* (grilling).
- *Módulos e seams principais* (grilling, depois do glossário do domínio estar de pé).
- *Integrações externas* (pesquisa por serviço: API, limites, preço).
