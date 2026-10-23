// Protótipo — Primitivos UI compartilhados
// TEMPLATE: adicione mais primitivos (Modal, Card, Field, etc.) conforme a tela exigir.

const { useState, useEffect, useRef, useMemo, useCallback } = React;

/* ---------- i18n hook ---------- */
const useT = (lang) => {
  const dict = TRANSLATIONS[lang] || TRANSLATIONS['pt-BR'];
  return useCallback((k, vars) => {
    let s = dict[k];
    if (s == null) return k;
    if (vars) for (const v in vars) s = s.replace('{' + v + '}', vars[v]);
    return s;
  }, [lang]);
};

/* ---------- Press (tappable feedback) ---------- */
const Press = ({ children, onClick, style, className, disabled, ...rest }) => {
  const [down, setDown] = useState(false);
  return (
    <div
      role="button"
      tabIndex={disabled ? -1 : 0}
      onPointerDown={() => !disabled && setDown(true)}
      onPointerUp={() => setDown(false)}
      onPointerLeave={() => setDown(false)}
      onClick={() => !disabled && onClick && onClick()}
      onKeyDown={(e) => { if (e.key === 'Enter' && !disabled && onClick) onClick(); }}
      aria-disabled={disabled || undefined}
      className={className}
      {...rest}
      style={{
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.5 : 1,
        transform: down ? 'scale(0.97)' : 'scale(1)',
        transition: 'transform 120ms ease, opacity 120ms ease',
        userSelect: 'none',
        outline: 'none',
        ...style,
      }}
    >
      {children}
    </div>
  );
};

/* ---------- Button variants ---------- */
const Button = ({ variant = 'primary', loading, disabled, children, ...rest }) => {
  const styles = {
    primary: {
      background: 'var(--primary)',
      color: 'var(--surface)',
      border: '1px solid var(--primary)',
    },
    secondary: {
      background: 'var(--surface)',
      color: 'var(--fg)',
      border: '1px solid var(--border-strong)',
    },
    danger: {
      background: 'var(--danger)',
      color: 'var(--surface)',
      border: '1px solid var(--danger)',
    },
    ghost: {
      background: 'transparent',
      color: 'var(--fg)',
      border: '1px solid transparent',
    },
  };
  return (
    <Press
      disabled={disabled || loading}
      {...rest}
      style={{
        ...styles[variant],
        padding: 'var(--space-3) var(--space-5)',
        borderRadius: 'var(--radius-md)',
        fontSize: 14,
        fontWeight: 700,
        display: 'inline-flex',
        alignItems: 'center',
        gap: 'var(--space-2)',
        justifyContent: 'center',
        minHeight: 44,
      }}
    >
      {loading && <IconLoader size={16} />}
      {children}
    </Press>
  );
};

/* ---------- Card ---------- */
const Card = ({ children, style }) => (
  <div
    style={{
      background: 'var(--surface)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-lg)',
      padding: 'var(--space-5)',
      boxShadow: 'var(--shadow-sm)',
      ...style,
    }}
  >
    {children}
  </div>
);

/* ---------- Field ---------- */
const Field = ({ label, value, onChange, type = 'text', placeholder, error, disabled }) => {
  const [focus, setFocus] = useState(false);
  return (
    <label style={{ display: 'flex', flexDirection: 'column', gap: 4, width: '100%' }}>
      {label && (
        <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--muted)' }}>{label}</span>
      )}
      <input
        type={type}
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        onChange={(e) => onChange && onChange(e.target.value)}
        onFocus={() => setFocus(true)}
        onBlur={() => setFocus(false)}
        style={{
          background: 'var(--input-bg)',
          color: 'var(--fg)',
          border: `1px solid ${error ? 'var(--danger)' : focus ? 'var(--primary)' : 'var(--border)'}`,
          borderRadius: 'var(--radius-md)',
          padding: 'var(--space-3) var(--space-4)',
          fontSize: 14,
          outline: 'none',
          opacity: disabled ? 0.5 : 1,
          transition: 'border-color 140ms ease',
          minHeight: 44,
        }}
      />
      {error && (
        <span style={{ fontSize: 12, color: 'var(--danger)' }}>{error}</span>
      )}
    </label>
  );
};

