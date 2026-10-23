// Protótipo — Superfícies de visualização (device frames).
// Cada aba (visão) liga só as superfícies dela, em visoes.jsx. Aqui ficam todas as
// que o padrão sabe desenhar: celular (iOS e Android), tablet em pé e deitado,
// computador, TV e totem.

const SURFACES = {
  desktop:       { label: 'Computador',     width: null, height: null, kind: 'free' },
  tablet:        { label: 'Tablet',         width: 834,  height: 1112, kind: 'tablet', moldura: 12 },
  tabletDeitado: { label: 'Tablet deitado', width: 1112, height: 834,  kind: 'tablet', moldura: 12 },
  ios:           { label: 'iOS',            width: 390,  height: 800,  kind: 'phone',  moldura: 9 },
  android:       { label: 'Android',        width: 390,  height: 800,  kind: 'phone',  moldura: 3 },
  tv:            { label: 'TV',             width: 1920, height: 1080, kind: 'tv',     moldura: 14 },
  totem:         { label: 'Totem',          width: 1080, height: 1920, kind: 'totem',  moldura: 28 },
};

const SURFACE_ORDER = ['desktop', 'tablet', 'tabletDeitado', 'ios', 'android', 'tv', 'totem'];

/* Tamanho do computador quando precisa de medida fixa (grade e prints). */
const DESKTOP_REFERENCIA = { width: 1440, height: 900 };

/* ---------- iOS: dynamic island + home indicator ---------- */
const IosFrame = ({ theme, children }) => {
  const { width: W, height: H } = SURFACES.ios;
  const chrome = theme === 'dark' ? '#F5F5F7' : '#1A1A1F';

  return (
    <div style={{
      width: W + 18, height: H + 18,
      background: '#0A0A0E', borderRadius: 56, padding: 9,
      boxShadow: 'var(--shadow-device)', flexShrink: 0,
    }}>
      <div style={{
        width: W, height: H, background: 'var(--bg)',
        borderRadius: 47, overflow: 'hidden', position: 'relative',
      }}>
        <div style={{
          position: 'absolute', top: 0, left: 0, right: 0, height: 44,
          display: 'flex', alignItems: 'flex-end', justifyContent: 'space-between',
          padding: '0 28px 8px', fontSize: 13, fontWeight: 700,
          color: chrome, zIndex: 30, pointerEvents: 'none',
        }}>
          <span style={{ fontVariantNumeric: 'tabular-nums' }}>9:41</span>
          <span>● ● ● 100%</span>
        </div>

        <div style={{
          position: 'absolute', top: 11, left: '50%', transform: 'translateX(-50%)',
          width: 120, height: 34, background: '#0A0A0E', borderRadius: 24, zIndex: 40,
        }} />

        <div style={{
          position: 'absolute', bottom: 8, left: '50%', transform: 'translateX(-50%)',
          width: 134, height: 5, background: chrome, borderRadius: 3,
          opacity: 0.92, zIndex: 30, pointerEvents: 'none',
        }} />

        <div style={{ position: 'absolute', inset: 0, paddingTop: 44, paddingBottom: 24, overflow: 'auto' }}>
          {children}
        </div>
      </div>
    </div>
  );
};

/* ---------- Android: status bar + gesture bar ---------- */
const AndroidFrame = ({ theme, children }) => {
  const { width: W, height: H } = SURFACES.android;
  const chrome = theme === 'dark' ? '#F5F5F7' : '#1A1A1F';

  return (
    <div style={{
      width: W + 6, height: H + 6,
      background: '#1A1A1F', borderRadius: 32, padding: 3,
      boxShadow: 'var(--shadow-device)', flexShrink: 0,
    }}>
      <div style={{
        width: W, height: H, background: 'var(--bg)',
        borderRadius: 30, overflow: 'hidden', position: 'relative',
      }}>
        <div style={{
          position: 'absolute', top: 0, left: 0, right: 0, height: 28,
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '0 16px', fontSize: 12, fontWeight: 700,
          color: chrome, zIndex: 30, pointerEvents: 'none',
        }}>
          <span style={{ fontVariantNumeric: 'tabular-nums' }}>9:41</span>
          <span>● ● 100%</span>
        </div>

        <div style={{
          position: 'absolute', bottom: 6, left: '50%', transform: 'translateX(-50%)',
          width: 108, height: 4, background: chrome, borderRadius: 2,
          opacity: 0.7, zIndex: 30, pointerEvents: 'none',
        }} />

        <div style={{ position: 'absolute', inset: 0, paddingTop: 28, paddingBottom: 18, overflow: 'auto' }}>
          {children}
        </div>
      </div>
    </div>
  );
};

