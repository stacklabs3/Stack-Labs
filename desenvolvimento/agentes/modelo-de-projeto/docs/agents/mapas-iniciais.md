# Mapas iniciais

Todo projeto abre cinco mapas do `/wayfinder` antes de qualquer código de produção (ADRs 0002 e 0007
do método). Cada mapa é uma issue com `wayfinder:map` **e** `mapa-inicial:<tipo>`, que é como os
watchdogs o encontram. Num projeto que já existia, os mapas são **retroativos**
(`/adotar-metodo-stack-labs`).

**Ao abrir ou conduzir um mapa**, leia o arquivo dele: o corpo da issue e os tickets que costumam nascer.

| Mapa | Destino | Trava a ativação de issue até fechar? |
|---|---|---|
| [Protótipo](mapas-iniciais/prototipo.md) | O desenho inicial do produto prototipado e aprovado | **Sim** |
| [Arquitetura](mapas-iniciais/arquitetura.md) | Linguagem, stack e módulos decididos, em ADRs | **Sim** |
| [Fases](mapas-iniciais/fases.md) | A divisão em fases que o desenvolvedor escolheu, com o que entra e o critério de pronto de cada uma | **Sim** |
| [Regras de código](mapas-iniciais/regras-de-codigo.md) | `docs/regras-de-codigo.md`, com os 22 princípios de código decididos, e o `docs/riscos.md` | **Sim** |
| [Infra](mapas-iniciais/infra.md) | Retrato da infra: custos, tecnologias e o que construir em volta | Não (informativo) |

## Como abrir

1. Uma sessão por mapa, com `/wayfinder` em modo *Chart the map*. Comece pelo Protótipo e pela
   Arquitetura; Fases, Regras de código e Infra dependem deles.
2. Crie o mapa com as duas labels e o corpo do arquivo dele; ajuste o destino ao projeto no grilling.
3. Os tickets do arquivo são ponto de partida: o grilling breadth-first decide quais existem, e o que
   não tiver pergunta precisa vai para *Not yet specified*. Exceção: os tickets de grupo do mapa de
   Regras de código são obrigatórios.
4. Ligue os bloqueios com dependências nativas: ticket que depende de uma escolha de Arquitetura
   fica `blocked_by` o ticket dela.
5. Dispare os subagentes `/research` dos tickets de pesquisa no fim da sessão de mapear.
6. Feche a sessão com o `/nova-sessao`.

Pronto quando a issue existe com as duas labels, *Destination* preenchido, os tickets criados com
bloqueios e as pesquisas disparadas.

## Em todo mapa

- Ticket de decisão se resolve por grilling (`/grilling` + `/domain-modeling`).
- Escolha técnica (linguagem, framework, banco, provedor) só vai ao grilling com o ticket de pesquisa
  dela fechado, e o grilling cita o arquivo de `docs/research/`
  ([pesquisa-antes-de-recomendar.md](pesquisa-antes-de-recomendar.md)).
- Escolha difícil de reverter vira ADR em `docs/adr/`. O ticket aponta a ADR; o mapa aponta o ticket.
