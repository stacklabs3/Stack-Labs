# Mapa inicial de Infra

Parte dos [mapas iniciais](../mapas-iniciais.md). Label `mapa-inicial:infra`.

## Corpo do mapa

```markdown
## Destination

Um retrato da infra que o projeto vai usar: os serviços e tecnologias disponíveis, o custo estimado e
o que teremos que construir em volta deles. É conhecimento de causa para os outros mapas; não trava
a implementação e não precisa fechar.

## Notes

- Mapa leve: poucos tickets, quase todos de pesquisa. Ambientes, deploy e CI ganham mapa próprio
  quando a implementação chegar perto.
- Preço, limite e região de provedor só entram com fonte primária e data da consulta.

## Decisions so far

## Not yet specified

## Out of scope
```

## Tickets que costumam nascer

- *Infra disponível para a stack* (pesquisa): serviços e tecnologias que atendem o que a Arquitetura decidiu.
- *Custos estimados* (pesquisa): quanto custa no começo e com o dobro de uso.
- *O que construir em volta* (grilling): o que a infra escolhida não resolve e vira trabalho nosso.
