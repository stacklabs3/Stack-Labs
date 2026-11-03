# Comparação da revisao-por-modelo

Experimento da issue [Stack-Labs#53](https://github.com/stacklabs1/Stack-Labs/issues/53). A pergunta é
se o trecho que o watchdog envia ao modelo, sem auxiliares nem `beforeEach`, gera suspeitos falsos, e
se dar mais contexto deixa RK errado passar.

O `comparar-revisao` (Rust, em `src/main.rs`) não altera o watchdog. Ele monta os prompts por conta própria e chama o `claude -p`
com as mesmas flags, **sem cache**. Só lê testes JS e TS (`it` e `test`).

## Variantes

| Variante | O que o modelo vê |
|---|---|
| `atual` | O prompt de hoje, igual ao do watchdog. |
| `auxiliares` | O trecho atual, mais as importações, as declarações de topo que o teste usa (até 3 níveis) e os `beforeEach`/`beforeAll` dos `describe` que o envolvem. Fica sem o título do `describe`. |
| `arquivo` | O arquivo inteiro, numerado, com a linha do teste a julgar. |

**Gabarito:** cada teste ganha uma cópia em que o RK é trocado, no arquivo e no prompt, por outro risco
do registro escolhido ao acaso (semente fixa). A resposta esperada é `exercita=false`.

## Rodar

Requisitos: `cargo` e o `claude` autenticado no PATH. Compile uma vez:

```bash
cargo build --release --manifest-path /caminho/do/Stack-Labs/desenvolvimento/agentes/experimentos/revisao-por-modelo/Cargo.toml
```

Depois, rode dentro do checkout do projeto, por exemplo o organiza-ai:

```bash
/caminho/do/Stack-Labs/desenvolvimento/agentes/experimentos/revisao-por-modelo/target/release/comparar-revisao --projeto . --simular src/estado.test.ts src/ultimo-botao.test.ts src/tela-de-entrada.test.tsx src/estado-da-fila.test.ts
```

`--help` lista as opções.

O `--simular` só monta os prompts e mede o tamanho, sem gastar nada. Confira alguns em
`.scratch/comparacao-revisao-por-modelo/prompts/`. Depois, rode sem `--simular`. Com 26 testes, são
cerca de 156 chamadas (26 × 2 × 3).

## Ler o resultado

Os arquivos ficam em `.scratch/comparacao-revisao-por-modelo/`:

- `resumo.md`: por variante, os suspeitos, o gabarito pego, as chamadas sem veredito e o custo médio e máximo.
- `resultados.csv`: uma linha por chamada, com o motivo dado pelo modelo.
- `revisao-humana.csv`: cada teste real com os três vereditos lado a lado. Preencha `humano_exercita`
  para contar os suspeitos falsos e os aprovados errados de cada variante.

Uma variante só vale a pena se reduzir os suspeitos falsos, continuar pegando os RKs trocados e não
estourar o teto de US$ 0,05 por chamada. Se um RK trocado passar, confira o motivo: o teste pode cobrir
também aquele risco.

## Limites

- A detecção de auxiliares e de ganchos é feita por regex e recuo, não por um parser de TypeScript.
- O roteiro porta a lógica do watchdog e pode divergir se ele mudar. As constantes do topo de `src/main.rs` estão copiadas de `revisao_por_modelo.rs`.
