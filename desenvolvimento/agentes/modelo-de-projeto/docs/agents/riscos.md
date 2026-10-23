# Registro de riscos e testes que valem

Todo teste do projeto existe para pegar um **risco**: algo que, se der errado, causa dano a quem usa
ou a quem mantém. O registro desses riscos é `docs/riscos.md`, e todo teste cita o risco que cobre
(regra 15 da constituição, princípio P5, ADR 0014 do método). Teste que não cobre risco do registro
não entra, mesmo que passe.

## O que é risco, e o que não é

**Ao propor um risco**, ele precisa caber numa destas classes, e o dano precisa ser concreto:

| Classe | Exemplo |
|---|---|
| Dinheiro ou dado perdido, duplicado ou corrompido | cobrar duas vezes o mesmo pedido; apagar a comanda errada |
| Segurança e permissão | garçom vê o caixa de outra loja; token vencido ainda entra |
| Regra de negócio que a spec decidiu | cupom vencido aceito; mesa fechada recebe pedido |
| Erro esperado na borda (E3, E4) | entrada inválida vira tipo do domínio; falha do provedor some calada |
| Contrato com o que não é nosso | resposta do gateway muda de formato; adapter não cumpre a port (L) |
| Tempo, ordem e concorrência | dois caixas fecham a mesma conta; virada de dia muda o total |
| Bug que já aconteceu | o defeito da issue #41 volta |

Não é risco, e por isso não ganha teste: getter, setter, construtor que só guarda campo, constante,
mapeamento campo a campo sem regra, o que o compilador ou o typecheck já garante, o comportamento do
framework ou da biblioteca, e o mock devolvendo o que mandaram ele devolver.

## O registro: `docs/riscos.md`

```markdown
# Riscos

Os riscos que os testes do projeto cobrem. Risco novo entra só com a origem: a issue, o ticket ou a
ADR em que o humano o aprovou.

| Risco | O que dá errado | Dano | Onde | Origem | Situação |
|---|---|---|---|---|---|
| RK1 | Cobrar duas vezes o mesmo pedido | estorno e cliente perdido | pagamento | #12 | |
| RK2 | Cupom vencido aceito | margem perdida | cupom | docs/adr/0003-cupom.md | |
| RK3 | Mesa fechada recebe pedido | pedido perdido na cozinha | comanda | #15 | pendente #31 |
```

- **ID** `RK<n>` em sequência, nunca reaproveitado. Risco que deixou de existir sai da tabela junto
  com os testes dele, por ticket.
- **Origem** é obrigatória: link ou `#n` da issue, do ticket ou da ADR em que o humano aprovou.
- **Situação** fica vazia quando há teste. Risco aprovado e ainda sem teste leva `pendente #<n>`,
  com a issue que vai cobri-lo.

### De onde vem cada risco

- **Mapa de Regras de código**, ticket *Riscos que os testes cobrem*: o registro nasce com os riscos
  do produto que o mapa de Arquitetura e a spec já mostram.
- **`/to-spec`**: a spec lista os riscos da funcionalidade; ao fechar o mapa, eles entram no registro
  como `pendente #<spec>`, e cada fatia diz quais cobre.
- **`/triage` de bug:** o bug vira risco (classe *bug que já aconteceu*), e a correção traz o teste
  que o reproduz.

Você **propõe** risco; o humano **aprova**. Teste que você quer escrever e que não acha risco no
registro é o sinal de que ele não deve existir, ou de que falta uma decisão: pergunte, não invente.

## Como o teste cita o risco

O ID aparece no nome do teste, num comentário, atributo ou anotação logo acima, ou no bloco que o
contém (`describe`, `mod`, classe). Um teste pode cobrir mais de um risco.

```ts
describe("RK1: cobrança única por pedido", () => {
  it("recusa a segunda cobrança do mesmo pedido", async () => {
    await cobrar(pedidoPago, gateway);
    await expect(cobrar(pedidoPago, gateway)).rejects.toThrow(PedidoJaCobrado);
  });
});
```

```rust
#[test]
fn rk2_recusa_cupom_vencido() {
    let cupom_vencido = Cupom::novo("NATAL", data(2025, 12, 31));
    assert_eq!(aplicar(&cupom_vencido, hoje(2026, 1, 2)), Err(ErroDeCupom::Vencido));
}
```

## O que faz um teste valer

1. **Falha quando o risco acontece.** Antes do verde, veja o teste vermelho pelo motivo certo (P6):
   o vermelho prova que ele enxerga o risco.
2. **Afirma o resultado observável** pela interface pública (P5): o estado ou a resposta, não a
   chamada interna. Conferir chamada de mock só vale na seam com o que não é nosso (o e-mail saiu, o
   gateway foi cobrado).
3. **O esperado vem de fora do código:** um exemplo do risco ou da spec, um literal conhecido. Nunca
   recalculado como o código calcula, nunca um valor comparado com ele mesmo.
4. **Roda sempre:** nada de `.skip`, `.only`, `#[ignore]` ou `@Disabled`. Risco sem teste fica
   visível no registro como `pendente`, não escondido na suíte.

O auxiliar de afirmação do projeto tem nome que começa por `assert`, `expect`, `verify` ou
`conferir`: é assim que a máquina o reconhece.

## O que a máquina confere

| Onde | O quê |
|---|---|
| `vigia-teste` (hook, a cada edição de teste) | No arquivo editado: teste sem `RK<n>`, `RK<n>` fora do registro, teste sem afirmação, afirmação trivial, teste desligado ou focado. O agente recebe o motivo e corrige na hora |
| Validador (`teste-cobre-risco` e os modelos de ast-grep do P5) | O mesmo nos testes alterados, mais o registro inteiro: risco sem origem, pendente sem issue, risco não pendente sem teste ([validador.md](validador.md)) |
| Revisor da orquestração | O que a máquina não vê: o teste cita o risco mas não o exercita; o esperado copia o código; o teste só confere interação |

**Teste de mutação** mede se os testes pegam o defeito, e não só se rodam a linha: o projeto o adota
no grilling para os módulos de risco alto, só no diff, pela receita da linguagem em
[validador.md](validador.md). Cobertura de linha não é portão: ela mede que a linha rodou, não que o
teste afirmou algo sobre ela.
