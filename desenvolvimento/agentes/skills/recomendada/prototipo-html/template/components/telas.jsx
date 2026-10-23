// Protótipo — Telas de exemplo.
// TEMPLATE: substitua por components/<dominio>.jsx com as telas reais.
// O que importa preservar: cada tela recebe `estado` e responde aos 8 estados
// visuais e ao offline (sem internet), e a navegação usa useRouter() em vez de
// trocar aba. A tela é um passo de uma rotina declarada em visoes.jsx.

const TelaHome = ({ lang, estado }) => {
  const t = useT(lang);
  const { navigate } = useRouter();

  if (estado === 'loading') {
    return (
      <TelaShell titulo={t('homeTitulo')}>
        <Skeleton w="70%" h={18} />
        <Skeleton w="100%" h={56} r="var(--radius-md)" />
        <Skeleton w="100%" h={56} r="var(--radius-md)" />
        <Skeleton w="100%" h={56} r="var(--radius-md)" />
      </TelaShell>
    );
  }

  if (estado === 'empty') {
    return (
      <TelaShell titulo={t('homeTitulo')}>
        <EmptyState title={t('emptyTitle')} body={t('emptyBody')} cta={t('emptyCta')} onCta={() => {}} />
      </TelaShell>
    );
  }

  if (estado === 'error') {
    return (
      <TelaShell titulo={t('homeTitulo')}>
        <ErrorState title={t('errorTitle')} body={t('errorBody')} cta={t('errorCta')} onCta={() => {}} />
      </TelaShell>
    );
  }

  const disabled = estado === 'disabled';
  const semInternet = estado === 'offline';

  return (
    <TelaShell titulo={t('homeTitulo')}>
      {semInternet && (
        <AvisoSemInternet
          titulo={t('semInternetTitulo')}
          continuaFuncionando={t('homeContinua')}
          espera={t('homeEspera')}
        />
      )}
      <div style={{ fontSize: 13, color: 'var(--muted)', marginBottom: 'var(--space-2)' }}>
        {t('homeSubtitulo')}
      </div>

      {MOCK_ITEMS.map((item) => (
        <Press
          key={item.id}
          onClick={() => !disabled && navigate('DETAIL', { id: item.id })}
          disabled={disabled}
          style={{
            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
            padding: 'var(--space-3) var(--space-4)',
            background: 'var(--surface)', border: '1px solid var(--border)',
            borderRadius: 'var(--radius-md)', width: '100%', textAlign: 'left',
          }}
        >
          <span style={{ fontSize: 14, fontWeight: 600 }}>{item.label}</span>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 13, color: 'var(--muted)' }}>
            {item.value}
          </span>
        </Press>
      ))}

      <Button variant="primary" disabled={disabled || semInternet}>{t('ctaPrimary')}</Button>
    </TelaShell>
  );
};

const TelaDetalhe = ({ lang, estado, params }) => {
  const t = useT(lang);
  const { navigate } = useRouter();
  const item = MOCK_ITEMS.find((itemMock) => String(itemMock.id) === String(params.id)) || MOCK_ITEMS[0];

  if (estado === 'loading') {
    return (
      <TelaShell titulo={t('detalheTitulo')} onVoltar={() => navigate('HOME')}>
        <Skeleton w="60%" h={22} />
        <Skeleton w="100%" h={14} />
        <Skeleton w="90%" h={14} />
      </TelaShell>
    );
  }

  if (estado === 'error') {
    return (
      <TelaShell titulo={t('detalheTitulo')} onVoltar={() => navigate('HOME')}>
        <ErrorState title={t('errorTitle')} body={t('errorBody')} cta={t('errorCta')} onCta={() => {}} />
      </TelaShell>
    );
  }

  const disabled = estado === 'disabled';
  const semInternet = estado === 'offline';

  return (
    <TelaShell titulo={t('detalheTitulo')} onVoltar={() => navigate('HOME')}>
      {semInternet && (
        <AvisoSemInternet
          titulo={t('semInternetTitulo')}
          continuaFuncionando={t('homeContinua')}
          espera={t('homeEspera')}
        />
      )}
      <div style={{ fontSize: 20, fontWeight: 800 }}>{item.label}</div>
      <div style={{ fontSize: 13, color: 'var(--muted)' }}>{t('detalheDescricao')}</div>
      <Field label="Email" value={MOCK_USER.email} onChange={() => {}} disabled={disabled} />
      <Button variant="primary" disabled={disabled || semInternet}>{t('ctaPrimary')}</Button>
    </TelaShell>
  );
};

/* Casca comum das telas: cabeçalho opcional + coluna com respiro consistente. */
const TelaShell = ({ titulo, onVoltar, children }) => (
  <div style={{
    display: 'flex', flexDirection: 'column', gap: 'var(--space-3)',
    padding: 'var(--space-5)', minHeight: '100%',
  }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
      {onVoltar && (
        <Press onClick={onVoltar} aria-label="Voltar" style={{ padding: 'var(--space-1)', background: 'transparent' }}>
          <IconArrow size={18} style={{ transform: 'rotate(180deg)', color: 'var(--fg)' }} />
        </Press>
      )}
      <div style={{ fontSize: 22, fontWeight: 800, letterSpacing: '-0.4px' }}>{titulo}</div>
    </div>
    {children}
  </div>
);
