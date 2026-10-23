# Convenções do protótipo HTML+JSX

Contrato compartilhado por todas as skills `*-prototipo`. Leia antes de criar,
editar, auditar ou portar qualquer protótipo.

Padrão nascido em um projeto interno. Substitui Figma como fonte de verdade visual
antes da implementação na stack real: React 18 + Babel standalone via CDN, zero build
step, abre no browser e funciona.

## Estrutura canônica

```
<raiz-de-prototipos>/<nome-do-prototipo>/
├── index.html            # tokens CSS + chrome do showcase + orquestrador
├── prints/               # prints aprovados da conferência, um diretório por rotina
└── components/
    ├── i18n.jsx          # TRANSLATIONS centralizadas
    ├── data.jsx          # mocks (nunca PII real) + DEFAULTS
    ├── icons.jsx         # SVG inline
    ├── ui.jsx            # primitivos: Button, Card, Field, Skeleton, EmptyState, ErrorState, AvisoSemInternet
    ├── device.jsx        # superfícies: computador, tablet (em pé e deitado), iOS, Android, TV, totem
    ├── router.jsx        # rotas hash + navegação real entre telas
    ├── visoes.jsx        # abas, superfícies e tema de cada aba, rotinas e meta de toques
    ├── painel.jsx        # painel de controle flutuante do showcase
    ├── <dominio>.jsx     # telas, uma ou mais por domínio
    └── app.jsx           # Showcase shell + mapa rota→tela + ErrorBoundary + lista de prints
```

A raiz e o nome seguem [deteccao-projeto.md](deteccao-projeto.md): o que o projeto
declarar vence o padrão (`prototipos_html/` e `YYYY-MM-DD-<slug>`). Nunca crie uma segunda
raiz.

## Protótipo único e abas

Um projeto com várias superfícies (cardápio do cliente, caixa, cozinha, painel do dono…)
tem **um protótipo só**, com **uma aba por superfície ou papel**. O `/novo-prototipo` roda
uma vez e cria esse protótipo com todas as abas; daí em diante cada rotina entra pelo
`/integrar-ao-prototipo`, reusando as peças e os tokens que já estão lá. Dois protótipos
do mesmo produto são duas paletas e dois `ui.jsx` divergindo, e ninguém sabe qual vale.

Projeto pequeno, de uma superfície só, segue a mesma estrutura com uma aba: o header
esconde a faixa de abas quando há uma só.

As abas vivem em `visoes.jsx`, o único arquivo com a organização do projeto. As skills
continuam genéricas; o que é do projeto entra ali:

```js
const META_DE_TOQUES = 3;   // padrão do projeto; cada rotina pode declarar a sua

const VISOES = [
  { id: 'caixa',   rotulo: 'Caixa',   superficies: ['desktop', 'tablet', 'ios', 'android'], temaPadrao: 'light' },
  { id: 'cozinha', rotulo: 'Cozinha', superficies: ['tabletDeitado', 'tv'], temaPadrao: 'dark', letraMinima: 32 },
];

const ROTINAS = [
  { id: 'turno-do-caixa', visao: 'caixa', rotulo: 'Turno do caixa',
    telas: [{ rota: 'CAIXA_ABRIR' }, { rota: 'CAIXA_SANGRIA' }, { rota: 'CAIXA_FECHAR' }] },
];
```

- **`superficies`** — as superfícies em que a aba se confere, na ordem do painel; a
  primeira é a de abertura. Chaves de `SURFACES` em `device.jsx`: `desktop` (computador),
  `tablet`, `tabletDeitado` (1112×834), `ios`, `android`, `tv` (1920×1080, deitada) e
  `totem` (1080×1920, em pé).
- **`temaPadrao`** — o tema com que a aba abre. Tela ligada o dia inteiro (cozinha) abre
  no escuro; o tema continua trocável pelo painel.
- **`letraMinima`** — opcional, para tela lida de longe: o menor tamanho de letra, em px
  na escala 1:1, que o texto da aba pode ter. O valor de partida se acerta no protótipo,
  olhando a tela de onde a pessoa vai ler.

## Rotinas e toques

A unidade do protótipo é a **rotina**: o trabalho inteiro de alguém, com as telas na ordem
em que acontecem ("Turno do caixa: abrir → sangria e suprimento → fechar e conferir").
Tela solta não existe: toda tela é passo de uma rotina declarada em `ROTINAS`, e é
alcançada pela navegação da tela anterior.

