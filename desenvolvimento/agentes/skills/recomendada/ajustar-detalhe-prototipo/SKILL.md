---
name: ajustar-detalhe-prototipo
description: Constrói uma ferramenta interativa com sliders, arrasto e guias visuais para o usuário escolher ele mesmo valores visuais exatos — posição, espaçamento, tamanho — e exporta os valores mapeados às linhas de código. Use quando o usuário pede ajuste fino que não converge por descrição ("sobe um pouco… demais… menos"), quando cada tentativa exige rebuild lento, ou quando o controle que você mexe quase não move o resultado. NÃO use quando o valor certo é óbvio, quando ninguém precisa julgar visualmente, ou para mudança de design mais ampla (melhorar-prototipo).
---

# ajustar-detalhe-prototipo

Quando um humano precisa escolher **valores visuais exatos** e seu loop é
`edita → rebuild → olha → "um pouco mais… demais"`, pare de adivinhar. Construa uma
ferramenta que renderiza o componente fielmente, expõe os valores como controles ao
vivo com guias objetivas, e exporta os números já mapeados às linhas de código.

O humano ajusta; você porta uma vez. Seu papel muda de *"adivinhar o número"* para
*"construir o botão e ler o resultado"*.

**Leia primeiro:** `~/.claude/skills/prototipo-html/references/convencoes.md`.

## O gatilho

**Depois de 1 ou 2 palpites que não acertaram — não depois de cinco.** Reconhecer cedo é
metade do valor. Nomeie em voz alta: "isso é problema de ajuste, não de valor — vou
construir o controle."

Sinais de que já passou da hora:
- O pedido é vago por natureza ("mais pra baixo", "respirando melhor")
- Cada ciclo custa caro (rebuild nativo, deploy de preview)
- O controle que você mexe quase não move a coisa

**Quando não usar:** valor único e óbvio, nenhum julgamento humano necessário, ou o loop
já é instantâneo **e** você sabe nomear o alvo exato.

## Por que o loop não converge

Duas falhas se somam:

1. **Superfície de controle errada.** Controles que distribuem sobra (`flex`, `Spacer`,
   `justify-content`) só têm o que distribuir se sobrar espaço. Com conteúdo quase
   preenchendo a tela, mudar a proporção move poucos pixels. O usuário pede "bem mais
   pra baixo" e o botão não entrega. Caso real: mudar de 2:1 para 3:1 moveu 15px.

2. **Você é um tradutor com perda.** O humano descreve uma sensação → você chuta um
   inteiro → rebuild lento → ele reage a um número que nunca viu. O viés de recência
   apaga a versão anterior e ninguém consegue segurar um valor.

A solução ataca os dois: muda o meio (browser, tempo real) **e** entrega o controle
direto à mão de quem julga.

## Os 5 detalhes que fazem funcionar

Sem qualquer um deles, o vaivém volta.

| # | Detalhe | Se faltar |
|---|---|---|
| 1 | **Reconhecer o gatilho cedo** — 1 ou 2 palpites, e troca de abordagem | Cinco ciclos de rebuild e frustração |
| 2 | **Inicializar nos valores reais de hoje**, com botão "resetar para o atual" | A ferramenta mente: ele ajusta contra uma ficção |
| 3 | **Exportar mapeado às linhas de código** (`gap: 36` → `linha 84 de x.dart`) + botão de copiar | Você re-traduz e reintroduz o erro |
| 4 | **Guias objetivas e aviso de validade** — linha de centro, marcador do alvo, delta ao vivo, aviso de overflow/corte | Ele aprova um layout que estoura no aparelho |
| 5 | **Modelar como o código real dispõe, e escolher controle que alcança o alvo** | Ajusta em `flex` (não chega lá) ou em px absoluto que não sobrevive ao porte |

Debaixo dos cinco: **renderizar fielmente**. Assets e SVG reais, ordem real dos
elementos, altura real dos campos, moldura de aparelho com safe area, e seletor de
tamanho de tela — "o centro" depende da altura da tela.

## Fluxo

1. **Nomeie o gatilho** e diga que vai construir o controle.
2. **Trave a granularidade** com `AskUserQuestion`: arrastar o bloco inteiro · ajuste
   fino leve · controle por gap individual. Granularidade demais também trava a decisão.
3. **Leia os valores atuais** no código de produção — a ferramenta abre neles (detalhe 2).
4. **Construa** em `<raiz>/<task-id>/`, seguindo o contrato (React 18 + Babel via CDN,
   tokens em CSS var, light e dark). Cada valor ganha slider **e** campo numérico:
   slider para explorar, campo para fechar.
5. **Verifique você mesmo** antes de entregar: suba com `/iniciar-prototipo`, console
   limpo, exercite um slider e o arrasto, confira nos dois temas. A ferramenta precisa
   renderizar fiel **antes** do humano tocar nela.
6. **Entregue a URL.** Ele ajusta até as guias confirmarem e aprova.
7. **Leia os valores exportados** e porte **de uma vez só** — uma edição, valores 1:1.
   Sem re-interpretar.
8. **Verifique o resultado real** e **abra a imagem**. Comparar hash ou tamanho de
   arquivo não é verificação.

## O painel de export

O que separa a ferramenta útil da ferramenta bonita:

```
topo:        88px   →  linha 42: SizedBox(height: 88)
logo:       240px   →  linha 51: BrandLogo(width: 240)
logo→email:  55px   →  linha 63: SizedBox(height: 55)
                                              [copiar]
```

Cada valor ao lado do lugar exato que muda. Assim o porte é mecânico, não uma segunda
rodada de adivinhação.

## Guias que tornam "quase no meio" mensurável

- Linha de centro do espaço útil
- Marcador no elemento que o usuário citou como referência
- Delta ao vivo ("38px acima do centro")
- **Aviso de overflow ou corte** — o valor mais bonito às vezes empurra conteúdo para
  fora da tela; sem o aviso ele aprova sem saber

Caso real: o bloco de login (587px) era mais alto que a área útil (711px), então centrar
o e-mail jogava o link de cadastro para fora. O aviso tornou o trade-off visível, e o
usuário escolheu conscientemente outra composição.

## Anti-padrões

| Erro | Correção |
|---|---|
| Continuar mexendo no mesmo controle porque "mais um pouco resolve" | Depois de 2 palpites, construa a ferramenta |
| Ferramenta abre em valores arbitrários | Inicialize nos valores reais + botão de reset |
| Exportar números soltos | Exporte mapeado às linhas, com botão de copiar |
| Buscar o centro perfeito ignorando se cabe | Aviso de overflow; deixe o trade-off explícito |
| Ajustar em `flex` e depois brigar com o porte | Modele como o código real dispõe |
| Dizer "ficou bom" a partir do tamanho do arquivo de imagem | Abra e **olhe** a imagem |
