# Atualização do método sem intervenção humana

Atualizar um projeto exigia várias ações de uma pessoa: rodar `--atualizar-watchdogs`, gravar o
`settings.json`, ativar o pre-push, aplicar a label `altera-watchdogs` e fazer o merge. O motivo era a
regra 21: o agente não altera os watchdogs que o vigiam. Mas numa atualização o agente não escolhe o
conteúdo, só o traz da fonte oficial, e o custo de cada projeto ficar versões atrás era maior que o
risco que essas etapas cobriam.

Decidimos: `/setup-stacklabs` é o único gesto humano numa atualização. O instalador atualiza os
watchdogs por padrão e mescla `.gitignore` e `settings.json`. A cada invocação, as skills de máquina
se conferem com a fonte oficial (`--conferir-skill`, por injeção de comando no `SKILL.md`, antes de o
texto chegar ao modelo). O `--concluir-pr` substitui o aval humano por uma prova: instalar de novo
por cima do commit não muda nada, todo arquivo da PR vem do instalador, a seção do projeto e o
restante do `settings.json` não mudaram e a verificação passa. Só então aplica a label e, com o CI
verde, faz o merge. Fora disso, a PR fica para uma pessoa.

A regra 21 continua valendo para todo o resto: o agente não edita a pasta protegida, não escolhe a
fonte (`STACKLABS_ORIGEM`, `MATT_ORIGEM`), não volta de versão (`--permitir-regressao`) nem aplica a
label direto. Fonte que não compila não entra, porque um watchdog quebrado trava o agente em todo
hook sem ele poder consertar.

**Consequências:** nas PRs de atualização, a label `altera-watchdogs` deixa de ser aval de uma pessoa
e a conferência roda na máquina do agente: um agente decidido a contornar o método poderia forjá-la,
limite que os watchdogs já admitem. Com o Stack-Labs privado, o CI do projeto não consegue repetir a
comparação; e sem proteção de branch no plano atual, CI vermelho não barra merge, por isso o
`--concluir-pr` espera o verde. Um defeito publicado no Stack-Labs chega aos projetos sem revisão
humana em cada um, então a revisão da PR no Stack-Labs passa a ser a barreira. Instalação nova, adoção
e mudança de etapa continuam com o merge humano. Levar um token ao CI de cada projeto foi descartado
por exigir configuração humana por repositório.
