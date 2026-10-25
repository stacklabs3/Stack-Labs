# Princípios de código: roteiro do grilling

Parte dos [princípios de código](../principios-de-codigo.md). Os tickets do mapa, com o grupo de
princípios de cada um, estão em [mapas-iniciais/regras-de-codigo.md](../mapas-iniciais/regras-de-codigo.md).

## Num ticket de grupo

1. **Leia o arquivo do grupo**, o arquivo da linguagem do projeto (TypeScript, Java, Rust ou Dart, em
   [principios-de-codigo.md](../principios-de-codigo.md)) e, em projeto com código, o resultado do
   ticket *Atrito do código existente*. Pronto quando cada princípio do grupo tem a sua pergunta-guia
   **No grilling**, a linha da linguagem e, se houver, o atrito que o quebra.
2. **Monte a rodada.** Cada pergunta-guia vira uma pergunta do projeto, no formato da regra 10 da
   constituição, com o atrito como exemplo quando houver. A recomendação é **adotar**, **adaptar**
   (e como) ou **afastar** (e por quê; o E1 e o P5 nunca se afastam), com a fonte: *Fontes* do arquivo do grupo para o princípio,
   `docs/research/` para o que depende da stack, a linha do arquivo da linguagem para a regra que o
   confere, as restrições para o que depende do time. Princípio
   que a linha **No validador** confere por máquina leva, na mesma pergunta, o parâmetro que a
   checagem precisa (a pasta do domínio, o formato do commit). Pronto quando todo princípio do grupo
   está numa pergunta numerada.
3. **Resolva o ticket** com uma linha por princípio (`E3: adaptado, erro esperado como exceção com
   tipo próprio`) e as `## Fontes`. Atrito que o projeto vai corrigir vira ticket novo. Pronto quando
   todo princípio do grupo tem linha.

## No ticket *Lint e formatador*

Além das ferramentas, decida cada prática **só desta linguagem** (a seção do arquivo da linguagem):
adotada ou afastada, com o porquê. A adotada vira regra com o ID da prática em `Nasce de:`. Pronto
quando toda prática da linguagem do projeto tem decisão.

## No ticket *Riscos que os testes cobrem*

1. **Levante os candidatos** nas fontes que já existem: as decisões do mapa de Arquitetura, o
   protótipo aprovado, o `GLOSSARY.md` e, em projeto com código, os bugs fechados e o resultado do
   *Atrito do código existente*. Encaixe cada um numa classe de [riscos.md](../riscos.md), com o dano
   concreto. Pronto quando cada classe tem candidato ou o motivo de não ter.
2. **Monte a rodada** no formato da regra 10: cada candidato vira uma pergunta ("Cobrar duas vezes o
   mesmo pedido é risco que o teste cobre?"), com o exemplo do projeto e a recomendação. Use o
   `/human-architect-mindset` para o que é caro de errar e o `/domain-modeling` para o nome de cada
   risco na língua do glossário. Pronto quando todo candidato está numa pergunta numerada.
3. **Escreva `docs/riscos.md`** só com os aprovados, a origem apontando este ticket e a situação
   `pendente #<n>` até a fatia que os cobre. Pronto quando o humano aprovou o registro.

## No ticket *Escrever docs/regras-de-codigo.md*

Junte as decisões no modelo abaixo e escreva `docs/regras-de-codigo.json` a partir de
`.claude/watchdogs/modelos/regras-de-codigo.json`, uma entrada por regra, como manda
[validador.md](../validador.md). O P5 cita a regra com `teste-cobre-risco`, que lê o `docs/riscos.md`
do ticket de riscos. Pronto quando `sh .claude/watchdogs/rodar validar --so-cobertura` termina em
`VALIDADOR: APROVADO` e o humano aprovou o documento.

## Modelo de `docs/regras-de-codigo.md`

```markdown
# Regras de código

Régua do implementador e do revisor. Nasce dos princípios de código da casa
(docs/agents/principios-de-codigo.md); muda só por ticket.

## Garantido pelo lint e pelo formatador

- <ferramenta e versão>: <o que ela garante sozinha>

## Regras

### R1. <a regra, em uma linha>
Nasce de: E1 (adotado | adaptado)
Por quê: <motivo do projeto>
Exemplo: <trecho do próprio projeto>

## Cobertura dos princípios

| Princípio | Decisão | Onde ou por quê |
|---|---|---|
| E1 | adotado | R1 |
| E2 | garantido pelo lint | <regra do lint> |
| E6 | afastado | <o porquê, da restrição ou da pesquisa> |
| ... uma linha para cada um dos 22 (E1 a E8, P1 a P9, S, O, L, I, D) |

## Práticas da linguagem

| Prática | Decisão | Onde ou por quê |
|---|---|---|
| RS2 | adotado | R12 |
| RS5 | afastado | <o porquê> |
```

Regra que nasce de prática leva o ID dela (`TS<n>`, `JV<n>`, `RS<n>`, `DT<n>`) em `principio` no `.json`; a
`cobertura` confere que a tabela de práticas a cita.
