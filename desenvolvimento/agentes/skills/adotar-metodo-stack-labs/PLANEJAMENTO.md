# Planejamento: passagem para completo

Nesta etapa só falta o código passar a exigir issue ativa. O trabalho do dia segue o método normal;
este ramo confere se o projeto está pronto para a última etapa.

## 7. Passagem para completo

Mostre ao humano, com evidência, cada condição:

- as PRs abertas antes da adoção foram mergeadas ou fechadas;
- ao menos uma issue foi do `ready-for-agent` à PR pelo método, sem contorno de watchdog;
- o `/auditar-metodo` não aponta desvio aberto.

Condição que falha vira o próximo passo do plano, não exceção. Com as três certas, ele faz a PR que
muda `etapaDeAdocao` para `completo`. Pronto quando ela está mergeada: o projeto segue o método
inteiro.
