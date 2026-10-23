---
name: voltar-a-principal
description: Fecha esta sessão de trabalho atualizando o HANDOFF do projeto com o que foi feito, valida o trabalho e o handoff, e entrega uma mensagem de copiar e colar para levar de volta à sessão principal.
argument-hint: "(opcional) contexto extra pra destacar pra sessão principal"
disable-model-invocation: true
---

# Voltar à sessão principal

Esta sessão é satélite de uma sessão principal que continua aberta em outro lugar. Feche o
trabalho documentando o que foi feito, não o que falta explicar — quem lê depois é a
principal (já conhece o projeto) e, eventualmente, uma sessão nova neste tópico especifico
(não conhece nada além do arquivo). Execute os três passos abaixo na mesma resposta.

Se o usuário passou argumento, trate como contexto extra (um aviso, uma ressalva, uma decisão
pendente) e incorpore no resumo do Passo 3.

## Passo 1 — Atualizar o documento HANDOFF

Identifique o projeto/tópico desta sessão pelo que foi trabalhado (ex.: tower-defense,
trilha-arquitetura-docusaurus). Se ambíguo, pergunte em uma linha antes de continuar.

Procure `HANDOFF-<projeto>.md` na raiz do repo atual:

- **Existe** → atualize as seções afetadas no próprio arquivo. Não crie um segundo arquivo
  nem duplique seção já presente — a versão mais recente de uma decisão substitui a anterior.
- **Não existe** → crie seguindo a mesma estrutura de outros handoffs do repo, se houver
  (título + 1 frase de contexto, "Implementado", "Decisões fechadas (não reabrir)",
  "Pendências/próximos passos", "Notas"). Sem exemplo local, use essas seções como padrão.

Aponte pra outros artefatos (designs, ADRs, planos) por caminho em vez de duplicar o
conteúdo deles. Redija qualquer informação sensível (chaves, senhas, dados pessoais) antes
de salvar.

Pronto quando: o arquivo no disco reflete o estado real do que foi implementado e decidido
nesta sessão especificamente — decisão revisada no meio do caminho entra só na forma final.

## Passo 2 — Validação final (caça a lacunas)

Valide em duas frentes e reporte o resultado em bullets curtos — "✅ nada ficou de fora"
ou a lista do que faltava:

**O trabalho da sessão** — releia a conversa comparando com o estado real no disco:

- O que a conversa diz que foi feito existe mesmo (arquivo salvo, teste rodado, edição aplicada)?
- Cada pedido do usuário foi atendido ou está registrado como pendência?
- Decisões revisadas no caminho: só a forma final sobreviveu?

**O HANDOFF** — releia o arquivo salvo no disco como se você fosse a sessão nova:

- Sustenta a retomada sozinho? Os caminhos que cita existem?
- O que afirma bate com o que a frente anterior confirmou?

Lacuna encontrada tem destino, não só relato: se é registro, corrija o HANDOFF agora; se a
principal precisa saber, entra na mensagem do Passo 3.

Pronto quando: as duas frentes reportadas e nenhuma lacuna sem destino.

## Passo 3 — Mensagem de retorno

Gere uma mensagem pronta pra colar na sessão principal:

- 3-6 linhas explicando o que foi feito nesta sessão (o delta, não o histórico completo —
  a principal já tem o contexto do projeto).
- O caminho do HANDOFF atualizado/criado, como ponteiro pro detalhe.
- Qualquer pendência ou decisão que a principal precisa saber pra não travar depois.

Template (adapte; corte o que não se aplica):

```
Voltando da sessão de [tópico] pra principal.

O que foi feito: [resumo 2-4 linhas]

Handoff em: caminho/HANDOFF-projeto.md
```

Entregue a mensagem dentro de um único bloco de código, como último elemento da resposta,
nada depois dele — o usuário copia com um gesto. Se a mensagem contiver blocos de código
internos, use cerca de 4 crases no bloco externo.
