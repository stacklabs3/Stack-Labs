# O contrato da fatia vem antes do código, e a máquina cobra a regra de código na edição

Os watchdogs, o validador e o `ci` barravam código ruim, mas tarde: a primeira escrita de uma issue
saía quase sempre errada, o `ci` ficava vermelho por muito tempo nela, e a correção, feita achado a
achado pelo plano do diagnosticador, passava a seguir mas já não era mais o que a issue, a regra ou a
ADR pediam. Em 2026-11-03 o Matheus trouxe o problema e pediu uma forma de o agente seguir a
documentação e as regras de código **na primeira execução**. A leitura, pelo `/human-architect-mindset`,
achou três causas: o passo "leia os documentos" do implementador não tinha critério de pronto (ler e
aplicar são coisas diferentes); a primeira régua de código de produção era o validador antes do
commit, dezenas de edições depois da escrita, ao contrário do teste, que o `vigia-teste` devolve na
hora; e nada ancorava a correção ao documento, só ao achado. Havia três caminhos: subir o esforço do
implementador e confiar, pôr o lint inteiro num hook, ou um **compromisso escrito antes do código**
mais a cobrança das regras de forma fixa na edição. O Matheus concordou com o terceiro e com as
quatro escolhas abaixo.

**Consequências:** a regra 24 e o `guarda-edicao` **bloqueiam** o código até a issue ativa ter o
comentário `## Contrato da fatia` (`docs/agents/contrato-da-fatia.md`): o que o diff toca e o
**vizinho** que imita, as seams com os `RK<n>`, as regras de código que o alcançam e como cada uma se
cumpre, os termos do glossário, as ADRs e os critérios de aceite, cada linha com o documento de onde
veio. Bloquear, e não avisar, porque o defeito era justamente pular a etapa; a consulta é uma por
sessão e por subagente, e fica no estado. O contrato mora **na issue**, não em arquivo local, para que
os quatro papéis e o humano o leiam e para que o ciclo de correção o reencontre. Ele é compromisso,
não fonte: o documento vence, e a correção muda uma linha do contrato só pelo passo `contrato:` do
plano do diagnosticador. O hook `vigia-codigo` devolve, a cada edição, a regra de
`docs/regras-de-codigo.json` que o arquivo quebra, só as de forma fixa (`ast-grep`, `termo-evitado`,
`todo-sem-issue`); lint e typecheck ficam no `ci`, pela latência. O implementador do primeiro ciclo
passa a `medium`: o contrato é trabalho de leitura e julgamento, e um ciclo de correção custa mais do
que essa diferença. O `guarda-edicao` confere que o comentário existe, não que o contrato está
completo: isso é do revisor (linha do contrato sem trecho, regra que alcança o diff e falta no
contrato) e do conferente (critério fora do contrato). Medir o efeito com `rodar custo` e com a
contagem de ciclos de correção por issue fica para depois.