/* ---------- Tablet (em pé ou deitado): moldura sóbria, sem chrome de SO ---------- */
const TabletFrame = ({ surface, children }) => {
  const { width: W, height: H, moldura } = SURFACES[surface];

  return (
    <div style={{
      width: W + moldura * 2, height: H + moldura * 2,
      background: '#1A1A1F', borderRadius: 28, padding: moldura,
      boxShadow: 'var(--shadow-device)', flexShrink: 0,
    }}>
      <div style={{
        width: W, height: H, background: 'var(--bg)',
        borderRadius: 18, overflow: 'auto', position: 'relative',
      }}>
        {children}
      </div>
    </div>
  );
};

/* ---------- TV: borda fina, tela deitada, lida de longe ---------- */
const TvFrame = ({ children }) => {
  const { width: W, height: H, moldura } = SURFACES.tv;

  return (
    <div style={{
      width: W + moldura * 2, height: H + moldura * 2,
      background: '#0A0A0E', borderRadius: 10, padding: moldura,
      boxShadow: 'var(--shadow-device)', flexShrink: 0,
    }}>
      <div style={{
        width: W, height: H, background: 'var(--bg)',
        overflow: 'auto', position: 'relative',
      }}>
        {children}
      </div>
    </div>
  );
};

/* ---------- Totem: corpo alto em pé, tela de toque ---------- */
const TotemFrame = ({ children }) => {
  const { width: W, height: H, moldura } = SURFACES.totem;

  return (
    <div style={{
      width: W + moldura * 2, height: H + moldura * 2,
      background: '#1A1A1F', borderRadius: 36, padding: moldura,
      boxShadow: 'var(--shadow-device)', flexShrink: 0,
    }}>
      <div style={{
        width: W, height: H, background: 'var(--bg)',
        borderRadius: 12, overflow: 'auto', position: 'relative',
      }}>
        {children}
      </div>
    </div>
  );
};

const FRAME_POR_SUPERFICIE = {
  ios: IosFrame,
  android: AndroidFrame,
  tablet: TabletFrame,
  tabletDeitado: TabletFrame,
  tv: TvFrame,
  totem: TotemFrame,
};

/* ---------- Computador: sem moldura, ocupa o espaço disponível ---------- */
const FreeSurface = ({ children }) => (
  <div style={{
    width: '100%', height: '100%',
    background: 'var(--bg)', overflow: 'auto',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
  }}>
    {children}
  </div>
);

/* Tamanho externo (tela + moldura). O computador não tem moldura e usa a referência. */
const tamanhoComMoldura = (surface) => {
  const especificacao = SURFACES[surface];
  if (especificacao.kind === 'free') return DESKTOP_REFERENCIA;
  return {
    width: especificacao.width + especificacao.moldura * 2,
    height: especificacao.height + especificacao.moldura * 2,
  };
};

/* ---------- Escala para caber na área disponível ----------
   Sem isto, o frame estoura a viewport em tela pequena e a revisão vira scroll.
   Só reduz: ampliar distorceria a percepção de tamanho real. */
const ScaledSurface = ({ surface, theme, avail, children }) => {
  const especificacao = SURFACES[surface];

  if (especificacao.kind === 'free') return <FreeSurface>{children}</FreeSurface>;

  const { width: larguraExterna, height: alturaExterna } = tamanhoComMoldura(surface);

  const margem = 48;
  const larguraDisponivel = (avail && avail.width) || 0;
  const alturaDisponivel = (avail && avail.height) || 0;
  const escala = (larguraDisponivel && alturaDisponivel)
    ? Math.min(1, (larguraDisponivel - margem) / larguraExterna, (alturaDisponivel - margem) / alturaExterna)
    : 1;

  const Frame = FRAME_POR_SUPERFICIE[surface];

  return (
    <div style={{
      width: larguraExterna * escala, height: alturaExterna * escala,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
    }}>
      <div style={{ transform: `scale(${escala})`, transformOrigin: 'center center', flexShrink: 0 }}>
        <Frame surface={surface} theme={theme}>{children}</Frame>
      </div>
    </div>
  );
};

/* ---------- Mede o container para alimentar ScaledSurface ---------- */
const useElementSize = () => {
  const ref = React.useRef(null);
  const [size, setSize] = React.useState({ width: 0, height: 0 });

  React.useEffect(() => {
    const elemento = ref.current;
    if (!elemento) return;
    const observador = new ResizeObserver((medicoes) => {
      const retangulo = medicoes[0].contentRect;
      setSize({ width: retangulo.width, height: retangulo.height });
    });
    observador.observe(elemento);
    return () => observador.disconnect();
  }, []);

  return [ref, size];
};
