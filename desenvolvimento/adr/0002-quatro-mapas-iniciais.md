# Todo projeto abre quatro mapas iniciais antes do código

Todo projeto começa abrindo quatro mapas do wayfinder: **Protótipo**, **Arquitetura**, **Fases** e
**Infra**. Havia a opção de um mapa único de escopo ou de decidir arquitetura e infra conforme a
necessidade surgisse. Escolhemos quatro mapas separados porque são quatro destinos diferentes, com
donos e ritmos diferentes: o protótipo conversa com o cliente, a arquitetura e a infra com o time
técnico, e as fases amarram as três. Num mapa só, as decisões técnicas ficavam esperando as de produto.

**Consequências:** nenhuma issue de implementação é ativada enquanto os quatro mapas não existirem,
e enquanto os mapas de Protótipo, Arquitetura e Fases não estiverem fechados (o watchdog
`guarda-bash` barra). O mapa de Protótipo fecha com o **desenho inicial** aprovado; protótipos que
surgirem depois entram como tickets nos mapas de cada esforço. O mapa de Infra é **informativo**:
responde que infra vamos usar, quanto custa e o que teremos que construir em volta, para dar
conhecimento de causa. É leve, não precisa fechar e não trava a implementação. A divisão das fases
(front, backend, integração ou outra) é decidida com o desenvolvedor que vai conduzir, no grilling;
a casa não impõe uma ordem.