A pergunta de aparência e a regra de toques só se conferem na rotina inteira. Por isso o
painel tem o grupo **Rotina**: escolher uma rotina abre a primeira tela e conta cada toque
(botão, link, campo, item tocável) até a última tela. O painel mostra
`Turno do caixa: 3 de 3 toques · concluída`, marca "acima da meta" em texto quando passa, e
`window.contagemDeToques` expõe o mesmo resultado para quem audita.

A meta é `metaDeToques` da rotina, ou `META_DE_TOQUES` do projeto. Rotina acima da meta é
achado do protótipo: a tela muda, não a meta.

## Dúvidas A/B

Quando uma tela da rotina tem **dúvida real de aparência** entre duas formas, o protótipo
traz as duas e o humano escolhe na aprovação. Sem dúvida real, uma opção só: A/B não é
jeito de adiar decisão que os documentos já tomaram, e dúvida de regra do negócio não se
resolve com A/B (é pergunta para o humano antes de construir).

A rotina declara a dúvida em `visoes.jsx`, e a tela lê a escolha com `useOpcao(id)`:

```jsx
// visoes.jsx
{ id: 'estados-da-vitrine', visao: 'cardapio', rotulo: 'Estados da vitrine', telas: [...],
  duvidas: [{ id: 'erro-da-vitrine', pergunta: 'Como aparece o erro?',
              opcoes: { A: 'Aviso no topo', B: 'Tela inteira' } }] }

// telas
const opcaoDoErro = useOpcao('erro-da-vitrine');   // 'A' | 'B'
return opcaoDoErro === 'A' ? <AvisoNoTopo /> : <ErroTelaInteira />;
```

- O painel ganha o grupo **Opção** com a pergunta e os botões `A · Aviso no topo` e
  `B · Tela inteira`. Ele só aparece quando a aba (ou a rotina em contagem) tem dúvida, e
  o resumo do painel mostra a opção ativa.
- `&opcao=erro-da-vitrine:B` abre direto na opção (várias dúvidas separadas por vírgula).
- A contagem de toques vale para a opção ativa; trocar a opção recomeça a rotina.
  `window.contagemDeToques.opcoes` diz em que opção se contou.
- Os prints saem de cada opção, em pastas `prints/<rotina>/opcao-<duvida>-<letra>/`.
- **Escolhida a opção**, a perdedora sai do código, a dúvida sai de `visoes.jsx` e
  `useOpcao` sai da tela, antes do merge. `useOpcao` de dúvida que não existe mais avisa no
  console.

## Itens que seguem o padrão

Quando a lista de telas do projeto marca um item como "segue o padrão" (de formulário, de
tabela, de outra rotina já aprovada), ele **não ganha protótipo**: o padrão já foi decidido e
aprovado na rotina que o criou. Esse item é conferido por print **na implementação**,
contra o padrão que ele segue, e o `/portar-prototipo` cobra isso.

## Glossário do projeto

Toda palavra que aparece na tela vem do glossário do projeto (`CONTEXT.md`, `GLOSSARY.md`
ou o que a [detecção](deteccao-projeto.md) achar), na língua de quem usa. Termo da lista
_Avoid_ do glossário, sigla solta e inglês ficam fora da tela, inclusive no `i18n.jsx`. Termo
que falta no glossário é pergunta para o humano, não palavra inventada.

## As 5 regras que carregam o peso

### 1. Ordem de carregamento é load-bearing

Os `.jsx` são carregados como scripts globais via `<script type="text/babel" src=...>`.
Não há módulos, não há `import`/`export`: tudo vira global, na ordem em que aparece no
`index.html`. A ordem é obrigatória:

```
i18n → data → icons → ui → device → router → visoes → painel → <telas> → app
```

Cada camada só pode usar o que veio antes dela. `app.jsx` é sempre o último — é ele que
monta o mapa rota → componente, porque é o único ponto onde router e telas já existem.

Inverter a ordem produz `X is not defined` em runtime, sem erro de sintaxe — o protótipo
abre em branco e o motivo só aparece no console.

Ao adicionar um arquivo, insira-o na posição correta da cadeia: tela nova entra
**depois** de `painel.jsx` e **antes** de `app.jsx`.

### 2. Cache-bust obrigatório

Todo `<script>` de componente carrega com `?v=N`:

```html
<script type="text/babel" src="components/ui.jsx?v=3"></script>
```

O browser cacheia `.jsx` agressivamente. Sem o bump você edita, recarrega e vê a versão
antiga — e queima a iteração debugando um arquivo que não está rodando.

Ao alterar qualquer componente, incremente o `?v=` de **todos** os scripts de uma vez.
Um número único para o protótipo inteiro é mais simples e menos sujeito a erro do que
versionar arquivo a arquivo.

### 3. Tokens só via CSS variables

