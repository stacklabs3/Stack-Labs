# Nenhuma recomendação da IA sem pesquisa que a sustente

O agente só recomenda (no grilling, num ticket, num review) o que uma pesquisa em fonte primária
sustenta, e cita a fonte junto da recomendação. Havia a opção de confiar no conhecimento do modelo e
pesquisar só quando o humano pedisse. Descartamos porque o modelo recomenda com a mesma confiança o
que sabe e o que imagina, e versões, preços e limites de serviços mudam mais rápido que o treinamento
dele. Uma escolha de stack ou de infra com base num palpite custa meses.

**Consequências:** todo ticket de grilling, tarefa ou pesquisa fecha com uma seção `## Fontes` no
comentário de resolução, com link para `docs/research/` ou para a fonte primária (o watchdog
`guarda-bash` barra o fechamento sem ela). No grilling, a pergunta que precisa de pesquisa espera o
subagente `/research` voltar; as demais seguem. Tickets de protótipo ficam fora dessa regra: a
evidência deles é o próprio protótipo e a aprovação explícita.
