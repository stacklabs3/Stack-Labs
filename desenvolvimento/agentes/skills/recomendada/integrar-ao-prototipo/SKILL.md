---
name: integrar-ao-prototipo
description: Adiciona uma rotina, tela ou funcionalidade nova a um protótipo HTML+JSX que já existe, na aba certa, reusando os primitivos e tokens dele. Use quando o usuário pedir para incluir, somar, acrescentar ou integrar mais uma rotina, tela ou feature ao protótipo, ou para consolidar uma tela aprovada num protótipo maior (vitrine). NÃO use para criar protótipo do zero (novo-prototipo) nem para refinar tela que já está no protótipo (melhorar-prototipo).
---

# integrar-ao-prototipo

Adiciona uma rotina nova a um protótipo existente **sem duplicar o que já está lá**.

**Leia primeiro:** `~/.claude/skills/prototipo-html/references/convencoes.md`.

Esta é a operação mais frequente e a que mais gera drift. Dois modos de falha dominam:
criar um primitivo que já existia no `ui.jsx`, e quebrar a ordem de carregamento do
`index.html`. Os passos 2 e 5 existem para impedir cada um deles.

## Passo 1 — Achar o protótipo, a aba e a rotina

Liste a raiz de protótipos e escolha o alvo (argumento, o protótipo único do projeto, ou a
pasta mais recente, dizendo qual escolheu). Leia o `visoes.jsx` e liste `components/` para
ver as abas, as rotinas e as telas que já existem.

Se não existir protótipo, pare e oriente `/novo-prototipo`. Se o item **segue o padrão**
(seção do contrato), pare: ele não ganha protótipo e é conferido por print na
implementação.

Feche a **rotina** do pedido antes de escrever: a aba onde ela vive, as telas em ordem, a
meta de toques e a pergunta de aparência que ela responde. Quando o projeto tem lista de
telas, ela dá tudo isso; quando o pedido é uma tela solta, descubra de que rotina ela é
passo.

Em modo rigoroso (`AGENTS.md` + `controle/`), obedeça o protocolo do projeto antes de
editar — veja `~/.claude/skills/prototipo-html/references/deteccao-projeto.md`.

## Passo 2 — Ler antes de escrever (obrigatório)

**Leia `ui.jsx` e `i18n.jsx` por inteiro antes de criar qualquer componente.**

Não é formalidade. Em protótipo maduro o `ui.jsx` passa de mil linhas; escrever um
segundo `Button` porque não se olhou o primeiro cria dois sistemas visuais divergindo em
silêncio, e o handoff para a stack real vira adivinhação. A moldura e as peças comuns que
outras rotinas já criaram estão lá: a rotina nova as usa.

Leia também os tokens no `<style>` do `index.html`, uma tela existente da mesma aba — para
copiar a convenção de composição, não só a de nomes — e o glossário do projeto, para as
palavras da tela.

Decida, para cada peça da rotina nova:

1. Primitivo já existe → **use**
2. Existe mas não cobre o caso → **estenda via prop**, não copie
3. Não existe e é reutilizável → **adicione em `ui.jsx`**
4. Não existe e é só desta tela → local no `<dominio>.jsx`

Antes de escrever, diga ao usuário o que vai reusar e o que vai criar. Se a lista de
"criar" estiver grande, provavelmente você não leu o `ui.jsx` direito.

## Passo 3 — Escrever as telas da rotina

Crie ou estenda `components/<dominio>.jsx`. Regras que não mudam:

- Tokens via `var(--*)` — zero hex, zero px mágico
- Strings via `t('chave')`, com as palavras do glossário — adicione as chaves em
  `i18n.jsx`, em **todos** os locales
- Mocks em `data.jsx`, nunca PII real
- Os 8 estados e o offline cobertos (default, hover, focus, active, disabled, loading,
  empty, error, offline); no offline, o que continua funcionando e o que espera são os
  desta tela
- Cada tela navega para a próxima da rotina por `useRouter().navigate(...)`

