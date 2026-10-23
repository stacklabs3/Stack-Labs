# Mapa inicial de Regras de código

Parte dos [mapas iniciais](../mapas-iniciais.md). Label `mapa-inicial:regras-de-codigo`. O grilling
roda em torno dos [princípios de código](../principios-de-codigo.md): um ticket por grupo, cada
princípio sai **adotado**, **adaptado** ou **afastado**. **Ao conduzir qualquer ticket deste mapa**,
leia [principios-de-codigo/grilling.md](../principios-de-codigo/grilling.md): como cada rodada anda
e o modelo de `docs/regras-de-codigo.md`.

## Corpo do mapa

```markdown
## Destination

docs/regras-de-codigo.md escrito e aprovado: os 22 princípios de código da casa (E1 a E8, P1 a P9,
S, O, L, I, D) decididos na tabela de cobertura, as regras que saíram deles com exemplo do próprio
projeto e o que o lint e o formatador garantem sozinhos. docs/regras-de-codigo.json diz como o
validador confere cada regra. É a régua do implementador e do revisor.

## Notes

- Roteiro de cada ticket: docs/agents/principios-de-codigo/grilling.md. Ticket de grupo fecha com uma
  linha de decisão por princípio no comentário de resolução.
- Depende do mapa de Arquitetura: tickets que dependem de uma escolha de stack ficam `blocked_by` o
  ticket dela. As restrições vêm antes e bloqueiam os tickets de grupo.
- Lint, formatador e typecheck passam por pesquisa como toda escolha técnica. Configurá-los, e pôr o
  validador (`sh .claude/watchdogs/rodar validar`) no CI, é a primeira fatia da primeira fase.
- Regra que o revisor não consegue apontar num diff não entra.

## Decisions so far

## Not yet specified

## Out of scope
```

## Tickets

Os de grupo são obrigatórios; um grupo que não cabe numa sessão vira mais de um ticket.

| Ticket | Tipo | Princípios | Skill |
|---|---|---|---|
| *Restrições para as regras*: quem mantém, nível do time, quanto o agente escreve, o que já deu errado | grilling | (bloqueia os grupos) | `/human-architect-mindset` |
| *Atrito do código existente*, só em projeto com código: cada atrito ligado ao princípio que quebra | tarefa | todos | `/improve-codebase-architecture` |
| *Escrita de código* | grilling | E1, E2, E6, E7, E8 ([escrita](../principios-de-codigo/escrita.md)) | `/domain-modeling` |
| *Tipos, erros e validação* | grilling | E3, E4, E5 ([escrita](../principios-de-codigo/escrita.md)) | `/domain-modeling` |
| *Desenho de módulos e SOLID* | grilling | P1 a P4 ([padrões](../principios-de-codigo/padroes.md)), S, O, L, I, D ([SOLID](../principios-de-codigo/solid.md)) | `/codebase-design` |
| *Testes e fluxo de trabalho* | grilling | P5 a P9 ([padrões](../principios-de-codigo/padroes.md)) | `/tdd` |
| *Opções de lint, formatador e typecheck* → *Lint e formatador* | pesquisa → grilling | os que a ferramenta garante sozinha | `/research` |
| *Escrever docs/regras-de-codigo.md e .json*: junta as decisões nos modelos; fecha o mapa com `--so-cobertura` aprovado | tarefa | os 22 na cobertura | |
