# Busca determinística para contexto: repo map, BM25 e dossiê por modelo barato

Pesquisa de 2026-10-25 (segunda passada no mesmo dia: toda afirmação conferida na fonte primária;
seção 6 nova). Pergunta: para gastar menos tokens e dar contexto melhor ao agente (Claude Code), a
proposta abaixo tem apoio em fonte primária? E o que já existe para adotar em vez de escrever
(ADR 0010: "ferramenta que não cobra a gente usa; a que cobra a gente escreve")? Pergunta somada:
grafos e memória persistente ("segundo cérebro") ajudam o agente a entender o contexto?

1. camada determinística em Rust: símbolos por tree-sitter + grafo de referências ranqueado
   (repo map do Aider, PageRank), BM25 pelo tantivy, busca literal pelos crates do ripgrep (`grep`,
   `ignore`), cache por hash de arquivo;
2. um modelo barato (Haiku, por `claude -p`) recebe os candidatos e devolve um **dossiê** de esquema
   fixo com citações `caminho:linha`; o Rust confere que cada citação existe;
3. o modelo forte (Sonnet/Opus) lê o dossiê e abre só os trechos citados.

## Resultado

| Proposta | Fontes que sustentam | Fontes que contestam ou limitam | Conclusão |
|---|---|---|---|
| 1. Camada determinística | SWE-bench usa BM25 como recuperação base (Jimenez et al., ICLR 2024); Agentless localiza pela árvore do repo + LLM e foi o melhor aberto no SWE-bench Lite (32,00%, US$ 0,70 por issue; Xia et al. 2024); Aider ranqueia o repo map com PageRank personalizado (código `aider/repomap.py`); Anthropic, *Contextual Retrieval*: embeddings + BM25 cortam 49% das falhas contra 35% só de embeddings; Claude Code: glob e grep guiados pelo modelo venceram banco vetorial local (Boris Cherny, entrevista ao Pragmatic Engineer, 2026-03-04); Sourcegraph tirou embeddings do Cody na 5.3 e passou a usar a própria busca, "tão bem quanto embeddings e às vezes melhor" | No SWE-bench, o BM25 não traz nenhum arquivo do patch em quase metade das issues; CodeRAG-Bench: denso supera BM25 com frequência (no SWE-bench Lite, 47,8 contra 43,0), mas BM25 vence no RepoEval (93,2) e indexa centenas de vezes mais rápido; Agentless soma busca por embeddings à localização; Cursor (2025-11-06): somar busca semântica ao grep deu +12,5% de acerto médio em perguntas sobre código; a evidência do Claude Code é interna, sem benchmark público | **Sustentada como base**, não como substituto de tudo. Lexical + símbolos é barato e determinístico; denso fica fora até a medição mostrar falta |
| 2. Haiku filtra e cita | Docs da Anthropic põem o Haiku em "subagent tasks" e processamento de alto volume; *Building effective agents*: encadeamento de prompts com portão programático entre etapas; cascatas (FrugalGPT: até 98% menos custo com a qualidade do GPT-4) e reranking por LLM (RankGPT) | RankGPT: modelos mais fracos ranqueiam bem pior (curie-001 contra GPT-4, Tabela 4); *lost in the middle* (Liu et al., TACL 2024) vale para o filtro também | **Sustentada com portão**: a conferência de citação em Rust é obrigatória e o dossiê nunca é a única fonte (o forte pode buscar) |
| 3. Forte abre só o citado | Docs do Claude Code: subagente roda em contexto próprio e devolve só o resumo | Se o dossiê erra, o forte erra com confiança; Anthropic: agentes gastam ~4x os tokens de um chat e multiagente ~15x (*How we built our multi-agent research system*) | **Plausível, não provada**: só a medição por issue (`rodar custo`) decide |
| 4. Grafo de código (seção 6) | RepoGraph: +32,8% relativo médio na taxa de resolução do SWE-bench somado a 4 métodos; LocAgent: até 92,7% de acerto na localização por arquivo, com custo ~86% menor; CodexGraph: competitivo no SWE-bench Lite com grafo por análise estática | Ganho medido em Python e em sistemas de pesquisa; RepoGraph mostra que achatar 2 saltos do grafo no prompt piora (26,00%, abaixo da linha de base) | **Sustentada** quando as arestas saem de parser (tree-sitter/LSP/SCIP), sem LLM; adotar por ferramenta pronta |
| 5. Grafo de conhecimento por LLM (GraphRAG, LightRAG) | GraphRAG ganha em perguntas "globais" sobre corpus de ~1 milhão de tokens | Indexação chama o LLM por bloco de texto (600 tokens no artigo; 281 min num corpus); arestas extraídas por modelo erram e envelhecem; código já tem estrutura exata de graça | **Não adotar** para código |
| 6. Memória persistente ("segundo cérebro") | A-MEM (notas ligadas no estilo Zettelkasten), MemGPT/Letta, memory tool da Anthropic e CLAUDE.md/auto memory do Claude Code; servidor MCP de memória (grafo em JSONL) | Docs da Anthropic mandam limitar tamanho e expirar memória; memória escrita pelo modelo envelhece sem aviso | **O método já é um segundo cérebro** (glossário, ADRs, mapas, riscos); o que falta são links conferidos por máquina, não um banco novo |

