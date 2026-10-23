---
name: melhorar-prototipo
description: Refina uma tela que já está no protótipo HTML+JSX — hierarquia visual, espaçamento, tipografia, microcopy, paleta, estados faltando, acessibilidade. Use quando o usuário disser que uma tela do protótipo está feia, confusa, poluída, "faltando algo", ou pedir para melhorar, polir, refinar ou repaginar. NÃO use para adicionar tela nova (integrar-ao-prototipo), para acertar um valor numérico específico de posição ou espaçamento (ajustar-detalhe-prototipo), nem para só verificar conformidade (auditar-prototipo).
---

# melhorar-prototipo

Refina o que já existe no protótipo. Diagnóstico antes de proposta, proposta antes de
edição.

**Leia primeiro:** `~/.claude/skills/prototipo-html/references/convencoes.md`.

Se o pedido for por um **valor exato** ("sobe um pouco", "mais espaço aqui", "logo
maior") e você já errou o palpite uma ou duas vezes, pare: isso é
`/ajustar-detalhe-prototipo`, não iteração de design.

## Passo 1 — Achar o alvo e ler

Localize o protótipo (argumento ou o mais recente, dizendo qual). Liste `components/`.

Leia o que vai tocar **antes** de opinar: a tela em questão, o `ui.jsx` e os tokens no
`index.html`. Crítica de design sem ler o código produz proposta que já está
implementada ou que quebra outra tela.

Em modo rigoroso (`AGENTS.md` + `controle/`), obedeça o protocolo do projeto antes de
editar.

## Passo 2 — Escopo

Use `AskUserQuestion` para travar o alvo, salvo quando o usuário já foi específico:

- **O quê** — uma tela, um componente do `ui.jsx`, ou os tokens/identidade visual
- **Qual** — a tela ou o primitivo, listando os que existem no protótipo
- **O problema** — pergunta aberta: o que incomoda hoje, o que quer no lugar,
  referências visuais se houver

A terceira é a que mais importa. "Melhore a tela" sem sintoma leva a mudança cosmética
aleatória; o sintoma concreto ("o preço some no meio dos adicionais") aponta o
diagnóstico.

## Passo 3 — Diagnóstico

Consulte as skills de design relevantes ao escopo — não todas, só as que se aplicam:

- `ui-ux-pro-max` — padrões de UX e sistemas de design
- `design:design-critique` — crítica estruturada
- `design-audit` — auditoria visual sistemática
- `emil-design-eng` — polimento e microinterações
- `ui-typography` — **sempre** que a mudança envolver texto visível

Cruze o que elas dizem com o sintoma do usuário e com o código que você leu. Opiniões
conflitantes vão para a síntese como trade-off explícito — não escolha em silêncio.

## Passo 4 — Propor antes de aplicar

Apresente, nesta ordem:

1. **Estado atual** — o que está lá hoje, citando arquivo e linha
2. **Problemas** — cada um ligado ao sintoma que o usuário relatou ou ao que você
   encontrou; sem "poderia ser mais moderno"
3. **Propostas** — 3 a 5, priorizadas por impacto ÷ esforço, cada uma dizendo o que
   muda concretamente
4. **Trade-offs** — o que se perde em cada uma
5. **Impacto sistêmico** — se toca `ui.jsx` ou tokens, quais outras telas mudam junto

Mudança em token ou primitivo compartilhado propaga. Diga em quais telas antes de
aplicar, não depois.

Confirme o escopo final com `AskUserQuestion` (aprovar tudo, selecionar algumas,
ajustar antes).

## Passo 5 — Aplicar

Só o que foi aprovado. Sempre:

- Tokens via `var(--*)`; token novo entra nos **dois** temas
- Strings via `i18n.jsx`, em todos os locales
- Estados preservados — refino não pode apagar empty, error, loading ou offline
- Rotina dentro da meta de toques — refino não pode somar toque à rotina da tela
- **Bump do `?v=`** em todos os scripts

## Passo 6 — Verificar

Suba com `/iniciar-prototipo`. Pelo preview MCP: console limpo, screenshot **antes e
depois** nos dois temas, telas vizinhas intactas se mexeu em `ui.jsx` ou token.

Entregue o antes/depois lado a lado — é o que permite ao usuário julgar em um olhar.

## Anti-padrões

- Opinar sobre a tela sem ter lido o arquivo
- Aplicar sem propor, quando o usuário só descreveu o incômodo
- Mudar `ui.jsx` ou token sem dizer que outras telas vão junto
- Refinar o visual e derrubar os estados empty/error/loading/offline no caminho
- Rodar todas as skills de design "por garantia" — ruído, não rigor
- Ficar tentando adivinhar um valor numérico → é `/ajustar-detalhe-prototipo`
- Esquecer o bump do `?v=` e revisar a versão antiga
