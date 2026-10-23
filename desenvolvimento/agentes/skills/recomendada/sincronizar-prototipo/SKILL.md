---
name: sincronizar-prototipo
description: Reflete uma rotina ou tela já aprovada no protótipo consolidado (vitrine) para evitar drift, faz o bump do cache-bust em todos os scripts e revalida a ordem de carregamento do index.html. Use no fechamento de uma tela, quando o usuário falar em sincronizar, consolidar, refletir na vitrine, atualizar o protótipo principal, ou quando a vitrine estiver defasada em relação ao que foi implementado. NÃO use para adicionar tela inédita a um protótipo (integrar-ao-prototipo) nem para portar para a stack real (portar-prototipo).
---

# sincronizar-prototipo

Mantém o protótipo consolidado — a vitrine que alguém abre para ver o produto — igual ao
que foi de fato aprovado e implementado.

**Leia primeiro:** `~/.claude/skills/prototipo-html/references/convencoes.md`.

Vitrine desatualizada é pior que vitrine inexistente: ela mostra com confiança um produto
que não existe mais. Como costuma ser a primeira coisa que o dono do produto abre, o
drift aqui custa credibilidade, não só organização.

## Passo 1 — Identificar origem e destino

- **Origem** — o protótipo da tela aprovada, ou a implementação na stack real quando ela
  já avançou além do protótipo
- **Destino** — o protótipo consolidado do projeto (o que agrega várias telas)

Com protótipo único (seção do contrato), origem e destino são o mesmo protótipo em dois
momentos: a branch da rotina, onde ela foi feita e aprovada, e a base para onde a branch
volta (a `main`, em geral). Sincronizar aqui é garantir que a rotina chega à base inteira,
na aba certa, sem desfazer o que outras rotinas trouxeram para a base nesse meio-tempo.

Não há consolidado? Pergunte se é para criar um, e qual protótipo vira a base. Não eleja
uma vitrine por conta própria.

Em modo rigoroso, obedeça o protocolo do projeto antes de editar.

## Passo 2 — Diferença real

Compare origem e destino antes de tocar em qualquer coisa:

- A tela existe no consolidado? Ausente, defasada, ou já igual?
- Tokens divergiram? (paleta, espaçamento, raio)
- Primitivos do `ui.jsx` divergiram entre os dois?
- Strings mudaram?
- Estados que existem na origem e faltam no destino (incluindo o offline)?
- A rotina em `visoes.jsx` — aba, telas em ordem, meta — é a mesma nos dois lados? As
  abas e superfícies de `VISOES` divergiram?

Reporte a diferença **antes** de sincronizar. Às vezes a resposta certa é atualizar a
origem, não o destino — quando a vitrine já tinha a decisão mais recente.

## Passo 3 — Refletir

**Merge dentro do consolidado, não uma aba nova.** O erro clássico é adicionar a tela
como tab separada e deixar a versão velha no lugar de sempre: agora a vitrine mostra as
duas, e quem olha não sabe qual vale.

A tela nova ocupa o lugar da antiga, na rotina em que ela vive: a entrada em `ROTINAS`
aponta para a tela nova, e a versão velha sai do `SCREEN_MAP` e do `index.html`.

Aplicando:

- Reuse os primitivos do `ui.jsx` do consolidado; se o da origem divergiu, **decida qual
  vence e diga qual** — não deixe os dois
- Strings para o `i18n.jsx` do consolidado, em todos os locales
- Token novo entra nos **dois** temas
- Preserve os 8 estados e o offline
- `visoes.jsx` do consolidado recebe a rotina e a aba dela; abas e rotinas de outras
  rotinas ficam como estão

## Passo 4 — Revalidar o index.html

Duas coisas, sempre:

**a) Ordem de carregamento** — `i18n → data → icons → ui → device → router → visoes →
painel → <telas> → app`. Tela nova entra depois de `painel.jsx` e antes de `app.jsx`. Fora de ordem dá `X is not defined` em
runtime, com tela em branco e sem erro de sintaxe.

**b) Bump do `?v=`** em **todos** os scripts, para o mesmo número. Este é o passo mais
esquecido da sincronização e o que faz o usuário abrir a vitrine e jurar que nada mudou.

## Passo 5 — Verificar

Suba com `/iniciar-prototipo` e, pelo preview MCP:

- Console limpo, sem 404 em `components/*.jsx`
- A tela sincronizada renderiza nos dois temas, em cada superfície da aba dela
- Cada rotina tocada continua dentro da meta de toques (grupo Rotina do painel)
- **As outras telas da vitrine continuam de pé** — o risco real aqui é regressão em tela
  que ninguém pediu para mexer
- Screenshot da tela sincronizada

Se mexeu em `ui.jsx` ou token do consolidado, percorra todas as tabs. Mudança em
primitivo compartilhado propaga para telas que você não abriu.

## Passo 6 — Publicar, se houver deploy

Vitrine com deploy (Vercel, Pages, host estático) precisa do redeploy: sincronizar
localmente e não publicar deixa o problema exatamente onde estava.

Publicar é ação externa e visível. **Confirme com o usuário antes** — verifique o que o
projeto usa (push que dispara deploy automático, comando manual) e siga o fluxo dele,
sem inventar um novo.

Depois, confira o resultado no ar, não só o build verde.

## Passo 7 — Entregar

- O que estava divergente e o que foi sincronizado
- Divergências de primitivo ou token resolvidas, e qual lado venceu
- Novo `?v=`
- Screenshot antes/depois
- Status do deploy, se houve

## Anti-padrões

- Adicionar tela nova como aba e deixar a versão velha viva na vitrine
- Sincronizar sem comparar antes — às vezes o destino é que estava certo
- Resolver divergência de primitivo mantendo os dois
- Esquecer o bump do `?v=` — a sincronização fica invisível
- Não conferir as outras telas depois de mexer em `ui.jsx` compartilhado
- Publicar sem confirmar com o usuário
- Dar deploy como concluído sem abrir a URL publicada
