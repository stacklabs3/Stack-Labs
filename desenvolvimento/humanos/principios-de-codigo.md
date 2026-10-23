# Princípios de código

A base da casa para escrever código bem, em qualquer projeto e em qualquer linguagem
([ADR 0008](../adr/0008-principios-de-codigo-da-casa.md)). A versão completa, com exemplo de cada
item e o que o revisor procura no diff, é a que os agentes leem:
[docs/agents/principios-de-codigo.md](../agentes/modelo-de-projeto/docs/agents/principios-de-codigo.md).

## Para que serve

Ela é o **ponto de partida do mapa de Regras de código**. Em vez de começar do zero em todo projeto,
o grilling passa pela base item por item e pergunta: adotamos como está, adaptamos ou deixamos de
fora? O que for decidido vai para o `docs/regras-de-codigo.md` do projeto, que é a régua dura. O que
o projeto não adotou continua valendo como bom senso: o revisor pode apontar, mas só reprova se
mostrar o estrago concreto.

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

## No grilling do mapa de Regras de código

Você vai ouvir perguntas como: "Os erros que o negócio prevê (cupom vencido) voltam como resposta ou
como exceção nesta stack?", "Criamos interface para o banco agora ou só quando houver a segunda
implementação?". Cada uma vem com a recomendação do agente, a fonte e um exemplo do seu projeto.
Afastar um princípio é legítimo: diga o porquê, e ele fica registrado.

## Mudar a base

A base é do método, não do projeto: muda por PR no Stack-Labs e chega aos projetos pelo
`/setup-stacklabs`. Uma exceção só de um projeto vai para o `docs/regras-de-codigo.md` dele.
