# Princípios de código

A base da casa para escrever código, em todo projeto e em qualquer linguagem: como escrever, como
desenhar e como ler SOLID sem cair em módulo raso. É o **guia do mapa de Regras de código**
(ADR 0008 do método): o grilling do mapa roda em torno destes princípios, um a um, e o mapa só fecha
quando todos têm decisão. Depois, é a régua de julgamento do revisor onde `docs/regras-de-codigo.md`
cala.

## Como esta base se relaciona com as regras do projeto

- **`docs/regras-de-codigo.md` vence.** Ele é a régua dura do projeto (ADR 0007). Um princípio que o
  projeto adotou vira regra lá, com exemplo do próprio código, e o revisor reprova quem a quebra.
- **O resto é julgamento.** Onde nenhuma regra do projeto alcança o trecho, ou o projeto ainda não
  tem a cobertura completa (projeto anterior à ADR 0008), o revisor aponta o princípio como
  observação ("possível quebra de E3") e só reprova se explicar o dano concreto nesta issue, como faz
  com os *smells* do `/code-review`. Princípio que o projeto **afastou** não é apontado.
- **Todo princípio recebe decisão.** No grilling do mapa, cada princípio é adotado, adaptado ou
  afastado, com o porquê registrado. Afastar é legítimo; pular não é.
- **O que o lint garante sai daqui.** Formatação, ordem de import e afins são da ferramenta.
- Cada princípio tem um **ID** (E1, P3, S...). `docs/regras-de-codigo.md` cita o ID da base de que a
  regra nasceu, e o revisor cita o ID no achado.

Precedência: `docs/regras-de-codigo.md` → ADRs do projeto → esta base → *smells* do `/code-review`.

Cada princípio traz a regra, o porquê, um exemplo, a pergunta-guia **No grilling** e o que o revisor
procura **No diff**. Os exemplos estão em TypeScript e no domínio de um app de pedidos, só para ilustrar. O projeto troca
pelos dele.

---

## E. Escrita de código

### E1. O nome diz o que é, na língua do domínio

Variável diz o que guarda, função diz o que faz, tipo diz o que representa, com as palavras do
`GLOSSARY.md`. Sem abreviação que só o autor entende. Booleano lê como pergunta (`estaPago`,
`temEstoque`). Quantidade com unidade leva a unidade no nome (`prazoEmMinutos`, `valorEmCentavos`).

**Por quê:** o leitor (humano ou agente) entende o código pelo nome antes de ler o corpo. Nome
vago esconde design confuso: se não sai um nome honesto, o módulo faz coisa demais.

```ts
// Evitar
const p = x.filter((i) => i.s === 2);
// Preferir
const pedidosEmPreparo = pedidos.filter((pedido) => pedido.status === StatusPedido.EmPreparo);
```

**No grilling:** Os nomes no código ficam em português ou em inglês? Um termo do glossário ("comanda", por exemplo) vira exatamente o nome do tipo?

**No diff:** nome de uma letra fora de laço curto, sinônimo de termo do glossário, `data`, `info`,
`manager`, `helper`, `utils` como nome de módulo.

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

### E4. Erro nunca é engolido; erro esperado é parte da interface

Separe dois tipos de erro:

- **Esperado** (o domínio prevê: estoque acabou, cupom vencido): é resultado, com tipo e nome
  próprios, e faz parte da interface do módulo. Quem chama é obrigado a tratar.
- **Inesperado** (bug, rede caiu, banco fora): sobe até a borda, que registra e traduz para quem
  está fora (status HTTP, mensagem na tela).

`catch` vazio, `catch` que só loga e segue, e erro genérico sem contexto são proibidos. A mensagem
diz o que falhou e com qual dado (sem dado pessoal sensível).

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
`string` comparada por texto.

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

### E7. Duplicação de conhecimento se remove; semelhança de forma, não sempre

A mesma **regra de negócio** em dois lugares é defeito: extraia. Dois trechos que só **parecem**
iguais, mas mudam por razões diferentes, podem ficar duplicados. Na dúvida, espere a terceira
ocorrência antes de abstrair.

**Por quê:** DRY é sobre conhecimento, não sobre texto. Abstrair duas coisas que só coincidem cria
um módulo que muda por dois motivos (veja **S**).

**No grilling:** Quando a mesma regra aparece em dois lugares, extraímos na hora ou esperamos a terceira vez?

**No diff:** a mesma fórmula de preço, taxa ou prazo em dois arquivos; ou o oposto, uma função com
um `if (origem === "app")` criada para unir dois casos que não tinham nada em comum.

