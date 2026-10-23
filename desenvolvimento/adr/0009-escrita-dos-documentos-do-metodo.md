# Os documentos do método se escrevem como skill: fonte única e divulgação progressiva

O método cresceu em pares (um guia para pessoas e um doc para agentes sobre o mesmo assunto), e a
mesma explicação passou a existir em dois, três lugares: o glossário, o guia humano, o doc do agente
e a skill. Arquivos como o dos mapas iniciais e o dos princípios de código juntavam num só o que cada
sessão só usa em parte. Isso custa duas vezes: o agente carrega contexto que não usa (e a constituição
está carregada em todo turno), e mudar um comportamento exige achar e editar todas as cópias.
Havia a opção de aceitar a repetição como reforço ou de manter um só documento por assunto para os
dois públicos. Escolhemos escrever todo documento do método com a régua do `/writing-for-agents` do
Matt Pocock (antes chamado `writing-great-skills`): cada significado tem um dono, os outros apontam
para ele, e o que só um ramo usa vai para um arquivo irmão atrás de um ponteiro que diz quando ler.

**Consequências:** a tabela de donos fica em [agentes/README.md](../agentes/README.md#mudar-o-método):
o glossário define, as ADRs explicam o porquê, a constituição traz uma linha por regra, os docs de
`docs/agents/` têm a mecânica, os guias humanos dizem o que a pessoa decide e as skills, o
procedimento. Os arquivos grandes viraram índice mais arquivos por ramo, no mesmo caminho, para não
quebrar hooks nem ponteiros: `docs/agents/mapas-iniciais/` (um por mapa) e
`docs/agents/principios-de-codigo/` (um por grupo e o roteiro do grilling). Toda mudança no método,
e todo documento que um agente escreve num projeto para outro agente ler (`docs/regras-de-codigo.md`,
uma exceção no `AGENTS.md`), passa pelo `/writing-for-agents`. As ADRs são registro: não se
reescrevem.
