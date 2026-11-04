---
name: setup-stacklabs
description: Instala o método StackLabs num projeto novo, ou atualiza um projeto que já o segue.
disable-model-invocation: true
allowed-tools: Bash(sh ${CLAUDE_SKILL_DIR}/rodar *)
---

Resultado da conferência da skill com a fonte oficial, feita agora:

!`sh ${CLAUDE_SKILL_DIR}/rodar --conferir-skill`

Se a linha acima diz `SKILL ATUALIZADA setup-stacklabs`, este texto está velho: leia
`${CLAUDE_SKILL_DIR}/SKILL.md` de novo e siga a versão nova. Com `SKILL NÃO CONFERIDA`, pare: o
instalador também depende da fonte oficial, então diga o erro ao humano.

Leva ao projeto o `modelo-de-projeto` (constituição, `docs/agents/`, watchdogs), as skills da casa e
as skills publicadas do Matt Pocock, todas em `.claude/skills/`. Quem copia, mescla, confere e faz o
merge é o instalador em Rust desta pasta (`src/`), que o [`rodar`](rodar) compila na primeira vez;
você conduz e resolve o que ele devolver. Numa atualização, **a pessoa só dispara a skill**: nenhum
passo pede confirmação ([ADR 0024](../../../adr/0024-atualizacao-do-metodo-sem-humano.md)).

O instalador é idempotente e só instala da fonte oficial: as variáveis de fonte local e voltar de
versão são do humano, e o `guarda-bash` as barra.

## Passos

### 1. Diagnóstico

Na raiz do projeto, descubra o **caso**:

- **atualização**: há `.claude/skills/ORIGEM.json`;
- **instalação**: repositório sem código nem histórico além do inicial;
- **adoção**: código em andamento sem o método. Pare e indique o `/adotar-metodo-stack-labs`.

Pronto quando o caso está definido.

### 2. Branch

Crie `chore/atualizar-metodo` (ou `chore/setup-metodo`) a partir da `origin/main` atualizada. Com
mudança não commitada fora de `.claude/skills/`, trabalhe num worktree novo ao lado do projeto, para
ela não entrar na atualização, e passe `--destino <worktree>` a todo comando seguinte.

Pronto quando a branch existe e está limpa, fora o que a conferência da skill trouxe.

### 3. Instalação

Rode `sh ${CLAUDE_SKILL_DIR}/rodar` (com `--versao <x.y.z>` se o pedido citou uma versão).

Pronto quando o instalador terminou sem erro e imprimiu o relatório.

### 4. Pendências do relatório

A linha `VERSÃO` diz de qual versão do método para qual o projeto foi: leia as entradas do
`desenvolvimento/CHANGELOG.md` entre as duas e trate o que elas mandam fazer à mão. As linhas
`COPIADO`, `MESCLADO`, `REMOVIDO`, `SKILL` e `COMPILADO` não pedem nada.

- **`MESCLAR`** (`AGENTS.md` sem a seção `## Regras deste projeto`, ou `CLAUDE.md` próprio): o modelo
  novo em `<arquivo>.stacklabs-novo` é a base. Cada regra própria do arquivo antigo vai para
  `### Acrescentadas`, e cada regra do modelo que o projeto não segue, para `### Afastadas`, no
  formato da seção. Apague o `.stacklabs-novo`. O que não couber vira pergunta na PR, e ela fica para
  uma pessoa.
- **`FALHA` dos watchdogs:** a fonte não compilou e os watchdogs do projeto ficaram como estavam.
  Mostre o erro ao humano e pare: é defeito do Stack-Labs.
- **`FALHA` das labels:** sem `origin` no GitHub ou `gh` sem login. Diga o que falta e rode de novo
  depois de resolvido; o instalador só refaz o que falta.

Pronto quando `find . -name '*.stacklabs-novo' -not -path './.git/*'` não lista nada.

### 5. Verificação

Rode `sh ${CLAUDE_SKILL_DIR}/rodar --verificar`. Cada `REPROVADO` é defeito da instalação ou violação
no código do projeto. Corrija o defeito; violação do validador vira ajuste em
`docs/regras-de-codigo.json` ou ticket, e a PR fica para uma pessoa (o `lint-cobre`, a `cobertura` e
o P5 passam a reprovar regra afastada depois das ADRs 0012 a 0014).

Pronto quando imprime `VERIFICADO`, ou cada reprovação restante tem ticket.

### 6. PR

Um commit com tudo o que o instalador e a mesclagem mudaram, push e PR. Escreva o corpo com a skill
`/pr`, trazendo a versão do método e os commits de origem de `ORIGEM.json`. Num repositório sem
`main`, diga ao humano que a PR depende de ele criar a `main`.

**Atualização:** rode `sh ${CLAUDE_SKILL_DIR}/rodar --concluir-pr <número>`. Ele prova que a PR só traz o
que a fonte oficial manda, aplica a label `altera-watchdogs` se precisar, espera o CI e faz o merge.
Saída 3 (`FICA PARA UM HUMANO`) lista os motivos: repasse-os ao humano com o link da PR.

**Instalação:** o merge é do humano.

Pronto quando a PR foi mesclada, ou o humano tem o link e os motivos.

### 7. Próximo passo

Os hooks só carregam no início de uma sessão: gere com o `/nova-sessao` o kickoff da **janela nova**
para o humano colar lá. O próximo trabalho depende do caso:

- **Instalação:** o `sessao-inicio` mostra o setup ✅ e os mapas iniciais ❌; o próximo trabalho é
  abrir os cinco mapas iniciais (`docs/agents/mapas-iniciais.md`).
- **Atualização:** se o `sessao-inicio` mostrar ❌ num mapa inicial que o projeto ainda não tinha (o
  de Regras de código, em projeto anterior à ADR 0007), avise o humano que nenhuma issue será ativada
  até esse mapa existir e fechar. O próximo trabalho é abri-lo como mapa retroativo.

Pronto quando o humano tem o kickoff e sabe qual é o próximo trabalho.