### E8. Valor com significado tem nome

Número, texto e limite que carregam regra (taxa de 10%, 3 tentativas, `"pago"`) viram constante
nomeada ou tipo enumerado, num lugar só. Configuração que muda por ambiente vem de configuração, não
do código.

**Por quê:** `0.1` no meio da conta não diz se é taxa de serviço, desconto ou imposto, e muda em
cinco lugares quando o cliente mudar a regra.

**No grilling:** Onde moram as constantes do negócio e a configuração por ambiente?

**No diff:** literal numérico que não é 0, 1 ou -1 numa regra de negócio; status comparado como
texto solto.

---

## P. Padrões de desenvolvimento

O vocabulário é o do `/codebase-design`: **módulo**, **interface**, **implementação**,
**profundidade**, **seam**, **adapter**.

### P1. Módulo profundo: interface pequena, muito comportamento atrás

Prefira poucos módulos que escondem bastante a muitos módulos finos que só repassam. A interface
inclui tudo o que quem chama precisa saber: tipos, invariantes, ordem de chamada, erros e custo.

**Por quê:** interface pequena dá **alavanca** a quem chama e **localidade** a quem mantém: a regra
muda num lugar só. Módulo raso espalha o conhecimento por todos os chamadores.

**Teste da exclusão:** imagine apagar o módulo. Se a complexidade some, era repasse (*Middle Man*).
Se ela reaparece em vários chamadores, o módulo se paga.

**No grilling:** Qual é o módulo típico deste projeto (pacote por conceito, classe, arquivo)? Ele esconde banco e regra atrás de poucas funções?

**No diff:** classe ou função que só chama outra com os mesmos parâmetros; o chamador precisando
chamar três métodos numa ordem certa para fazer uma coisa.

### P2. Seam só onde algo varia de verdade

Uma interface (port) com uma implementação só é indireção, não design. Crie a seam quando há duas
variações reais: produção e teste, dois provedores, dois canais. Dependência dentro do processo
(cálculo puro, memória) não precisa de seam: chame direto.

**Por quê:** cada seam é uma interface a mais para aprender e manter. "Um adapter é uma seam
hipotética; dois adapters, uma seam real."

**No grilling:** Criamos interface para banco e provedores desde o começo, ou só quando houver a segunda implementação (o substituto de teste conta)?

**No diff:** `interface ServicoDeX` com uma única `ServicoDeXImpl`; fábrica ou container de injeção
para algo que nunca varia.

### P3. Dependência entra, não é criada; o domínio não conhece a infraestrutura

O módulo recebe o que usa (parâmetro, construtor) em vez de instanciar por dentro. A regra de negócio
não importa banco, HTTP, fila ou SDK de terceiro: ela define a port de que precisa, e a infraestrutura
fornece o adapter. A montagem (quem liga o quê) fica num ponto só, na borda da aplicação.

**Por quê:** é o que deixa a regra testável sem rede e trocável sem reescrita. Ver **D**.

```ts
// Evitar
function fecharConta(mesa: Mesa) { const gateway = new StripeGateway(process.env.KEY); /* ... */ }
// Preferir
function fecharConta(mesa: Mesa, cobranca: Cobranca): ContaFechada { /* ... */ }
```

**No grilling:** Como as dependências chegam ao módulo: parâmetro, construtor ou container de injeção? Onde fica a montagem?

**No diff:** `new` de cliente externo dentro da regra; import de ORM ou de framework web num arquivo
do domínio; leitura de `process.env` fora da montagem.

### P4. O que muda junto fica junto

Organize pelo conceito do domínio (pedido, cardápio, pagamento), não pela camada técnica, salvo
decisão contrária da Arquitetura. Uma mudança de regra deveria tocar poucos arquivos vizinhos.

**Por quê:** mudança espalhada por muitos arquivos (*Shotgun Surgery*) é o custo mais caro de
manter, para humano e para agente, que precisa carregar tudo no contexto.

**No grilling:** As pastas se organizam por conceito do negócio (pedido, cardápio) ou por camada técnica (controllers, services)?

**No diff:** uma regra nova que obriga a editar cinco pastas distantes.

### P5. Teste pela interface, com o comportamento como frase

O teste exercita a interface pública do módulo e afirma o que é observável de fora. O nome do teste
é uma frase do domínio (`"recusa cupom vencido"`). Mock só na seam com o que não é nosso (provedor
externo, relógio, aleatório); o que é nosso roda de verdade ou com um substituto local (banco em
memória, por exemplo).

