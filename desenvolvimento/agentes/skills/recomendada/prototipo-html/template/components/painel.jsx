// Protótipo — Painel de controle flutuante do showcase.
//
// Todo controle de revisão vive aqui: superfície, tema, estado, rotina e reset. O
// header fica só com a marca, as abas e o modo, e o device frame ganha a tela inteira.
// Antes isto ocupava o header em barras de abas — que sumiam em viewport estreita
// justamente quando o protótipo tinha telas demais para caber.

/* ---------- Opções A/B ----------
   A escolha de cada dúvida de visoes.jsx vive no shell e no endereço
   (&opcao=erro-da-vitrine:B), para o print sair reproduzível. Sem escolha, vale a
   primeira opção declarada. A tela lê com useOpcao(id) e desenha a opção. */
const OpcoesContext = React.createContext({});

const lerOpcoesDoEndereco = (textoDoParametro) => Object.fromEntries(
  (textoDoParametro || '').split(',').filter(Boolean).map((parDuvidaOpcao) => parDuvidaOpcao.split(':')));

const textoDasOpcoes = (opcoesEscolhidas) => Object.entries(opcoesEscolhidas)
  .map(([idDaDuvida, letraDaOpcao]) => `${idDaDuvida}:${letraDaOpcao}`).join(',');

const duvidaPorId = (idDaDuvida) => ROTINAS.flatMap(duvidasDaRotina).find((duvida) => duvida.id === idDaDuvida);

const opcaoDaDuvida = (opcoesEscolhidas, duvida) => opcoesEscolhidas[duvida.id] || Object.keys(duvida.opcoes)[0];

const useOpcao = (idDaDuvida) => {
  const opcoesEscolhidas = React.useContext(OpcoesContext);
  const duvida = duvidaPorId(idDaDuvida);
  // Dúvida resolvida sai de visoes.jsx junto com a opção perdedora; useOpcao que sobra é resto.
  if (!duvida) {
    console.warn(`[Opção] a dúvida "${idDaDuvida}" não está em visoes.jsx`);
    return 'A';
  }
  return opcaoDaDuvida(opcoesEscolhidas, duvida);
};

/* "Opção B" ou, com mais de uma dúvida, "Opção A·B". */
const letrasDasOpcoes = (opcoesEscolhidas, duvidas) =>
  duvidas.map((duvida) => opcaoDaDuvida(opcoesEscolhidas, duvida)).join('·');

/* Os 8 estados de componente e o offline (sem internet), que é estado da tela. */
const ESTADOS = ['default', 'hover', 'focus', 'active', 'disabled', 'loading', 'empty', 'error', 'offline'];

const PainelGrupo = ({ label, options, value, onChange }) => (
  <div className="painel-grupo">
    <div className="painel-grupo-label">{label}</div>
    <div className="painel-opcoes">
      {options.map((opcao) => (
        <button
          key={opcao.value}
          className={value === opcao.value ? 'active' : ''}
          onClick={() => onChange(opcao.value)}
          aria-pressed={value === opcao.value}
        >
          {opcao.label}
        </button>
      ))}
    </div>
  </div>
);

/* "Ver um item: 2 de 3 toques · concluída". Acima da meta fica marcado em texto,
   não só em cor. */
const textoDaContagem = (t, contagem, opcoesEscolhidas) => {
  if (!contagem) return '';
  const meta = metaDaRotina(contagem.rotina);
  const duvidas = duvidasDaRotina(contagem.rotina);
  const rotuloComOpcao = duvidas.length
    ? `${contagem.rotina.rotulo} (${t('painelOpcao')} ${letrasDasOpcoes(opcoesEscolhidas, duvidas)})`
    : contagem.rotina.rotulo;
  const partes = [`${rotuloComOpcao}: ${t('rotinaToques', { toques: contagem.toques, meta })}`];
  if (contagem.concluida) partes.push(t('rotinaConcluida'));
  if (contagem.toques > meta) partes.push(t('rotinaAcimaDaMeta'));
  return partes.join(' · ');
};

