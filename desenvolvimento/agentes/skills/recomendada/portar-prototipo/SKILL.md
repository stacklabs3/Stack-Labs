---
name: portar-prototipo
description: Traduz um protótipo HTML+JSX aprovado para a stack real do projeto — mapeia componente para arquivo, token do protótipo para o tema da stack, e gera o plano de implementação. Use depois que o protótipo foi aprovado e o usuário pedir para implementar, portar, converter para Flutter/Next.js/React Native, ou "agora faz de verdade". NÃO use antes da aprovação explícita do protótipo, nem para editar o protótipo em si.
---

# portar-prototipo

Traduz o protótipo aprovado para a stack real. O protótipo é a fonte de verdade visual:
onde a implementação divergir, é ela que está errada.

**Leia primeiro:** `~/.claude/skills/prototipo-html/references/convencoes.md`.

## Passo 0 — Portão de aprovação

**O protótipo foi explicitamente aprovado?** Não se infere de silêncio, nem de "legal",
nem do fato de o usuário ter pedido para portar.

Sem aprovação, pare e peça. Em modo rigoroso, a aprovação é exact-match do termo que o
projeto define (veja `~/.claude/skills/prototipo-html/references/deteccao-projeto.md`).

Protótipo com dúvida A/B ainda aberta (`duvidas` em `visoes.jsx` ou `useOpcao` nas telas)
não está aprovado: a escolha não foi feita ou a perdedora não saiu. Pare e peça.

Rode `/auditar-prototipo` antes, se ainda não rodou. Portar protótipo com FAIL aberto
significa reproduzir o defeito em código de produção, onde consertar custa dez vezes mais.

## Passo 1 — Detectar a stack

Siga `deteccao-projeto.md`. Identifique a stack alvo, onde vivem os tokens de tema, e a
convenção de componentes já existente no projeto.

Mais de uma stack no repo (mobile + web): confirme qual é o alvo antes de mapear.

## Passo 2 — Inventário do protótipo

Leia o protótipo inteiro e levante:

- **Rotinas** — as de `visoes.jsx`: aba, telas em ordem, meta de toques e as superfícies
  em que cada aba se confere (elas viram critério de aceite)
- **Telas** — cada `components/<dominio>.jsx` e o que ele renderiza
- **Primitivos** — o que está em `ui.jsx` e quem usa cada um
- **Tokens** — a paleta completa dos dois temas, escalas de espaço, raio, sombra
- **Estados** — quais dos 8 e o offline cada tela cobre
- **Strings** — as chaves de `i18n.jsx`
- **Mocks** — o formato dos dados que as telas consomem (vira contrato de API/modelo)

## Passo 3 — Mapear

Duas tabelas. Elas são a entrega principal deste passo.

**Tokens → tema da stack:**

| Token do protótipo | Valor light | Valor dark | Destino na stack |
|---|---|---|---|
| `--primary` | `#0F172A` | `#F8FAFC` | `colorScheme.primary` |
| `--space-4` | `16px` | — | `Spacing.md` |

**Componente → arquivo:**

| Protótipo | Destino | Observação |
|---|---|---|
| `ui.jsx > Button` | `lib/widgets/app_button.dart` | já existe, estender variante |
| `checkout.jsx` | `lib/features/checkout/checkout_screen.dart` | novo |

Antes de marcar algo como novo, **procure o equivalente que já existe no projeto**.
Criar um segundo botão em produção é pior do que criar um segundo botão no protótipo.

Tokens vão para o arquivo central de tema — nunca hex espalhado pelos componentes.
A regra do protótipo continua valendo do outro lado.

## Passo 4 — Plano de implementação

Entregue antes de escrever código:

1. **Ordem de implementação** — tokens primeiro, depois primitivos, depois telas.
   Inverter significa refazer tela quando o token muda.
2. **Arquivos** — o que é novo, o que é alterado
3. **Reuso** — o que o projeto já tem e vai ser aproveitado
4. **Gaps** — o que o protótipo mostra e a stack ainda não suporta (dependência,
   permissão, endpoint), com o custo de cada um
5. **Critérios de aceite** — os 8 estados e o offline, os dois temas, cada superfície da
   aba, a rotina dentro da meta de toques, acessibilidade equivalente
6. **Fora de escopo** — o que fica para depois, explicitamente

Em modo rigoroso, o plano vai para onde o protocolo do projeto manda e passa pela
aprovação canônica antes da execução.

## Passo 5 — Implementar

Reproduza o protótipo. As invariantes atravessam a fronteira:

- Tokens no arquivo central de tema; zero hex ou px mágico nos componentes
- Strings no sistema de i18n da stack; nada hard-coded
- Os 8 estados e o offline, incluindo empty, error e loading
- As palavras da tela do glossário do projeto, como no protótipo
- Acessibilidade: contraste AA nos dois temas, foco visível, ordem de tabulação, alvo de
  toque ≥ 44px
- Sem PII em log — vale especialmente para telefone, e-mail e endereço que apareciam
  como mock no protótipo

Siga o idioma do código existente: nomenclatura, formatação e estrutura de pasta do
projeto, não as do protótipo.

## Passo 6 — Verificar contra o protótipo

Verificação é comparação, não impressão:

1. Rode o que o projeto usa como gate (análise estática, testes, lint)
2. Suba o protótipo com `/iniciar-prototipo`
3. Rode o app real na mesma tela
4. **Conferência por print** do app real, com as mesmas combinações do protótipo (seção
   do contrato): estado normal em cada superfície da aba, nos dois temas; vazio, erro,
   carregando e sem internet numa combinação. Ponha cada print **lado a lado** com o
   print aprovado em `prints/<rotina>/` — e **abra as imagens**
5. Percorra a rotina no app real contando os toques; ela continua dentro da meta
6. Os prints da implementação vão num comentário da PR, como os do protótipo

**Itens que seguem o padrão** não têm protótipo próprio: confira-os do mesmo jeito, por
print, contra a rotina aprovada que criou o padrão (o formulário, a tabela).

Divergência achada: ou corrige a implementação, ou registra a diferença deliberada com
o motivo. Divergência silenciosa é como o design drifta.

## Passo 7 — Entregar

- Tabela de mapeamento final (o que virou o quê)
- Prints lado a lado (protótipo × app real), com o link do comentário na PR
- Divergências deliberadas e por quê
- O que ficou fora de escopo
- Resultado dos gates do projeto

Se o projeto mantém protótipo consolidado (vitrine), use `/sincronizar-prototipo` para
refletir a tela aprovada lá.

**Se a implementação não cabe em uma sessão.** Pare depois do plano do Passo 4 e leve as duas
tabelas de mapeamento para o `/to-spec`: elas viram a seção de decisões de implementação da spec,
que o `/to-tickets` fatia em issues, cada uma construída por um `/implement` em sessão limpa.
O protótipo continua sendo a referência visual de cada uma dessas sessões — Passo 6 (prints
lado a lado) vale por issue, não só no fim. Fluxo completo: [FLUXOS.md](https://github.com/matheus3006/my-claude-skills/blob/main/FLUXOS.md).

## Anti-padrões

- Portar sem aprovação explícita
- Portar com FAIL de auditoria em aberto
- Espalhar hex pelos componentes em vez de alimentar o tema central
- Criar componente novo na stack sem procurar o que já existe
- Implementar tela antes de token e primitivo
- Declarar paridade sem print lado a lado em cada superfície
- Julgar screenshot por tamanho ou hash de arquivo em vez de olhar
- Deixar empty/error/loading/offline para "depois" — nunca vem