**Por quê:** teste pela interface sobrevive a refatoração. Teste que quebra quando a implementação
muda e o comportamento não, está testando além da interface e trava o código.

**No grilling:** O que se testa com o banco de verdade ou um substituto local, e o que se mocka? Em que idioma e formato vai o nome do teste?

**No diff:** teste que acessa método privado ou estado interno; mock de módulo nosso; teste chamado
`test1` ou `deve funcionar`.

### P6. Vermelho, verde, refatorar; e nunca os dois juntos

Nas seams combinadas na spec, use `/tdd`: um teste falhando, o mínimo para passar, depois limpar com
o teste verde. Refatoração (muda a forma, não o comportamento) e mudança de comportamento vão em
commits separados.

**Por quê:** refatorar sem teste verde é reescrever às cegas. Misturar as duas coisas no mesmo
commit torna impossível revisar o que mudou de comportamento.

**No grilling:** TDD vale para todo código ou só nas seams da spec? Refatoração vai sempre em commit separado?

**No diff:** commit "refatora X" que também altera teste de comportamento; código de produção sem o
teste que o motivou.

### P7. Nada para um requisito que não existe

Não acrescente parâmetro, opção, camada ou ponto de extensão para uma necessidade que a spec não
tem. Quando a necessidade chegar, ela chega com o exemplo real que diz a forma certa.

**Por quê:** generalidade especulativa (*Speculative Generality*; YAGNI) é código para manter,
testar e entender que não entrega nada hoje e costuma errar a forma do amanhã.

**No grilling:** Há alguma variação que a spec já pede e justifica preparar extensão agora? Se não, a regra é não preparar.

**No diff:** flag que só tem um valor em uso; classe abstrata com uma filha; configuração que
ninguém configura.

### P8. Consistência vence preferência

Siga o padrão que o projeto já usa, mesmo que você prefira outro. Mudar um padrão é decisão: abre
ticket no mapa de Regras de código, e a troca acontece no projeto inteiro, não num arquivo.

**Por quê:** dois jeitos de fazer a mesma coisa dobram o que todo leitor precisa saber. Arquitetura
mantida com disciplina vence a arquitetura "perfeita" abandonada no meio (`/human-architect-mindset`:
"estou otimizando ou traindo um compromisso?").

**No grilling:** Quais escolhas ficam fixas e só mudam por ticket (cliente HTTP, datas, validação, logs)?

**No diff:** segundo cliente HTTP, segundo jeito de tratar erro ou segunda biblioteca de datas sem
ticket que o decidiu.

### P9. Commit conta uma mudança, e a mensagem diz o porquê

Cada commit é uma mudança coerente que passa nos testes. A mensagem diz o que mudou e por quê, na
língua do domínio; o diff já mostra o como.

**Por quê:** é o histórico que o próximo agente lê para entender uma decisão, e o que permite
reverter uma coisa sem levar outra junto.

**No grilling:** Qual o formato e o idioma da mensagem de commit (Conventional Commits ou livre)?

**No diff:** commit "ajustes", "wip" ou "fix" que mistura três assuntos.

---

## SOLID, lido com o vocabulário da casa

SOLID nasceu na orientação a objetos (Robert C. Martin juntou os cinco), mas aqui vale para qualquer
**módulo**: função, classe, pacote. Lido ao pé da letra, ele produz o oposto do que a casa quer: uma
classe por método, uma interface para cada classe, módulos rasos. Cada princípio abaixo diz como ele
se aplica e **quando não se aplica**.

### S. Responsabilidade única: um módulo muda por um motivo só

O motivo é um **ator**: a pessoa ou a área que pede a mudança. Não é "fazer uma coisa só". Um
módulo pode fazer muito, desde que tudo mude pelo mesmo motivo.

**Exemplo:** `Pedido` calcula o total (regra do restaurante, pedida pelo dono) e também gera o
cupom fiscal (regra fiscal, pedida pelo contador). Dois atores: quando a lei muda, mexer no cálculo
do total vira risco. Separe o cupom fiscal.

**Quando não:** quebrar uma regra coesa em dez funções de três linhas, todas públicas, não é SRP:
é módulo raso (P1).

**No grilling:** Quem pede mudança neste produto (o dono, o fiscal, a operação)? Em que módulos o código separa o que cada um pede?

**No diff:** um arquivo editado por motivos sem relação (*Divergent Change*).

### O. Aberto para extensão, fechado para modificação, nas seams reais

Onde uma variação já existe e vai crescer (formas de pagamento, canais de notificação), a variante
nova entra como um **adapter novo** na seam, sem editar o código que já funciona.

