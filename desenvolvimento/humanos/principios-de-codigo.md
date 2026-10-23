# Princípios de código

São 22 princípios da casa para escrever código, que valem em qualquer projeto e linguagem: oito de
escrita (E1 a E8), nove de padrões de desenvolvimento (P1 a P9) e SOLID, lido para não virar código
picado. A lista, com exemplo de cada um, está em
[docs/agents/principios-de-codigo.md](../agentes/modelo-de-projeto/docs/agents/principios-de-codigo.md).
O porquê de existirem está na [ADR 0008](../adr/0008-principios-de-codigo-da-casa.md).

## O que você decide

Os princípios guiam o grilling do [mapa de Regras de código](mapas-iniciais.md#4-regras-de-código).
Cada um vira uma pergunta sobre o seu projeto, e você decide se ele é **adotado**, **adaptado** ou
**deixado de fora**, sempre com o porquê.

- **Restrições primeiro:** quem vai manter, o nível do time, quanto do código o agente escreve, o
  que já deu errado. Elas justificam adaptar ou deixar algo de fora.
- **Depois, um ticket por grupo**, uma sessão cada: escrita; tipos, erros e validação; módulos e
  SOLID; testes e fluxo de trabalho.
- **Cada pergunta** é curta, com um exemplo do seu projeto e a recomendação do agente com a fonte.
  Por exemplo: "Quando o cupom está vencido, o sistema devolve uma resposta 'cupom vencido' que a
  tela trata, ou lança um erro? Recomendo a resposta, pelo princípio E4: a tela fica obrigada a
  tratar o caso."
- **No fim**, você aprova o `docs/regras-de-codigo.md`, com os 22 princípios na tabela de cobertura.

## O que vale depois

O que você adotou ou adaptou vira regra do projeto. O que dá para conferir por máquina (imports,
`catch` vazio, formato do commit, termos do glossário) o **validador** confere no CI; o resto o
revisor confere, e reprova quem quebra uma regra. O que ficou de fora não é cobrado. Onde nenhuma regra alcança um trecho, o princípio vale como bom senso: o
revisor aponta, e só reprova mostrando o estrago concreto.

## Mudar a base

Por PR no Stack-Labs; chega aos projetos pelo `/setup-stacklabs`. Exceção de um projeto só vai para
o `docs/regras-de-codigo.md` dele.
