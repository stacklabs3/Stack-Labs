# Princípios de código: SOLID (S, O, L, I, D)

Parte dos [princípios de código](../principios-de-codigo.md).

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

**No validador:** `revisor`.

### O. Aberto para extensão, fechado para modificação, nas seams reais

Onde uma variação já existe e vai crescer (formas de pagamento, canais de notificação), a variante
nova entra como um **adapter novo** na seam, sem editar o código que já funciona.

**Exemplo:** aceitar Pix entra como `CobrancaPix` que cumpre a interface `Cobranca`, não como mais
um `case "pix"` repetido em quatro `switch` espalhados (*Repeated Switches*).

**Quando não:** não crie ponto de extensão antes da segunda variante (P2, P7). Com uma variação só,
um `if` é mais simples e honesto.

**No grilling:** Quais variações já existem ou estão na spec (formas de pagamento, canais) e merecem uma seam com adapters?

**No diff:** o mesmo `switch` sobre o mesmo tipo em dois ou mais lugares.

**No validador:** `revisor`.

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

**No validador:** `teste-de-contrato-por-port`.

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

**No validador:** `revisor`.

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

**No validador:** `ast-grep` com o modelo `dominio-sem-infra.yml`.

### SOLID em uma tabela

| | Pergunta que o revisor faz ao diff | *Smell* que costuma aparecer |
|---|---|---|
| **S** | Este módulo muda por mais de um ator? | Divergent Change |
| **O** | A variante nova editou código antigo que já funcionava? | Repeated Switches |
| **L** | Algum adapter se comporta diferente do contrato? | Refused Bequest |
| **I** | Quem chama depende de algo que não usa? | (interface gorda) |
| **D** | A regra de negócio importa detalhe de infraestrutura? | (dependência apontando para fora do domínio) |

## Fontes

- Robert C. Martin, *Clean Architecture* (2017), cap. 7 a 11: os cinco princípios; SRP como
  "responsável por um, e só um, ator". Também "The Single Responsibility Principle" (2014),
  blog.cleancoder.com.
- Robert C. Martin, "Design Principles and Design Patterns" (2000): a primeira reunião dos princípios.
- Bertrand Meyer, *Object-Oriented Software Construction* (1988): aberto-fechado.
- Barbara Liskov, "Data Abstraction and Hierarchy", OOPSLA 1987; Barbara Liskov e Jeannette Wing,
  "A Behavioral Notion of Subtyping", ACM TOPLAS, 1994: substituição pelo comportamento.
- Martin Fowler, *Refactoring*, 2ª ed. (2018), cap. 3: Divergent Change, Repeated Switches,
  Refused Bequest.
