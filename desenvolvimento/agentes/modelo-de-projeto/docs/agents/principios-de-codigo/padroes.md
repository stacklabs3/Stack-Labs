# Princípios de código: padrões de desenvolvimento (P1 a P9)

Parte dos [princípios de código](../principios-de-codigo.md). Como desenhar módulos, testar e entregar.

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

**No validador:** `revisor` (teste da exclusão).

### P2. Seam só onde algo varia de verdade

Uma interface (port) com uma implementação só é indireção, não design. Crie a seam quando há duas
variações reais: produção e teste, dois provedores, dois canais. Dependência dentro do processo
(cálculo puro, memória) não precisa de seam: chame direto.

**Por quê:** cada seam é uma interface a mais para aprender e manter. "Um adapter é uma seam
hipotética; dois adapters, uma seam real."

**No grilling:** Criamos interface para banco e provedores desde o começo, ou só quando houver a segunda implementação (o substituto de teste conta)?

**No diff:** `interface ServicoDeX` com uma única `ServicoDeXImpl`; fábrica ou container de injeção
para algo que nunca varia.

**No validador:** `revisor`.

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

**No validador:** `ast-grep` com os modelos `dominio-sem-infra.yml` e `ambiente-so-na-montagem.yml`.

### P4. O que muda junto fica junto

Organize pelo conceito do domínio (pedido, cardápio, pagamento), não pela camada técnica, salvo
decisão contrária da Arquitetura. Uma mudança de regra deveria tocar poucos arquivos vizinhos.

**Por quê:** mudança espalhada por muitos arquivos (*Shotgun Surgery*) é o custo mais caro de
manter, para humano e para agente, que precisa carregar tudo no contexto.

**No grilling:** As pastas se organizam por conceito do negócio (pedido, cardápio) ou por camada técnica (controllers, services)?

**No diff:** uma regra nova que obriga a editar cinco pastas distantes.

**No validador:** `comando` com o dependency-cruiser em JS/TS e o ArchUnit em Java; em Rust, crate por conceito no workspace; o resto, `revisor`.

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

**No validador:** `ast-grep` com o modelo `dominio-sem-infra.yml` apontado para os testes e para os módulos nossos que não se mockam; o nome do teste fica com o revisor.

### P6. Vermelho, verde, refatorar; e nunca os dois juntos

Nas seams combinadas na spec, use `/tdd`: um teste falhando, o mínimo para passar, depois limpar com
o teste verde. Refatoração (muda a forma, não o comportamento) e mudança de comportamento vão em
commits separados.

**Por quê:** refatorar sem teste verde é reescrever às cegas. Misturar as duas coisas no mesmo
commit torna impossível revisar o que mudou de comportamento.

**No grilling:** TDD vale para todo código ou só nas seams da spec? Refatoração vai sempre em commit separado?

**No diff:** commit "refatora X" que também altera teste de comportamento; código de produção sem o
teste que o motivou.

**No validador:** `revisor`.

### P7. Nada para um requisito que não existe

Não acrescente parâmetro, opção, camada ou ponto de extensão para uma necessidade que a spec não
tem. Quando a necessidade chegar, ela chega com o exemplo real que diz a forma certa.

**Por quê:** generalidade especulativa (*Speculative Generality*; YAGNI) é código para manter,
testar e entender que não entrega nada hoje e costuma errar a forma do amanhã.

**No grilling:** Há alguma variação que a spec já pede e justifica preparar extensão agora? Se não, a regra é não preparar.

**No diff:** flag que só tem um valor em uso; classe abstrata com uma filha; configuração que
ninguém configura.

**No validador:** `revisor`.

### P8. Consistência vence preferência

Siga o padrão que o projeto já usa, mesmo que você prefira outro. Mudar um padrão é decisão: abre
ticket no mapa de Regras de código, e a troca acontece no projeto inteiro, não num arquivo.

**Por quê:** dois jeitos de fazer a mesma coisa dobram o que todo leitor precisa saber. Arquitetura
mantida com disciplina vence a arquitetura "perfeita" abandonada no meio (`/human-architect-mindset`:
"estou otimizando ou traindo um compromisso?").

**No grilling:** Quais escolhas ficam fixas e só mudam por ticket (cliente HTTP, datas, validação, logs)?

**No diff:** segundo cliente HTTP, segundo jeito de tratar erro ou segunda biblioteca de datas sem
ticket que o decidiu.

**No validador:** `ast-grep` com o modelo `dominio-sem-infra.yml` apontado para o código todo e as bibliotecas que o padrão fixo exclui.

### P9. Commit conta uma mudança, e a mensagem diz o porquê

Cada commit é uma mudança coerente que passa nos testes. A mensagem diz o que mudou e por quê, na
língua do domínio; o diff já mostra o como.

**Por quê:** é o histórico que o próximo agente lê para entender uma decisão, e o que permite
reverter uma coisa sem levar outra junto.

**No grilling:** Qual o formato e o idioma da mensagem de commit (Conventional Commits ou livre)?

**No diff:** commit "ajustes", "wip" ou "fix" que mistura três assuntos.

**No validador:** `mensagem-de-commit`.

## Fontes

- John Ousterhout, *A Philosophy of Software Design* (2018): módulos profundos (P1).
- David Parnas, "On the Criteria To Be Used in Decomposing Systems into Modules", CACM, 1972:
  esconder o que pode mudar atrás de uma interface (P1, P4).
- Michael Feathers, *Working Effectively with Legacy Code* (2004): seams (P2).
- Kent Beck, *Test-Driven Development: By Example* (2002): vermelho, verde, refatorar (P6).
- Martin Fowler, *Refactoring*, 2ª ed. (2018), cap. 3: Middle Man, Shotgun Surgery, Speculative
  Generality.
- Skills do método: `/codebase-design` (vocabulário, teste da exclusão, disciplina de seam),
  `/human-architect-mindset` (compromisso com o padrão escolhido, P8).
