# Todo projeto abre quatro mapas iniciais antes do código

Todo projeto começa abrindo quatro mapas do wayfinder: **Protótipo**, **Arquitetura**, **Fases** e
**Infra**. Havia a opção de um mapa único de escopo ou de decidir arquitetura e infra conforme a
necessidade surgisse. Escolhemos quatro mapas separados porque são quatro destinos diferentes, com
donos e ritmos diferentes: o protótipo conversa com o cliente, a arquitetura e a infra com o time
técnico, e as fases amarram as três. Num mapa só, as decisões técnicas ficavam esperando as de produto.

**Consequências:** nenhuma issue de implementação é ativada enquanto os quatro mapas não existirem,
e enquanto os mapas de Arquitetura e de Fases não estiverem fechados (o watchdog `guarda-bash` barra).
O mapa de Infra pode fechar depois, até o primeiro deploy. O mapa de Protótipo segue aberto enquanto
houver rotina a prototipar.