/* ---------- Skeleton ---------- */
const Skeleton = ({ w = '100%', h = 16, r = 'var(--radius-sm)' }) => (
  <div
    aria-hidden
    style={{
      width: w,
      height: h,
      borderRadius: r,
      background: 'linear-gradient(90deg, var(--surface-2) 0%, var(--border) 50%, var(--surface-2) 100%)',
      backgroundSize: '200% 100%',
      animation: 'shimmer 1.4s ease-in-out infinite',
    }}
  />
);

/* ---------- Empty / Error states ---------- */
const EmptyState = ({ title, body, cta, onCta }) => (
  <div
    style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: 'var(--space-3)',
      padding: 'var(--space-6)',
      textAlign: 'center',
    }}
  >
    <div
      style={{
        width: 56, height: 56,
        borderRadius: 'var(--radius-full)',
        background: 'var(--surface-2)',
        display: 'grid', placeItems: 'center',
        color: 'var(--muted)',
      }}
    >
      <IconBox size={28} />
    </div>
    <div style={{ fontSize: 16, fontWeight: 700 }}>{title}</div>
    <div style={{ fontSize: 13, color: 'var(--muted)', maxWidth: 320 }}>{body}</div>
    {cta && <Button onClick={onCta}>{cta}</Button>}
  </div>
);

const ErrorState = ({ title, body, cta, onCta }) => (
  <div
    role="alert"
    style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: 'var(--space-3)',
      padding: 'var(--space-6)',
      textAlign: 'center',
    }}
  >
    <div
      style={{
        width: 56, height: 56,
        borderRadius: 'var(--radius-full)',
        background: 'var(--danger-soft)',
        display: 'grid', placeItems: 'center',
        color: 'var(--danger)',
      }}
    >
      <IconAlert size={28} />
    </div>
    <div style={{ fontSize: 16, fontWeight: 700 }}>{title}</div>
    <div style={{ fontSize: 13, color: 'var(--muted)', maxWidth: 320 }}>{body}</div>
    {cta && <Button variant="secondary" onClick={onCta}>{cta}</Button>}
  </div>
);

/* ---------- Sem internet ----------
   Avisa na hora que a conexão caiu, diz o que continua funcionando e o que espera.
   A tela continua de pé por baixo: sem internet não é tela de erro. */
const AvisoSemInternet = ({ titulo, continuaFuncionando, espera }) => (
  <div
    role="status"
    style={{
      display: 'flex', gap: 'var(--space-3)', alignItems: 'flex-start',
      padding: 'var(--space-3) var(--space-4)',
      background: 'var(--warn-soft)', color: 'var(--fg)',
      border: '1px solid var(--warn)', borderRadius: 'var(--radius-md)',
    }}
  >
    <IconWifiOff size={20} style={{ color: 'var(--warn)', flexShrink: 0, marginTop: 2 }} />
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-1)' }}>
      <div style={{ fontSize: 14, fontWeight: 800 }}>{titulo}</div>
      {continuaFuncionando && <div style={{ fontSize: 13 }}>{continuaFuncionando}</div>}
      {espera && <div style={{ fontSize: 13, color: 'var(--muted)' }}>{espera}</div>}
    </div>
  </div>
);

/* Aviso curto de que a conexão voltou. O Showcase mostra sozinho ao sair do
   estado offline; a tela não precisa cuidar disso. */
const AvisoConexaoVoltou = ({ texto }) => (
  <div
    role="status"
    style={{
      position: 'absolute', left: 'var(--space-4)', right: 'var(--space-4)', bottom: 'var(--space-5)',
      display: 'flex', gap: 'var(--space-2)', alignItems: 'center',
      padding: 'var(--space-3) var(--space-4)',
      background: 'var(--toast-bg)', color: 'var(--toast-fg)',
      borderRadius: 'var(--radius-md)', boxShadow: 'var(--shadow-lg)',
      fontSize: 13, fontWeight: 700, zIndex: 60, animation: 'fadeIn 160ms ease',
    }}
  >
    <IconWifi size={18} />
    <span>{texto}</span>
  </div>
);

/* ---------- Animation keyframes injection ---------- */
const styleId = '__prototype_keyframes__';
if (!document.getElementById(styleId)) {
  const style = document.createElement('style');
  style.id = styleId;
  style.textContent = `
    @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
    @keyframes shimmer { from { background-position: 200% 0; } to { background-position: -200% 0; } }
    @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  `;
  document.head.appendChild(style);
}
