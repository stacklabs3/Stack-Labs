# Transcrição do vídeo hWe-j78vNpc

Existe uma técnica que venho usando há

algum tempo que eu achava ser entediante

e óbvia demais para postar a respeito.

Então, em um horário de atendimento da

coorte que eu estava ministrando esta

semana, eu a mostrei e as pessoas

disseram: "uau, isso é muito legal". Eu

nunca pensei que poderia ser tão simples

. Depois, postei sobre isso no Twitter e

foi um sucesso estrondoso. Portanto,

obviamente, é algo sobre o qual as

pessoas precisam saber. E essa técnica

resolve um problema: como trabalhar com

agentes em vários repositórios ao mesmo

tempo. Então, vamos preparar isso.

Imagine que sua organização seja assim,

como a maioria das organizações é. Elas

possuem muitos repositórios diferentes,

todos com preocupações distintas. Temos

um que é a vitrine da loja, um que é a

API de pagamentos, um que é o painel

administrativo e mais 20 repositórios

que você pode precisar acessar

ocasionalmente. Isso é fácil quando você

precisa fazer alterações em apenas um

desses repositórios, então você

simplesmente executa o agente dentro da

vitrine. É até bem fácil se você

perceber, ao trabalhar na vitrine, que

precisa fazer alterações na API de

pagamentos, porque você pode fazer seu

agente trabalhar em vários diretórios

simultaneamente. Isso significa que você

acabaria com um pull request dentro da

vitrine da Acme e outro dentro da API de

pagamentos, tudo a partir da mesma

sessão.

Tudo bem, isso funciona, mas a questão

passa a ser: o que você faz com os dados

, com os arquivos de direcionamento e

com o que você quer que o agente tenha

em cada sessão?

O que acontece frequentemente é que você

terá suas habilidades, suas ferramentas

, seu agents.md e talvez até links de

wiki e coisas do tipo duplicados em

vários repositórios. Isso significa que

, sempre que você iniciar uma sessão de

agente, terá acesso a essas coisas.

Então, se você iniciá-lo na vitrine,

ainda terá as mesmas habilidades e

ferramentas. Mas isso é um enorme

transtorno, porque você precisa manter

todas essas habilidades e ferramentas

sincronizadas em diferentes repositórios

, talvez em 30 ou 40 deles. Você poderia

colocar todas essas habilidades e

ferramentas na máquina local das pessoas

, no diretório de usuário, para que

ficassem acessíveis a partir de cada um

desses repositórios. Mas, como fica meio

invisível para o usuário, isso significa

que elas não serão atualizadas com o

tempo e provavelmente ficarão defasadas

em relação à configuração global. Além

disso, penso no diretório do usuário

como o lugar onde ele pode fazer

alterações na configuração que se

adaptem à forma como ele trabalha, em

vez de como a equipe quer que ele

trabalhe. E nenhuma dessas soluções

parece ser muito boa, o que significa

que as pessoas que trabalham em

configurações multi-repo sentem que não

têm boas opções. Portanto, a solução que

eu proponho é chamada de home repo. Essa

é uma técnica que venho usando há algum

tempo, onde tenho um único repositório,

que é a base onde todas as minhas

habilidades, todos os meus arquivos de

direcionamento, um wiki e várias

ferramentas personalizadas residem. Isso

significa que, sempre que executo meu

agente, eu o executo sempre dentro do

home repo.

Isso quer dizer que o agente operando

dentro do home repo, assim como em um

repositório normal, consegue acessar a

vitrine e a API de pagamentos, operar em

vários repositórios diferentes ao mesmo

tempo e reunir todo o contexto de que

precisa.

Então você provavelmente está pensando:

"por favor, podemos ver seu home repo?"

Bem, aqui está ele. As coisas principais

às quais precisamos prestar atenção aqui

são, claro, o arquivo claw.md, que

veremos em um minuto, mas também,

crucialmente, as habilidades no topo

aqui, em "clawed skills". Essas

habilidades são privadas. Em outras

palavras, são habilidades que não quero

liberar por qualquer motivo, pois são

extremamente boas e sensíveis para o meu

negócio. E isso significa que elas estão

disponíveis em todas as sessões.

Portanto, se sua organização tem uma

lista de habilidades que você deseja

adicionar, você deve adicioná-las dentro

do home repo aqui mesmo. Este também é

um lugar onde você pode configurar

definições de uma forma versionada.

Então, se você estiver executando um

agente consistente aqui, pode ter

configurações no seu settings.json que

se aplicam imediatamente a toda a sua

organização. As principais estruturas

vêm do claw.md aqui ou do agents.md, e

dentro dele você pode ver que há

ponteiros de navegação para todos os

repositórios importantes com os quais

lido diariamente. Então, se houver

qualquer mudança de habilidade, por

exemplo, ela estará em repos AI skills.

Qualquer coisa que precise de

atualização no meu gerenciador de vídeos

de cursos, que provavelmente é meu

repositório mais ativo em termos de

commits, vai para lá. Qualquer coisa

para o projeto da turma 05, qualquer

coisa para o repositório do curso

intensivo de programação com IA. Então,

quando falo sobre esses links do home

repo para outros repositórios, é assim

que eles são feitos. Literalmente apenas

um caminho dentro do agents.md.

Em uma máquina nova, o que o agente faz

