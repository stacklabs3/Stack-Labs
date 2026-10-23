# Mapa inicial de Fases

Parte dos [mapas iniciais](../mapas-iniciais.md). Label `mapa-inicial:fases`.

## Corpo do mapa

```markdown
## Destination

A sequência de fases do desenvolvimento, com o que cada fase entrega, o critério de pronto de cada
uma e a ordem em que os mapas de implementação serão abertos.

## Notes

- Depende do mapa de Arquitetura (stack) e do mapa de Protótipo (desenho inicial aprovado).
- A divisão das fases é decisão do desenvolvedor que vai conduzir o projeto. Comece perguntando a
  ele como quer dividir; a casa não tem ordem padrão. A recomendação parte do que os outros mapas
  decidiram.
- Cada fase fecha com `/to-spec` e `/to-tickets` em fatias verticais.

## Decisions so far

## Not yet specified

## Out of scope
```

## Tickets que costumam nascer

- *Como dividir as fases* (grilling com o desenvolvedor).
- *Seams entre as partes* (grilling): onde ficam as interfaces que deixam uma fase avançar sem a outra.
- *Critério de pronto de cada fase* (grilling).
- *Estratégia de testes por fase* (grilling + `/tdd`).
