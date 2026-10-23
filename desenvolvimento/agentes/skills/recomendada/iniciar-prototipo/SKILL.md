---
name: iniciar-prototipo
description: Sobe o servidor HTTP local de um protótipo HTML+JSX e abre no browser. Use quando o usuário pedir para rodar, servir, abrir, subir ou ver o protótipo, ou quando precisar visualizar o protótipo depois de editá-lo. NÃO use para criar protótipo (novo-prototipo) nem para editar conteúdo — esta skill só serve e verifica.
---

# iniciar-prototipo

Sobe o protótipo HTML+JSX em servidor local, faz smoke test e abre no browser.

Protótipo **precisa** ser servido por HTTP: abrir o `index.html` via `file://` falha,
porque o Babel busca os `.jsx` por XHR e o navegador bloqueia por CORS. A tela abre em
branco e o erro só aparece no console.

Argumento opcional: um task-id específico (ex.: `2026-05-20-cliente-mvp`).

## Passo 1 — Achar o protótipo

Liste a raiz de protótipos do projeto (`prototipos_html/` por padrão), excluindo
`_template`.

- Argumento passado → usa ele
- Uma pasta só → usa ela
- Várias → a de `mtime` mais recente, **dizendo qual escolheu**
- Nenhuma → pare e oriente `/novo-prototipo`

Valide que existem `index.html` e `components/`. Se faltar, pare e reporte — não tente
consertar aqui.

## Passo 2 — Resolver a porta

Padrão: **8765**.

Se estiver ocupada, verifique **o que** está ocupando:

```bash
lsof -ti :8765
```

- Ocupada por um servidor **deste mesmo protótipo** → mate e suba de novo (é restart)
- Ocupada por **outra coisa** (outro projeto, Docker, outro protótipo) → **não mate**;
  suba na próxima porta livre (8766, 8767…) e informe a porta escolhida

Matar cegamente quem está em 8765 derruba o protótipo de outro projeto que o usuário
tem aberto. Dois protótipos em paralelo é caso normal, não conflito.

## Passo 3 — Subir

```bash
cd <raiz>/<task-id>
python3 -m http.server <porta> > /tmp/proto-srv-<task-id>.log 2>&1
```

Use `run_in_background: true` — em foreground o servidor morre no fim da chamada.

## Passo 4 — Smoke test

Antes de devolver a URL:

1. `curl -sI http://localhost:<porta>/` → HTTP 200
2. `curl -s http://localhost:<porta>/components/app.jsx | head -3` → JSX, não 404

Falhou? Mostre `/tmp/proto-srv-<task-id>.log` e pare. Entregar URL quebrada faz o
usuário debugar um problema que era seu.

## Passo 5 — Abrir e verificar

Abra no browser pelo preview MCP (`preview_start` com a URL) e verifique:

- `read_console_messages` — zero erro
- `read_network_requests` — nenhum 404 em `components/*.jsx`
- screenshot

Console limpo é parte da entrega, não extra. Erro comum: ordem de carregamento errada no
`index.html` produzindo `X is not defined` — confira `i18n → data → icons → ui → telas →
app`. Outro: `?v=` não incrementado depois de editar, servindo a versão antiga.

Sem preview MCP disponível, devolva a URL e diga que a verificação visual não foi feita.

## Passo 6 — Entregar

- URL completa
- Task-id servido e por que foi escolhido (se houve escolha)
- Porta, se não for a 8765
- Resultado do smoke test e do console
- Próximos passos: `/melhorar-prototipo`, `/integrar-ao-prototipo`, `/auditar-prototipo`

## Anti-padrões

- Rodar sem background → servidor morre e a URL não responde
- Matar quem estiver em 8765 sem checar o que é
- Devolver URL sem smoke test
- Dizer "está rodando" sem ter aberto e lido o console
