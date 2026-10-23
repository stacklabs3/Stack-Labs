# Práticas da linguagem: os 22 princípios em TypeScript, Java e Rust, e o que só cada uma tem

As principais linguagens da casa são TypeScript e Java, e o Rust passou a ser usado (ADR 0011). Os
22 princípios (ADR 0008) valem para todas, mas a regra de máquina que confere cada um muda de
linguagem para linguagem, e os modelos do validador só traziam TypeScript e Python. Uma pesquisa por
linguagem, em fonte primária (`research/2026-10-23-praticas-typescript.md`, `-java.md`, `-rust.md`),
ligou cada prática a um princípio e achou práticas que nenhum dos 22 cobre: `unsafe` e pânico em Rust,
recurso aberto e `equals`/`hashCode` em Java, as regras do React em TypeScript. Havia duas saídas:
fazer delas princípios novos da casa, mudando a tabela de cobertura de 22 para N, ou tratá-las como
**práticas da linguagem**. O humano escolheu a segunda: o que só uma linguagem tem não é princípio da
casa, e a tabela dos 22 continua igual em todo projeto.

**Consequências:** cada linguagem tem o seu arquivo em `docs/agents/principios-de-codigo/`
(`typescript.md`, `java.md`, `rust.md`), com a linha de cada princípio, as ressalvas e as práticas
só dela, com ID (`TS<n>`, `JV<n>`, `RS<n>`). O grilling decide cada prática no ticket *Lint e
formatador*; a adotada vira regra com o ID em `principio`, e a `cobertura` exige a linha dela na
tabela *Práticas da linguagem* do `docs/regras-de-codigo.md`, sem tocar nos 22. "Não calar o
verificador" vale para todas as linguagens e entrou no E4, não como princípio novo. As práticas
passaram pelas skills `/codebase-design` e `/human-architect-mindset`: interface com uma implementação
só é seam hipotético (o par `Service`/`ServiceImpl` do Spring), o teste mora na interface (o lint que
empurra o teste para dentro do módulo ficou fora), limite de tamanho mede a função e não desenha o
módulo, e toda ferramenta nas receitas tem versão fixa, porque a troca de flag do jscpd fez a receita
antiga reprovar todo projeto sem ninguém notar. Modelos por linguagem entram em
`.claude/watchdogs/modelos/` como referência; o projeto copia só o que decidiu. O modelo de TypeScript
fixa o TS 6.0 enquanto o `typescript-eslint` não aceitar o TS 7, com o `oxlint` como alternativa
documentada. Linguagem nova da casa ganha o seu arquivo e o seu prefixo por PR no Stack-Labs.
