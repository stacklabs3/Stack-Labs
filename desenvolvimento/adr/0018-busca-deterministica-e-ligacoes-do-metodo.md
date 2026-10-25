# Contexto do agente por busca determinística e pelas ligações do método, sem grafo feito por modelo

O agente gasta boa parte de cada issue procurando contexto, e a ADR 0017 passou a medir esse gasto.
A pesquisa `desenvolvimento/research/2026-10-25-busca-deterministica-para-contexto.md` comparou
quatro caminhos: busca lexical e grafo de código feitos por parser (probe, codanna, repo map do
Aider), embeddings, grafo de entidades extraído por modelo (GraphRAG, LightRAG) e memória escrita
pelo modelo (A-MEM, servidor MCP de memória). O grafo feito por parser tem o melhor apoio
(RepoGraph, LocAgent), desde que entre por consulta e não despejado no prompt; o grafo por modelo
custa uma chamada por trecho e só ganha em perguntas globais sobre texto; a memória escrita pelo
modelo envelhece, que é o motivo de o próprio Claude Code ter trocado o índice vetorial por glob e
grep. O método já é um segundo cérebro curado por gente (glossário, ADRs, mapas, `RK<n>`); falta
ligar as peças por máquina. O humano escolheu, em 2026-10-25, seguir a ordem da pesquisa, cada passo
condicionado à medição do anterior.

**Consequências:** a ordem é (1) linha de base com `rodar custo` e gabarito pelos diffs das PRs
mergeadas; (2) probe e codanna como CLI chamada por skill ou subagente, adotados só se a medição num
projeto com código de produção mostrar ganho, porque no Stack-Labs, quase só Markdown em português,
o probe acertou 0,34 contra 0,64 do `git grep` (seção 7 da pesquisa); (3) as **ligações do
método** no motor Rust: ADR, `RK<n>`, issue e termo do glossário ligados a arquivo e símbolo, tirados
de IDs citados, do git e do tree-sitter, refeitos a cada execução com cache por hash, servindo à
consulta do agente e a um watchdog de ligação quebrada; (4) repo map próprio só se faltar grafo
ranqueado; (5) dossiê do Haiku como subagente com `model: haiku` e conferência das citações em
Rust, só se a etapa 2 mostrar economia. Ficam de fora: grafo de entidades por modelo, memória escrita
pelo modelo como fonte de verdade, servidor MCP e índice BM25 próprios, embeddings antes de a
medição mostrar falha da busca lexical, Haiku em hook de todo prompt e o dossiê como única fonte de
contexto. Cada passo que entrar no método ganha a sua ADR com a medição que o justificou.