**Exemplo:** aceitar Pix entra como `CobrancaPix` que cumpre a interface `Cobranca`, não como mais
um `case "pix"` repetido em quatro `switch` espalhados (*Repeated Switches*).

**Quando não:** não crie ponto de extensão antes da segunda variante (P2, P7). Com uma variação só,
um `if` é mais simples e honesto.

**No grilling:** Quais variações já existem ou estão na spec (formas de pagamento, canais) e merecem uma seam com adapters?

**No diff:** o mesmo `switch` sobre o mesmo tipo em dois ou mais lugares.

### L. Substituição de Liskov: todo adapter cumpre a interface inteira

Quem implementa uma interface cumpre **toda** ela, não só a assinatura: as mesmas invariantes, os
mesmos erros, a mesma ordem e garantias. Quem chama não pode precisar saber qual adapter recebeu.

**Exemplo:** o repositório em memória dos testes não rejeita pedido com número duplicado, e o do
Postgres rejeita. Os testes passam, a produção quebra. O substituto precisa ter o mesmo
comportamento observável que o adapter real.

**Quando não:** não há exceção. Se uma implementação não consegue cumprir a interface, ela não é
daquela interface: prefira composição a herança (*Refused Bequest*).

**No grilling:** O substituto de teste de cada seam roda os mesmos testes de contrato que o adapter real?

**No diff:** subclasse que lança "não suportado" num método herdado; `if (x instanceof Y)` em quem
chama; fake de teste que aceita o que a produção recusa.

### I. Segregação de interface: quem chama conhece só o que usa

A interface é desenhada do ponto de vista de quem chama, e pequena. Quem só lê o cardápio não
depende de uma interface que também edita preço e publica promoção.

**Exemplo:** a tela do garçom recebe `ConsultaDeCardapio` (buscar, listar), não o
`ServicoDeCardapio` inteiro com as operações de administração.

**Quando não:** não corte um módulo profundo em várias interfaces minúsculas que quem chama precisa
juntar de novo. Interface pequena e implementação grande é o mesmo objetivo de P1, não o contrário.

**No grilling:** As interfaces nascem de quem usa (leitura separada da administração) ou é uma interface por módulo?

**No diff:** chamador recebendo um objeto com vinte métodos para usar um; implementação forçada a
escrever métodos vazios.

### D. Inversão de dependência: a regra define a abstração, a infraestrutura obedece

O módulo de regra de negócio depende de uma interface que **ele mesmo define** (a port), e o detalhe
(banco, provedor, framework) a implementa. A seta de dependência aponta para o domínio.

**Exemplo:** o módulo de pagamento declara `Cobranca` com o que precisa (`cobrar`, `estornar`); o
adapter do provedor vive fora do domínio e é ligado na montagem (P3).

**Quando não:** não inverta dependência dentro do processo sem variação (P2). Cálculo puro se chama
direto; container de injeção só se o projeto decidiu por ele.

**No grilling:** A regra de negócio pode importar alguma biblioteca de fora (ORM, framework, SDK)? Quais são exceção aceita?

**No diff:** import de SDK, ORM ou framework dentro do domínio; interface definida no lado da
infraestrutura e importada pelo domínio.

### SOLID em uma tabela

| | Pergunta que o revisor faz ao diff | *Smell* que costuma aparecer |
|---|---|---|
| **S** | Este módulo muda por mais de um ator? | Divergent Change |
| **O** | A variante nova editou código antigo que já funcionava? | Repeated Switches |
| **L** | Algum adapter se comporta diferente do contrato? | Refused Bequest |
| **I** | Quem chama depende de algo que não usa? | (interface gorda) |
| **D** | A regra de negócio importa detalhe de infraestrutura? | (dependência apontando para fora do domínio) |

---

## Roteiro do grilling do mapa de Regras de código

O mapa de Regras de código é organizado por esta base. Cada ticket de grilling cobre um grupo de
princípios e só fecha quando cada princípio do grupo tem decisão.

| Ticket | Princípios | Skills |
|---|---|---|
| *Restrições para as regras* | nenhum: quem mantém, nível do time, quanto o agente escreve, o que já deu errado | `/human-architect-mindset` |
| *Escrita de código* | E1, E2, E6, E7, E8 | `/domain-modeling` |
| *Tipos, erros e validação* | E3, E4, E5 | `/domain-modeling` |
| *Desenho de módulos e SOLID* | P1, P2, P3, P4, S, O, L, I, D | `/codebase-design` |
| *Testes e fluxo de trabalho* | P5, P6, P7, P8, P9 | `/tdd` |

