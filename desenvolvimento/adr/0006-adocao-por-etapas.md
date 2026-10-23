# Projeto que já existia adota o método por etapas, com os watchdogs ligados aos poucos

Instalar o método inteiro num projeto em execução trava o time no mesmo dia: o `guarda-edicao` barra
código sem issue ativa, o `guarda-bash` só ativa issue com os mapas iniciais fechados, e o CI reprova
as branches em andamento. Havia três caminhos: abrir exceções nos watchdogs para projetos antigos,
adotar só as regras escritas sem os watchdogs, ou ligar os watchdogs em etapas. Escolhemos as
**etapas** (`observacao`, `planejamento`, `completo`), gravadas em `.claude/metodo.json`: as regras
são as mesmas desde o primeiro dia, e só o bloqueio por máquina chega aos poucos, quando o projeto já
tem o que ele cobra.

**Consequências:** a adoção é a skill `/adotar-metodo-stack-labs`, separada do `/setup-stacklabs`
(que fica para projeto novo e atualização). Os mapas iniciais de um projeto antigo são
**retroativos**: registram as decisões já tomadas e fecham na hora quando nada está em aberto, sem
exceção no portão dos watchdogs. As issues abertas passam por `/triage` antes de entrar na
orquestração. Branches e PRs anteriores à data de adoção terminam fora das regras de branch e de
issue. Mudar de etapa é uma PR do humano, com a label `altera-watchdogs`.
