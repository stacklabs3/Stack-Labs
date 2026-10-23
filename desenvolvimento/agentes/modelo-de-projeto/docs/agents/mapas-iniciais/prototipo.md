# Mapa inicial de Protótipo

Parte dos [mapas iniciais](../mapas-iniciais.md). Label `mapa-inicial:prototipo`.

## Corpo do mapa

```markdown
## Destination

O desenho inicial do produto: as rotinas principais prototipadas no protótipo único
(`prototipos/<projeto>/`), auditadas e com aprovação explícita do humano, prontas para `/portar-prototipo`.

## Notes

- Aparência se prototipa com a família `*-prototipo`, na ordem de docs/agents/roteiro-de-prototipo.md.
- Comportamento (estado, regra, lógica) se prototipa com `/prototype`, não com a família visual.
- Tickets de protótipo de um mesmo lote fecham juntos, depois da aprovação.
- Este mapa trava a implementação até fechar. Protótipos que surgirem depois do desenho inicial
  entram como tickets de protótipo nos mapas de cada esforço, não reabrem este.

## Decisions so far

## Not yet specified

## Out of scope
```

## Tickets que costumam nascer

- *Superfícies e aparelhos* (grilling): em que aparelho cada papel usa o produto (iOS, Android, web, tablet, TV).
- *Princípios de UI/UX do projeto* (grilling + pesquisa de referências do mercado).
- *Criar o protótipo único* (protótipo): `/novo-prototipo`, com as abas e as visões decididas.
- *Rotinas do desenho inicial* (grilling): quais rotinas entram agora; cada uma vira um ticket de protótipo, em lotes.
