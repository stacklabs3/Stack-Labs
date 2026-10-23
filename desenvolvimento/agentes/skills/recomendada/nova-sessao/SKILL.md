---
name: nova-sessao
description: Fecha o planejamento da sessão atual e prepara a execução em uma sessão nova. Faz a validação final da conversa (confirma que não deixamos nada de fora do plano), analisa o que a tarefa pede, seleciona as skills disponíveis necessárias para executá-la e entrega uma mensagem autossuficiente de copiar e colar que dá início à tarefa na nova sessão. Use sempre que o usuário pedir "validação final", "confirmar que não deixamos nada de fora", uma "mensagem de copiar e colar" para "darmos início a essa tarefa em uma nova sessão", pedir para "selecionar as skills necessárias" para executar uma tarefa, falar em handoff/kickoff de sessão, ou quando o contexto da sessão atual passar de ~50-60% e for hora de destilar e recomeçar (regra de ouro do handoff).
argument-hint: "(opcional) qual tarefa a nova sessão vai executar"
---

# Nova sessão — validação final + kickoff

Este fluxo fecha uma sessão de planejamento e abre caminho para uma sessão de execução limpa.
A sessão nova nasce com ZERO memória desta conversa — tudo o que não estiver na mensagem final
(ou em arquivo referenciado por ela) deixa de existir. Esse é o critério que governa os três
passos abaixo. Execute os três na mesma resposta, nesta ordem, todos visíveis ao usuário.
Se o usuário passou argumentos, eles dizem qual tarefa a nova sessão vai executar — foque
a validação e a mensagem nessa tarefa.

## Passo 1 — Validação final (caça a lacunas)

Releia a conversa inteira comparando com o plano/tarefa na sua forma final. Procure:

- Pedidos ou requisitos mencionados no caminho e ausentes do plano final
- Decisões revisadas depois — vale sempre a versão final; descarte as anteriores
- Pontas soltas: perguntas sem resposta, "depois a gente vê", TODOs implícitos
- Arquivos, paths, comandos e artefatos citados que a nova sessão precisará conhecer
- Critério de aceite: dá para saber objetivamente quando a tarefa termina?

Reporte em bullets curtos: "✅ nada ficou de fora" ou a lista do que faltava — e incorpore
cada item encontrado à mensagem do Passo 3. Se um item encontrado depende de um parâmetro
que nunca foi decidido na conversa, não deixe o buraco viajar para a nova sessão: proponha
um default concreto na mensagem e avise o usuário para ajustar antes de copiar.

## Passo 2 — Análise da tarefa + seleção de skills

Resuma em 2-4 linhas o que a tarefa pede. Isso aparece aqui, na sessão atual, para o usuário
validar o entendimento antes de levar a tarefa adiante.

Depois selecione as skills necessárias para EXECUTAR a tarefa, escolhendo apenas entre as
skills realmente disponíveis no ambiente (listadas no system prompt) — nunca invente nome de
skill nem recomende uma que não está instalada. Para cada selecionada, 1 linha: por que ela
entra. Se nenhuma se aplica, diga isso explicitamente em vez de forçar uma escolha.

## Passo 3 — Mensagem de kickoff (o entregável)

Regras da mensagem:

- Autossuficiente: prefira paths absolutos ou relativos à raiz do repo (nomeando o repo),
  decisões com racional, zero referências do tipo "como vimos acima"
- Decisões fechadas viajam com o porquê — é isso que impede a sessão nova de reabri-las
- A mensagem manda EXECUTAR, não re-planejar — sessão nova recebendo plano tende a
  re-planejar; corte isso na primeira linha
- Se o plano já vive em um arquivo (plano, ADR, doc de handoff), referencie o arquivo em
  "Leia primeiro" em vez de duplicar o conteúdo; o arquivo é a fonte da verdade
- Se o contexto for grande demais para caber numa mensagem, destile antes um doc de handoff
  no repo e faça a mensagem apontar para ele
- Skills entram como invocações `/nome` logo na primeira linha, para a sessão nova
  carregá-las antes de trabalhar
- A mensagem é uma foto com data, e pode ser colada horas ou dias depois, quando outra
  sessão já fez o trabalho. Por isso ela sempre abre com um **Pré-voo**: a data e a hora
  em que foi gerada, as premissas que tornam a tarefa necessária (ex.: "a #467 está aberta
  e sem resolução", "ainda não existe PRD do mapa #463", "a branch X não foi mergeada") e,
  para cada premissa, o comando que a confere. Inclua também uma checagem de trabalho
  *posterior* à premissa (ex.: issues criadas depois da data, PRD que cite o mapa, PR
  aberta), porque o próximo passo da cadeia pode já ter andado.
  Se alguma premissa falhar, a sessão para antes de qualquer escrita, relata o estado
  encontrado e pergunta. Nenhuma "primeira escrita obrigatória" (claim, branch, commit)
  vem antes do pré-voo; ela vem logo depois dele
- No Passo 1, confira as premissas você mesmo no momento de gerar a mensagem (rode os
  comandos). Não entregue um kickoff cuja premissa já é falsa agora
- Entregue a mensagem DENTRO de um único bloco de código, como último elemento da resposta,
  nada depois dele — o usuário copia com um gesto. Se a mensagem contiver blocos de código
  internos, use cerca de 4 crases no bloco externo.

Template (adapte; corte seções que não se aplicam):

```
Invoque as skills /skill-a e /skill-b e execute a tarefa abaixo. O planejamento já foi
feito e validado — não re-planejar, não reabrir decisões.

## Pré-voo (antes de qualquer escrita)
Mensagem gerada em [AAAA-MM-DD HH:MM]. Confira que ela ainda vale:
- [premissa] — `[comando que confere]`
- Nada posterior já fez isto — `[comando: itens criados depois da data / PR / PRD]`
Se qualquer premissa falhar: não escreva nada, relate o estado encontrado e pergunte.

## Contexto
[2-4 frases: projeto, diretório de trabalho, momento atual, o que já existe]

## Leia primeiro
- caminho/do/arquivo.md — [o que contém e por que importa]

## Tarefa
[objetivo claro e verificável]

## Decisões já tomadas (não re-discutir)
- [decisão] — porque [racional curto]

## Escopo
1. [passo]
2. [passo]

## Fora do escopo
- [o que foi decidido NÃO fazer agora]

## Critérios de aceite
- [condição objetivamente verificável]
```
