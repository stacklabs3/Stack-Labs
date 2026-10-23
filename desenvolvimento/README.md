# Método de desenvolvimento com IA

Como a StackLabs constrói software com agentes de IA, em todo projeto. O método é o do Matt Pocock,
com o `/wayfinder` como porta de entrada, mais as regras da casa.

> **Para usar num projeto:** instale as skills uma vez na sua máquina
> ([instruções](agentes/README.md#instalar-num-projeto-novo)). Projeto novo: rode
> **`/setup-stacklabs`**. Projeto que já está em execução:
> **[`/adotar-metodo-stack-labs`](humanos/adotar-em-projeto-existente.md)**, que adota por etapas.

| Pasta | Para quem | O que é |
|---|---|---|
| [humanos/](humanos/README.md) | Sócios e funcionários | O guia: como decidir, aprovar e conduzir os agentes |
| [agentes/](agentes/README.md) | Os agentes de IA | A constituição, os watchdogs e as skills, prontos para copiar em cada projeto |
| [GLOSSARY.md](GLOSSARY.md) | Os dois | A linguagem do método, fonte única dos termos |
| [adr/](adr/) | Os dois | As decisões do próprio método e o porquê |

## As regras da casa, em uma linha cada

1. Todo projeto abre quatro **mapas iniciais**: Protótipo, Arquitetura, Fases e Infra.
2. Todo mapa decide por **grilling**: o agente pergunta e recomenda, o humano responde.
3. **Nenhuma recomendação sem pesquisa**: o agente cita a fonte primária ou não recomenda.
4. Aparência se prototipa com a família **`*-prototipo`** e só vira código com aprovação explícita.
5. **Uma janela por trabalho**: trabalho novo ou continuação abre com o kickoff do **`/nova-sessao`**;
   assunto paralelo vai para uma **sessão satélite** e volta pelo **`/voltar-a-principal`**.
6. Uma issue só chega a **`ready-for-agent`** por uma rota do método e depois da **definição de
   pronto** ([humanos](humanos/caminho-ate-ready-for-agent.md) ·
   [agentes](agentes/modelo-de-projeto/docs/agents/caminho-ate-ready-for-agent.md)).
7. Issues `ready-for-agent` vão para a **orquestração**: uma sessão orquestradora dispara, em
   sequência, quem implementa, quem revisa e quem confere com o spec, numa branch de integração.
8. **Watchdogs** barram o que foge do método, na sessão, no push e na PR.

Projeto de referência: [appetito_v2](https://github.com/stacklabs1/appetito_v2).
