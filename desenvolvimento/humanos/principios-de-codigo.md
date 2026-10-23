# Princípios de código

A base da casa para escrever código bem, em qualquer projeto e em qualquer linguagem
([ADR 0008](../adr/0008-principios-de-codigo-da-casa.md)). A versão completa, com exemplo de cada
item e o que o revisor procura no diff, é a que os agentes leem:
[docs/agents/principios-de-codigo.md](../agentes/modelo-de-projeto/docs/agents/principios-de-codigo.md).

## Para que serve

Ela é o **guia do mapa de Regras de código**. O grilling desse mapa roda em torno destes princípios:
cada um vira uma pergunta sobre o seu projeto, e você decide se ele é **adotado**, **adaptado** ou
**deixado de fora**, sempre com o porquê. Nenhum fica sem decisão: o mapa só fecha quando os 22 estão
na tabela de cobertura do `docs/regras-de-codigo.md`.

O que você adotou ou adaptou vira regra do projeto, e o revisor reprova quem a quebra. O que você
deixou de fora não é cobrado. Onde nenhuma regra alcança um trecho, o princípio vale como bom senso:
o revisor pode apontar, mas só reprova se mostrar o estrago concreto.

## Os princípios, em uma linha cada

**Escrita de código**

- **E1.** O nome diz o que é, com as palavras do glossário (`pedidosEmPreparo`, nunca `p`).
- **E2.** Função curta, num só nível, que sai cedo no caso inválido.
- **E3.** Conceito do domínio tem tipo próprio (dinheiro, CPF), validado uma vez, na entrada.
- **E4.** Erro nunca é engolido; o erro que o negócio prevê faz parte da resposta.
- **E5.** Uma função ou responde ou faz, nunca as duas; dado imutável por padrão.
- **E6.** Comentário explica o porquê; código morto se apaga.
- **E7.** A mesma regra de negócio não se repete; parecença de forma, nem sempre.
- **E8.** Número ou texto com significado tem nome e mora num lugar só.

**Padrões de desenvolvimento**

- **P1.** Poucos módulos que escondem bastante, em vez de muitos que só repassam.
- **P2.** Interface (seam) só onde algo varia de verdade.
- **P3.** O módulo recebe o que usa; a regra de negócio não conhece banco nem provedor.
- **P4.** O que muda junto fica junto.
- **P5.** Teste pela interface, com o nome do teste em frase do negócio.
- **P6.** Teste vermelho, verde, depois refatora; refatoração e mudança de comportamento em commits separados.
- **P7.** Nada para um requisito que não existe.
- **P8.** Consistência vence preferência: mudar um padrão é decisão, por ticket.
- **P9.** Commit conta uma mudança, e a mensagem diz o porquê.

**SOLID**, lido para não virar código picado:

- **S.** Um módulo muda por um motivo só, e motivo é *quem pede a mudança* (o dono, o contador).
- **O.** Variante nova (outra forma de pagamento) entra como peça nova, sem mexer no que funciona.
- **L.** Toda peça que cumpre um contrato cumpre ele inteiro, inclusive os erros.
- **I.** Quem usa conhece só o que usa.
- **D.** A regra de negócio define o que precisa; banco e provedor se adaptam a ela.

## Como o grilling anda

1. **Restrições primeiro:** quem vai manter, o nível do time, quanto do código o agente escreve, o
   que já deu errado. São elas que justificam adaptar ou deixar algo de fora.
2. **Depois, um ticket por grupo**, uma sessão cada:

   | Ticket | Princípios |
   |---|---|
   | Escrita de código | E1, E2, E6, E7, E8 |
   | Tipos, erros e validação | E3, E4, E5 |
   | Desenho de módulos e SOLID | P1 a P4, S, O, L, I, D |
   | Testes e fluxo de trabalho | P5 a P9 |

3. **Cada pergunta** é curta, com um exemplo do seu projeto e a recomendação do agente com a fonte.
   Por exemplo: "Quando o cupom está vencido, o sistema devolve uma resposta 'cupom vencido' que a
   tela trata, ou lança um erro? Recomendo a resposta, pelo princípio E4, com a fonte dele: a tela fica obrigada a tratar o caso."
4. **Você responde por número.** "Sim" aceita; qualquer outra coisa, explique. Deixar de fora é
   legítimo: diga o porquê, e ele fica registrado.
5. **No fim**, o agente junta tudo no `docs/regras-de-codigo.md`, com a tabela dos 22 princípios, e
   você aprova o documento.

## Mudar a base

A base é do método, não do projeto: muda por PR no Stack-Labs e chega aos projetos pelo
`/setup-stacklabs`. Uma exceção só de um projeto vai para o `docs/regras-de-codigo.md` dele.