Precisa de token que não existe? Adicione ao `:root` **e** ao `[data-theme="dark"]`.
Token que só existe em um tema quebra o outro.

## Passo 4 — Registrar a rotina

Três edições:

**a) `router.jsx`** — acrescente as rotas em `ROUTES` (com `:param` se a tela receber id).

**b) `app.jsx`** — registre cada tela no `SCREEN_MAP` (`ROTA: Componente`).

**c) `visoes.jsx`** — acrescente a rotina em `ROTINAS`, na aba dela, com as telas em
ordem (e `metaDeToques` só se for diferente da do projeto). A grade do Showcase e os
prints saem daqui. Rotina de uma aba que ainda não existe ganha a aba em `VISOES`, com as
superfícies e o tema de abertura dela.

Dúvida real de aparência numa tela vira A/B: a rotina declara `duvidas` e a tela desenha
as duas opções com `useOpcao(id)` (convenções, "Dúvidas A/B"). Escolhida a opção, apague a
perdedora, a dúvida e o `useOpcao` antes do merge.

## Passo 5 — Registrar no index.html (onde tudo quebra)

Duas edições, ambas obrigatórias:

**a) Inserir o script na posição correta da cadeia.**

```
i18n → data → icons → ui → device → router → visoes → painel → <telas> → app
```

Tela nova entra **depois** de `painel.jsx` e **antes** de `app.jsx`. Os `.jsx` são
scripts globais, sem módulos: fora de ordem dá `X is not defined` em runtime, sem erro
de sintaxe — o protótipo abre em branco e o motivo só aparece no console.

**b) Bump do `?v=` em todos os scripts, para o mesmo número.**

Sem isso o browser serve o `.jsx` cacheado e você revisa a versão antiga.

## Passo 6 — Verificar antes de entregar

Suba com `/iniciar-prototipo` e, pelo preview MCP:

- Console limpo — sem erro e sem warning
- Nenhum 404 em `components/*.jsx`
- A rotina, pelo grupo Rotina do painel, chega à última tela dentro da meta de toques
  (`window.contagemDeToques`). Acima da meta, mude a tela antes de entregar
- Cada tela renderiza em cada superfície da aba, nos dois temas, e no offline
- As outras rotinas continuam funcionando (regressão é o risco aqui)

Depois de integrar em protótipo grande, dê uma passada em `/auditar-prototipo` —
integração é onde hex mágico, string hard-coded e termo fora do glossário entram sem
ninguém ver.

## Passo 7 — Conferir por print

Tire os prints da rotina (seção "Conferência por print" do contrato):

```bash
~/.claude/skills/prototipo-html/scripts/tirar-prints.sh http://localhost:8765 <id-da-rotina> <protótipo>/prints
```

**Abra cada print** e confira contra a checklist do projeto (os princípios de UI, quando
o projeto tem). O que falhar se corrige e se tira de novo, antes de mostrar ao humano.

## Passo 8 — Entregar

- A rotina: aba, telas em ordem e `toques/meta`
- O que foi reusado do `ui.jsx` e o que foi criado (e por quê)
- Chaves novas no `i18n.jsx`
- Tokens novos, se houve
- Posição do script na cadeia e o novo `?v=`
- A pasta dos prints, com os principais mostrados ao humano

Os prints aprovados pelo humano são commitados com a rotina e vão num comentário da PR.

## Anti-padrões

- Escrever componente antes de ler o `ui.jsx` → é o modo de falha nº 1
- Copiar um primitivo e mudar duas linhas em vez de estender via prop
- Tela solta, sem rotina em `visoes.jsx` → não dá para contar os toques
- Registrar o script no fim, depois do `app.jsx` → quebra em runtime
- Esquecer o bump do `?v=` → revisa a versão antiga sem perceber
- Adicionar token só no `:root` e não no dark
- Adicionar chave i18n em um locale só
- Mostrar prints ao humano sem ter aberto cada um
- Entregar sem conferir que as rotinas antigas continuam de pé
