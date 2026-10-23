# Roteiro de protótipo (família `*-prototipo`)

Como a casa prototipa aparência. As skills da família ficam no `~/.claude/skills/` de quem trabalha
(cópia oficial em `desenvolvimento/agentes/skills/recomendada/` do repositório Stack-Labs); este
roteiro diz quando usar cada uma. Comportamento e lógica
não passam por aqui: usam `/prototype`.

## As skills

| Skill | Quando | Quantas vezes |
|---|---|---|
| `/novo-prototipo` | Primeiro ticket do mapa de Protótipo: cria o **protótipo único** em `prototipos/<projeto>/`, com as abas e as visões decididas | Uma vez por projeto |
| `/iniciar-prototipo` | Servir o protótipo localmente e abrir no navegador para ver ou conferir | Quantas precisar |
| `/integrar-ao-prototipo` | Cada rotina nova, reusando peças comuns e tokens | Uma por rotina |
| `/melhorar-prototipo` | Refinar uma tela que já existe (hierarquia, espaçamento, tipografia, microcopy, estados, acessibilidade), inclusive os defeitos que a auditoria achou, mantendo a forma aprovada | Quantas precisar |
| `/ajustar-detalhe-prototipo` | Ajuste fino visual que não converge por descrição: o humano escolhe o valor com sliders e guias | Quando "sobe um pouco… demais" não converge |
| `/auditar-prototipo` | Antes de levar um lote à aprovação | Uma por rodada |
| `/sincronizar-prototipo` | Ao fechar uma tela aprovada: reflete na vitrine e sobe a versão de todos os scripts | Uma por fechamento |
| `/portar-prototipo` | Levar a rotina aprovada para a stack real, com o plano de implementação | Só com aprovação explícita |

As convenções, o checklist de qualidade, o modelo e o script de prints que essas skills usam ficam na
base compartilhada `prototipo-html` (em `~/.claude/skills/prototipo-html/`), que se instala junto.

## Como um lote anda

Um subagente por vez, nunca dois ao mesmo tempo.

1. **Construção** da rotina com `/integrar-ao-prototipo`: um subagente, um commit.
2. **Acabamento** da rotina por outro subagente, só sobre o diff da construção.
3. Repete 1 e 2 para cada rotina do lote, na ordem dos tickets.
4. **Auditoria** com `/auditar-prototipo`; correções com `/melhorar-prototipo`. A partir da segunda
   rodada, audite só o que a anterior corrigiu.
5. Os prints, a tabela de telas e os **pontos para aprovação** vão ao humano no ticket.
6. O humano escreve a **aprovação explícita** no comentário de resolução. Só então os tickets do lote
   fecham e o `/portar-prototipo` fica liberado.

## Rotina pronta

- caminho contado e toques medidos em cada superfície;
- estados vazio, carregando, erro e sem internet;
- abre em cada superfície decidida, claro e escuro, com o console limpo;
- foco visível, alvos ≥ 44 px no celular, contraste AA;
- só tokens e strings centralizadas; vocabulário do `GLOSSARY.md`;
- prints tirados **e abertos** pelo agente antes do commit.

## Pontos para aprovação

Em `.scratch/<lote>/pontos-para-aprovacao.md`: uma seção por rotina, os pontos numerados, em
palavras simples, com o cenário do cliente e o que foi escolhido em cada um. Dúvida real de aparência
vira A/B no protótipo, não pergunta solta.

O projeto pode ter o próprio roteiro de lote, mais detalhado (o `appetito_v2` tem
`docs/agents/roteiro-de-lote-de-prototipo.md`). Ele complementa este, nunca o contradiz.
