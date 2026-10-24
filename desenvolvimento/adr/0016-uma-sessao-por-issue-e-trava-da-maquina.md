# O orquestrador abre uma sessão por issue, e todo comando pesado passa pela trava da máquina

Emenda a ADR 0005. Nela o orquestrador disparava os subagentes dentro da própria janela, e só o
humano podia chamar o `/orquestrar-issues`. Com a fila longa, a janela do orquestrador enchia de
pareceres, e o kickoff colado travava no `disable-model-invocation`. Também havia filas
independentes (trilhas) esperando uma pela outra. Escolhemos três mudanças. **O agente chama a
skill**: a fila e a branch continuam decisão do humano, no passo 2. **Uma sessão do Claude por
issue**, de janela limpa, que leva a issue até o fim pelo ciclo fixo (implementador → revisor →
conferente → diagnosticador, até dois ciclos de correção); só quando ela termina, integrada ou
devolvida, o orquestrador confere e abre a da próxima. Sem sessões no ambiente, o ciclo roda com
subagentes, como antes. **Um orquestrador por trilha**, cada um com a sua branch de integração e o
seu estado em `.claude/orquestracao/`. Implementadores simultâneos dentro de um orquestrador foram
considerados e recusados pelo humano (2026-10-24): uma issue por vez, até o fim.

O paralelismo entre trilhas volta ao risco que a ADR 0005 evitava: sessões disputando a memória.
Numa máquina sem swap o kernel mata processos sem aviso; num projeto da casa, dois builds de Next ao
mesmo tempo derrubaram os dev servers, o build e o shell do agente (2026-09-13). Por isso a
**trava da máquina**: todo comando pesado (build, dev server, suíte de testes, e2e, build nativo, CI
local) roda dentro de `$HOME/.stacklabs/pesado.lock`, um por vez na máquina inteira, com
`NODE_OPTIONS=--max-old-space-size=4096`.

**Consequências:** a trava é um `mkdir` atômico com o PID do dono; quem espera limpa a de um dono
morto. Ela vale entre projetos, orquestradores e qualquer agente da máquina, e está no papel
`implementador` e no brief de cada sessão de issue. O orquestrador acompanha a sessão pelo aviso de
fim, sem polling, e confere pareceres, branch e commit antes do merge, que continua um por vez. Rodar
issues em paralelo dentro de uma trilha exige reabrir esta ADR.