const PainelControle = ({
  lang,
  visao,
  surface, setSurface,
  theme, setTheme,
  estado, setEstado,
  contagem, iniciarRotina,
  opcoesEscolhidas, escolherOpcao,
  onReset,
}) => {
  const t = useT(lang);
  const [aberto, setAberto] = React.useState(false);
  const painelRef = React.useRef(null);
  const botaoRef = React.useRef(null);
  const rotinas = rotinasDaVisao(visao.id);
  // Com uma rotina em contagem, só as dúvidas dela; sem, as da aba inteira.
  const duvidas = contagem ? duvidasDaRotina(contagem.rotina) : duvidasDaVisao(visao.id);

  // Esc fecha e devolve o foco ao botão — sem isso a navegação por teclado fica presa.
  React.useEffect(() => {
    if (!aberto) return;
    const aoTeclar = (evento) => {
      if (evento.key === 'Escape') { setAberto(false); botaoRef.current?.focus(); }
    };
    const aoClicarFora = (evento) => {
      if (painelRef.current && !painelRef.current.contains(evento.target)) setAberto(false);
    };
    document.addEventListener('keydown', aoTeclar);
    document.addEventListener('mousedown', aoClicarFora);
    return () => {
      document.removeEventListener('keydown', aoTeclar);
      document.removeEventListener('mousedown', aoClicarFora);
    };
  }, [aberto]);

  const resumoDaContagem = contagem ? ` · ${contagem.toques}/${metaDaRotina(contagem.rotina)}` : '';
  const resumoDasOpcoes = duvidas.length ? ` · ${t('painelOpcao')} ${letrasDasOpcoes(opcoesEscolhidas, duvidas)}` : '';
  const resumo = `${SURFACES[surface].label} · ${theme === 'light' ? t('temaClaro') : t('temaEscuro')} · ${estado}${resumoDasOpcoes}${resumoDaContagem}`;

  return (
    <div className="painel" ref={painelRef}>
      {aberto && (
        <div className="painel-corpo" role="dialog" aria-label={t('painelTitulo')}>
          <PainelGrupo
            label={t('painelSuperficie')}
            value={surface}
            onChange={setSurface}
            options={visao.superficies.map((chave) => ({ value: chave, label: SURFACES[chave].label }))}
          />
          <PainelGrupo
            label={t('painelTema')}
            value={theme}
            onChange={setTheme}
            options={[
              { value: 'light', label: t('temaClaro') },
              { value: 'dark', label: t('temaEscuro') },
            ]}
          />
          <PainelGrupo
            label={t('painelEstado')}
            value={estado}
            onChange={setEstado}
            options={ESTADOS.map((nomeDoEstado) => ({ value: nomeDoEstado, label: nomeDoEstado }))}
          />
          {rotinas.length > 0 && (
            <div className="painel-grupo">
              <PainelGrupo
                label={t('painelRotina')}
                value={contagem ? contagem.rotina.id : ''}
                onChange={(idDaRotina) => iniciarRotina(idDaRotina || null)}
                options={[
                  { value: '', label: t('rotinaNenhuma') },
                  ...rotinas.map((rotina) => ({ value: rotina.id, label: rotina.rotulo })),
                ]}
              />
              {contagem && (
                <div
                  className={`painel-contagem ${contagem.toques > metaDaRotina(contagem.rotina) ? 'acima' : ''}`}
                  aria-live="polite"
                >
                  {textoDaContagem(t, contagem, opcoesEscolhidas)}
                </div>
              )}
            </div>
          )}
          {duvidas.map((duvida) => (
            <PainelGrupo
              key={duvida.id}
              label={`${t('painelOpcao')} · ${duvida.pergunta}`}
              value={opcaoDaDuvida(opcoesEscolhidas, duvida)}
              onChange={(letraDaOpcao) => escolherOpcao(duvida.id, letraDaOpcao)}
              options={Object.entries(duvida.opcoes).map(([letraDaOpcao, rotuloDaOpcao]) => ({
                value: letraDaOpcao, label: `${letraDaOpcao} · ${rotuloDaOpcao}`,
              }))}
            />
          ))}
          <button className="painel-reset" onClick={onReset}>{t('painelReset')}</button>
        </div>
      )}

      <button
        ref={botaoRef}
        className="painel-toggle"
        onClick={() => setAberto((estaAberto) => !estaAberto)}
        aria-expanded={aberto}
        aria-label={t('painelTitulo')}
      >
        <IconSliders size={15} />
        <span className="painel-resumo">{resumo}</span>
        <span className={`painel-chevron ${aberto ? 'aberto' : ''}`} aria-hidden="true">▾</span>
      </button>
    </div>
  );
};