| Peça | Versão (crates.io / GitHub, 2026-10-25) | Licença | Situação |
|---|---|---|---|
| `tree-sitter` (bindings Rust) | 0.27.1 (2026-10-08) | MIT | maduro. Gramáticas são crates separados que dependem só de `tree-sitter-language ^0.1` (o `tree-sitter` é dependência de dev): `tree-sitter-rust` 0.24.2, `-python`/`-javascript`/`-go` 0.25.0, `-typescript` 0.23.2, `-java` 0.23.5. O runtime aceita ABI 13 a 15 (`TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION`/`TREE_SITTER_LANGUAGE_VERSION` em `api.h`), conferido na carga da gramática; linkam juntos com o 0.27 |
| `tantivy` | 0.27.0 (2026-10-17) | MIT | maduro (Quickwit); BM25 nativo |
| `grep` | 0.4.1 (2025-10-22) | MIT/Unlicense | motor do ripgrep |
| `ignore` | 0.4.33 (2026-08-04) | MIT/Unlicense | respeita `.gitignore`, paralelo |
| `rmcp` (SDK MCP oficial em Rust) | 3.7.0 (2026-10-22) | Apache-2.0 | oficial em `modelcontextprotocol/rust-sdk` |
| `ast-grep` | 0.51.0 (2026-10-15) | MIT | já usado pelo validador (ADR 0010) |
| probe (`probelabs/probe`) | ativo (push 2026-10-22), 731 estrelas | Apache-2.0 | Rust: ripgrep + tree-sitter + BM25/TF-IDF/híbrido (`--reranker`), reranking BERT opcional; devolve blocos de código inteiros; CLI e MCP. **Sem PageRank nem grafo de referências** (README e `src/search/`) |
| codanna (`bartolli/codanna`) | ativo (2026-10-11), 756 estrelas | Apache-2.0 | Rust: índice de símbolos tree-sitter, chamadas/referências, MCP; busca semântica opcional |
| serena (`oraios/serena`) | ativo (2026-10-20), 30,7 mil estrelas | **GPL-3.0-or-later** (a biblioteca SolidLSP é MIT) | Python, usa LSP; MCP; pesado para instalar em todo projeto. Usar como ferramenta externa não obriga nada; copiar código dela para o motor obriga GPL |

## 1. Recuperação determinística funciona para código?

- **SWE-bench** (Jimenez et al., ICLR 2024, <https://arxiv.org/abs/2310.06770>, seção 4.1 e
  Tabela 3): a recuperação base é BM25. Com limite de 27 mil tokens, o BM25 traz um superconjunto
  dos arquivos do patch em ~40% das issues e nenhum deles em quase metade. Lição: BM25 sozinho é
  piso, não teto.
