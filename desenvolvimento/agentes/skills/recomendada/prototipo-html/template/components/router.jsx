// Protótipo — Router hash-based.
// Navegação real entre telas, como no app final: o showcase não troca de tela por
// aba, ele navega. Em produção isso vira go_router (Flutter) ou o router da stack.
//
// O mapa rota → componente vive em app.jsx, que carrega por último e enxerga as
// telas. Aqui ficam só as rotas, o estado e a navegação.

const ROUTES = {
  HOME:   '#/home',
  DETAIL: '#/detail/:id',
};

const matchRoute = (hash) => {
  if (!hash || hash === '#' || hash === '') return { name: 'HOME', params: {} };

  for (const [name, pattern] of Object.entries(ROUTES)) {
    const re = new RegExp('^' + pattern.replace(/:(\w+)/g, '(?<$1>[^/]+)') + '$');
    const m = hash.match(re);
    if (m) return { name, params: m.groups || {} };
  }
  return { name: 'HOME', params: {} };
};

const buildPath = (name, params = {}) => {
  let path = ROUTES[name] || ROUTES.HOME;
  for (const [k, v] of Object.entries(params)) path = path.replace(`:${k}`, v);
  return path;
};

const RouterContext = React.createContext(null);

const RouterProvider = ({ children }) => {
  const [route, setRoute] = React.useState(() => matchRoute(window.location.hash));

  React.useEffect(() => {
    const onHash = () => setRoute(matchRoute(window.location.hash));
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  const navigate = React.useCallback((name, params) => {
    window.location.hash = buildPath(name, params);
  }, []);

  const value = React.useMemo(() => ({ route, navigate }), [route, navigate]);

  return <RouterContext.Provider value={value}>{children}</RouterContext.Provider>;
};

const useRouter = () => {
  const ctx = React.useContext(RouterContext);
  if (!ctx) throw new Error('useRouter precisa estar dentro de <RouterProvider>');
  return ctx;
};
