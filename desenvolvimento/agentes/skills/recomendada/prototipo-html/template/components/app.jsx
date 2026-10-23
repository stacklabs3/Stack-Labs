// Protótipo — Showcase shell.
// Header enxuto (marca + abas + modo). Todo o resto dos controles fica no painel
// flutuante, para o device frame ganhar a tela inteira.
//
// O endereço guarda a combinação em revisão, para cada print ser reproduzível:
//   ?visao=<id>&superficie=<chave>&tema=light|dark&estado=<estado>&modo=prototype|showcase
//   &opcao=<duvida>:<A|B>    abre a dúvida A/B na opção (várias separadas por vírgula)
//   &print=1                 esconde header e painel (só a tela na moldura)
//   &idioma=pt-BR            fixa o idioma (o navegador sem janela diz que é inglês)
//   ?listarPrints=<rotina>   devolve a lista de prints da rotina (ou "todas") em JSON

const PARAMETROS_DO_ENDERECO = new URLSearchParams(window.location.search);

const detectLang = () => {
  const idiomaDoEndereco = PARAMETROS_DO_ENDERECO.get('idioma');
  if (idiomaDoEndereco && TRANSLATIONS[idiomaDoEndereco]) return idiomaDoEndereco;
  const idiomaDoNavegador = (navigator.language || 'pt-BR').toLowerCase();
  if (idiomaDoNavegador.startsWith('pt')) return 'pt-BR';
  if (idiomaDoNavegador.startsWith('es')) return 'es';
  return 'en';
};

/* Mapa rota → componente. Vive aqui porque app.jsx carrega por último e é o
   único que enxerga router e telas ao mesmo tempo. */
const SCREEN_MAP = {
  HOME: TelaHome,
  DETAIL: TelaDetalhe,
};

class PrototypeErrorBoundary extends React.Component {
  constructor(props) { super(props); this.state = { error: null }; }
  static getDerivedStateFromError(error) { return { error }; }
  componentDidCatch(error, info) { console.error('[Showcase]', this.props.contexto, error, info); }
  componentDidUpdate(propsAnteriores) {
    if (propsAnteriores.contexto !== this.props.contexto && this.state.error) this.setState({ error: null });
  }
  render() {
    if (this.state.error) {
      const erro = this.state.error;
      return (
        <div style={{
          padding: 'var(--space-6)', display: 'flex', flexDirection: 'column',
          gap: 'var(--space-3)', alignItems: 'center', justifyContent: 'center',
          color: 'var(--danger)', textAlign: 'center', height: '100%',
        }}>
          <div style={{ fontSize: 16, fontWeight: 800 }}>Render error em "{this.props.contexto}"</div>
          <code style={{
            fontSize: 12, padding: 'var(--space-3)', borderRadius: 'var(--radius-md)',
            background: 'var(--surface)', border: '1px solid var(--border)',
            color: 'var(--fg)', maxWidth: '100%', overflow: 'auto',
            whiteSpace: 'pre-wrap', textAlign: 'left',
          }}>{String(erro?.stack || erro?.message || erro)}</code>
        </div>
      );
    }
    return this.props.children;
  }
}

/* ---------- Prints da conferência ----------
   Estado normal em todas as combinações da aba (superfície × tema) e os estados
   vazio, erro, carregando e sem internet numa combinação só (a de abertura). */
const ESTADOS_NUMA_COMBINACAO = ['empty', 'error', 'loading', 'offline'];

const enderecoDaCombinacao = ({ visao, superficie, tema, estado, tela, opcoesEscolhidas }) => {
  const parametros = new URLSearchParams({ visao, superficie, tema, estado, print: '1' });
  if (Object.keys(opcoesEscolhidas).length) parametros.set('opcao', textoDasOpcoes(opcoesEscolhidas));
  return `?${parametros.toString()}${buildPath(tela.rota, tela.params || {})}`;
};

const tamanhoDaJanelaDePrint = (superficie) => {
  const { width: largura, height: altura } = tamanhoComMoldura(superficie);
  const folgaDaEscala = SURFACES[superficie].kind === 'free' ? 0 : 48;
  return { largura: largura + folgaDaEscala, altura: altura + folgaDaEscala };
};

/* Rotina com dúvida A/B: a conferência inteira sai em cada opção. A primeira combinação
   tem todas as dúvidas na primeira opção; as outras trocam uma dúvida por vez. Cada
   combinação vai para uma pasta própria (opcao-<duvida>-<letra>). */
