# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual label strings used in this repo's issue tracker.

| Label in mattpocock/skills | Label in our tracker | Meaning                                  |
| -------------------------- | -------------------- | ---------------------------------------- |
| `needs-triage`             | `needs-triage`       | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info`         | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent`    | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human`    | Requires human implementation            |
| `wontfix`                  | `wontfix`            | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the corresponding label string from this table.

## Issue adiada: o terceiro eixo

A label de **issue adiada** marca *quando* a issue entra: depois de um marco que o projeto nomeia
(um lançamento, uma fase). Não é estado nem tipo. A issue continua com o rótulo de triagem dela
(`needs-triage`, `ready-for-agent`…) ou com o `wayfinder:*` dela; a label de adiada vale nas duas
famílias e só tira a issue das fronteiras: a do `/orquestrar-issues` e a *Frontier query* do
`/wayfinder` ([issue-tracker.md](issue-tracker.md)). Tirar a label devolve a issue à fila.

| Papel | Label neste tracker |
| ----- | ------------------- |
| issue adiada | `adiada` |

Quando uma skill fala da label de adiada, use a string desta tabela. O projeto que usa outro nome
(por exemplo `depois-do-lancamento`) troca só a coluna da direita.
