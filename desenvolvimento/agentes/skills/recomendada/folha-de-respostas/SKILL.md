---
name: folha-de-respostas
description: Gera a folha de respostas .html de uma rodada de perguntas ao humano — perguntas numeradas com cenário, opções A/B/C com a recomendada marcada, campo de observação aberto e os botões "Marcar a recomendação nas que faltam" e "Copiar respostas" (texto "Q1 = A (recomendado) · Obs: …" para colar no chat). Use quando uma rodada de grilling, de dúvidas de um lote ou qualquer lista de decisões com recomendação precisar ir ao humano numa folha para marcar, ou quando o usuário pedir "o .html com as perguntas e o botão de copiar".
---

# Folha de respostas

Escreva as perguntas num JSON e rode o script. Ele valida o JSON e gera a folha: um arquivo só, sem internet, que abre com dois cliques, guarda o que foi marcado no navegador e tem tema claro e escuro.

## Quick start

```bash
node ~/.claude/skills/folha-de-respostas/scripts/gerar-folha.mjs perguntas.json respostas.html && open respostas.html
```

`perguntas.json`:

```json
{
  "titulo": "Minha Loja: rodada 2 de regras",
  "tituloDaAba": "Respostas 2 · Minha Loja",
  "intro": ["Cenário: o Caldo da Vila Centro, da Ana (dona). Esta rodada sai das suas observações na rodada 1."],
  "chave": "respostas-minha-loja-2",
  "cabecalhoDaCopia": "Respostas da rodada 2 de regras (Minha Loja, onda 1)",
  "perguntas": [
    { "bloco": "O menu" },
    { "id": "Q55", "titulo": "Quem muda a ordem do menu",
      "cena": "A Ana usa Cardápio todo dia; o Caio vive em Pedidos.",
      "opcoes": ["Cada pessoa organiza o próprio menu (…)", "A loja define uma ordem só"],
      "rec": 0 }
  ]
}
```

| Campo | Obrigatório | O que é |
|---|---|---|
| `titulo` | sim | O título da folha |
| `tituloDaAba` | não | O título da aba do navegador (sem ele, usa `titulo`) |
| `intro` | não | Parágrafos de abertura: cenário e o que a rodada não pergunta |
| `chave` | sim | Nome único da rodada, para o navegador guardar as marcas sem misturar com outra folha |
| `cabecalhoDaCopia` | sim | A primeira linha do texto copiado |
| `perguntas` | sim | Em ordem; `{ "bloco": "…" }` abre um grupo com título |
| `id` | sim | `Q1`, `Q2`… Só letras, números, `_` ou `-`, sem repetir |
| `cena` | não | O cenário concreto da pergunta, com a fonte da recomendação |
| `opcoes` | sim | De 2 a 5 textos; viram A, B, C… |
| `rec` | não | O índice (a partir de 0) da opção recomendada; ganha o selo "recomendado" |

O texto copiado sai assim, uma linha por pergunta, e `—` na pergunta sem marca:

```
Respostas da rodada 2 de regras (Minha Loja, onda 1)
Q55 = A (recomendado)
Q56 = B · Obs: o texto livre que a pessoa escreveu
```

## Como escrever uma boa rodada

- Uma pergunta por decisão independente. A que depende de uma resposta ainda aberta vai para a rodada seguinte.
- Cada pergunta tem uma cena com pessoas e números ("A Ana quer mudar o Caldo verde de R$ 24 para R$ 26"), sem jargão, e diz de onde vem a recomendação (concorrente real, pesquisa, decisão anterior).
- A opção recomendada vem primeiro (`"rec": 0`). Cada opção se lê sozinha e diz o efeito, não só o nome.
- Na rodada seguinte, continue a numeração (Q55, Q56…) e use uma `chave` nova.
- Os textos são texto puro. O script escapa o HTML, então não use tags.
- Se o script recusar, ele lista cada problema. Corrija o JSON e rode de novo.

## Depois de gerar

Abra a folha (`open`), diga ao humano onde ela está e espere ele colar as respostas no chat. Guarde o JSON junto da folha (ex.: `respostas-2.json`), para refazer a folha se uma pergunta mudar.
