# Remendos da casa nas skills formais do pstack

As duas `SKILL.md` desta pasta são cópia de `cursor/plugins` @ `b1fbf4b60ac6813b4583ccc356850dafcf492665`
(pstack 0.15.48, MIT, `Copyright (c) 2026 Lauren Tan`, em [LICENSE](LICENSE)):

- `pstack/skills/create-formal-verification-skill/SKILL.md`
- `pstack/skills/maintain-formal-verification-skill/SKILL.md`

O primeiro commit da branch traz a cópia sem mudança; o segundo aplica os remendos abaixo. Para ver
cada um no texto, `git diff` entre os dois commits. Quem atualizar a cópia a partir do pstack
reaplica esta lista. A decisão de onde isto mora no método (remendo do instalador ou skill da casa)
é do ticket
[Decidir o que da verificação formal entra no método](https://github.com/stacklabs3/Stack-Labs/issues/75).

Cada remendo diz onde entra e o porquê, com a fonte.

1. **O alvo vem do registro de riscos.** *create*, seção *Take the target from the risk register*
   (no lugar de *Choose the target from the code*). O alvo é um `RK<n>` de `docs/riscos.md` da classe
   tempo, ordem ou concorrência, aprovado pelo humano; risco novo o agente propõe e o humano aprova
   antes de gravar. No pstack o agente escolhe "o mais forte" por julgamento, sem critério que se
   confira. Fonte: ADR 0014 (todo teste cobre um risco) e a pesquisa do pstack, *O que serve ao
   método*.

2. **O RK citado na invariante.** *create*, mesma seção; *maintain*, passos 1, 3 e 5 e o relatório.
   Cada invariante nomeia o `RK<n>` que guarda, no comentário do `.tla` e no `README.md`. Assim a
   invariante liga ao dano já escrito no registro e o replay vira teste `RK<n>` (regra 15).

3. **A ferramenta é fixa: TLA+ com o TLC 1.7.4.** *create*, seção *The tool is TLA+ with TLC* (no
   lugar de *Let the question choose the tool*). `tla2tools.jar` 1.7.4 por sha256, `-workers 1`,
   `-coverage 1`, Java 8 ou mais novo. O mapa limita o esforço a TLA+ (rodada 1, Q3); a 1.7.4 é a
   *Latest* estável (a 1.8.0 era *Pre-release* em 25/11/2026). Fonte: pesquisa de infra do TLA+.

4. **Nada de `statechart`.** *create*, a seção *Generate the verify skill* saiu com o parágrafo do
   `/statechart`. O agente conta o contraexemplo em palavras, sem ferramenta. A `statechart` tem seis
   issues abertas de robustez, depende do mise e de rede e esconde o aviso do TLC (cursor/plugins#1235).

5. **A pasta e os nomes fixos, sem skill gerada no projeto.** *create*, seção *The folder and its
   names* (no lugar de *Generate the verify skill*); *maintain*, introdução, *Edit scope* e passos 1,
   2 e 7. Tudo mora em `verificacao/<fluxo>/`: `<Modulo>.tla`, um `.cfg` por variante
   (`<fluxo>.cfg`, `quebra-<guarda>.cfg`, um por limite), `check.sh` e `README.md` com o registro de
   cobertura. O registro que o pstack põe numa skill `verify` gerada mora no `README.md`. No pstack o
   caminho e o nome são livres, e a máquina não acha o que não tem lugar fixo; nenhuma skill se
   instala no projeto enquanto o #75 não decide.

6. **Trava 1, ver falhar guarda por guarda.** *create*, item *Able to fail, guard by guard* e seção
   *Prove it before you hand off*; *maintain*, passo 5, *Pass*. Uma quebra de propósito **por guarda**
   que o modelo diz cobrir, como constante booleana num `quebra-<guarda>.cfg` e espelhada no código.
   A guarda que não faz invariante nenhuma falhar sai da tabela. No pstack basta uma quebra; no
   crashlens a primeira versão afirmava cobrir uma guarda que nunca viu falhar (commit `105b782`).

7. **Trava 2, toda ação dispara.** *create*, item *Every action fires*; *maintain*, passo 5, *Pass*.
   O `check.sh` reprova se alguma ação de `Next` tiver zero passos na saída do `-coverage` ou se o TLC
   emitir aviso. Uma ação que nunca dispara faz a invariante valer de graça (cursor/plugins#1235,
   aberta).

8. **Trava 3, modelo contra código por um segundo leitor.** *create*, item *Checked by a second
   reader*; *maintain*, passo 6 (novo). Um subagente que não vê o raciocínio do autor confere cada
   linha da tabela ação→código: o código existe, os nomes batem e a atomicidade está justificada pela
   transação ou pela tranca. Só cerca de 41–46% dos modelos escritos por LLM batem com o código
   (SysMoBench). O item *True to the code* passa a dizer que um passo indivisível é uma transação ou
   uma tranca e que duas escritas no código são duas ações.

9. **O replay vira teste RK e issue `needs-triage`.** *create*, seção *When the checker finds a
   counterexample* (nova); *maintain*, passo 5, *Fail*. Todo passo acontece: teste de replay que cita
   o `RK<n>` e falha, issue de bug com `needs-triage` e o humano roda o `/triage`. Algum passo não
   acontece: o modelo está errado, corrige-se e registra-se como ruído. No pstack o replay fica só no
   relatório, "sem commitar no produto"; no método o achado termina num teste que cobre o risco
   (regra 15) e a correção é outra issue.

10. **A escada de limites registrada.** *create*, item *Cheap to rerun*. Estados, profundidade e
    tempo de parede de cada limite vão no `README.md`, para o próximo agente saber o que o `.cfg`
    mínimo custa.

11. **"Invariant" no lugar de "property".** *maintain*, todo o texto. No método, "regra" já quer
    dizer Regras de código; o termo da afirmação que o modelo prova é **invariante**.

12. **O que o humano lê.** *create*, seção *Hand off*. O humano lê as invariantes, o que fica de fora
    e o contraexemplo em palavras; o resto é do agente e do segundo leitor.