const combinacoesDeOpcoes = (rotina) => {
  const duvidas = duvidasDaRotina(rotina);
  if (!duvidas.length) return [{}];
  const opcoesIniciais = Object.fromEntries(duvidas.map((duvida) => [duvida.id, Object.keys(duvida.opcoes)[0]]));
  const variacoes = duvidas.flatMap((duvida) => Object.keys(duvida.opcoes).slice(1)
    .map((letraDaOpcao) => ({ ...opcoesIniciais, [duvida.id]: letraDaOpcao })));
  return [opcoesIniciais, ...variacoes];
};

const pastaDasOpcoes = (opcoesEscolhidas) => {
  const pares = Object.entries(opcoesEscolhidas);
  if (!pares.length) return '';
  return `opcao-${pares.map(([idDaDuvida, letraDaOpcao]) => `${idDaDuvida}-${letraDaOpcao}`).join('--')}/`;
};

const printsDaRotina = (rotina) => {
  const visao = visaoPorId(rotina.visao);
  const prints = [];
  combinacoesDeOpcoes(rotina).forEach((opcoesEscolhidas) => {
    const registrar = (tela, indiceDaTela, superficie, tema, estado) => {
      const ordem = String(indiceDaTela + 1).padStart(2, '0');
      prints.push({
        arquivo: `${rotina.id}/${pastaDasOpcoes(opcoesEscolhidas)}${ordem}-${tela.rota.toLowerCase()}--${superficie}--${tema}--${estado}.png`,
        endereco: enderecoDaCombinacao({ visao: visao.id, superficie, tema, estado, tela, opcoesEscolhidas }),
        ...tamanhoDaJanelaDePrint(superficie),
      });
    };
    rotina.telas.forEach((tela, indiceDaTela) => {
      visao.superficies.forEach((superficie) => {
        ['light', 'dark'].forEach((tema) => registrar(tela, indiceDaTela, superficie, tema, 'default'));
      });
      ESTADOS_NUMA_COMBINACAO.forEach((estado) => {
        registrar(tela, indiceDaTela, visao.superficies[0], visao.temaPadrao || 'light', estado);
      });
    });
  });
  return prints;
};

const ListaDePrints = ({ idDaRotina }) => {
  const rotinas = idDaRotina === 'todas' ? ROTINAS : ROTINAS.filter((rotina) => rotina.id === idDaRotina);
  const prints = rotinas.flatMap(printsDaRotina);
  return <pre id="lista-de-prints">{JSON.stringify(prints, null, 2)}</pre>;
};

/* Tela ativa segundo a rota. */
const TelaAtual = ({ lang, estado }) => {
  const { route } = useRouter();
  const Screen = SCREEN_MAP[route.name] || SCREEN_MAP.HOME;
  return <Screen lang={lang} estado={estado} params={route.params} />;
};

/* ---------- Modo Showcase: as rotinas da aba, cada uma com as telas em ordem ---------- */
const ALTURA_DO_QUADRO_NA_GRADE = 520;

const QuadroDaGrade = ({ surface, rotulo, children }) => {
  const especificacao = SURFACES[surface];
  const { width: largura, height: altura } = especificacao.kind === 'free' ? DESKTOP_REFERENCIA : especificacao;
  const escala = Math.min(1, ALTURA_DO_QUADRO_NA_GRADE / altura);
  return (
    <div className="grade-tile">
      <div className="grade-tile-label">{rotulo}</div>
      <div className="grade-tile-viewport" style={{ width: largura * escala, height: altura * escala }}>
        <div style={{ width: largura, height: altura, transform: `scale(${escala})`, transformOrigin: 'top left', overflow: 'auto', background: 'var(--bg)' }}>
          {children}
        </div>
      </div>
    </div>
  );
};

const GradeDeTelas = ({ lang, estado, visao, surface, opcoesEscolhidas }) => {
  const t = useT(lang);
  return (
    <div className="showcase-grade">
      {rotinasDaVisao(visao.id).map((rotina) => (
        <section key={rotina.id} className="grade-rotina">
          <div className="grade-rotina-titulo">
            {rotina.rotulo} · {t('rotinaMeta', { meta: metaDaRotina(rotina) })}
            {duvidasDaRotina(rotina).length > 0 && ` · ${t('painelOpcao')} ${letrasDasOpcoes(opcoesEscolhidas, duvidasDaRotina(rotina))}`}
          </div>
          <div className="grade-rotina-telas">
            {rotina.telas.map((tela, indiceDaTela) => {
              const Screen = SCREEN_MAP[tela.rota];
              const contexto = `${rotina.id}/${tela.rota}`;
              return (
                <React.Fragment key={contexto + indiceDaTela}>
                  {indiceDaTela > 0 && <IconArrow size={18} style={{ color: 'var(--muted)', alignSelf: 'center' }} />}
                  <QuadroDaGrade surface={surface} rotulo={`${indiceDaTela + 1}. ${tela.rota}`}>
                    <PrototypeErrorBoundary contexto={contexto}>
                      <Screen lang={lang} estado={estado} params={tela.params || {}} />
                    </PrototypeErrorBoundary>
                  </QuadroDaGrade>
                </React.Fragment>
              );
            })}
          </div>
        </section>
      ))}
    </div>
  );
};

