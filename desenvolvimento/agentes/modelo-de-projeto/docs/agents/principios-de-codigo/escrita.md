# Princípios de código: escrita (E1 a E8)

Parte dos [princípios de código](../principios-de-codigo.md). Como escrever cada linha: nomes,
funções, tipos, erros, efeitos, comentários, duplicação e valores.

### E1. O nome diz o que é, na língua do domínio

**Obrigatório** (regra 20 da constituição): o grilling adapta o E1 ao projeto, mas não o afasta.

Variável diz o que guarda, função diz o que faz, tipo diz o que representa, com as palavras do
`GLOSSARY.md`: `produto = valor - custo`, nunca `x = y - z`. Nome de uma letra nunca, nem em laço,
lambda, `catch` ou genérico (`<Item>`, não `<T>`); a única exceção é `_` para valor descartado. Sem
abreviação que só o autor entende. Booleano lê como pergunta (`estaPago`,
`temEstoque`). Quantidade com unidade leva a unidade no nome (`prazoEmMinutos`, `valorEmCentavos`).

**Por quê:** o leitor (humano ou agente) entende o código pelo nome antes de ler o corpo. Nome
vago esconde design confuso: se não sai um nome honesto, o módulo faz coisa demais.

```ts
// Evitar
const p = x.filter((i) => i.s === 2);
// Preferir
const pedidosEmPreparo = pedidos.filter((pedido) => pedido.status === StatusPedido.EmPreparo);
```

**No grilling:** Os nomes no código ficam em português ou em inglês? Um termo do glossário ("comanda", por exemplo) vira exatamente o nome do tipo? (Se o E1 vale não se pergunta: vale sempre.)

**No diff:** nome de uma letra (fora o `_` descartado), sinônimo de termo do glossário, `data`,
`info`, `manager`, `helper`, `utils` como nome de módulo.

**No validador:** `termo-evitado`; nome de uma letra e a convenção de caixa pelo `lint-cobre`; o genérico de uma letra pelo `ast-grep` com o modelo `parametro-de-tipo-sem-nome.yml` (as regras da linguagem estão no arquivo dela). A `cobertura` reprova o E1 afastado.

### E2. Função curta, num só nível de abstração, com saída cedo

Uma função ou orquestra passos com nome, ou faz um passo concreto; não as duas coisas. Trate o caso
inválido primeiro e saia (`return` antecipado) em vez de aninhar `if`. Mais de três parâmetros é
sinal de um tipo querendo nascer.

**Por quê:** cada nível de aninhamento é um estado a mais que o leitor carrega. Parâmetros que
viajam juntos são um conceito sem nome (*Data Clumps*).

```ts
// Evitar
function confirmar(pedido, usuario, mesa, desconto, cupom) {
  if (pedido) { if (pedido.itens.length > 0) { /* ...30 linhas... */ } }
}
// Preferir
function confirmarPedido(pedido: Pedido, condicoes: CondicoesDeVenda): PedidoConfirmado {
  if (pedido.itens.length === 0) throw new PedidoVazio(pedido.id);
  const valorTotal = calcularValorTotal(pedido, condicoes);
  return registrarConfirmacao(pedido, valorTotal);
}
```

**No grilling:** O revisor cobra limites (tamanho de função, níveis de aninhamento, número de parâmetros) ou isso fica no julgamento?

**No diff:** três ou mais níveis de aninhamento, função que mistura regra de negócio com SQL ou HTML,
lista longa de parâmetros soltos.

**No validador:** `lint-cobre` com os limites de aninhamento, parâmetros e tamanho (as regras da linguagem estão no arquivo dela).

### E3. Conceito do domínio tem tipo próprio; valide na borda, uma vez

Dinheiro, e-mail, CPF, identificador de pedido: cada um tem um tipo, não `string` ou `number`
solto. A entrada externa (HTTP, formulário, arquivo, fila) é convertida nesse tipo **na borda**; dali
para dentro, o tipo garante que o dado é válido e ninguém valida de novo.

**Por quê:** validar espalhado é validar de menos em algum lugar. Um tipo que só se constrói válido
transforma um erro de execução em erro de compilação (*Primitive Obsession*; "parse, don't validate").

```ts
// Evitar: todo mundo confere se o valor é positivo, ou esquece
function aplicarDesconto(valor: number, percentual: number) {}
// Preferir
const valorDoItem = Dinheiro.deCentavos(entrada.valorEmCentavos); // falha aqui, na borda
function aplicarDesconto(valor: Dinheiro, desconto: Percentual): Dinheiro {}
```

**No grilling:** Quais conceitos do domínio ganham tipo próprio desde o início (dinheiro, documento, identificadores)? Em que ponto a entrada externa é convertida?

**No diff:** a mesma checagem em dois lugares, `string` passando por três funções até virar algo,
`any` ou `unknown` atravessando a borda.

**No validador:** `lint-cobre` e o typecheck contra o tipo genérico (`any`, `Object`, `serde_json::Value`; as regras da linguagem estão no arquivo dela); o tipo próprio na borda fica com o revisor.

### E4. Erro nunca é engolido; erro esperado é parte da interface

Separe dois tipos de erro:

- **Esperado** (o domínio prevê: estoque acabou, cupom vencido): é resultado, com tipo e nome
  próprios, e faz parte da interface do módulo. Quem chama é obrigado a tratar.
- **Inesperado** (bug, rede caiu, banco fora): sobe até a borda, que registra e traduz para quem
  está fora (status HTTP, mensagem na tela).