Cores, espaçamentos, raios e sombras vivem no `<style>` do `index.html`: `:root` define
o tema light, `[data-theme="dark"]` sobrescreve. O switch de tema muda o atributo no
`<html>`, nada mais.

Nos componentes, sempre `var(--token)`. Zero hex e zero px mágico espalhado pelo JSX —
é o que permite trocar a identidade visual inteira em um arquivo, e o que faz o
mapeamento para o tema da stack real ser mecânico em vez de arqueológico.

Escalas mínimas: `--space-1..6`, `--radius-sm/md/lg/full`, `--shadow-sm/md/lg`, mais
tokens semânticos (`--bg`, `--fg`, `--surface`, `--border`, `--muted`, `--ok`, `--warn`,
`--danger` e variantes `-soft`).

### 4. Os 8 estados visuais e o sem internet

Todo componente interativo cobre: **default, hover, focus, active, disabled, loading,
empty, error**. Toda tela cobre também **offline** (sem internet). O showcase permite ver
cada um sem editar código.

Os mais esquecidos, e os que custam mais caro depois:
- **empty** — ícone ou ilustração + copy que diz o que vai aparecer + o primeiro passo
- **error** — o que houve em palavras simples + ação de recuperação, sem perder o que foi
  digitado (não só "algo deu errado")
- **loading** — skeleton que ocupa a mesma caixa do conteúdo final, sem layout shift
- **offline** — `AvisoSemInternet` no topo da tela, que continua de pé por baixo: diz o
  que continua funcionando e o que espera a conexão, e a ação que espera fica
  indisponível. Ao sair do offline, o showcase mostra sozinho o aviso de que a conexão
  voltou. Sem internet não é tela de erro.

`focus` precisa de outline visível de verdade — não `outline: none` com um box-shadow
que desaparece no dark.

### 5. Reuso antes de escrita

**Antes de criar qualquer componente, leia `ui.jsx` e `i18n.jsx` do protótipo.**

Em protótipos maduros o `ui.jsx` passa de mil linhas. Criar um segundo `Button` porque
não se olhou o primeiro é o modo de falha número um: o protótipo passa a ter dois
sistemas visuais divergindo em silêncio, e o handoff para a stack real vira adivinhação.

Ordem correta ao integrar algo novo:
1. Ler `ui.jsx` → o primitivo já existe? Use.
2. Existe mas não cobre o caso? Estenda via prop, não copie.
3. Não existe e é reutilizável? Adicione em `ui.jsx`.
4. Não existe e é específico da tela? Aí sim, local no `<dominio>.jsx`.

Mesma lógica para `i18n.jsx`: nenhuma string hard-coded no JSX, nunca. Toda string
visível passa pelo dicionário, mesmo em projeto de um idioma só — a estrutura pronta
evita um refactor completo depois.

## i18n

```js
const TRANSLATIONS = {
  'pt-BR': { chave: 'valor', comParam: 'Olá, {name}' },
};
```

Acesso via hook `useT(lang)`, que resolve interpolação `{param}`. Idioma padrão pt-BR;
adicionar locale é acrescentar uma chave de topo, sem tocar nos componentes.

Se o projeto é monolíngue, mantenha a estrutura e omita o switcher de idioma do showcase.

## Showcase

O `app.jsx` monta o shell de revisão. A divisão é deliberada:

**Header — marca, abas e modo.** As abas (uma por visão de `visoes.jsx`) rolam na
horizontal, e `Protótipo | Showcase` fica à direita. O header não cresce com o número de
telas, então nunca corta nem empurra controle para fora da vista.

**Painel de controle flutuante** (`painel.jsx`) — canto inferior direito, fechado
mostrando o estado ativo (`iOS · Escuro · loading · 2/3`), expandindo para:

- **Superfície** — só as da aba ativa, na ordem de `visoes.jsx`
- **Tema** — Claro · Escuro
- **Estado** — os 8 estados e o offline, aplicados à tela ativa
- **Rotina** — inicia a contagem de toques de uma rotina da aba
- **Opção** — só com dúvida A/B na aba: escolhe a opção de cada dúvida
- **Reset** — volta à abertura da aba e limpa a rota

Fecha com Esc (devolvendo o foco ao botão) e com clique fora.

Controle de revisão não pode disputar espaço com o conteúdo revisado. Quando os
switchers moram no header, o primeiro protótipo com telas demais os empurra para fora —
e some justamente o controle de tema, que é o que menos se pode perder.

**Abas separam visões; telas se navegam pelo router.** O protótipo navega como o app
final navega. Trocar de aba abre a primeira rotina dela, na superfície e no tema de
abertura da aba.

