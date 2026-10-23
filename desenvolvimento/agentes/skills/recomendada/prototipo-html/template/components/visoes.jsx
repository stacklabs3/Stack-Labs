// Protótipo — Abas (visões) e rotinas.
// O único arquivo com a organização de cada projeto: quais abas existem, em que
// superfícies cada uma se confere, com que tema abre, e quais rotinas ela tem.
// As skills *-prototipo são genéricas; o que é do projeto entra aqui.
//
// TEMPLATE: troque as duas abas de exemplo pelas do projeto.

/* Meta de toques do projeto: quantos toques a rotina principal pode levar, da
   primeira à última tela. Cada rotina pode declarar a sua; sem isso, vale esta. */
const META_DE_TOQUES = 3;

/* Uma aba por superfície ou papel. `superficies` são chaves de SURFACES, na ordem
   em que aparecem no painel; a primeira é a de abertura. `temaPadrao` é o tema com
   que a aba abre ('light' ou 'dark'). */
const VISOES = [
  { id: 'cliente', rotulo: 'Cliente', superficies: ['ios', 'android', 'desktop'], temaPadrao: 'light' },
  { id: 'balcao',  rotulo: 'Balcão',  superficies: ['tabletDeitado', 'tv'],      temaPadrao: 'dark' },
];

/* Uma rotina é o trabalho inteiro de alguém, em telas na ordem em que acontecem.
   `telas` são rotas de router.jsx, com os params da tela quando ela precisa. */
const ROTINAS = [
  {
    id: 'ver-item',
    visao: 'cliente',
    rotulo: 'Ver um item',
    telas: [{ rota: 'HOME' }, { rota: 'DETAIL', params: { id: 1 } }],
  },
  {
    id: 'conferir-item',
    visao: 'balcao',
    rotulo: 'Conferir um item',
    telas: [{ rota: 'HOME' }, { rota: 'DETAIL', params: { id: 2 } }],
    metaDeToques: 2,
  },
];

const visaoPorId = (idDaVisao) => VISOES.find((visao) => visao.id === idDaVisao) || VISOES[0];

const rotinasDaVisao = (idDaVisao) => ROTINAS.filter((rotina) => rotina.visao === idDaVisao);

const metaDaRotina = (rotina) => rotina.metaDeToques ?? META_DE_TOQUES;

/* Dúvida de aparência em aberto (A/B). Quando uma tela da rotina tem dúvida real entre
   duas formas, a rotina declara `duvidas` e a tela lê a escolha com useOpcao(id):

     duvidas: [{ id: 'erro-da-vitrine', pergunta: 'Como aparece o erro?',
                 opcoes: { A: 'Aviso no topo', B: 'Tela inteira' } }]

   O painel troca a opção, os prints saem das duas e os toques se contam em cada uma.
   Escolhida a opção, a perdedora e a dúvida saem do código antes do merge. */
const duvidasDaRotina = (rotina) => rotina.duvidas || [];

const duvidasDaVisao = (idDaVisao) => rotinasDaVisao(idDaVisao).flatMap(duvidasDaRotina);