- **Agentless** (Xia et al. 2024, <https://arxiv.org/abs/2407.01489>): localização hierárquica
  (árvore do repo → arquivo → classe/função → linha) pelo LLM, **somada a uma busca por embeddings**
  depois de filtrar pastas irrelevantes. 32,00% no SWE-bench Lite a US$ 0,70 por issue, o melhor
  entre os abertos na época. Sustenta "esqueleto de símbolos + LLM escolhe" (etapas 1 e 2), mas
  não prova que dá para dispensar o denso.
- **RepoCoder** (Zhang et al., EMNLP 2023, <https://arxiv.org/abs/2303.12570>): recuperação
  iterativa por similaridade melhora em mais de 10% a completação no nível do repositório.
- **CodeRAG-Bench** (Wang et al. 2024, <https://arxiv.org/abs/2406.14497>, Tabelas 3 e 4): "dense
  embedding models frequently surpass BM25", mas o quadro é misto no código de repositório: BM25
  93,2 contra 81–83 dos GIST no RepoEval; no SWE-bench Lite, BM25 43,0 contra 47,8 do GIST-large.
  BM25 codifica em 0,15 ms por documento; o GIST-large em 13 ms.
- **Aider, repo map** (<https://aider.chat/docs/repomap.html>, post
  <https://aider.chat/2023/10/22/repomap.html>, código `aider/repomap.py`): extrai definições e
  referências por tree-sitter, monta um grafo e roda `pagerank` do networkx com personalização
  pelos arquivos e identificadores citados; corta no orçamento `--map-tokens` (padrão 1k). É a
  especificação pronta da etapa 1; as consultas `tags.scm` podem ser reaproveitadas (Aider é
  Apache-2.0).
- **Anthropic, *Contextual Retrieval*** (<https://www.anthropic.com/news/contextual-retrieval>):
  sobre o top-20, embeddings contextuais cortam 35% das falhas, somados ao BM25 contextual 49%, e
  com reranking 67%. "Codebases" estão entre os domínios testados. Experimento de RAG, não de agente.
- **Claude Code**: na entrevista *Building Claude Code with Boris Cherny* (Pragmatic Engineer,
  2026-03-04, <https://newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny>),
  o resumo do episódio registra que o time tentou bancos vetoriais locais e indexação recursiva por
  modelo, todos com defeito (índice velho, complexidade de permissão), e que glob e grep guiados
  pelo modelo "beat everything". Não há benchmark público. A frase de que a avaliação foi por
  "vibes" internas só aparece em fonte secundária (OfficeChai) e num post dele no X de 2026-02-01
  citado sem link **[confirmar: o post no X não foi achado por busca; precisa da URL]**. Leitura
  para o método: o Claude Code já faz busca lexical; a proposta só se paga se entregar *menos*
  tokens que o grep agêntico para o mesmo acerto.
- **Sourcegraph/Cody** (FAQ da 5.3, <https://5.3.sourcegraph.com/cody/faq>): embeddings saíram na
  5.3; o Cody passou a usar a Sourcegraph Search como provedor principal de contexto, e "early
  evidence suggests that this context fetching works as well as embeddings and is sometimes
  better". Não achei post de blog; a FAQ é a fonte oficial.
- **Cursor** (*Improving agent with semantic search*, 2025-11-06, <https://cursor.com/blog/semsearch>):
  contesta. Somar busca semântica ao grep deu +12,5% de acerto médio (6,5% a 23,5% conforme o
  modelo) e +2,6% de retenção do código em bases com mais de mil arquivos; "a combinação dos dois"
  é o melhor. Medição interna do Cursor.

## 2. Modelo pequeno filtra, modelo grande decide

- **Escolha de modelo** (<https://platform.claude.com/docs/en/about-claude/models/choosing-a-model>):
  a matriz põe o Haiku 5.5 em "real-time applications, high-volume intelligent processing, ...
  subagent tasks". Subagentes do Claude Code aceitam `model: haiku` e rodam "in its own context
  window", devolvendo só o resumo (<https://code.claude.com/docs/en/sub-agents>). O Explore embutido
  usa o modelo da conversa, não o Haiku; para Haiku, definir um subagente próprio.
- ***Building effective agents*** (<https://www.anthropic.com/engineering/building-effective-agents>):
  "the simplest solution possible"; *prompt chaining* com "gates" programáticos entre etapas é
  exatamente a conferência de citações em Rust.
- **Custo de multiagente** (<https://www.anthropic.com/engineering/multi-agent-research-system>):
  agentes usam ~4x os tokens de um chat; multiagente ~15x. O dossiê precisa provar economia.
- **Cascatas**: FrugalGPT (Chen et al. 2023, <https://arxiv.org/abs/2305.05176>): iguala o GPT-4
  com até 98% menos custo. **Reranking por LLM**: RankGPT (Sun et al., EMNLP 2023,
  <https://arxiv.org/abs/2304.09542>): GPT-4 ranqueia acima de supervisionados; modelos mais fracos
  (curie-001, davinci-003) ficam bem abaixo (Tabela 4). Os modelos testados são de 2023; o Haiku
  atual precisa da própria medição.
- **Riscos**: o filtro descarta o arquivo certo (falso negativo, invisível ao forte) e inventa
  `caminho:linha`. *Lost in the Middle* (Liu et al., TACL 2024, <https://arxiv.org/abs/2307.03172>):
  o meio de um contexto longo é o mais ignorado; mandar poucos candidatos ranqueados ao Haiku.
  Mitigações: Rust confere existência de arquivo, faixa de linha e que o trecho contém o símbolo
  citado; o dossiê lista também o que foi descartado (top‑N com escore) para o forte poder abrir.

## 3. Adotar antes de escrever

- **probe** (<https://github.com/probelabs/probe>): a etapa 1 quase inteira (ripgrep + tree-sitter
  + BM25/TF-IDF/híbrido, blocos AST completos, limite de tokens, CLI e MCP), Apache-2.0, Rust.
  Não tem PageRank nem grafo de referências: o ranqueamento é por texto (`src/search/`,
  `early_ranker` BM25).
- **codanna** (<https://github.com/bartolli/codanna>): grafo de símbolos (definições, chamadas,
  quem chama), Rust, Apache-2.0, com MCP. Cobre o que o probe não tem.
- **ast-grep** já está no método e resolve consulta estrutural pontual; não ranqueia.
- **serena** (<https://github.com/oraios/serena>): LSP (mais preciso), mas Python, exige servidores
  de linguagem e a aplicação é GPL-3.0-or-later. Fica para projeto que já tenha LSP à mão.
- Pela ADR 0010, nenhuma delas cobra: **usar**. O que nenhuma faz, e é nosso: o dossiê de esquema
  fixo, a conferência de citações e a medição.

## 4. Integração

- **CLI + skill**: o binário dos watchdogs ganha um subcomando (ex.: `rodar contexto`) que imprime
  candidatos/dossiê; uma skill ou subagente o chama por Bash. Sem processo residente, versionado com
  o motor (ADR 0011).
- **MCP em Rust** (`rmcp`, <https://github.com/modelcontextprotocol/rust-sdk>): ferramentas
  tipadas. O custo de contexto por ferramenta caiu: a busca de ferramentas do Claude Code vem ligada
  por padrão e só lista os nomes até o modelo procurar (<https://code.claude.com/docs/en/mcp>,
  "Scale with MCP tool search"). Ainda exige servidor configurado por projeto; probe e codanna já
  oferecem MCP, então escrever o nosso não se justifica.
- **Hooks** (<https://code.claude.com/docs/en/hooks>): `SessionStart` e `UserPromptSubmit` injetam
  `additionalContext`; `PreToolUse` bloqueia (`deny`) ou troca a entrada (`updatedInput`). O
  `UserPromptSubmit` tem timeout padrão de 30 s (os demais, 600 s), então não serve para chamar o
  Haiku em todo prompt; o `SessionStart` serve para injetar um repo map curto em cache.

## 5. Medição

- Unidade: a issue, com o `rodar custo` (ADR 0016) comparando tokens de entrada/saída e custo por
  issue com e sem a camada, em issues reais pareadas por tamanho.
- Qualidade da recuperação, offline: para issues já fechadas, o conjunto de arquivos do diff do
  merge é o gabarito; medir revocação@k dos candidatos (etapa 1) e do dossiê (etapa 2), como o
  SWE-bench faz com o oracle.
- Citações: taxa de citações inválidas barradas pelo Rust; meta zero que passe.
- Resultado final: a issue fecha com a mesma revisão aprovada (ADR 0015) e sem mais retrabalho.

## 6. Grafos e memória persistente ("segundo cérebro")

### 6.1 Grafo de conhecimento por LLM (GraphRAG, LightRAG)

- **GraphRAG** (Edge et al. 2024, <https://arxiv.org/abs/2404.16130>; `microsoft/graphrag`, MIT):
  o LLM extrai um grafo de entidades de cada bloco de texto e gera resumos de comunidades; a
  pergunta é respondida por map-reduce sobre os resumos. Ganha de RAG vetorial em abrangência e
  diversidade nas perguntas "globais" ("quais os temas do corpus?") sobre ~1 milhão de tokens.
  Custo: uma ou mais chamadas de LLM por bloco de 600 tokens (com "gleanings" para achar entidades
  perdidas); indexar o corpus de podcasts levou 281 minutos. Na consulta, os resumos de raiz usam
  97% menos tokens que resumir o texto-fonte.
- **LightRAG** (Guo et al. 2024, <https://arxiv.org/abs/2410.05779>; `HKUDS/LightRAG`, MIT): grafo +
  vetores com recuperação em dois níveis e atualização incremental; mais barato que o GraphRAG, mas
  a extração de entidades ainda é por LLM.
- **Encaixe em código**: fraco. A pergunta típica do agente é local ("onde muda isto?", "quem chama
  aquilo?"), não "global"; e o código já tem a estrutura exata (definição, chamada, import) que o
  parser extrai sem modelo. Pagar LLM por bloco para obter arestas aproximadas do que o tree-sitter
  dá exato, e reindexar a cada commit, não fecha a conta.

### 6.2 Grafo de código para agentes

- **Formatos de índice**: SCIP (`sourcegraph/scip`, Apache-2.0) e LSP dão definição/referência
  precisas por compilador; tree-sitter dá definição/referência sintática, mais barata e multilíngue,
  com falsos positivos de nome igual. *Code property graph* (AST + fluxo de controle + dados) é
  ferramenta de análise de segurança; não achei estudo dele como contexto de agente.
- **CodexGraph** (Liu et al. 2024, <https://arxiv.org/abs/2408.03910>): análise estática grava
  módulos, classes, funções e suas relações num banco de grafo (Neo4j); o agente escreve consultas
  Cypher. Competitivo no CrossCodeEval, SWE-bench Lite e EvoCodeBench.
- **RepoGraph** (Ouyang et al. 2024, <https://arxiv.org/abs/2410.14684>): grafo por linha de código
  (definição/referência), plugável; +32,8% relativo médio na resolução do SWE-bench somado a quatro
  métodos (+2,34 pontos no Agentless). Achado útil: achatar o ego-grafo de 2 saltos no prompt
  derrubou a taxa para 26,00%, abaixo da linha de base. Grafo bom, entregue em excesso, piora.
- **LocAgent** (Chen et al., ACL 2025, <https://arxiv.org/abs/2503.09089>; Apache-2.0): grafo
  heterogêneo (arquivo, classe, função; import, chamada, herança) por parsing, navegado pelo agente
  em vários saltos. Até 92,7% de acerto na localização por arquivo; com Qwen-2.5-Coder-32B ajustado
  iguala modelos proprietários a ~86% menos custo; +12% no Pass@10 da resolução.
- **Ferramentas prontas**: codanna (tree-sitter, chamadas/quem chama, MCP) e serena (LSP, MCP) são
  esses grafos já empacotados. O repo map do Aider é a versão "grafo ranqueado e cortado no
  orçamento".

### 6.3 Memória persistente

- **A-MEM** (Xu et al. 2025, <https://arxiv.org/abs/2502.12110>): notas no estilo Zettelkasten;
  cada memória nova vira nota com palavras-chave e tags, o LLM liga notas parecidas e reescreve as
  antigas ("memory evolution"). Ganho medido em conversa longa, não em código.
- **MemGPT** (Packer et al. 2023, <https://arxiv.org/abs/2310.08560>; hoje `letta-ai/letta`,
  Apache-2.0): memória em camadas como sistema operacional, o modelo pagina entre contexto e
  armazenamento.
- **Anthropic memory tool**
  (<https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool>): diretório
  `/memories` do lado do cliente, comandos de arquivo; os próprios docs mandam limitar o tamanho,
  "periodically delete memory files that haven't been accessed in a long time" e, para software,
  manter log de progresso e marcar feito só após verificação de ponta a ponta.
- **Claude Code** (<https://code.claude.com/docs/en/memory>): CLAUDE.md/AGENTS.md (escrito pelo
  humano) e auto memory (escrita pelo Claude, carrega as primeiras 200 linhas ou 25 KB), ambos
  lidos em toda sessão e tratados "as context, not enforced configuration".
- **Servidor MCP de memória** (`modelcontextprotocol/servers`, `src/memory`, MIT): grafo de
  entidades, relações e observações num JSONL local; quem escreve é o modelo.

### 6.4 O método já é um segundo cérebro

O Stack-Labs já guarda memória ligada e curada pelo humano: `GLOSSARY.md` (linguagem, com
_Evitar_), ADRs (decisões difíceis de reverter; contradizer exige apontar), mapas `wayfinder:map`
(índice com as decisões tomadas, uma linha e um link cada, névoa e fora de escopo), registro de
riscos `RK<n>` em `docs/riscos.md` e o CLAUDE.md/AGENTS.md. É Zettelkasten com dono: cada nota tem
ID, origem e link. Isso já resolve o defeito que A-MEM e o servidor MCP têm, memória escrita pelo
modelo sem revisão.

O que um grafo acrescentaria são **arestas conferidas por máquina** entre o que já existe:
ADR ↔ arquivo de código, issue ↔ commit/diff, teste ↔ `RK<n>` ↔ código onde o risco mora,
termo do glossário ↔ símbolo. Todas saem de texto e git sem LLM (IDs citados, `git log`, símbolos
do tree-sitter). O ganho é duplo: o agente acha "a ADR que vale para este arquivo" por consulta, e
o watchdog acusa link quebrado (ADR que cita arquivo apagado, `RK` sem teste, termo que sumiu).

**Risco de memória velha**: memória que o agente lê como verdade e ninguém confere é pior que não
ter memória; os docs da Anthropic pedem expiração, e o Pragmatic Engineer registra "stale indexes"
entre os motivos de o Claude Code largar o índice vetorial. Regra: grafo gerado a cada execução a
partir do repositório (cache por hash), nunca um banco paralelo editado à mão ou pelo modelo.

### 6.5 Construção determinística contra extração por LLM

| | Arestas por parser (tree-sitter, SCIP, LSP, git) | Entidades extraídas por LLM (GraphRAG, LightRAG, A-MEM, MCP memory) |
|---|---|---|
| Custo de indexar | CPU, segundos; incremental por hash | chamadas de LLM por bloco; minutos a horas |
| Exatidão | exata (LSP/SCIP) ou sintática (tree-sitter) | aproximada, varia por execução |
| Atualização | regenera a cada commit | reextração cara; envelhece |
| Serve para | quem chama, onde define, o que importa, links ADR/RK/issue | perguntas globais sobre texto livre |
| Evidência em código | RepoGraph, LocAgent, CodexGraph, Aider | nenhuma achada para repositório de código |

## 7. Medição local: probe contra `git grep` neste repositório

Rodado em 2026-10-25 com o probe 0.6.0 (`cargo install probe-code --locked`). Gabarito: as PRs
mergeadas do Stack-Labs com até 20 arquivos; pergunta é o título da PR, resposta certa são os
arquivos alterados que ainda existem. Métrica: acerto@10 (fração dos arquivos certos entre os 10
primeiros devolvidos). Linha de base: `git grep -c -i -F` por palavra do título com 4 letras ou
mais, ranqueando pelo número de palavras distintas e depois de ocorrências.

| PR | Arquivos certos | probe | git grep |
|---|---|---|---|
| Custo de tokens por issue nos watchdogs | 12 | 0,10 | 0,60 |
| Orquestrador: uma sessão por issue | 5 | 0,60 | 0,60 |
| CI local no motor dos watchdogs | 16 | 0,00 | 0,90 |
| Grilling em linguagem simples | 3 | 0,33 | 0,33 |
| Caminho até ready-for-agent | 11 | 0,60 | 0,80 |
| Sessões: /nova-sessao e /voltar-a-principal | 13 | 0,40 | 0,60 |
| **Média (6 PRs)** | | **0,34** | **0,64** |

Limites: amostra pequena; o repositório é quase todo Markdown em português, e o probe tira radical e
palavra vazia em inglês ("de", "por", "nos" viram termo de busca); o título da PR foi escrito com as
mesmas palavras dos arquivos, o que favorece a busca literal. O `-f` (só arquivos) do probe não
ranqueia, e a saída `-o json` vem seguida de texto: quem integrar precisa ler só o primeiro objeto
JSON. **Conclusão:** neste repositório, o probe não paga a adoção; a medição precisa ser repetida
num projeto com código de produção antes de decidir o passo 2 da recomendação.

## Recomendação

Ordem:

1. **Medir a linha de base** com `rodar custo` em 5–10 issues sem nada novo, e montar o gabarito
   de arquivos pelos diffs.
2. **Adotar probe** (texto) e **codanna** (grafo de chamadas) como CLI chamada por uma
   skill/subagente; medir revocação@k e custo por issue. Entregar o grafo por consulta, não
   despejado no prompt (lição do RepoGraph).
3. **Construir só os links do método**, determinísticos, no motor Rust: ADR/`RK<n>`/issue/termo ↔
   arquivo e símbolo, extraídos de IDs citados, git e tree-sitter, regenerados por hash; servem a
   consulta do agente e um watchdog de link quebrado. É o "segundo cérebro" que falta, sobre o que o
   método já tem.
4. **Só se faltar grafo ranqueado**: repo map no motor com `tree-sitter` + `tags.scm` do Aider +
   PageRank, injetado por hook `SessionStart` com orçamento fixo.
5. **Dossiê pelo Haiku** como subagente (`model: haiku`), esquema fixo, com conferência de citações
   no Rust; ligar só se a etapa 2 mostrar economia e a revocação do dossiê ficar perto da dos
   candidatos.

Não construir:

- GraphRAG/LightRAG ou outro grafo de entidades extraído por LLM sobre o código;
- memória escrita pelo modelo sem revisão (servidor MCP de memória, A-MEM) como fonte de verdade:
  auto memory do Claude Code basta para preferência pessoal;
- servidor MCP próprio (probe/codanna já têm);
- índice BM25 próprio em tantivy enquanto probe resolver;
- embeddings/banco vetorial antes de a medição mostrar falha da busca lexical (o Cursor mostra que
  pode faltar em base grande: medir, não supor);
- chamada ao Haiku em hook de todo prompt;
- dossiê como única fonte: o forte continua podendo buscar.

## Fontes

- Jimenez et al., *SWE-bench*, ICLR 2024: <https://arxiv.org/abs/2310.06770>
- Xia et al., *Agentless*, 2024: <https://arxiv.org/abs/2407.01489>
- Zhang et al., *RepoCoder*, EMNLP 2023: <https://arxiv.org/abs/2303.12570>
- Wang et al., *CodeRAG-Bench*, 2024: <https://arxiv.org/abs/2406.14497>
- Aider, repo map: <https://aider.chat/docs/repomap.html>, <https://aider.chat/2023/10/22/repomap.html>,
  <https://github.com/Aider-AI/aider/blob/main/aider/repomap.py>
- Anthropic, *Contextual Retrieval*: <https://www.anthropic.com/news/contextual-retrieval>
- Anthropic, *Building effective agents*: <https://www.anthropic.com/engineering/building-effective-agents>
- Anthropic, *How we built our multi-agent research system*: <https://www.anthropic.com/engineering/multi-agent-research-system>
- Anthropic, escolha de modelo: <https://platform.claude.com/docs/en/about-claude/models/choosing-a-model>
- Anthropic, memory tool: <https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool>
- Claude Code, subagentes: <https://code.claude.com/docs/en/sub-agents>
- Claude Code, hooks: <https://code.claude.com/docs/en/hooks>
- Claude Code, MCP e busca de ferramentas: <https://code.claude.com/docs/en/mcp>
- Claude Code, memória: <https://code.claude.com/docs/en/memory>
- Orosz, *Building Claude Code with Boris Cherny*, Pragmatic Engineer, 2026-03-04:
  <https://newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny>
- Sourcegraph, Cody FAQ 5.3: <https://5.3.sourcegraph.com/cody/faq>
- Cursor, *Improving agent with semantic search*, 2025-11-06: <https://cursor.com/blog/semsearch>
- Chen et al., *FrugalGPT*, 2023: <https://arxiv.org/abs/2305.05176>
- Sun et al., *RankGPT*, EMNLP 2023: <https://arxiv.org/abs/2304.09542>
- Liu et al., *Lost in the Middle*, TACL 2024: <https://arxiv.org/abs/2307.03172>
- Edge et al., *From Local to Global: A Graph RAG Approach*, 2024: <https://arxiv.org/abs/2404.16130>
- Guo et al., *LightRAG*, 2024: <https://arxiv.org/abs/2410.05779>
- Liu et al., *CodexGraph*, 2024: <https://arxiv.org/abs/2408.03910>
- Ouyang et al., *RepoGraph*, 2024: <https://arxiv.org/abs/2410.14684>
- Chen et al., *LocAgent*, 2025: <https://arxiv.org/abs/2503.09089>
- Xu et al., *A-MEM*, 2025: <https://arxiv.org/abs/2502.12110>
- Packer et al., *MemGPT*, 2023: <https://arxiv.org/abs/2310.08560>
- crates.io (versões e dependências lidas pela API em 2026-10-25): <https://crates.io/crates/tree-sitter>,
  <https://crates.io/crates/tantivy>, <https://crates.io/crates/grep>,
  <https://crates.io/crates/ignore>, <https://crates.io/crates/rmcp>,
  <https://crates.io/crates/ast-grep-core>, gramáticas `tree-sitter-{rust,typescript,python,javascript,go,java}`
- tree-sitter, ABI: <https://github.com/tree-sitter/tree-sitter/blob/master/lib/include/tree_sitter/api.h>
- Repositórios (licença pela API do GitHub e pelo arquivo LICENSE): <https://github.com/probelabs/probe>,
  <https://github.com/bartolli/codanna>, <https://github.com/oraios/serena> (LICENSE),
  <https://github.com/ast-grep/ast-grep>, <https://github.com/modelcontextprotocol/rust-sdk>,
  <https://github.com/microsoft/graphrag>, <https://github.com/HKUDS/LightRAG>,
  <https://github.com/letta-ai/letta>, <https://github.com/gersteinlab/LocAgent>,
  <https://github.com/sourcegraph/scip>,
  <https://github.com/modelcontextprotocol/servers/tree/main/src/memory>
- Secundária, só para a frase "vibes": <https://officechai.com/ai/claude-researcher-explains-how-agentic-search-performed-better-than-rag-for-code-generation/>
