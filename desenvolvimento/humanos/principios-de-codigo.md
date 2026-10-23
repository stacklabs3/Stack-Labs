# Princípios de código

São 22 princípios da casa para escrever código, que valem em qualquer projeto e linguagem: oito de
escrita (E1 a E8), nove de padrões de desenvolvimento (P1 a P9) e SOLID, lido para não virar código
picado. A lista, com exemplo de cada um, está em
[docs/agents/principios-de-codigo.md](../agentes/modelo-de-projeto/docs/agents/principios-de-codigo.md).
O porquê de existirem está na [ADR 0008](../adr/0008-principios-de-codigo-da-casa.md).

## O que você decide

Os princípios guiam o grilling do [mapa de Regras de código](mapas-iniciais.md#4-regras-de-código).
Cada um vira uma pergunta sobre o seu projeto, e você decide se ele é **adotado**, **adaptado** ou
**deixado de fora**, sempre com o porquê. São duas as exceções, que valem em todo projeto e o
grilling só decide como: o **E1**, nome que diz o que é (`produto = valor - custo`, nunca
`x = y - z`), e o **P5**, todo teste cobre um risco.

- **Restrições primeiro:** quem vai manter, o nível do time, quanto do código o agente escreve, o
  que já deu errado. Elas justificam adaptar ou deixar algo de fora.
- **Depois, um ticket por grupo**, uma sessão cada: escrita; tipos, erros e validação; módulos e
  SOLID; testes e fluxo de trabalho.
- **Cada pergunta** é curta, com um exemplo do seu projeto e a recomendação do agente com a fonte.
  Por exemplo: "Quando o cupom está vencido, o sistema devolve uma resposta 'cupom vencido' que a
  tela trata, ou lança um erro? Recomendo a resposta, pelo princípio E4: a tela fica obrigada a
  tratar o caso."
- **Práticas da linguagem:** para TypeScript, Java e Rust, cada princípio já vem com a regra de
  ferramenta que o confere naquela linguagem, e cada linguagem traz práticas só dela (em Rust, por
  exemplo, não usar `unsafe`). Você decide cada uma no ticket de lint; elas ficam fora da conta dos 22
  ([ADR 0012](../adr/0012-praticas-da-linguagem.md)).
- **Riscos que os testes cobrem:** num ticket próprio, você decide o que, se der errado, causa dano
  de verdade no produto (cobrar duas vezes, aceitar cupom vencido, mostrar dado de outra loja). Cada
  risco aprovado entra no `docs/riscos.md` com um ID (`RK1`). Depois, a spec e os bugs trazem riscos
  novos, e você aprova cada um.
- **No fim**, você aprova o `docs/regras-de-codigo.md`, com os 22 princípios na tabela de cobertura.

## O que vale depois

O que você adotou ou adaptou vira regra do projeto. O que dá para conferir por máquina (imports,
`catch` vazio, formato do commit, termos do glossário) o **validador** confere no CI; o resto o
revisor confere, e reprova quem quebra uma regra. O que ficou de fora não é cobrado. Onde nenhuma regra alcança um trecho, o princípio vale como bom senso: o
revisor aponta, e só reprova mostrando o estrago concreto.

## Testes que valem

Todo teste cita o risco que cobre. Teste que não protege nada (de getter, do framework, sem
afirmação, que compara um valor com ele mesmo, desligado) não entra: o hook `vigia-teste` devolve ao
agente na hora em que ele escreve, o validador reprova no CI e o revisor confere se o teste de fato
exercita o risco ([ADR 0014](../adr/0014-todo-teste-cobre-um-risco.md)). Por isso o risco é decisão
sua: é ele que diz onde vale a pena testar.

## Mudar a base

Por PR no Stack-Labs; chega aos projetos pelo `/setup-stacklabs`. Exceção de um projeto só vai para
o `docs/regras-de-codigo.md` dele.
