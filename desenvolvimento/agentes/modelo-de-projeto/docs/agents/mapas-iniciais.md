# Mapas iniciais

Todo projeto abre cinco mapas do `/wayfinder` antes de qualquer código de produção
(ADRs 0002 e 0007 do método). Cada mapa é uma issue com `wayfinder:map` **e** `mapa-inicial:<tipo>`, que é
como os watchdogs encontram o mapa. Num projeto que já existia antes do método, os mapas são
**retroativos**: registram as decisões já tomadas e fecham na hora quando nada está em aberto
(`/adotar-metodo-stack-labs`).

| Mapa | Label | Destino | Precisa estar fechado para ativar issue? |
|---|---|---|---|
| Protótipo | `mapa-inicial:prototipo` | O desenho inicial do produto prototipado e aprovado | **Sim** |
| Arquitetura | `mapa-inicial:arquitetura` | Linguagem, stack e módulos decididos, em ADRs | **Sim** |
| Fases | `mapa-inicial:fases` | A divisão em fases que o desenvolvedor escolheu, com o que entra e o critério de pronto de cada uma | **Sim** |
| Regras de código | `mapa-inicial:regras-de-codigo` | As regras que todo código do projeto segue, escritas em `docs/regras-de-codigo.md`, e o lint que as garante escolhido | **Sim** |
| Infra | `mapa-inicial:infra` | Retrato da infra que vamos usar: custos, tecnologias e o que construir em volta | Não (informativo) |

## Como abrir

1. Uma sessão por mapa, com `/wayfinder` em modo *Chart the map*. Comece pelo Protótipo e pela
   Arquitetura; Fases, Regras de código e Infra dependem deles. Infra é o menor dos cinco: poucos
   tickets, quase todos de pesquisa.
2. Crie o mapa com as duas labels e o corpo do modelo abaixo. Ajuste o destino ao projeto no grilling.
3. Os tickets sugeridos são ponto de partida, não lista fechada (exceto os tickets de grupo do mapa
   de Regras de código, obrigatórios). O grilling breadth-first decide
   quais existem; o que não tiver pergunta precisa vai para *Not yet specified*.
4. Ligue os bloqueios entre mapas com dependências nativas: os tickets de Fases, de Regras de código e
   de Infra que dependem de uma escolha de Arquitetura ficam `blocked_by` o ticket dela.
5. Dispare os subagentes `/research` dos tickets de pesquisa no fim da sessão de mapear.
6. Feche a sessão com o `/nova-sessao`: o kickoff abre a próxima janela, que pega o próximo ticket
   da fronteira (um ticket de decisão por sessão).

## Regras que valem nos cinco

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

## Mapa de Regras de código

O grilling deste mapa roda em torno dos **princípios de código** da casa
([principios-de-codigo.md](principios-de-codigo.md), ADR 0008 do método): cada ticket cobre um grupo
de princípios, e cada princípio sai com uma decisão. O roteiro das rodadas e o modelo do documento
final estão na seção *Roteiro do grilling* de lá.

```markdown
## Destination

docs/regras-de-codigo.md escrito e aprovado: cada um dos 22 princípios de código da casa (E1 a E8,
P1 a P9, S, O, L, I, D) adotado, adaptado ou afastado com o porquê, as regras que saíram deles com
exemplo do próprio projeto, e o que o lint e o formatador garantem sozinhos. É a régua do
implementador e do revisor da orquestração.

## Notes

- Guia do grilling: docs/agents/principios-de-codigo.md. Cada pergunta nasce da pergunta-guia
  "No grilling" do princípio, reescrita com um exemplo real deste projeto, e traz a recomendação
  (adotar, adaptar ou afastar) com a fonte. Ticket de grupo só fecha com uma linha de decisão por
  princípio no comentário de resolução.
- Depende do mapa de Arquitetura: as regras são da linguagem e da stack decididas lá. Os tickets que
  dependem de uma escolha de stack ficam `blocked_by` o ticket dela.
- As restrições (quem mantém, nível do time, o que o agente erra) vêm primeiro, com
  `/human-architect-mindset`, e bloqueiam os tickets de grupo: são elas que justificam adaptar ou
  afastar um princípio.
- Projeto com código existente: o `/improve-codebase-architecture` roda antes dos grupos; cada
  atrito vira exemplo na pergunta do princípio que ele quebra.
- Lint, formatador e checagem de tipo são pesquisados como qualquer escolha técnica: versão atual,
  manutenção e o que cada regra pega, com fonte. O princípio que a ferramenta garante sozinha entra
  na cobertura como "garantido pelo lint", sem repetir a regra.
- Configurar o lint e o CI é implementação: entra como a primeira fatia da primeira fase, não como
  ticket deste mapa.
- O documento é curto e verificável. Regra que o revisor não consegue apontar num diff não é regra.
- Somam-se aos princípios as regras da constituição e a lista de smells do `/code-review`.

## Decisions so far

## Not yet specified

## Out of scope
```

Tickets que costumam nascer (os de grupo são obrigatórios; um grupo pode virar mais de um ticket se
não couber numa sessão):

- *Restrições para as regras* (grilling + `/human-architect-mindset`): quem mantém, nível do time,
  quanto do código o agente escreve, o que já deu errado em projetos parecidos. Bloqueia os grupos.
- *Atrito do código existente* (tarefa, só em projeto com código): `/improve-codebase-architecture`,
  com cada atrito ligado ao princípio que ele quebra.
- *Escrita de código: E1, E2, E6, E7, E8* (grilling + `/domain-modeling`): nomes e idioma, tamanho de
  função, comentários, duplicação, constantes.
- *Tipos, erros e validação: E3, E4, E5* (grilling + `/domain-modeling`): tipos do domínio, onde se
  valida a entrada, como um erro sobe, o que se loga, imutabilidade.
- *Desenho de módulos e SOLID: P1 a P4, S, O, L, I, D* (grilling + `/codebase-design`): profundidade,
  onde ficam as seams, quando um adapter se justifica, como chegam as dependências, organização das pastas.
- *Testes e fluxo de trabalho: P5 a P9* (grilling + `/tdd`): o que se testa e o que se mocka, TDD,
  generalidade, padrões fixos, commits.
- *Opções de lint, formatador e typecheck* (pesquisa) → *Lint e formatador* (grilling): quais
  princípios a ferramenta garante sozinha.
- *Escrever docs/regras-de-codigo.md* (tarefa): junta as decisões no modelo de
  principios-de-codigo.md, com a tabela de cobertura dos 22 princípios; fecha o mapa.

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