**Uma moldura por vez.** Dentro da aba, a superfície ativa é a plataforma: palco e grade
mostram só aquela moldura. Tela de computador nunca aparece no meio das telas de iPhone —
a revisão de uma plataforma não pode tropeçar na outra (referência:
cadillac-prototipos.vercel.app, uma aba por visão com Showcase próprio).

**Modo Showcase é a grade das rotinas da aba:** uma linha por rotina, com as telas em
ordem e a meta de toques no título, todas na superfície ativa.

Celular, tablet, TV e totem renderizam dentro de device frame (status bar, dynamic island,
home indicator, safe area quando há) e escalam para caber; o computador ocupa o espaço
livre.

**ErrorBoundary** envolve a tela ativa e cada tile da grade — tela quebrada não derruba
o showcase, e o boundary mostra o stack em vez de deixar em branco.

**O endereço guarda a combinação.**
`?visao=<id>&superficie=<chave>&tema=light|dark&estado=<estado>&modo=prototype|showcase&idioma=<locale>&opcao=<duvida>:<letra>`
abre o protótipo exatamente naquela combinação, e `&print=1` esconde header e painel. É o
que torna cada print reproduzível.

## Conferência por print

Ao fim de cada rotina, antes da aprovação, a rotina é conferida por prints:

- **Estado normal em todas as combinações:** cada tela × cada superfície da aba × claro e
  escuro.
- **Vazio, erro, carregando e sem internet numa combinação só:** a de abertura da aba.

O script tira todos de uma vez, a partir de `visoes.jsx`:

```bash
~/.claude/skills/prototipo-html/scripts/tirar-prints.sh http://localhost:8765 <id-da-rotina> <protótipo>/prints
```

O script precisa do Node 22 ou mais novo e de um Chrome ou Chromium (`CHROME=<caminho>`
escolhe outro). Ele abre o Chrome sem janela pelo protocolo de depuração, com a área de
tela exata de cada combinação, e só fotografa quando a tela montou: o `--screenshot` do
Chrome cortava e deslocava a moldura.

Os arquivos saem como `prints/<rotina>/<ordem>-<tela>--<superficie>--<tema>--<estado>.png`.
Rotina com dúvida A/B tira a conferência inteira em cada opção, em
`prints/<rotina>/opcao-<duvida>-<letra>/`.
Abra os prints e confira cada um contra a checklist antes de mostrar ao humano. Os prints
aprovados são commitados na branch da rotina e vão **num comentário da PR**, com link fixo
ao commit (`https://github.com/<dono>/<repo>/blob/<sha>/<caminho>?raw=true`), para a
evidência não mudar depois do merge.

## Mocks

`data.jsx` guarda os dados de exemplo. **Nunca PII real** — nem em código, nem em
comentário. Nomes, telefones, e-mails e endereços são sempre fictícios e visivelmente
fictícios.

Protótipo não tem lógica de negócio: rede, cálculo de preço e regra de domínio são
simulados. O que se valida é a superfície visual e o fluxo, não a implementação.

## Anti-padrões

| Não faça | Por quê |
|---|---|
| Tailwind, Bootstrap ou framework de CSS | Contradiz o modelo de tokens e infla o protótipo |
| Build step (Vite, webpack, esbuild) | O ganho inteiro do padrão é o zero-friction |
| `import`/`export` nos `.jsx` | Não há módulos; quebra o carregamento global |
| Lógica de negócio real | Protótipo simula com mocks — valida visual, não implementação |
| Usar a stack final no protótipo | Perde a iteração rápida que justifica o padrão |
| Hex ou px direto no JSX | Quebra a troca de tema e o mapeamento para a stack real |
| String hard-coded no componente | Fura o i18n e espalha copy pelo código |
| Editar sem bump do `?v=` | Você revisa a versão antiga sem perceber |
| Segundo protótipo do mesmo produto | Duas paletas e dois `ui.jsx`; integre na aba certa |
| Tela fora de qualquer rotina | Não dá para contar os toques nem responder à pergunta de aparência |

## Servir

```bash
cd <raiz-de-prototipos>/<nome-do-prototipo>
python3 -m http.server 8765
```

Precisa ser servido por HTTP — abrir o `index.html` via `file://` falha, porque o Babel
busca os `.jsx` por XHR e o navegador bloqueia por CORS.

A skill `/iniciar-prototipo` cuida disso, incluindo porta ocupada e smoke test.

## Aprovação

Protótipo é fonte de verdade visual: só depois de aprovado vira código de produção.
A aprovação é explícita do usuário — não se infere de silêncio nem de um "legal".

O nível de cerimônia depende do projeto; veja `deteccao-projeto.md`.
