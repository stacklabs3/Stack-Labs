# Evidências sobre verificação formal com agentes: o que se confirma e o que é hype

Pesquisa de 2026-11-25, ticket #71 do mapa #69. Pergunta: o que outros vídeos, artigos, estudos e
relatos da indústria mostram sobre verificação formal escrita ou mantida por agentes de IA, o que de
fato funciona, quais as críticas sérias, quais as alternativas ao TLA+, e quais afirmações do vídeo
`bQCBfcZZj0Y` (transcrição em `transcricao_bQCBfcZZj0Y.md`, raiz do repo) se sustentam.

Método: cada afirmação foi seguida até a fonte que a possui (post do autor, paper, blog de engenharia
da empresa, código). Onde só achei fonte secundária, digo isso. Onde a fonte primária bloqueou a
leitura automática (X/Twitter, CACM), digo o que li no lugar.

## Resultado

1. **O vídeo acerta no essencial e erra no título.** A Anthropic **não** provou a hipótese de
   Riemann; um modelo melhorou um limite inferior (41,6% → 67,2% dos zeros na reta crítica). As 123
   bibliotecas, os números do seL4, o bug de 35 passos do DynamoDB e o relato de Boris Cherny se
   confirmam, cada um com ressalvas de escopo que o vídeo omite.
2. **O que funciona, com fonte primária:** model checking de um **desenho** (TLA+ ou P) pequeno, de
   algumas centenas de linhas, sobre uma parte do sistema sujeita a corrida, falha ou repetição. AWS,
   Datadog, MongoDB e Azure relatam bugs reais achados assim. Agentes já escrevem esses modelos e até
   provas (TLAPS, Rocq, Lean), e encontram bugs de código.
3. **O que não funciona sozinho:** o modelo escrito por LLM sem conferência. Os estudos medem
   ~41–46% de acerto em conformidade com o código e nas invariantes (SysMoBench), 8,6% de acerto
   semântico em geração a partir de linguagem natural, e invariantes tautológicas em specs públicas.
   Os dois riscos que o vídeo cita ("a regra pode estar errada" e "o modelo fica para trás") são os
   mesmos que a literatura aponta, e há bug aberto no pstack que demonstra o primeiro.
4. **Implicação para o mapa #69:** a receita precisa de três defesas que a literatura sustenta: ver
   a invariante falhar (mutação no modelo ou no código), conferir o modelo contra o código (trace
   validation ou revisão ação a ação) e registrar o commit coberto. Isso bate com o desenho do
   `create-formal-verification-skill` do pstack.

## 1. Afirmações do vídeo, uma a uma

