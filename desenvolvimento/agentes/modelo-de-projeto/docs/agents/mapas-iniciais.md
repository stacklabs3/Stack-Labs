# Mapas iniciais

Todo projeto abre quatro mapas do `/wayfinder` antes de qualquer código de produção
(ADR 0002 do método). Cada mapa é uma issue com `wayfinder:map` **e** `mapa-inicial:<tipo>`, que é
como os watchdogs encontram o mapa.

| Mapa | Label | Destino | Precisa estar fechado para ativar issue? |
|---|---|---|---|
| Protótipo | `mapa-inicial:prototipo` | O desenho inicial do produto prototipado e aprovado | **Sim** |
| Arquitetura | `mapa-inicial:arquitetura` | Linguagem, stack e módulos decididos, em ADRs | **Sim** |
| Fases | `mapa-inicial:fases` | A divisão em fases que o desenvolvedor escolheu, com o que entra e o critério de pronto de cada uma | **Sim** |
| Infra | `mapa-inicial:infra` | Retrato da infra que vamos usar: custos, tecnologias e o que construir em volta | Não (informativo) |

## Como abrir

1. Uma sessão por mapa, com `/wayfinder` em modo *Chart the map*. Comece pelo Protótipo e pela
   Arquitetura; Fases e Infra dependem deles. Infra é o menor dos quatro: poucos tickets, quase todos de pesquisa.
2. Crie o mapa com as duas labels e o corpo do modelo abaixo. Ajuste o destino ao projeto no grilling.
3. Os tickets sugeridos são ponto de partida, não lista fechada. O grilling breadth-first decide
   quais existem; o que não tiver pergunta precisa vai para *Not yet specified*.
4. Ligue os bloqueios entre mapas com dependências nativas: os tickets de Fases e de Infra que
   dependem de uma escolha de Arquitetura ficam `blocked_by` o ticket dela.
5. Dispare os subagentes `/research` dos tickets de pesquisa no fim da sessão de mapear.

## Regras que valem nos quatro

- **Grilling em todo ticket de decisão** (`/grilling` + `/domain-modeling`), em rodadas, com o
  humano respondendo.
- **Nenhuma recomendação sem pesquisa** ([pesquisa-antes-de-recomendar.md](pesquisa-antes-de-recomendar.md)).
  Antes do grilling de uma escolha técnica (linguagem, framework, banco, provedor), o ticket de
  pesquisa dela já está fechado e o grilling cita o arquivo de `docs/research/`.
- Toda escolha difícil de reverter vira ADR em `docs/adr/`. O ticket aponta a ADR; o mapa aponta o ticket.

---

## Mapa de Protótipo

```markdown
## Destination

O desenho inicial do produto: as rotinas principais prototipadas no protótipo único
(`prototipos/<projeto>/`), auditadas e com aprovação explícita do humano, prontas para `/portar-prototipo`.

## Notes

- Família `*-prototipo`: `/novo-prototipo` roda UMA vez (primeiro ticket); cada rotina depois entra
  pelo `/integrar-ao-prototipo`; `/melhorar-prototipo` corrige o que a auditoria achou;
  `/auditar-prototipo` antes de toda aprovação; `/sincronizar-prototipo` antes de todo commit;
  `/portar-prototipo` só com aprovação explícita. Roteiro: docs/agents/roteiro-de-prototipo.md.
- Comportamento (estado, regra, lógica) se prototipa com `/prototype`, não com a família visual.
- Tickets de protótipo de um mesmo lote fecham juntos, depois da aprovação.
- Este mapa trava a implementação até fechar. Protótipos que surgirem depois do desenho inicial
  entram como tickets de protótipo nos mapas de cada esforço, não reabrem este.
- Vocabulário: GLOSSARY.md do projeto.

## Decisions so far

## Not yet specified

## Out of scope
```

Tickets que costumam nascer:

- *Superfícies e aparelhos* (grilling): em que aparelho cada papel usa o produto (iOS, Android, web, tablet, TV).
- *Princípios de UI/UX do projeto* (grilling + pesquisa de referências do mercado).
- *Criar o protótipo único* (protótipo): `/novo-prototipo`, com as abas e as visões decididas.
- *Rotinas do desenho inicial* (grilling): quais rotinas entram agora; cada uma vira um ticket de protótipo, em lotes.

## Mapa de Arquitetura

```markdown
## Destination

A linguagem, a stack e os módulos principais do projeto decididos, cada escolha difícil de reverter
registrada como ADR, com a pesquisa que a sustenta em docs/research/.

## Notes

- Grilling com `/grilling` + `/domain-modeling` em todo ticket; vocabulário de arquitetura do
  `/codebase-design` (módulo, interface, profundidade, seam, adapter). Projeto com código existente:
  comece por `/improve-codebase-architecture`.
- Nenhuma opção técnica entra no grilling sem ticket de pesquisa fechado comparando as alternativas
  (versão atual, licença, maturidade, custo, quem mantém).
- Restrições do cliente (equipe que vai manter, orçamento, integrações obrigatórias) vêm antes da stack.

## Decisions so far

## Not yet specified

## Out of scope
```

Tickets que costumam nascer:

- *Restrições do projeto* (grilling): quem mantém, orçamento, prazos, integrações, LGPD.
- *Opções de linguagem e framework* (pesquisa) → *Linguagem e framework* (grilling, bloqueado pela pesquisa).
- *Opções de banco de dados* (pesquisa) → *Banco de dados* (grilling).
- *Módulos e seams principais* (grilling, depois do glossário do domínio estar de pé).
- *Integrações externas* (pesquisa por serviço: API, limites, preço).

## Mapa de Fases

```markdown
## Destination

A sequência de fases do desenvolvimento, com o que cada fase entrega, o critério de pronto de cada
uma e a ordem em que os mapas de implementação serão abertos.

## Notes

- Depende do mapa de Arquitetura (stack) e do mapa de Protótipo (desenho inicial aprovado).
- A divisão das fases é decisão do desenvolvedor que vai conduzir o projeto. Comece perguntando a
  ele como quer dividir; a casa não tem ordem padrão. Recomende só com base no que os outros mapas
  decidiram e na pesquisa, com a fonte citada.
- Cada fase fecha com `/to-spec` e `/to-tickets` em fatias verticais.

## Decisions so far

## Not yet specified

## Out of scope
```

Tickets que costumam nascer:

- *Como dividir as fases* (grilling com o desenvolvedor).
- *Seams entre as partes* (grilling): onde ficam as interfaces que deixam uma fase avançar sem a outra.
- *Critério de pronto de cada fase* (grilling).
- *Estratégia de testes por fase* (grilling + `/tdd`).

## Mapa de Infra

```markdown
## Destination

Um retrato da infra que o projeto vai usar: os serviços e tecnologias disponíveis, o custo estimado e
o que teremos que construir em volta deles. É conhecimento de causa para os outros mapas; não trava
a implementação e não precisa fechar.

## Notes

- Mapa leve: poucos tickets, quase todos de pesquisa. Não detalhe ambientes, deploy ou CI aqui;
  isso vira mapa próprio quando a implementação chegar perto.
- Preço, limite e região de provedor só entram com fonte primária e data da consulta.

## Decisions so far

## Not yet specified

## Out of scope
```

Tickets que costumam nascer:

- *Infra disponível para a stack* (pesquisa): serviços e tecnologias que atendem o que a Arquitetura decidiu.
- *Custos estimados* (pesquisa): quanto custa no começo e com o dobro de uso.
- *O que construir em volta* (grilling): o que a infra escolhida não resolve e vira trabalho nosso.
