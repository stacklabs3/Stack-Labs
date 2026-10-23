// Protótipo — Dicionário i18n (pt-BR default, en, es).
// TEMPLATE: substitua as strings de exemplo pelas strings reais das telas.
// Projeto monolíngue: mantenha só 'pt-BR' — a estrutura evita refactor depois.

const TRANSLATIONS = {
  'pt-BR': {
    appName: 'Protótipo',

    // Chrome do showcase
    modoPrototipo: 'Protótipo',
    modoShowcase: 'Showcase',

    // Painel de controle flutuante
    painelTitulo: 'Controles do showcase',
    painelSuperficie: 'Superfície',
    painelTema: 'Tema',
    painelEstado: 'Estado',
    painelReset: 'Resetar',
    painelAbas: 'Abas do protótipo',
    painelRotina: 'Rotina',
    painelOpcao: 'Opção',
    rotinaNenhuma: 'Nenhuma',
    rotinaToques: '{toques} de {meta} toques',
    rotinaConcluida: 'concluída',
    rotinaMeta: 'meta de {meta} toques',
    rotinaAcimaDaMeta: 'acima da meta',
    conexaoVoltou: 'A conexão voltou. O que estava esperando já foi enviado.',
    temaClaro: 'Claro',
    temaEscuro: 'Escuro',

    // Telas de exemplo
    homeTitulo: 'Início',
    homeSubtitulo: 'TEMPLATE — substitua pelas telas reais',
    detalheTitulo: 'Detalhe',
    detalheDescricao: 'TEMPLATE — descrição do item selecionado',

    ctaPrimary: 'Ação principal',
    ctaSecondary: 'Ação secundária',
    emptyTitle: 'Nada por aqui ainda',
    emptyBody: 'Comece adicionando seu primeiro item.',
    emptyCta: 'Adicionar',
    errorTitle: 'Algo deu errado',
    errorBody: 'Não foi possível concluir a operação. Tente novamente.',
    errorCta: 'Tentar de novo',
    semInternetTitulo: 'Sem internet',
    homeContinua: 'Você continua vendo os itens.',
    homeEspera: 'A ação principal espera a conexão voltar.',
  },

  'en': {
    appName: 'Prototype',

    modoPrototipo: 'Prototype',
    modoShowcase: 'Showcase',

    painelTitulo: 'Showcase controls',
    painelSuperficie: 'Surface',
    painelTema: 'Theme',
    painelEstado: 'State',
    painelReset: 'Reset',
    painelAbas: 'Prototype tabs',
    painelRotina: 'Routine',
    painelOpcao: 'Option',
    rotinaNenhuma: 'None',
    rotinaToques: '{toques} of {meta} taps',
    rotinaConcluida: 'done',
    rotinaMeta: 'goal of {meta} taps',
    rotinaAcimaDaMeta: 'over the goal',
    conexaoVoltou: 'You are back online. Everything that was waiting has been sent.',
    temaClaro: 'Light',
    temaEscuro: 'Dark',

    homeTitulo: 'Home',
    homeSubtitulo: 'TEMPLATE — replace with the real screens',
    detalheTitulo: 'Detail',
    detalheDescricao: 'TEMPLATE — description of the selected item',

    ctaPrimary: 'Primary action',
    ctaSecondary: 'Secondary action',
    emptyTitle: 'Nothing here yet',
    emptyBody: 'Start by adding your first item.',
    emptyCta: 'Add',
    errorTitle: 'Something went wrong',
    errorBody: 'We could not complete the operation. Please try again.',
    errorCta: 'Retry',
    semInternetTitulo: 'No internet',
    homeContinua: 'You can still see the items.',
    homeEspera: 'The primary action waits until you are back online.',
  },

  'es': {
    appName: 'Prototipo',

    modoPrototipo: 'Prototipo',
    modoShowcase: 'Showcase',

    painelTitulo: 'Controles del showcase',
    painelSuperficie: 'Superficie',
    painelTema: 'Tema',
    painelEstado: 'Estado',
    painelReset: 'Reiniciar',
    painelAbas: 'Pestañas del prototipo',
    painelRotina: 'Rutina',
    painelOpcao: 'Opción',
    rotinaNenhuma: 'Ninguna',
    rotinaToques: '{toques} de {meta} toques',
    rotinaConcluida: 'completada',
    rotinaMeta: 'meta de {meta} toques',
    rotinaAcimaDaMeta: 'por encima de la meta',
    conexaoVoltou: 'La conexión volvió. Lo que estaba esperando ya se envió.',
    temaClaro: 'Claro',
    temaEscuro: 'Oscuro',

    homeTitulo: 'Inicio',
    homeSubtitulo: 'TEMPLATE — sustituye con las pantallas reales',
    detalheTitulo: 'Detalle',
    detalheDescricao: 'TEMPLATE — descripción del elemento seleccionado',

    ctaPrimary: 'Acción principal',
    ctaSecondary: 'Acción secundaria',
    emptyTitle: 'Nada por aquí aún',
    emptyBody: 'Comienza agregando tu primer elemento.',
    emptyCta: 'Agregar',
    errorTitle: 'Algo salió mal',
    errorBody: 'No pudimos completar la operación. Intenta de nuevo.',
    errorCta: 'Reintentar',
    semInternetTitulo: 'Sin internet',
    homeContinua: 'Sigues viendo los elementos.',
    homeEspera: 'La acción principal espera a que vuelva la conexión.',
  },
};