`catch` vazio, `catch` que só loga e segue, e erro genérico sem contexto são proibidos. A mensagem
diz o que falhou e com qual dado (sem dado pessoal sensível). O aviso do compilador e do lint também
é erro: calar o verificador (`@ts-ignore`, `eslint-disable`, `@SuppressWarnings`, `#[allow]`) só com
o porquê escrito ao lado.

**Por quê:** erro engolido vira dado corrompido descoberto semanas depois, longe da causa.

```ts
// Evitar
try { await cobrar(pedido); } catch (e) { console.log(e); }
// Preferir
const cobranca = await cobrar(pedido);
if (cobranca.tipo === "recusada") return avisarClienteDaRecusa(pedido, cobranca.motivo);
```

**No grilling:** Nesta stack, o erro que o negócio prevê (cupom vencido) volta como resultado ou como exceção com tipo próprio? Quem registra o erro inesperado, e com o quê?

**No diff:** `catch` sem relançar nem tratar, `throw new Error("erro")`, erro de domínio como
`string` comparada por texto, verificador calado sem o porquê.

**No validador:** `ast-grep` com o modelo `catch-vazio.yml`; promessa e `Result` descartados e verificador calado pelo `lint-cobre` (as regras da linguagem estão no arquivo dela).

### E5. Comando ou consulta, e imutável por padrão

Uma função ou **responde** (consulta, sem efeito colateral) ou **faz** (comando, muda estado), e o
nome deixa claro qual. Dado que não precisa mudar é imutável (`const`, `readonly`, cópia em vez de
mutação). Efeito colateral (gravar, enviar, cobrar) fica explícito e concentrado.

**Por quê:** consulta sem efeito pode ser chamada, testada e reordenada sem medo. Mutação escondida
é a causa mais comum de bug que "só acontece às vezes" (separação comando-consulta, Meyer).

```ts
// Evitar: o nome diz consulta, o corpo muda o carrinho
function calcularTotal(carrinho) { carrinho.total = soma(carrinho.itens); return carrinho.total; }
// Preferir
function calcularTotal(carrinho: Carrinho): Dinheiro { return somar(carrinho.itens.map((item) => item.valor)); }
```

**No grilling:** Imutável é regra (o lint cobra) ou preferência? Existe exceção aceita, como um trecho crítico de desempenho?

**No diff:** `get`/`calcular`/`buscar` que grava, parâmetro recebido e alterado por dentro,
`let` que nunca é reatribuído.

**No validador:** `lint-cobre` (as regras da linguagem estão no arquivo dela).

### E6. Comentário explica o porquê; o resto se apaga

O código diz o quê e o como. Comentário fica para o que o código não consegue dizer: a razão de uma
escolha, uma restrição externa, um link para a ADR ou para a issue. Código comentado e código morto
se apagam (o git guarda). `TODO` só com a issue que vai resolvê-lo.

**Por quê:** comentário que repete o código mente assim que o código muda.

```ts
// Evitar
// soma 1 no contador
contador += 1;
// Preferir
// A operadora cobra por tentativa, não por aprovação: por isso contamos antes da resposta (ADR 0012).
tentativasDeCobranca += 1;
```

**No grilling:** Alguma interface pública exige comentário de documentação (JSDoc, docstring), ou só se comenta o porquê?

**No diff:** bloco comentado, `TODO` sem issue, comentário que parafraseia a linha de baixo.

**No validador:** `todo-sem-issue`; código comentado fica com o revisor.

### E7. Duplicação de conhecimento se remove; semelhança de forma, não sempre

A mesma **regra de negócio** em dois lugares é defeito: extraia. Dois trechos que só **parecem**
iguais, mas mudam por razões diferentes, podem ficar duplicados. Na dúvida, espere a terceira
ocorrência antes de abstrair.

**Por quê:** DRY é sobre conhecimento, não sobre texto. Abstrair duas coisas que só coincidem cria
um módulo que muda por dois motivos (veja **S**).

**No grilling:** Quando a mesma regra aparece em dois lugares, extraímos na hora ou esperamos a terceira vez?

**No diff:** a mesma fórmula de preço, taxa ou prazo em dois arquivos; ou o oposto, uma função com
um `if (origem === "app")` criada para unir dois casos que não tinham nada em comum.

**No validador:** `comando` com o jscpd; duplicação de conhecimento sem texto igual fica com o revisor.

### E8. Valor com significado tem nome

Número, texto e limite que carregam regra (taxa de 10%, 3 tentativas, `"pago"`) viram constante
nomeada ou tipo enumerado, num lugar só. Configuração que muda por ambiente vem de configuração, não
do código.

**Por quê:** `0.1` no meio da conta não diz se é taxa de serviço, desconto ou imposto, e muda em
cinco lugares quando o cliente mudar a regra.

**No grilling:** Onde moram as constantes do negócio e a configuração por ambiente?

**No diff:** literal numérico que não é 0, 1 ou -1 numa regra de negócio; status comparado como
texto solto.

**No validador:** `lint-cobre` (as regras da linguagem estão no arquivo dela; em Rust, só o revisor).

## Fontes

- Bertrand Meyer, *Object-Oriented Software Construction* (1988): separação comando-consulta (E5).
- Martin Fowler, *Refactoring*, 2ª ed. (2018), cap. 3: Mysterious Name, Long Parameter List, Data
  Clumps, Primitive Obsession, Duplicated Code.
- Alexis King, "Parse, don't validate" (2019), lexi-lambda.github.io: converter na borda em tipo
  que garante a validade (E3).
- Andrew Hunt e David Thomas, *The Pragmatic Programmer*, ed. de 20 anos (2019): DRY como
  duplicação de conhecimento (E7).