/* Elementos que contam como toque na contagem da rotina. */
const SELETOR_DE_TOCAVEIS = 'button, a, input, select, textarea, label, [role="button"], [role="tab"], [role="link"], [role="checkbox"], [role="radio"], [role="switch"], [role="option"]';

const ShowcaseConteudo = () => {
  const { route, navigate } = useRouter();
  const visaoInicial = visaoPorId(PARAMETROS_DO_ENDERECO.get('visao'));
  const [lang] = React.useState(detectLang());
  const [idDaVisao, setIdDaVisao] = React.useState(visaoInicial.id);
  const [surface, setSurface] = React.useState(PARAMETROS_DO_ENDERECO.get('superficie') || visaoInicial.superficies[0]);
  const [theme, setTheme] = React.useState(PARAMETROS_DO_ENDERECO.get('tema') || visaoInicial.temaPadrao || 'light');
  const [estado, setEstado] = React.useState(PARAMETROS_DO_ENDERECO.get('estado') || DEFAULTS.estado);
  const [mode, setMode] = React.useState(PARAMETROS_DO_ENDERECO.get('modo') || DEFAULTS.mode);
  const [contagem, setContagem] = React.useState(null);
  const [opcoesEscolhidas, setOpcoesEscolhidas] = React.useState(lerOpcoesDoEndereco(PARAMETROS_DO_ENDERECO.get('opcao')));
  const [conexaoVoltou, setConexaoVoltou] = React.useState(false);
  const estadoAnteriorRef = React.useRef(estado);
  const [bodyRef, bodySize] = useElementSize();
  const t = useT(lang);
  const visao = visaoPorId(idDaVisao);
  const modoPrint = PARAMETROS_DO_ENDERECO.get('print') === '1';

  React.useEffect(() => { document.documentElement.setAttribute('data-theme', theme); }, [theme]);

  // Sair do offline mostra o aviso de que a conexão voltou, como o app fará.
  React.useEffect(() => {
    const saiuDoOffline = estadoAnteriorRef.current === 'offline' && estado !== 'offline';
    estadoAnteriorRef.current = estado;
    if (!saiuDoOffline) return;
    setConexaoVoltou(true);
    const temporizador = setTimeout(() => setConexaoVoltou(false), 4000);
    return () => clearTimeout(temporizador);
  }, [estado]);

  // A contagem começa quando a primeira tela abre e termina na última tela da rotina.
  React.useEffect(() => {
    if (!contagem || contagem.concluida) return;
    const { telas } = contagem.rotina;
    if (contagem.aguardandoPrimeiraTela) {
      if (route.name === telas[0].rota) setContagem({ ...contagem, aguardandoPrimeiraTela: false });
      return;
    }
    if (route.name === telas[telas.length - 1].rota) setContagem({ ...contagem, concluida: true });
  }, [route, contagem]);

  // Leitura da contagem por quem audita (javascript_tool), sem abrir o painel.
  React.useEffect(() => {
    window.contagemDeToques = contagem && {
      rotina: contagem.rotina.id,
      opcoes: Object.fromEntries(duvidasDaRotina(contagem.rotina)
        .map((duvida) => [duvida.id, opcaoDaDuvida(opcoesEscolhidas, duvida)])),
      toques: contagem.toques,
      meta: metaDaRotina(contagem.rotina),
      concluida: contagem.concluida,
    };
  }, [contagem, opcoesEscolhidas]);

  const iniciarRotina = (idDaRotina) => {
    const rotina = ROTINAS.find((rotinaDaLista) => rotinaDaLista.id === idDaRotina);
    if (!rotina) { setContagem(null); return; }
    const primeiraTela = rotina.telas[0];
    navigate(primeiraTela.rota, primeiraTela.params);
    setContagem({ rotina, toques: 0, concluida: false, aguardandoPrimeiraTela: true });
  };

  // Trocar a opção no meio da contagem recomeça a rotina: os toques valem para uma opção só.
  const escolherOpcao = (idDaDuvida, letraDaOpcao) => {
    setOpcoesEscolhidas((opcoesAnteriores) => ({ ...opcoesAnteriores, [idDaDuvida]: letraDaOpcao }));
    if (contagem) iniciarRotina(contagem.rotina.id);
  };

  const registrarToque = (evento) => {
    if (!contagem || contagem.concluida || contagem.aguardandoPrimeiraTela) return;
    const tocavel = evento.target.closest(SELETOR_DE_TOCAVEIS);
    if (!tocavel || tocavel.disabled || tocavel.getAttribute('aria-disabled') === 'true') return;
    setContagem((contagemAtual) => ({ ...contagemAtual, toques: contagemAtual.toques + 1 }));
  };

  const trocarVisao = (idDaNovaVisao) => {
    const novaVisao = visaoPorId(idDaNovaVisao);
    setIdDaVisao(novaVisao.id);
    setSurface(novaVisao.superficies[0]);
    setTheme(novaVisao.temaPadrao || 'light');
    setContagem(null);
    const [primeiraRotina] = rotinasDaVisao(novaVisao.id);
    if (primeiraRotina) navigate(primeiraRotina.telas[0].rota, primeiraRotina.telas[0].params);
  };

  const resetar = () => {
    setSurface(visao.superficies[0]);
    setTheme(visao.temaPadrao || 'light');
    setEstado(DEFAULTS.estado);
    setMode(DEFAULTS.mode);
    setContagem(null);
    setOpcoesEscolhidas({});
    window.location.hash = '';
  };

  const palco = (
    <ScaledSurface surface={surface} theme={theme} avail={bodySize}>
      <div onClickCapture={registrarToque} style={{ position: 'relative', minHeight: '100%' }}>
        <PrototypeErrorBoundary contexto={`${visao.id}/${surface}/${estado}`}>
          <TelaAtual lang={lang} estado={estado} />
        </PrototypeErrorBoundary>
        {conexaoVoltou && <AvisoConexaoVoltou texto={t('conexaoVoltou')} />}
      </div>
    </ScaledSurface>
  );

  if (modoPrint) {
    return (
      <OpcoesContext.Provider value={opcoesEscolhidas}>
        <main className="showcase-body showcase-body--print" ref={bodyRef}>{palco}</main>
      </OpcoesContext.Provider>
    );
  }

  return (
    <OpcoesContext.Provider value={opcoesEscolhidas}>
      <div className="showcase-shell">
        <header className="showcase-header">
          <div className="showcase-brand">
            <span>{t('appName')}</span>
            <span className="accent">•</span>
          </div>

          {VISOES.length > 1 && (
            <nav className="showcase-abas" aria-label={t('painelAbas')}>
              {VISOES.map((visaoDaAba) => (
                <button
                  key={visaoDaAba.id}
                  className={visaoDaAba.id === visao.id ? 'active' : ''}
                  onClick={() => trocarVisao(visaoDaAba.id)}
                  aria-pressed={visaoDaAba.id === visao.id}
                >
                  {visaoDaAba.rotulo}
                </button>
              ))}
            </nav>
          )}

          <div className="showcase-modo">
            <button
              className={mode === 'prototype' ? 'active' : ''}
              onClick={() => setMode('prototype')}
              aria-pressed={mode === 'prototype'}
            >
              {t('modoPrototipo')}
            </button>
            <button
              className={mode === 'showcase' ? 'active' : ''}
              onClick={() => setMode('showcase')}
              aria-pressed={mode === 'showcase'}
            >
              {t('modoShowcase')}
            </button>
          </div>
        </header>

        <main className={`showcase-body ${mode === 'showcase' ? 'showcase-body--grade' : ''}`} ref={bodyRef}>
          {mode === 'prototype'
            ? palco
            : <GradeDeTelas lang={lang} estado={estado} visao={visao} surface={surface} opcoesEscolhidas={opcoesEscolhidas} />}
        </main>

        <PainelControle
          lang={lang}
          visao={visao}
          surface={surface} setSurface={setSurface}
          theme={theme} setTheme={setTheme}
          estado={estado} setEstado={setEstado}
          contagem={contagem} iniciarRotina={iniciarRotina}
          opcoesEscolhidas={opcoesEscolhidas} escolherOpcao={escolherOpcao}
          onReset={resetar}
        />
      </div>
    </OpcoesContext.Provider>
  );
};

const Showcase = () => {
  const idDaRotinaParaListar = PARAMETROS_DO_ENDERECO.get('listarPrints');
  if (idDaRotinaParaListar) return <ListaDePrints idDaRotina={idDaRotinaParaListar} />;
  return (
    <RouterProvider>
      <ShowcaseConteudo />
    </RouterProvider>
  );
};
