---
name: novo-prototipo
description: Cria do zero o protótipo HTML+JSX de um projeto (React 18 + Babel standalone, zero build step), com uma aba por superfície e as primeiras rotinas. Use quando o projeto ainda não tem protótipo e o usuário pedir protótipo, mockup, wireframe interativo, showcase visual ou "quero ver como fica antes de implementar". NÃO use se o projeto já tem protótipo e a intenção é somar rotina ou tela (integrar-ao-prototipo) ou refinar tela existente (melhorar-prototipo).
---

# novo-prototipo

Cria o protótipo HTML+JSX de um projeto, do zero, em qualquer projeto.
Zero build step: React 18 + Babel standalone via CDN, abre no browser e funciona.

**Leia primeiro:** `~/.claude/skills/prototipo-html/references/convencoes.md` — é o contrato.
Não improvise estrutura, ordem de carregamento, tokens, abas ou rotinas.

**Onde isto se encaixa.** Se o protótipo nasceu de um ticket de decisão do `/wayfinder`, a
resposta do ticket é a decisão visual — e a aprovação que o `/portar-prototipo` vai exigir
depois. Se nasceu solto, o caminho é o mesmo, só sem o mapa por cima. O fluxo completo, da
ideia solta até o código, está em [FLUXOS.md](https://github.com/matheus3006/my-claude-skills/blob/main/FLUXOS.md).

## Antes de qualquer coisa: existe protótipo?

Liste a raiz de protótipos do projeto. O projeto tem **um protótipo só** (seção
"Protótipo único e abas" do contrato), então, se já existir um:

- Quer somar rotina ou tela → **pare e use `/integrar-ao-prototipo`**
- Quer refinar tela que já está lá → **pare e use `/melhorar-prototipo`**

Se o pedido for um item que **segue o padrão** (seção do contrato), não há protótipo a
criar: diga ao usuário qual padrão o item segue e que ele é conferido por print na
implementação.

Só siga aqui se o projeto ainda não tem protótipo.

## Passo 1 — Detectar o projeto

Siga `~/.claude/skills/prototipo-html/references/deteccao-projeto.md` e reporte o resumo:
raiz e nome do protótipo, modo de rigor, stack de destino, origem dos tokens de marca,
glossário e lista de telas.

Em **modo rigoroso** (`AGENTS.md` + `controle/`), obedeça o protocolo do projeto antes
de editar: task ativa, plano aprovado, o que o `AGENTS.md` mandar. Em **modo leve**,
siga direto.

## Passo 2 — Fechar o escopo

O escopo do protótipo único é a organização inteira e as rotinas do primeiro pedido:

- **Abas** — uma por superfície ou papel, com as superfícies em que cada uma se confere e
  o tema com que abre
- **Rotinas deste pedido** — cada uma com as telas em ordem e a meta de toques
- **Identidade visual** — reaproveitar tokens encontrados no projeto, ou definir novos
- **Idiomas** — monolíngue (sem switcher) ou multi-locale

Quando o projeto tem lista de telas, ela responde abas, superfícies e rotinas: diga o que
tirou dela e siga. Use `AskUserQuestion` só para o que nem a detecção nem a lista
responderam — não transforme a skill em interrogatório.

## Passo 3 — Scaffold

```bash
cp -r ~/.claude/skills/prototipo-html/template <raiz>/<nome-do-prototipo>
```

O nome é o que o projeto declarou; sem declaração, `YYYY-MM-DD-<slug>`.

Depois de copiar, adapte na ordem:

1. **Tokens** no `index.html` — `--primary`, `--accent` e derivados, nos dois temas.
   Use os valores reais do projeto quando a detecção os encontrou. Quando não, use os
   neutros do template e **diga ao usuário que são placeholder**.
2. **`i18n.jsx`** — as strings reais das telas, com as palavras do glossário (seção
   "Glossário do projeto" do contrato). Monolíngue: mantenha só `pt-BR` e remova o
   switcher de idioma do `app.jsx`.
3. **`data.jsx`** — mocks das telas. **Nunca PII real**, nem em comentário.
4. **`visoes.jsx`** — todas as abas do projeto (`VISOES`), a meta de toques
   (`META_DE_TOQUES`) e as rotinas deste pedido (`ROTINAS`). As abas das rotinas que ainda
   não foram prototipadas já entram, para a organização nascer inteira.
5. **`components/<dominio>.jsx`** — as telas, substituindo `telas.jsx`. Uma por domínio,
   não uma por arquivo quando forem da mesma família. Cada tela recebe `estado`, responde
   aos 8 estados e ao offline, e navega para a próxima tela da rotina.
6. **`router.jsx`** — as rotas reais em `ROUTES`.
7. **`index.html`** — registre cada arquivo de tela **depois** de `painel.jsx` e
   **antes** de `app.jsx`, com `?v=1`.
8. **`app.jsx`** — atualize `SCREEN_MAP` (rota → componente). A grade do Showcase sai de
   `ROTINAS`. Mantenha o header enxuto e o `PrototypeErrorBoundary`.

## Passo 4 — Cobrir os estados

default, hover, focus, active, disabled, loading, empty, error e offline — cada um
visível pelo seletor de Estado do painel flutuante, sem editar código.

Os que somem quando se tem pressa, e que são justamente os que a implementação vai
precisar: **empty** (o que vai aparecer + o primeiro passo), **error** (o que houve +
recuperação), **loading** (skeleton na mesma caixa do conteúdo, sem layout shift) e
**offline** (`AvisoSemInternet` com o que continua funcionando e o que espera).

**Uma moldura por vez:** a aba mostra só a superfície ativa, no palco e na grade. Ver
"Showcase" em `convencoes.md`.

## Passo 5 — Servir e verificar você mesmo

Suba com `/iniciar-prototipo` (ou `python3 -m http.server 8765` na pasta).

**Não entregue sem ter verificado.** Pelo preview MCP:

- `read_console_messages` — zero erro, zero warning do React
- `read_network_requests` — nenhum 404 em `components/*.jsx`
- cada aba abre na superfície e no tema dela
- cada rotina, pelo grupo Rotina do painel, dentro da meta de toques
- percorrer os estados

Depois tire os prints da conferência (seção "Conferência por print" do contrato) e **abra
cada um**.

Console sujo ou tela em branco quase sempre é ordem de carregamento errada no
`index.html` — confira a cadeia `i18n → data → icons → ui → device → router → visoes →
painel → telas → app`.

## Passo 6 — Entregar

Devolva ao usuário:

- URL do protótipo
- Abas criadas, com superfícies e tema de cada uma
- Rotinas, com `toques/meta` de cada uma
- Estados cobertos
- O que ficou como placeholder (tokens de marca, copy provisória, mock raso)
- Onde estão os prints da conferência

E o próximo passo: `/melhorar-prototipo` para refinar, `/integrar-ao-prototipo` para
somar rotina, `/auditar-prototipo` antes de aprovar, `/portar-prototipo` depois de aprovado.

## Anti-padrões

- Criar protótipo quando o projeto já tem um → integre
- Inventar cor de marca sem procurar a paleta real do projeto
- Inventar palavra de tela que o glossário já nomeia
- Pular os estados empty/error/loading/offline "porque é só um mockup" — são eles que
  revelam o que falta no fluxo
- Tela solta, fora de qualquer rotina
- Entregar sem abrir no browser
- Colocar lógica de negócio real; protótipo simula
- PII real nos mocks