é: se o sandcastle não existir neste

local, ele o clona. E então, se houver

vários agentes rodando ao mesmo tempo,

ele simplesmente usará uma árvore de

trabalho dentro desse mesmo repositório

. Basicamente, é configuração zero.

Funciona sozinho e, sabe, eventualmente

você precisa fazer uma limpeza normal na

árvore de trabalho conforme avança. Se

eu quiser adicionar outro repositório à

lista, basta editar o agents.md e ele

entra na mesma configuração. Essa

configuração funciona para talvez 20, 30

, 40 repositórios, possivelmente. Não

escalaria para, digamos, 400

repositórios. Mas para essas situações,

eu apenas adicionaria um pouco de

revelação progressiva. Dentro do seu

agents.md, em vez de linkar os

repositórios brutos, você diria: ok,

aqui estão os repositórios front-end,

aqui os back-end e aqui os internos para

ferramentas internas.

E então, lá dentro, você pode

simplesmente criar links para os

repositórios. Isso significa que o

agente conseguiria encontrar o

repositório certo com base em táticas

simples de sistema de arquivos. E, a

propósito, vale dizer que eu adoro

monorepos. Se tudo isso pudesse ficar em

um único repositório, eu faria. Sabe, eu

trabalhava na Vercel como dev rel de

turbo repo. Sabe, eu curto muito

monorepos. Mas nem sempre é possível usá

-los, especialmente se você precisar de

uma mistura onde alguns repositórios

sejam públicos e outros privados.

Então, o que eu diria é: se você puder

mover tudo isso para um único

repositório, ótimo.

Se não puder, e provavelmente não

conseguirá por questões políticas, então

um "home repo" é o caminho. Há mais uma

coisa a dizer sobre "home repos": notei

que pude ser muito mais ambicioso com as

ferramentas que criei para meus agentes

desde que adotei um. Chamo meu home repo

de minha wiki pessoal, porque é uma wiki

enorme com todos os dados um tanto

sensíveis para o meu negócio.

Fica aqui dentro, na wiki, e é

essencialmente uma wiki para LLM, estilo

Carpathy, com vários arquivos markdown

de coisas que meu público acha legal ou

interessante. Vou falar sobre como gero

essa wiki e como a mantenho atualizada

em um vídeo futuro, porque as pessoas

são obcecadas por essa ideia.

Mas quero dizer, é muito importante para

o meu negócio porque consigo consultá-

la, extrair dados realmente ricos que

vão para a criação de cursos, de

conteúdo, etc. E por ter esse home repo

, ele se torna o lugar óbvio para

abrigar essa wiki.

Eu também andei escondendo o jogo sobre

ferramentas, porque elas são

absolutamente essenciais, dando ao seu

agente ferramentas personalizadas que

ele pode usar para interagir com o que

você usa no seu negócio. Se eu rodar a

ajuda do wiki dentro do diretório do meu

wiki pessoal, podemos ver que há muita

coisa aqui integrando com Gmail, Slack,

Discord, YouTube, GitHub, Gcal, AI Hero

, Cold Turkey para, tipo, fazer meus

bloqueios, coisas assim. Eu publico

relatórios HTML avulsos, adiciono ao

butter, um buffer do butter. Essas

coisas conseguiram crescer organicamente

dentro do wiki pessoal e somar a esse

ecossistema enorme que consigo construir

, baseado no fato de que centralizei

tudo e que sempre rodo tudo em um único

local. Isso significa que sou capaz de

fazer coisas malucas, como dizer: "Ok,

procure essa mensagem no Discord, então

vá para o repositório do curso intensivo

, faça uma correção lá usando todas as

habilidades que você já possui". A

partir daí, você pode usar minha

habilidade CVM dentro do meu gerenciador

de vídeos de cursos, onde quer que ele

esteja aqui em cima, e usar isso para ir

lá e realmente editar o curso para fazer

a correção. Em outras palavras, o

repositório principal se torna a

ferramenta que conecta todas as

diferentes partes do seu negócio, não

apenas o código, mas dando ao agente

acesso a fontes de dados ricas e

ferramentas poderosas que ele pode usar

para trabalhar melhor. Então, como você

configura um para si mesmo? Bem, eu

escrevi um artigo chamado "O Padrão de

Repositório Principal"(Home Repo Pattern

). E neste artigo, incluí um prompt aqui

embaixo que você pode usar para

configurar o seu. Ele basicamente te

questiona sobre sua própria configuração

, analisa todos os repositórios com os

quais você precisa se preocupar e, então

, começa a mover as coisas para um

repositório principal. E, claro, o link

para isso está aqui embaixo. E, a

propósito, se você curte o que estou

postando aqui, você realmente vai

aproveitar meu curso intensivo de

programação com IA no seu próprio ritmo

. Esta é a introdução perfeita se você

quer aprender sobre agentes, se nunca

tentou usar agentes antes, ou apenas

sente que está ficando para trás neste

mundo grande e emocionante. Mas enfim,

esse é o repositório principal, o padrão

que eu achei ser básico demais para

postar, mas agora que falo sobre ele,

percebo que, ok, ele realmente resolve

vários problemas que as pessoas estão

tendo. Então aproveite. Divirta-se

configurando e aproveite seu novo lar,

eu acho. Bom trabalho e vejo você na

próxima.
