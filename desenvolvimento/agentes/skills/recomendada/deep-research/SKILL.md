---
name: deep-research
description: >-
  Pesquisa profunda multi-fonte com triangulação e relatório com fonte por afirmação.
  Use quando o usuário pedir deep research, pesquisa profunda, levantamento técnico,
  investigação de mercado/tecnologia/fornecedor, ou quando a resposta exigir várias
  fontes verificadas em vez de uma busca rápida. NÃO use para consulta pontual de um
  fato só (WebSearch direto resolve) nem para pesquisa dentro do código do repo.
---

# deep-research

Pesquisa é processo, não busca. O entregável é um relatório em que **cada afirmação-chave
carrega a fonte** — nunca "a internet diz". Responder de memória, sem buscar, é o
anti-padrão número um: memória de modelo tem data de validade e mercado muda.

## Passo 1 — Fechar a pergunta antes de abrir o navegador

Decomponha o pedido em **sub-perguntas verificáveis** (3 a 7). "Como funciona X?" não é
verificável; "quais linguagens o SDK de X aceita, segundo a documentação oficial?" é.

- Se o pedido for ambíguo em escopo, moeda, região ou período: **AskUserQuestion** antes
  de gastar uma hora pesquisando a pergunta errada.
- Escreva as sub-perguntas no início do trabalho. Elas viram as seções do relatório —
  e o critério de "terminei".

## Passo 2 — Varredura multi-ângulo

Uma query só encontra o que aquela query encontra. Para cada sub-pergunta, busque por
**ângulos diferentes**:

- Termo técnico em inglês E em português (docs oficiais quase sempre em EN; prática
  local, fóruns e regulamentação, em PT)
- Nome do produto/fabricante ("PAX A920 SDK") E nome da categoria ("smart POS Android
  development")
- Busca dirigida à fonte primária: `site:` do fabricante, da norma, do órgão regulador

**Hierarquia de fontes** — mais alto vence quando houver conflito:
1. Documentação oficial do fabricante/plataforma, norma publicada, regulador
2. Repositório/changelog oficial, fórum de desenvolvedores do próprio fabricante
3. Integrador/parceiro com relação comercial declarada
4. Blog, revendedor, agregador, IA de terceiros — só como pista, nunca como prova

Registre a query usada quando um achado for importante: quem revisar precisa conseguir
refazer o caminho.

## Passo 3 — Fan-out com subagentes (opcional)

Com 3+ sub-perguntas independentes e ferramenta de agentes disponível, dispare um
subagente por sub-pergunta, em paralelo. Contrato de cada agente:

- Devolver **achados + URL de cada achado + a query que o encontrou**
- Marcar o que NÃO encontrou (ausência declarada vale mais que silêncio)

O orquestrador não aceita afirmação de subagente sem URL — reenvia ou rebaixa para
"não confirmado".

## Passo 4 — Leitura profunda

Busca acha o documento; a resposta mora dentro dele. Use WebFetch nas fontes-chave e
leia de verdade — data de publicação inclusive.

- PDF ou página que a ferramenta não lê: tente rota alternativa (versão HTML, cache,
  renderizar localmente). Se nada abrir, o relatório diz **"fonte não lida"** — nunca
  finja que leu.
- Paywall e login idem: declare o limite.

## Passo 5 — Triangulação

- **Afirmação-chave exige 2+ fontes independentes.** Com uma só, marque `[fonte única]`
  — o leitor decide quanto confiar. Independente = não deriva uma da outra (dez blogs
  copiando o mesmo press release contam como um).
- **Conflito entre fontes vira parágrafo próprio**, com as duas versões e as datas.
  Escolher em silêncio a versão mais conveniente é fraude de pesquisa.
- **Date os achados.** "O SDK aceita Kotlin (doc de 2024)" envelhece melhor que
  "aceita Kotlin".

## Passo 6 — Entrega em arquivos: um .md por pesquisa + consolidação

A pesquisa entrega uma **pasta**, não um arquivo solto:

```
<raiz-de-pesquisa>/<YYYY-MM-DD>-<slug>/
├── 01-<sub-pergunta>.md      # uma investigação por arquivo
├── 02-<sub-pergunta>.md
├── ...
└── consolidacao.md           # a síntese, e nada além dela
```

**Cada `NN-<sub-pergunta>.md`** é o registro bruto daquela investigação:
a sub-pergunta, as **queries usadas**, as **fontes** (URL + data + tipo na hierarquia
do Passo 2), os **achados** com nível de confiança, e o que não foi encontrado ou não
abriu. É o arquivo que permite refazer ou auditar a pesquisa depois.

**O `consolidacao.md`** é puro: só síntese, sem achado novo — tudo nele rastreia para
um arquivo `NN-*.md` (linke-os). Estrutura:

1. **Resumo executivo** — as respostas em 5-10 linhas, sem jargão
2. **Uma seção por sub-pergunta** — cada afirmação-chave com a URL inline e o nível:
   `[confirmado]` (2+ fontes) · `[fonte única]` · `[inferido]` (dedução sua, diga de quê)
3. **O que segue aberto** — o que a pesquisa não conseguiu fechar, e por quê
4. **Perguntas para quem sabe** — o que só o fornecedor/parceiro/regulador responde;
   prontas para copiar num e-mail
5. **Limites da pesquisa** — fontes não lidas, regiões não cobertas, data do trabalho

Se o projeto tiver convenção de onde guardar pesquisa (ex.: `pesquisa-tecnica/`),
a pasta nasce lá; senão, pergunte antes de espalhar arquivo.

## Anti-padrões

- Responder de memória sem nenhuma busca — o motivo de a skill existir
- Uma query por sub-pergunta e pronto
- Citar agregador/blog quando a fonte primária está a um clique
- Esconder conflito entre fontes escolhendo a mais conveniente
- "Não achei" sem dizer o que buscou e onde
- Fingir leitura de fonte que não abriu (paywall, PDF quebrado)
- Misturar achado com recomendação sem rotular qual é qual
- Relatório sem data: pesquisa é fotografia, e fotografia tem carimbo