As restrições vêm primeiro e bloqueiam os outros quatro: são elas que justificam adaptar ou afastar.

**Como cada rodada anda**

1. O agente lê o princípio inteiro (regra, porquê, exemplo) e transforma a pergunta-guia **No
   grilling** numa pergunta do projeto: curta, em linguagem simples e com um exemplo real do
   domínio dele, mostrando o que muda em cada opção (regra 10 da constituição).
2. Toda pergunta traz a recomendação: **adotar**, **adaptar** (e como) ou **afastar** (e por quê),
   com a fonte: a seção *Fontes* abaixo para o princípio, a pesquisa de `docs/research/` para o que
   depende da stack, as restrições do primeiro ticket para o que depende do time.
3. Os princípios independentes do grupo vão numa rodada só, numerados. O humano responde; o agente
   não decide por ele.
4. O comentário de resolução do ticket traz uma linha por princípio do grupo (`E3: adaptado, ...`) e
   as `## Fontes`. Princípio sem linha deixa o ticket aberto.
5. Num projeto que já tem código, o ticket *Atrito do código existente* roda o
   `/improve-codebase-architecture` antes dos grupos: cada atrito vira exemplo na pergunta do
   princípio que ele quebra, e a decisão diz se o projeto assume (regra) ou corrige (ticket novo).

**Fechar o mapa**

O ticket *Escrever docs/regras-de-codigo.md* junta as decisões no modelo abaixo. O mapa fecha quando
a tabela de cobertura tem as 22 linhas preenchidas e o humano aprovou o documento.

## Modelo de `docs/regras-de-codigo.md`

```markdown
# Regras de código

Régua do implementador e do revisor. Nasce dos princípios de código da casa
(docs/agents/principios-de-codigo.md); muda só por ticket.

## Garantido pelo lint e pelo formatador

- <ferramenta e versão>: <o que ela garante sozinha>

## Regras

### R1. <a regra, em uma linha>
Nasce de: E1 (adotado | adaptado)
Por quê: <motivo do projeto>
Exemplo: <trecho do próprio projeto>

## Cobertura dos princípios

| Princípio | Decisão | Onde ou por quê |
|---|---|---|
| E1 | adotado | R1 |
| E2 | garantido pelo lint | <regra do lint> |
| E6 | afastado | <o porquê, da restrição ou da pesquisa> |
| ... uma linha para cada um dos 22 (E1 a E8, P1 a P9, S, O, L, I, D) |
```

Princípio **afastado** continua fora da régua dura e fora do julgamento do revisor: o projeto decidiu
não segui-lo. Princípio **adotado** ou **adaptado** vira regra dura pela regra que o cita.

## Fontes

- Robert C. Martin, *Clean Architecture* (2017), cap. 7 a 11: os cinco princípios SOLID; SRP como
  "responsável por um, e só um, ator". Também "The Single Responsibility Principle" (2014),
  blog.cleancoder.com.
- Robert C. Martin, "Design Principles and Design Patterns" (2000): a primeira reunião dos princípios.
- Bertrand Meyer, *Object-Oriented Software Construction* (1988): princípio aberto-fechado e
  separação comando-consulta.
- Barbara Liskov, "Data Abstraction and Hierarchy", OOPSLA 1987; Barbara Liskov e Jeannette Wing,
  "A Behavioral Notion of Subtyping", ACM TOPLAS, 1994: substituição pelo comportamento, não pela
  assinatura.
- John Ousterhout, *A Philosophy of Software Design* (2018): módulos profundos e interfaces pequenas.
- David Parnas, "On the Criteria To Be Used in Decomposing Systems into Modules", CACM, 1972:
  esconder o que pode mudar atrás de uma interface.
- Michael Feathers, *Working Effectively with Legacy Code* (2004): seams.
- Martin Fowler, *Refactoring*, 2ª ed. (2018), cap. 3: os *smells* citados.
- Kent Beck, *Test-Driven Development: By Example* (2002): vermelho, verde, refatorar.
- Andrew Hunt e David Thomas, *The Pragmatic Programmer*, ed. de 20 anos (2019): DRY como
  duplicação de conhecimento.
- Alexis King, "Parse, don't validate" (2019), lexi-lambda.github.io: converter na borda em tipo
  que garante a validade.
- Skills do método: `/codebase-design` (vocabulário, teste da exclusão, disciplina de seam),
  `/code-review` (lista de *smells*), `/human-architect-mindset` (compromisso com o padrão escolhido).
