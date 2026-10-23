# Princípios de código

A base da casa para escrever código em todo projeto, em qualquer linguagem (ADR 0008 do método). É
o **guia do grilling do mapa de Regras de código** e, depois, a régua de julgamento do revisor onde
`docs/regras-de-codigo.md` não alcança.

## Como vale num projeto

- **`docs/regras-de-codigo.md` vence.** Princípio **adotado** ou **adaptado** no mapa vira regra
  dura lá, com exemplo do projeto e o ID de onde nasceu (`Nasce de: E3`).
- **Princípio afastado** fica fora da régua e do revisor: o projeto decidiu, com o porquê registrado.
  O **E1** é a exceção: é regra 20 da constituição, e o projeto o adapta, mas não o afasta.
- **Onde nenhuma regra alcança o trecho**, o princípio é julgamento, como os *smells* do
  `/code-review`: o revisor aponta pelo ID e só reprova com o dano concreto nesta issue.
- O que o lint garante é da ferramenta.

Precedência: `docs/regras-de-codigo.md` → ADRs do projeto → estes princípios → *smells* do `/code-review`.

## Os 22 princípios

Cada um traz a regra, o porquê, um exemplo, a pergunta **No grilling**, o que procurar **No diff** e
a checagem que o confere **No validador** ([validador.md](validador.md)).
**Ao escrever ou revisar código** que toca um grupo, leia o arquivo dele.

**[Escrita](principios-de-codigo/escrita.md)**: E1 nome na língua do domínio · E2 função curta, um
nível, saída cedo · E3 tipo próprio para conceito do domínio, validado na borda · E4 erro nunca
engolido, erro esperado na interface · E5 comando ou consulta, imutável por padrão · E6 comentário
diz o porquê · E7 duplicação de conhecimento se remove · E8 valor com significado tem nome.

**[Padrões](principios-de-codigo/padroes.md)**: P1 módulo profundo · P2 seam só onde algo varia ·
P3 dependência entra, domínio sem infraestrutura · P4 o que muda junto fica junto · P5 teste pela
interface · P6 vermelho, verde, refatorar · P7 nada para requisito que não existe · P8 consistência
vence preferência · P9 commit conta uma mudança.

**[SOLID](principios-de-codigo/solid.md)**, lido com o vocabulário do `/codebase-design`: S um ator
por módulo · O variante nova como adapter, nas seams reais · L adapter cumpre a interface inteira ·
I quem chama conhece só o que usa · D a regra define a port.

**Ao escrever, revisar ou conduzir o grilling em TypeScript, Java ou Rust**, leia também o arquivo
da linguagem: como cada princípio aparece nela, a regra que o confere e as práticas só dela.
[TypeScript](principios-de-codigo/typescript.md) (TS1 a TS5) ·
[Java](principios-de-codigo/java.md) (JV1 a JV6) · [Rust](principios-de-codigo/rust.md) (RS1 a RS6).

**Ao conduzir um ticket do mapa de Regras de código**, leia
[principios-de-codigo/grilling.md](principios-de-codigo/grilling.md): os tickets por grupo, como
cada rodada anda e o modelo de `docs/regras-de-codigo.md`.
