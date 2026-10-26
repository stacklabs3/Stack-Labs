# Método versionado e regras por projeto

O `modelo-de-projeto` já se instalava em qualquer projeto pelo `/setup-stacklabs`, mas sem versão: o
`ORIGEM.json` guardava só o commit, ninguém sabia se um projeto estava uma mudança ou dez atrás, e
não havia como instalar um estado conhecido. A exceção de cada projeto morava numa seção solta do
`AGENTS.md`, que toda atualização devolvia como `MESCLAR` para mesclagem à mão, e não havia formato
para afastar uma regra do método.

Decidimos: o método tem versão SemVer em `desenvolvimento/VERSAO`, com tag `v<versão>` e entrada em
`desenvolvimento/CHANGELOG.md` a cada PR que muda `desenvolvimento/agentes/`. O instalador grava a
versão no `ORIGEM.json`, mostra a mudança (`VERSÃO`) e aceita `--versao <x.y.z>` para instalar a
tag. O `AGENTS.md` do modelo termina na seção `## Regras deste projeto`, com `### Acrescentadas`
(IDs `RP<n>`) e `### Afastadas` (pelo título da regra, com motivo e link). Ela é do projeto: na
atualização o instalador troca o resto do arquivo pelo modelo novo e mantém a seção (`MESCLADO`).
P5, E1 e as regras dos watchdogs não se afastam; afastar regra **[W]** não desliga o watchdog. Regra
de código continua no `docs/regras-de-codigo.json` do projeto. O que a máquina bloqueia o projeto
ajusta no campo `ajustes` do `.claude/metodo.json` (caminhos livres, branches descartáveis, limites
da smart zone), sem mudar o código dos watchdogs. O agente não abre PR no Stack-Labs para levar uma
regra do projeto ao método: as docs antigas mandavam "propor ao Stack-Labs" e cada projeto acabava
abrindo PR aqui com regra que só servia a ele. Levar algo ao método é decisão do humano.

**Consequências:** editar o `AGENTS.md` fora da seção do projeto se perde na atualização. O
`AGENTS.md` sem a seção (projeto anterior à 1.0.0) ainda cai em `MESCLAR` uma vez; depois da
mesclagem ele tem a seção e as atualizações seguintes são automáticas. Uma alternativa, um arquivo
de regras separado em JSON, foi descartada: as regras do método são texto para o agente, não
verificação de máquina, e a máquina já tem o `metodo.json` e o `regras-de-codigo.json`.