| Afirmação do vídeo | Veredito | Fonte |
|---|---|---|
| "Um matemático da Anthropic e Claude provaram a hipótese de Riemann" | **Falso.** Um modelo não lançado melhorou o limite inferior da proporção de zeros na reta crítica de 41,6% para 67,2%. A própria Anthropic escreve: "We don't expect that the techniques Claude used will lead to proving the Riemann hypothesis." Quem disparou foi Jarred Sumner (não matemático); os matemáticos Levent Alpöge e Ralph Furman revisaram; Brian Conrey e Dan Goldston leram o paper. Post de 2026-08-10. | [Anthropic, riemann-zeta](https://www.anthropic.com/research/riemann-zeta) |
| "São dezessete milhões de linhas, e computadores checaram cada passo" | **Não confirmado.** Existe formalização em Lean que passa no validador `comparator`, mas o post não dá contagem de linhas. Os "17 milhões" não aparecem em nenhuma fonte achada. Suspeita: confusão com os ~180 milhões de linhas de Rocq do trabalho das 123 bibliotecas. | idem |
| "Lean é a linguagem em que a prova de Riemann foi publicada" | **Confirmado** para o resultado parcial (formalização em Lean). | idem |
| "A Anthropic fez Claude provar que 123 bibliotecas não travam" | **Confirmado, com escopo estreito.** 239 pontos de entrada de fuzz (OSS-Fuzz) em 123 bibliotecas C/C++: nenhum crash (erro de memória ou assert detectado pelo AddressSanitizer) para entrada < 64 MB. Provas em **Rocq** (não Lean), ~180 milhões de linhas, ~600 mil por prova, sobre o binário **RISC-V**, estático, single-thread, com musl. Achou centenas de bugs, entre eles um underflow num descompressor e a CVE-2026-56132 do libexpat. A própria Anthropic: "'Formally verified' does not mean 'bug-free.'" Feito com a Theorem, em "a few weeks", ao custo de "hundreds of millions of tokens" por prova. Post de 2026-10-22. | [Anthropic, formal-verification](https://www.anthropic.com/research/formal-verification) |
| "seL4: provar cerca de 8.700 linhas de C levou cerca de vinte pessoas-ano" | **Confirmado.** O paper do SOSP 2009: "8,700 lines of C code and 600 lines of assembler"; "The cost of the proof is higher, in total about 20 py. This includes ... about 9 py invested in formal language frameworks, proof tools ... The total effort for the seL4-specific proof was 11 py." Ressalvas: 7.500 linhas de C são verificadas (boot e assembly ficam de fora); o projeto inteiro (desenho + código + prova) deu ~25 pessoas-ano; os autores estimam 6 pessoas-ano para refazer a prova de um kernel novo. | [Klein et al., SOSP 2009 (PDF)](https://trustworthy.systems/publications/papers/Klein_EHACDEEKNSTW_09.pdf); [números do l4.verified](https://trustworthy.systems/projects/OLD/l4.verified/numbers) |
| "A Amazon relatou em 2015 que um check desses achou um bug de desenho no DynamoDB a 35 passos de profundidade" | **Confirmado.** "This was a very subtle bug; the shortest error trace exhibiting the bug contained 35 high-level steps." O bug podia perder dados numa sequência de falha e recuperação. O rascunho de 2014 lista o modelo: 939 linhas de TLA+, 3 bugs. | [Newcombe et al., CACM 2015](https://cacm.acm.org/magazines/2015/4/184701-how-amazon-web-services-uses-formal-methods/fulltext); [trecho abreviado, site do Lamport](https://lamport.azurewebsites.net/tla/amazon-excerpt.html) |
| Boris Cherny usou TLA+ e Lean e disse "não conheço bem nenhuma das duas, mas Claude é excelente nas duas" | **Confirmado em substância.** Post de 2026-09-22: "I used Opus 5.5 to formally verify the Claude Agent SDK using Lean. A couple short prompts = 16 PRs fixing various bugs and race conditions. ... TLA+ also works well. I sometimes combine Lean and TLA+ to look for issues around data flow, concurrency, and state mgmt." No post seguinte ele descreve o laço (modelar a parte com corrida, achar contraexemplo, reproduzir no código, corrigir) e avisa que o código todo **não** está verificado, só "the hairiest parts". O X bloqueou a leitura direta; li o texto pelo título indexado do post e pelo post de Hillel Wayne, que cita o episódio. A frase exata "Claude is excellent at both" não foi conferida palavra a palavra. | [post 1](https://x.com/bcherny/status/2102543349102338309); [post 2](https://x.com/bcherny/status/2102898067133595992); [Hillel Wayne, 2026-09-30](https://buttondown.com/hillelwayne/archive/what-tla-can-and-cant-check/) |
| Boris "landed 259 PRs in a month, all written by Claude" | **Confirmado como autorrelato** (dez/2025: 259 PRs e 497 commits em 30 dias, "every single line" pelo Claude Code com Opus 4.5). | [Simon Willison citando o post, 2025-12-27](https://simonwillison.net/2025/Dec/27/boris-cherny/) |
| Time de Lauren Tan na xAI: 11 mil PRs, bots fazem merge dos próprios fixes | **Só fonte secundária** (resumos da palestra no AI Engineer CODE 2026: ~11 mil PRs/mês, 0,5% revertidos). Não achei o vídeo da palestra nem post dela com o número. | [resumo TeamDay](https://www.teamday.ai/ai/lauren-tan-spacexsi-grok-bot-11000-agent-prs-lint-rules) |
| "Matt Pocock chama provas de essenciais" | **Não achado.** Nenhuma fonte liga Matt Pocock a verificação formal. | — |
| A skill `create-formal-verification-skill` do pstack e o bug "a compra nunca pode ser paga, então nunca é paga duas vezes" | **Confirmado.** A skill está em `cursor/plugins`, pasta `pstack/skills/`, e exige "Able to fail. A check that you have never seen fail proves nothing." O bug é a issue aberta [cursor/plugins#1235](https://github.com/cursor/plugins/issues/1235): a ação `BankAccepts` faz `paid' = TRUE` e também `UNCHANGED paid`, nunca dispara, e o `statechart` relata `NeverPaidTwice` como "Rules that hold" sem mostrar o aviso do TLC. | [SKILL.md](https://github.com/cursor/plugins/blob/main/pstack/skills/create-formal-verification-skill/SKILL.md) |
| "Learn TLA+ diz: TLA+ testa desenhos, não código" | **Confirmado em substância.** learntla.com: "a means of designing systems that lets you directly test those designs." | [learntla.com](https://learntla.com/) |
| Caso crashlens; frase de Lauren "already found several bugs" | **Não conferido.** Fora do tempo desta pesquisa. | — |

## 2. Casos da indústria (fonte primária)

| Empresa e sistema | Ferramenta | O que achou | Custo relatado | Fonte |
|---|---|---|---|---|
| AWS: DynamoDB, S3 e outros 10–14 sistemas (2011–2014) | TLA+ e PlusCal com TLC | Bug de perda de dados a 35 passos no DynamoDB; outros dois bugs sérios em outros algoritmos | Engenheiros "from entry level to principal" com resultado útil em "two to three weeks"; "managers now allocate engineering time to TLA+" | [CACM 2015](https://cacm.acm.org/magazines/2015/4/184701-how-amazon-web-services-uses-formal-methods/fulltext); [trecho](https://lamport.azurewebsites.net/tla/amazon-excerpt.html) |
| AWS hoje: S3 (consistência forte), EBS, DynamoDB, Aurora, MemoryDB, EC2 | **P** (desde 2019), PObserve (logs contra a spec, em teste e produção), métodos leves (property-based no ShardStore do S3), simulação determinística | Modelos dos protocolos da consistência forte do S3 | — (artigo bloqueou leitura; dados das páginas do P) | [Brooker e Desai, CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services); [P](https://p-org.github.io/P/) |
| Microsoft: Azure Cosmos DB | TLA+ "leve" da semântica vista pelo cliente | Dois problemas na documentação pública dos níveis de consistência, corrigidos; causa raiz de um incidente de 28 dias (Cosmos DB + Service Bus) | — | [Hackett, Rowe, Kuppe, arXiv 2210.13661](https://arxiv.org/abs/2210.13661) |
| Microsoft: Azure Storage | LLM deriva spec TLA+ do código existente | Um bug "potencial" de perda de dados (só slides, sem detalhe) | — | relatado por [Hillel Wayne, 2025-06-05](https://buttondown.com/hillelwayne/archive/ai-is-a-gamechanger-for-tla-users); ferramenta [lamport-agent](https://zfhuang99.github.io/github%20copilot/formal%20verification/tla+/2025/11/14/lamport-agent.html) |
| MongoDB: replicação do servidor | Trace checking (logs contra `RaftMongo.tla`) | **Fracassou por custo**: "It took us a month to figure out how to instrument MongoDB"; "the marginal cost ... wasn't worth the marginal value" | Orçamento do experimento esgotado | [Davis, blog MongoDB 2025](https://mongodb.com/company/blog/engineering/conformance-checking-at-mongodb-testing-our-code-matches-our-tla-specs); [VLDB 2020, arXiv 2006.00915](https://arxiv.org/abs/2006.00915) |
| MongoDB: Mobile SDK (transformação operacional) | TLA+ gerando casos de teste | Bug real de recursão infinita; 4.913 testes gerados com 100% de cobertura de ramos (testes à mão: 21%; AFL: 92%) | 21 regras (~1.000 linhas C++) traduzidas em ~2 semanas | idem |
| MongoDB: reconfiguração do Raft | TLA+ e prova TLAPS | Protocolo novo prototipado e provado | — | [blog MongoDB](https://mongodb.com/blog/post/technical/rapid-prototyping-safe-logless-reconfiguration-protocol-mongodb-tla-plus); [arXiv 2109.11987](https://arxiv.org/abs/2109.11987) |
| Datadog: Courier (fila de mensagens) | TLA+ (TLC, ~5,5 milhões de estados), SimPy, chaos testing | Violação de `NoLostMsgs` depois da mudança do sequenciador; revisar o código contra o modelo "found several bugs" de concorrência otimista do FoundationDB | Mantém o modelo só para mudanças grandes de desenho, porque mantê-lo "is costly" | [Datadog, 2024-11-20](https://www.datadoghq.com/blog/engineering/formal-modeling-and-simulation) |
| Datadog: Husky | **FETCh**: TLA+ vira monitor em Rust que lê a telemetria de produção | Colocações conflitantes (< 0,01%) que derrubariam o backfill numa recuperação | ~2% de CPU p90 e < 0,2% de memória em 1.300+ pods | [Datadog, 2026-11-05](https://www.datadoghq.com/blog/engineering/continuous-formal-spec-validation) |

Lição comum: o valor vem de **um modelo pequeno sobre a parte difícil** e de **uma ligação
explícita entre modelo e código** (revisão ação a ação, testes gerados do modelo, ou logs checados
contra a spec). Quem tentou a ligação pesada (trace checking do servidor MongoDB) pagou caro.

## 3. Agentes escrevendo verificação formal: estudos

| Fonte | O que mostra |
|---|---|
| [SysMoBench (arXiv 2509.23130; ICLR 2026)](https://arxiv.org/abs/2509.23130) e [post SIGOPS, 2026-05-08](https://sigops.org/2026/can-llms-model-real-world-systems-in-tla) | 11 sistemas reais (Raft do etcd e do Redis, eleição do ZooKeeper, locks do Asterinas). Sintaxe ~100%; execução no TLC 30–92%; **conformidade com o código ~46% e invariantes ~41%** em média; 10–50% nos sistemas distribuídos difíceis. Os modelos saem "structurally complete, type-correct" a partir de padrões de livro, não do código. Lição: TLC rodar não prova que o modelo descreve o sistema. |
| [Specula (arXiv 2607.25333)](https://www.alphaxiv.org/abs/2607.25333) | Agente que escreve TLA+ a partir do código e se corrige contra ele: 249 bugs em 48 projetos (68 confirmados, 24 corrigidos no paper). Em 5 sistemas: Specula com Opus 4.8 achou 62 bugs; Claude Code sozinho, 2; Claude Code com TLA+, 3. Custo mediano US$ 57 e 3,7 h por projeto. Com Haiku 4.5 achou 0. Venceu o desafio GenAI da [TLA+ Foundation](https://foundation.tlapl.us/challenge/). |
| [Bisharat et al., arXiv 2606.05792 (ICSOFT 2026)](https://arxiv.org/abs/2606.05792) | 30 LLMs, 205 specs a partir de linguagem natural: até 26,6% sintaticamente corretas e **8,6% semanticamente corretas**. Conclusão: sem supervisão de especialista, não são confiáveis. |
| [William Schultz, 2026-04-03](https://will62794.github.io/formal-methods/2026/04/03/autonomous-protocol-proofs.html) | Claude Code (Opus 4.6) provou em TLAPS os 12 lemas de uma invariante indutiva do Raft (2.559 obrigações) em ~3h45, quase sem ajuda; antes levava semanas a meses de um aluno de doutorado. Ressalvas dele: o Raft está muito documentado; ele prefere model checking finito na prática. |
| [Anthropic, 123 bibliotecas](https://www.anthropic.com/research/formal-verification) | Prova em escala industrial sobre o binário, com bugs reais achados. A especificação é derivada do teste de fuzz existente, por isso a "regra" já vinha pronta. |
| [Boris Cherny](https://x.com/bcherny/status/2102543349102338309) | Relato de praticante: Lean e TLA+ nas partes com corrida do Agent SDK, 16 PRs. Sem números independentes. |

## 4. Críticas sérias

1. **"Prova a spec, não o código."** Hillel Wayne: "correct designs don't automatically translate
   into correct code" e "formal methods will solve the problem of agentic software development once
   and for all, and that's nonsense" ([2026-09-30](https://buttondown.com/hillelwayne/archive/what-tla-can-and-cant-check/)).
   O mesmo post lista o que TLA+ **não expressa** nativamente: propriedades de dois passos (desfazer
   restaura o estado), tempo real e ponto flutuante, alcançabilidade ("existe um caminho em que P"),
   hiperpropriedades (muitas de segurança) e metapropriedades.
2. **LLM escreve invariante fraca ou tautológica.** Hillel, sobre specs públicas feitas por IA:
   "These are tautologically true!", "So we know the person did not actually run these specs", e "the
   actual value in using formal methods comes from the subtle properties" ([2026-03-10](https://buttondown.com/hillelwayne/archive/llms-are-bad-at-vibing-specifications)).
   Cerca de 4% das specs TLA+ no GitHub já citam Claude. Ele e um cliente não conseguiram que Claude
   escrevesse boa propriedade de liveness.
3. **LLM "conserta" o modelo afrouxando a regra.** Em 2025 Claude propôs deixar a spec permissiva em
   vez de corrigir o bug: "if you say bugs are okay, then the spec finds that bugs are okay!" ([2025-06-05](https://buttondown.com/hillelwayne/archive/ai-is-a-gamechanger-for-tla-users)).
4. **Modelo que nunca falha.** [cursor/plugins#1235](https://github.com/cursor/plugins/issues/1235):
   ação que nunca dispara deixa a invariante "verdadeira" por vacuidade, e o relatório esconde o aviso
   do TLC. É o caso citado no vídeo.
5. **O modelo fica para trás do código.** Datadog só atualiza o modelo em mudança grande de desenho,
   porque é caro. A validação de traces ([Cirstea, Kuppe, Loillier, Merz, SEFM 2024](https://arxiv.org/abs/2404.16075))
   achou divergência entre spec e implementação **em todos** os programas testados. O FETCh da
   Datadog e o PObserve da AWS existem para fechar esse buraco em produção.
6. **"Verificado" tem escopo.** A própria Anthropic: "The claim may not cover everything we care
   about." A prova das 123 bibliotecas vale para RISC-V e musl; a maioria roda x86 ou ARM com glibc.
7. **Adoção.** Hillel, no Pragmatic Engineer, estima que a IA leve o uso de 0,1% para 0,3% da
   indústria. Martin Kleppmann prevê o contrário, que a IA tornará a verificação formal mainstream e
   que o limite é cultural ([2025-12-08](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html)).

## 5. Alternativas ao TLA+ (com TLC) para o mesmo problema

| Ferramenta | O que é | Força | Custo e limite | Fonte |
|---|---|---|---|---|
| **PlusCal** | Linguagem de algoritmo com cara de pseudocódigo que o tradutor converte em TLA+, checada pelo TLC | Mais fácil de ler para quem programa; mesmo checker e ecossistema; foi o que a AWS usou em 2015 | Ainda precisa ler a TLA+ gerada em erros; menos flexível para modelar alto nível | [Lamport, PlusCal User's Manual 1.8](https://lamport.azurewebsites.net/tla/p-manual.pdf) |
| **Apalache** | Model checker **simbólico** de TLA+ via SMT (Z3, CVC5) | Checa invariante indutiva e execuções limitadas sem enumerar estados; bom para espaço de estados enorme | Exige anotação de tipo; checagem limitada (bounded); mais lento por estado; hoje mantido pela TLA+ Foundation | [apalache-mc/apalache](https://github.com/apalache-mc/apalache) |
| **Quint** | Sintaxe moderna (estilo TypeScript) sobre a lógica do TLA+, com checador de tipos, REPL e simulador | Mais legível para devs e agentes; `quint run` (simulação aleatória rápida), `quint verify` com Apalache ou `--backend=tlc`; modo de teste baseado em modelo (`--mbt`, crate `quint-connect` para Rust) | Simulador não dá garantia; suporte parcial a propriedades temporais; comunidade menor | [quint.sh](https://quint.sh/docs/what-does-quint-do); [repo](https://github.com/quint-co/quint) |
| **P** (AWS) | Linguagem de máquinas de estado que trocam eventos, com P-Checker, PObserve e PVerifier | Parece código; usado no S3, DynamoDB, EBS; **PObserve** checa logs de produção contra a spec; **PeasyAI** gera modelos com IA e integra com Claude Code via MCP | Modelo é mais próximo de implementação (menos abstrato); exige .NET; foco em sistemas por mensagens | [p-org.github.io/P](https://p-org.github.io/P/); [Brooker e Desai](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services) |

Leitura para o método: TLA+ com TLC continua o padrão mais documentado e é o padrão do pstack
("TLA+ is the default for that in any language"). Quint é a alternativa mais forte quando a leitura
humana e o teste gerado do modelo pesam; P quando o sistema é de mensagens e se quer checar logs. O
mapa #69 já decidiu "só TLA+"; nada aqui obriga a rever, mas Quint merece nota no ADR.

## 6. Fontes que mais valem estudo

1. [Newcombe et al., "How AWS Uses Formal Methods" (CACM 2015)](https://cacm.acm.org/magazines/2015/4/184701-how-amazon-web-services-uses-formal-methods/fulltext): o caso fundador, com custo de aprendizado e o bug de 35 passos.
2. [Brooker e Desai, "Systems Correctness Practices at AWS" (CACM 2025)](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services): como a AWS combina TLA+, P, PObserve, métodos leves e simulação dez anos depois.
3. [Datadog, "Continuous formal spec validation" (2026)](https://www.datadoghq.com/blog/engineering/continuous-formal-spec-validation) e [Courier (2024)](https://www.datadoghq.com/blog/engineering/formal-modeling-and-simulation): como evitar que o modelo fique para trás, com custo medido.
4. [MongoDB, "Conformance checking at MongoDB" (2025)](https://mongodb.com/company/blog/engineering/conformance-checking-at-mongodb-testing-our-code-matches-our-tla-specs): o que deu errado (trace checking) e o que deu certo (testes gerados do modelo).
5. [SysMoBench, post SIGOPS (2026)](https://sigops.org/2026/can-llms-model-real-world-systems-in-tla) e [Specula](https://www.alphaxiv.org/abs/2607.25333): medição de quão bem LLMs modelam sistemas reais e como um agente que se confere contra o código melhora isso.
6. [Hillel Wayne, "LLMs are bad at vibing specifications"](https://buttondown.com/hillelwayne/archive/llms-are-bad-at-vibing-specifications) e ["What TLA+ can and can't check"](https://buttondown.com/hillelwayne/archive/what-tla-can-and-cant-check/): a crítica mais concreta, com exemplos de invariante tautológica.
7. [Anthropic, "LLMs are making formal software verification feasible at a much larger scale"](https://www.anthropic.com/research/formal-verification): o teto atual, com escopo e base de confiança explícitos.
8. [`create-formal-verification-skill` do pstack](https://github.com/cursor/plugins/blob/main/pstack/skills/create-formal-verification-skill/SKILL.md) com [a issue #1235](https://github.com/cursor/plugins/issues/1235): a skill que o mapa avalia e o defeito de vacuidade que ela ainda tem.

## 7. Veredito: demonstrado e hype

**Demonstrado com fonte primária:**
- Model checking de desenho acha bugs profundos de concorrência e falha que teste não acha (AWS,
  Datadog, MongoDB, Azure).
- Agentes atuais escrevem TLA+ sintaticamente correto quase sempre, provam lemas TLAPS de protocolo
  conhecido e escrevem provas Rocq e Lean em escala, achando bugs reais (Anthropic, Schultz, Cherny).
- O custo de aprender e escrever caiu: o gargalo passou a ser **ler e conferir** a invariante e o
  modelo.

**Hype ou exagero:**
- "Claude provou a hipótese de Riemann" (falso) e "17 milhões de linhas" (sem fonte).
- "A IA pode provar que seu código funciona em todos os casos": TLA+ checa o modelo, dentro de
  limites, e só a invariante escrita; não o código, nem o que não foi escrito.
- Modelo escrito pelo agente sem conferência: cerca de metade não descreve o código real, e
  invariantes saem tautológicas.

**O que a receita do método precisa ter, se adotada (insumo para #72–#75):**
1. Ver cada invariante falhar num código ou modelo quebrado de propósito, e checar que toda ação do
   modelo dispara (contra o defeito da #1235).
2. Conferir o modelo contra o código ação a ação, citando arquivo e função, antes de confiar no
   "passou".
3. Registrar commit e arquivos cobertos, e reconferir quando eles mudam.
4. Leitura humana da invariante e da lista do que ela deixa de fora.
5. Escopo pequeno: uma promessa de risco alto por vez, modelo de algumas centenas de linhas.
