# Contrato da fatia

O comentário `## Contrato da fatia` que você publica na issue **antes da primeira linha de código**
(regra 24 da constituição, ADR 0023 do método). Ele traduz os documentos do projeto para este diff:
cada linha diz o que o código vai cumprir e de qual documento isso veio. O `guarda-edicao` libera o
código só depois que ele está na issue; o revisor e o conferente leem o diff com ele na mão.

O contrato é **compromisso**, não fonte: a verdade continua na issue, no spec, em
`docs/regras-de-codigo.md`, no `GLOSSARY.md`, em `docs/adr/` e em `docs/riscos.md`. Quando o
contrato e um documento divergem, o documento vence e o contrato se corrige por escrito.

## Escrever o contrato

1. **Ache o terreno.** Rode a consulta de contexto (regra 23) com cada termo da issue e cada símbolo
   que vai mudar, e abra os trechos citados. Para cada arquivo que vai nascer ou mudar, escolha o
   **vizinho**: o arquivo existente que faz o mesmo tipo de coisa e que o seu vai imitar em forma,
   nomes, tratamento de erro e testes (P8: consistência vence preferência). Leia o vizinho inteiro.
   Pronto quando todo arquivo de *Toca* tem vizinho, ou a palavra `primeiro` com o porquê.
2. **Recolha o que alcança o diff**, documento a documento, só o que alcança:
   - `docs/regras-de-codigo.md`: cada regra cujo escopo pega um arquivo de *Toca*, com **como** o diff
     a cumpre em meia linha. A regra que o lint ou o validador garante entra do mesmo jeito: o
     `vigia-codigo` e o `vigia-teste` a cobram a cada edição, e o contrato diz o que você vai fazer
     para passar na primeira.
   - `GLOSSARY.md`: cada conceito que o diff nomeia, com o nome que ele terá no código. Conceito sem
     termo é lacuna: `BLOQUEADO` com a pergunta ao humano, pelo `/domain-modeling`.
   - `docs/adr/`: cada ADR que fixa algo neste trecho, com o que ela fixa aqui.
   - `docs/riscos.md`: cada seam que a issue ou o spec combinam, o `RK<n>` que o teste nela cobre e
     o nome do teste que vai existir. Fatia sem risco a cobrir escreve `sem teste, pela issue`.
   - A issue e o spec: cada critério de aceite, citado, com o teste ou a conferência que o prova.
   Pronto quando toda regra que alcança *Toca* tem linha, todo critério tem prova, todo teste tem
   `RK<n>`, todo conceito tem termo e toda ADR do trecho está citada.
3. **Publique** na issue, no modelo abaixo, com `gh issue comment <n> --body-file <arquivo>` (o
   arquivo em `.scratch/`). Pronto quando o comentário está na issue: é ele que o `guarda-edicao`
   procura antes de liberar o código.

## Modelo

```markdown
## Contrato da fatia

**Toca:** `src/cupom/aplicar.ts` (vizinho: `src/pedido/fechar.ts`) · `src/cupom/aplicar.test.ts` (vizinho: `src/pedido/fechar.test.ts`)
**Seams e riscos:** `aplicarCupom(pedido, cupom)` → RK2 em `recusa cupom vencido` · RK7 em `aplica uma vez por pedido`
**Regras de código:** R3 erro esperado vira `ResultadoDeCupom`, não exceção · R6 `src/cupom/` não importa `src/infra/` · R11 nome do teste cita o risco
**Linguagem:** Cupom → `Cupom` · Comanda → `Comanda` (nunca `Ticket`)
**ADRs:** ADR 0003 (cupom vence à meia-noite do fuso da loja): a data vem de `RelogioDaLoja`
**Critérios de aceite:** "cupom vencido é recusado com a mensagem X" → `recusa cupom vencido` · "o desconto aparece no total" → `aplica o desconto no total`
**Fora do contrato:** cupom cumulativo (a issue não pede)
```

Uma linha por item, com o ID do documento de onde veio. O contrato cabe na tela: o que não alcança
este diff fica fora dele.

## Implementar contra o contrato

Cada edição começa pela linha do contrato que ela cumpre e pelo vizinho que ela imita. O
`vigia-codigo` e o `vigia-teste` devolvem na hora a regra quebrada, com o porquê de
`docs/regras-de-codigo.md`: a correção vai para o arquivo, do jeito que a regra pede. Correção que
sai de uma linha do contrato (outra seam, outro nome, outro teste) atualiza o contrato por escrito,
editando o comentário, antes de seguir: o diff e o contrato andam juntos.

Antes de dizer "pronto", percorra o contrato linha a linha e aponte onde cada uma está no diff.
Pronto quando toda linha tem o seu trecho, e nenhum trecho do diff está fora de uma linha ou de
*Fora do contrato*.

## Na revisão e no ciclo de correção

- O **revisor** lê o diff com o contrato e com os documentos: cada linha do contrato cumprida no
  diff, e cada regra de `docs/regras-de-codigo.md` que alcança o diff e **falta no contrato** é
  achado, porque mostra o documento que não foi lido.
- O **conferente** confere a coluna *Critérios de aceite* contra a issue e contra o diff: critério da
  issue fora do contrato é achado (a); trecho fora de *Fora do contrato* é achado (b).
- O **diagnosticador** escreve cada passo do plano com a linha do contrato que ele preserva. Achado
  que exige sair do contrato vira um passo `contrato:` que diz a linha nova; assim a correção muda o
  compromisso por escrito em vez de deixar o código à deriva do documento.
- O **implementador** de um ciclo de correção relê o contrato antes do primeiro edit e cumpre o
  plano item a item. A correção **cumpre o contrato**: resolve o achado dentro das linhas que ele
  fixa, e muda uma linha só pelo passo `contrato:` do plano.
