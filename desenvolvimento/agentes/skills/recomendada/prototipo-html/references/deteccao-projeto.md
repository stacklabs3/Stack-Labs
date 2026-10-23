# Detecção de projeto

As skills `*-prototipo` rodam em qualquer projeto da máquina. Antes de agir, detecte
onde ficam os protótipos, qual o modo de rigor, qual a stack de destino, a identidade
visual e o glossário.

## 1. Raiz de protótipos

Ordem de precedência:

0. O projeto declara raiz e nome (no `AGENTS.md`, em `docs/`, no ticket ou no mapa que
   pediu o protótipo, ex.: "protótipo único em `prototipos/appetito-v2/`") → usa exatamente
   esse caminho, mesmo que a pasta ainda não exista
1. Já existe `prototipos_html/` → usa
2. Existe outra raiz com o padrão (`prototypes/`, `prototipos/`, `mockups/` contendo
   `index.html` + `components/*.jsx`) → usa a que existe
3. Nada existe → cria `prototipos_html/`

**Nunca crie uma segunda raiz.** Duas convenções no mesmo repo é como o padrão morre.

Sem nome declarado, o protótipo se chama `YYYY-MM-DD-<slug>`. Protótipo único (veja
`convencoes.md`) usa o nome declarado pelo projeto, sem data: ele vive enquanto o produto
viver.

Se houver ambiguidade real (duas raízes candidatas), pergunte em vez de escolher.

## 2. Modo de rigor

Detectado pela presença de artefatos de controle de contexto no projeto:

| Sinal | Modo |
|---|---|
| Existe `AGENTS.md` **e** diretório `controle/` | **Rigoroso** |
| Caso contrário | **Leve** |

### Modo rigoroso

O projeto tem protocolo próprio de controle de contexto. Respeite-o:

- Leia o `AGENTS.md` antes de editar — ele define o ciclo de vida da task
- Só edite fora de `controle/` com task ativa e plano aprovado
- Registre evidência no ledger da task, se o protocolo pedir
- Aprovação é exact-match do termo que o projeto define (ex.: `aprovado`,
  `/aprovar-plano`) — um "pode seguir" não conta

Nesse modo o projeto pode ter comandos locais em `.claude/commands/` que sobrescrevem
estas skills globais. Comando de projeto vence skill global: se existir um
`.claude/commands/<mesmo-nome>.md`, ele é a versão canônica ali e é ele que roda.

### Modo leve

Sem protocolo de controle. O fluxo é:

1. Cria/edita o protótipo
2. Serve e verifica (console limpo, screenshot)
3. Pede aprovação explícita antes de portar para a stack real

Sem `controle/`, sem ledger, sem cerimônia. A aprovação do protótipo continua sendo
obrigatória — é ela que separa protótipo de código de produção.

Em qualquer modo, um `AGENTS.md` presente vale por inteiro: branch, issue ativa, quem
aprova e como se registra a aprovação são do projeto.

## 3. Stack de destino

Detectada por arquivo-marcador na raiz. Importa para `/portar-prototipo` e para as
decisões de layout do protótipo (mobile vs web).

| Marcador | Stack | Onde vivem os tokens |
|---|---|---|
| `pubspec.yaml` | Flutter | `ThemeData` / arquivo de tema em `lib/` |
| `next.config.*` | Next.js | `tailwind.config.*` ou CSS vars em `globals.css` |
| `app.json` + `package.json` com `expo` | React Native / Expo | tema em JS/TS |
| `vite.config.*` + `svelte.config.*` | Svelte | CSS vars globais |
| `package.json` com `react`, sem framework | React puro | CSS vars ou styled-components |
| `Package.swift` / `*.xcodeproj` | SwiftUI | `Color` assets + extensões |

Projeto pode ter mais de uma stack (mobile + web no mesmo repo). Nesse caso, detecte
todas e pergunte qual é o alvo do protótipo — a resposta muda o frame do showcase
(device frame com safe area vs viewport de browser).

Sem marcador reconhecível: pergunte a stack em vez de assumir.

## 4. Identidade visual

Antes de definir tokens em protótipo novo, procure a paleta que já existe:

- `tailwind.config.*` → `theme.extend.colors`
- CSS global com `:root { --* }`
- Arquivo de tema da stack (Flutter `ThemeData`, tokens em TS)
- `docs/` com design tokens ou guia de marca
- Logo ou assets em `assets/`, `public/`, `static/`

Achou? Reaproveite os valores reais — protótipo com cor inventada valida a tela errada.
Não achou? Use a paleta neutra do template e sinalize ao usuário que os tokens de marca
são placeholder, para ele confirmar ou substituir.

## 5. Glossário e lista de telas

O glossário dá as palavras da tela. Procure, nesta ordem: `CONTEXT.md`, `GLOSSARY.md`,
`UBIQUITOUS_LANGUAGE.md`, um glossário em `docs/`. Leia o termo de cada coisa que a tela
mostra e a lista _Avoid_ de cada termo — essas palavras ficam fora da tela.

Se o projeto tem uma lista de telas a prototipar (rotinas, ordem, "nasce em", itens que
"seguem o padrão"), ela diz quais rotinas existem, em que superfícies cada uma se confere
e quais itens não ganham protótipo. Ela alimenta o `visoes.jsx`.

Sem glossário: use as palavras de quem usa a tela, em português claro, e diga ao usuário
que não havia glossário.

## 6. Resumo da detecção

Ao iniciar, reporte o que foi detectado em duas ou três linhas, antes de agir:

```
Raiz: prototipos_html/ (existente)
Modo: rigoroso — AGENTS.md + controle/ presentes
Stack: Flutter (pubspec.yaml) + Next.js (next.config.ts)
Tokens: encontrados em tailwind.config.ts
Glossário: CONTEXT.md · Lista de telas: docs/telas-a-prototipar.md
```

Se algo veio de suposição e não de evidência, diga qual e por quê. O usuário corrige
em uma linha; descobrir a suposição errada depois de três telas prontas custa muito mais.
