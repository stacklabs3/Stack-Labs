# Piloto de verificação formal

Relato do ticket [Rodar o piloto de verificação formal e registrar o experimento](https://github.com/stacklabs3/Stack-Labs/issues/74),
do mapa [Mapa: verificação formal no método](https://github.com/stacklabs3/Stack-Labs/issues/69). O plano
veio de [Escolher projeto, risco e invariante do piloto](https://github.com/stacklabs3/Stack-Labs/issues/73).
Este relato serve ao ticket [Decidir o que da verificação formal entra no método](https://github.com/stacklabs3/Stack-Labs/issues/75):
ele registra o que aconteceu e não decide o que entra.

- **Projeto:** excurfly, issue [#2910](https://github.com/stacklabs3/excurfly/issues/2910), PR [#2914](https://github.com/stacklabs3/excurfly/pull/2914),
  pasta `verificacao/cobranca-sem-resposta/`, commit de sincronia `33573772c`.
- **Receita seguida:** a vertente desta pasta, em `vertente/`: as duas skills formais do pstack
  (cursor/plugins @ `b1fbf4b`, MIT) com os doze remendos de `vertente/REMENDOS.md`.
- **Fluxo:** o ciclo da Cobrança por Pix quando a emissão fica sem resposta.

## Resultado

**Não é bug na compra.** A sequência suspeita da #73 não acontece no código, porque a emissão que
falha cancela a Compra em aberto (`payment.purchase.ts:576-580`) e a tentativa nova nasce noutra
Reserva. O piloto achou um lugar onde a sequência pode acontecer, a **Recompra**, que emite de novo
na mesma Reserva (`payment.rebuy.ts:500-550`). A Recompra ficou fora do modelo, e a pista foi para
[Recompra paga por Pix pode voltar como Devolução quando a emissão fica sem resposta](https://github.com/stacklabs3/excurfly/issues/2913)
(`needs-triage`), sem replay.

| Invariante | Risco | No código como está | Visto falhando |
|---|---|---|---|
| Pago no prazo vira Reserva (`PagoNoPrazoViraReserva`) | RK329, novo | vale nos dois limites | sim, em 2 quebras |
| Cobrado uma vez só (`CobradoUmaVezSo`) | RK1 | vale nos dois limites | **não**: nenhuma quebra a fez falhar |

**Cobrado uma vez só** vale por construção do modelo: o Comprador só paga o Pix que a emissão lhe
mostrou, e cada Compra mostra um Pix só. Pela trava 1, um "passa" assim não é evidência.

## As três travas

### Trava 1: ver falhar

Foram sete quebras de propósito, uma por guarda, cada uma como constante booleana num `.cfg`.

| Guarda | Resultado | Espelho no código |
|---|---|---|
| A emissão sem resposta encerra a Compra, e a tentativa nova é outra Compra (`payment.purchase.ts:576-580`) | viola RK329 em 14 estados | Não há rota que emita duas vezes na mesma Compra. O replay da v1 mostrou o caminho que o código toma. |
| A consulta lista só as cobranças da própria Reserva (`payment.reconciliation.ts:1696`) | viola RK329 em 13 estados | O PSP falso devolve também a cobrança da Compra cancelada: o aviso do Pix pago dá `refunded_duplicate`, e a Compra fica `awaiting_payment` com o id velho. |
| O id da emissão só se for NULL (`payment.charging.ts:432-435`) | não falha | sai da tabela |
| A adoção só na linha sem id (`payment.cash-released.ts:56-60`) | não falha | sai da tabela |
| O aviso com linha viva com id vira `duplicate` (`payment.reconciliation.ts:2057-2058`) | não falha | sai da tabela |
| A consulta confirma a cobrança recebida (`payment.reconciliation.ts:1698-1713`) | não falha | sai da tabela |
| O soft-delete da emissão sem resposta só sem id (`payment.charging.ts:380`) | não falha | sai da tabela |

Das sete guardas, cinco não mudam nenhuma das duas invariantes no limite mínimo. A guarda que a
planilha de partida tratava como central, o `IS NULL` do id da emissão, é uma delas. Na compra, quem
segura o RK329 é o cancelamento da Compra que falha.

### Trava 2: toda ação dispara

No `.cfg` do código, as 15 ações de `Next` deram passo, e o TLC não deu aviso. O `check.sh`
foi provado nos dois sentidos, numa cópia do modelo:

- uma ação com guarda impossível reprova, com a lista das ações sem passo;
- um `UNCHANGED` sobre a variável que a ação muda, o caso da cursor/plugins#1235, reprova pelo aviso do TLC.

**Limite da trava:** ela olha a ação, não o ramo. Em `Confirmar`, no código como está, só roda o ramo
da linha com o id. O `duplicate`, a adoção pelo aviso e a volta da tentativa retirada ficaram com
zero avaliações no `-coverage`, e nenhuma ação ficou com zero passos.

### Trava 3: modelo contra código

Um subagente separado, sem o raciocínio do autor, recebeu o `.tla`, a tabela e o código, e conferiu
as 14 linhas: 4 confere, 9 confere com ressalva, 1 não confere. A que não conferia era o ramo
`buyerReplacedReservation` (`payment.reconciliation.ts:1996-2012`), que faltava em `Confirmar`. O
ramo entrou no modelo, e os números não mudaram.

As ressalvas que ficaram registradas no README da pasta:

- a consulta decide a guarda pelo relógio do começo da passada;
- `applyConfirmation` pode gravar a adoção e confirmar em transações diferentes;
- o cancelamento da Compra lê e escreve sem tranca;
- faltam escritas que não mexem nas invariantes: a marca em dinheiro, a expiração das guardas da
  Viagem dentro de outra compra e a segunda volta da passada.

## A escada de limites

| `.cfg` | Tentativas | Aviso repete | Rodadas | Gerados | Distintos | Profundidade | Parede |
|---|---|---|---|---|---|---|---|
| mínimo (`cobranca-sem-resposta.cfg`) | 2 | 1 | 1 | 4.680 | 2.120 | 17 | 0,72 s |
| maior (`limite-maior.cfg`) | 3 | 2 | 1 | 38.717 | 14.306 | 23 | 0,90 s |
| v1, descartada | 2 | 1 | 1 | 1.740 até a violação | 1.028 | 13 | 1,24 s |

Medido com TLC 1.7.4, `-workers 1`, no Apple M5 com OpenJDK 25. A parede é quase toda partida da
JVM. O `check.sh` inteiro, com o limite maior, leva 2,96 s. O mínimo fica abaixo de 1 s. A v1 passou
de 1 s, mas ela parou na violação, e o tempo é de partida.

## O bug

Nenhum na compra. A pista na Recompra está em #2913, sem replay e sem teste. O `/triage` dela é do
humano.

## O custo

| Medida | Valor |
|---|---|
| `sh .claude/watchdogs/rodar custo 2910` | 0 tokens, 0 sessões (ver *Ruído*) |
| A sessão inteira, pelos pesos do `custo.rs` lidos do transcript | ≈ 5,21 M tokens equivalentes (131 respostas) |
| O subagente da trava 3 | ≈ 0,61 M (24 respostas, 5,5 min) |
| **Total** | **≈ 5,8 M**, acima dos 3 M da ADR 0017: **sinal contra** |

A sessão inteira inclui o pré-voo, a vertente, a issue, o modelo, os replays, as PRs e este relato.
A maior parte é leitura de cache (28,9 M tokens lidos até a metade da sessão), que vem de ler
`payment.reconciliation.ts`, um arquivo de mais de 2.300 linhas, várias vezes.

**Tempo de parede por fase** (UTC, 27 e 28/11/2026, anotado na sessão):

| Fase | Tempo |
|---|---|
| Pré-voo, claim e vertente | ≈ 7 min |
| Issue, RK e troca de pasta (o clone, ver *Ruído*) | ≈ 15 min |
| Contexto e contrato da fatia | ≈ 6 min |
| Escrever o modelo v1 | ≈ 3 min |
| Replay da v1 (com a instalação do backend) | ≈ 2 min |
| Escrever o modelo v2 e o `check.sh` | ≈ 3 min |
| Ver falhar (quebras, espelho no código, prova do `check.sh`) | ≈ 3 min |
| Conferir a tabela (subagente e correção) | ≈ 7 min |
| README, PRs e relato | ≈ 15 min |

Os runs do TLC estão na escada de limites. Cada run levou menos de 1,3 s.

## Ruído

- **O modelo v1 estava errado.** Ele seguiu a tabela de partida da #73, em que o Comprador tenta de
  novo na mesma Compra. O TLC achou a sequência suspeita. O replay pela rota, com o PSP falso,
  mostrou que o passo 5 não acontece. O autor já tinha visto o cancelamento da Compra no código antes
  de rodar, e seguiu o método assim mesmo, para medir o caminho inteiro. A v1 virou a quebra
  `quebra-tentativa-na-mesma-compra.cfg` da v2. **Lição:** a sequência suspeita veio da leitura da
  emissão e da conciliação, sem a rota que as chama. A tabela de partida precisa da linha de quem
  chama.
- **O modelo v2 deixou de fora um ramo** (`buyerReplacedReservation`). A trava 3 o achou. O ramo não
  mudou o resultado.
- **O `rodar custo` não mediu.** A sessão nasceu no Stack-Labs e mudou de pasta. Os hooks de custo
  do excurfly não a acompanharam, e o custo saiu do transcript, com os pesos do `custo.rs`.
- **O ambiente custou mais que o modelo.** A raiz do excurfly na máquina tinha trabalho não
  commitado de outra issue. Uma worktree em `.claude/worktrees/` cairia em caminho livre do
  watchdog. O humano escolheu clonar o excurfly dentro do Stack-Labs. O clone nasceu sem
  `core.hooksPath`, então os git hooks do método não rodaram nos commits, e a regra 21 proíbe o
  agente de ligá-lo.
- **A sonda do pré-voo ativou issue errada.** `rodar ativar-issue 74` ativou a #74 do excurfly, uma PR
  mergeada sem rótulo, em vez de recusar, contra o que a regra 3 pede. Fica para o humano olhar o
  watchdog.
- **Uma edição do `.tla` foi por script,** e não pelo Edit, contra a regra da casa. As outras foram
  pelo Edit e pelo Write.

## O que o humano precisou ler e decidir

1. A issue do piloto e a linha do RK329, numa rodada só: aprovou as duas como vieram.
2. O trabalho não commitado na raiz do excurfly: interrompeu o stash e mandou clonar o projeto dentro
   do Stack-Labs.
3. A Situação do RK329 depois do piloto e a pista da Recompra, numa rodada: escolheu "pendente na
   Recompra" e a issue `needs-triage`.

O que o método pede que o humano leia, as invariantes, o que fica de fora e o contraexemplo em
palavras, coube no corpo da issue e no README da pasta. O TLA+ ele não precisou ler.

## Fontes

- excurfly @ `33573772c`:
  - `backend/src/modules/payment/payment.charging.ts`, `payment.purchase.ts`, `payment.reconciliation.ts`, `payment.cash-released.ts` e `payment.rebuy.ts`;
  - `backend/src/modules/reservation/reservation.claim.ts`;
  - `docs/riscos.md` (RK1, RK329);
  - `GLOSSARY.md` (Cobrança, Compra em aberto, Devolução, Reserva).
- excurfly, PR [#2914](https://github.com/stacklabs3/excurfly/pull/2914): `verificacao/cobranca-sem-resposta/README.md`, com a tabela ação → código e o parecer da trava 3.
- [cursor/plugins](https://github.com/cursor/plugins) @ `b1fbf4b`: as duas skills formais do pstack e a issue [#1235](https://github.com/cursor/plugins/issues/1235).
- As pesquisas do mapa, nas branches `research/infra-tla` e `research/pstack-verificacao-formal`.
- ADR 0014 (todo teste cobre um risco) e ADR 0017 (orçamento de 3 M por issue).
