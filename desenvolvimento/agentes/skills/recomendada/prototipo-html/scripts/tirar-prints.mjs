// Tira os prints da conferência de uma rotina (ou de todas) do protótipo servido.
//
// Uso: node tirar-prints.mjs <endereco> <id-da-rotina|todas> <pasta-de-saida> [idioma] [paralelo]
//   ex.: node tirar-prints.mjs http://localhost:8765 turno-do-caixa prototipos/meu-app/prints
// O idioma padrão é pt-BR; o paralelo padrão é 6 abas. CHROME=<caminho> escolhe o navegador.
//
// A lista vem do próprio protótipo (?listarPrints=<rotina>). Cada print abre numa aba do
// Chrome sem janela com a área de tela exata da combinação (CDP), espera a tela montar e
// fotografa. O --screenshot do Chrome não serve: a janela dele sai 87 px mais baixa e com
// no mínimo 500 px de largura, o que cortava e deslocava a moldura.
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';

const [enderecoDoPrototipo, idDaRotina, pastaDeSaida, idiomaTexto, paraleloTexto] = process.argv.slice(2);
if (!enderecoDoPrototipo || !idDaRotina || !pastaDeSaida) {
  console.error('Uso: node tirar-prints.mjs <endereco> <id-da-rotina|todas> <pasta-de-saida> [idioma] [paralelo]');
  process.exit(2);
}
const endereco = enderecoDoPrototipo.replace(/\/$/, '');
const idioma = idiomaTexto || 'pt-BR';
const quantidadeEmParalelo = Number(paraleloTexto || 6);
const FOLGA_DEPOIS_DE_MONTAR_MS = 1500;
const LIMITE_PARA_MONTAR_MS = 20000;
const CANDIDATOS_A_CHROME = [
  process.env.CHROME,
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  '/Applications/Chromium.app/Contents/MacOS/Chromium',
  '/usr/bin/google-chrome', '/usr/bin/chromium', '/usr/bin/chromium-browser',
].filter(Boolean);
const CAMINHO_DO_CHROME = CANDIDATOS_A_CHROME.find((caminhoDoCandidato) => existsSync(caminhoDoCandidato));
if (!CAMINHO_DO_CHROME) { console.error('Nenhum Chrome ou Chromium encontrado para tirar os prints (use CHROME=<caminho>).'); process.exit(1); }

const dormir = (milissegundos) => new Promise((resolver) => setTimeout(resolver, milissegundos));

const portaDoChrome = 9400 + Math.floor(Math.random() * 500);
const perfilDoChrome = mkdtempSync(join(tmpdir(), 'prints-'));
const chrome = spawn(CAMINHO_DO_CHROME, [
  '--headless=new', '--disable-gpu', '--hide-scrollbars', `--remote-debugging-port=${portaDoChrome}`,
  // Abas em paralelo: nenhuma fica "em segundo plano" (sem isso a folha não abria a tempo).
  '--disable-background-timer-throttling', '--disable-renderer-backgrounding', '--disable-backgrounding-occluded-windows',
  `--user-data-dir=${perfilDoChrome}`, 'about:blank'], { stdio: 'ignore' });

let enderecoDoNavegador;
for (let tentativa = 0; tentativa < 100 && !enderecoDoNavegador; tentativa++) {
  try { enderecoDoNavegador = (await (await fetch(`http://127.0.0.1:${portaDoChrome}/json/version`)).json()).webSocketDebuggerUrl; } catch (erroAoSubir) { await dormir(100); }
}
if (!enderecoDoNavegador) { console.error('O Chrome não subiu.'); process.exit(1); }

const conexao = new WebSocket(enderecoDoNavegador);
await new Promise((resolver) => conexao.addEventListener('open', resolver));
let proximoId = 1;
const respostasPendentes = new Map();
conexao.addEventListener('message', (evento) => {
  const mensagem = JSON.parse(evento.data);
  if (mensagem.id && respostasPendentes.has(mensagem.id)) {
    respostasPendentes.get(mensagem.id)(mensagem);
    respostasPendentes.delete(mensagem.id);
  }
});
const enviar = (metodo, parametros = {}, idDaSessao) => new Promise((resolver) => {
  const id = proximoId++;
  respostasPendentes.set(id, resolver);
  conexao.send(JSON.stringify({ id, method: metodo, params: parametros, ...(idDaSessao ? { sessionId: idDaSessao } : {}) }));
});

const abrirAba = async () => {
  const { result: { targetId: idDaAba } } = await enviar('Target.createTarget', { url: 'about:blank', newWindow: true });
  const { result: { sessionId: idDaSessao } } = await enviar('Target.attachToTarget', { targetId: idDaAba, flatten: true });
  await enviar('Page.enable', {}, idDaSessao);
  await enviar('Emulation.setFocusEmulationEnabled', { enabled: true }, idDaSessao);
  return idDaSessao;
};

const avaliar = async (idDaSessao, expressao) => {
  const resposta = await enviar('Runtime.evaluate', { expression: expressao, returnByValue: true }, idDaSessao);
  return resposta.result && resposta.result.result ? resposta.result.result.value : undefined;
};

const esperarAte = async (idDaSessao, expressao) => {
  const inicio = Date.now();
  while (Date.now() - inicio < LIMITE_PARA_MONTAR_MS) {
    if (await avaliar(idDaSessao, expressao)) return true;
    await dormir(150);
  }
  return false;
};

// A lista de prints da rotina.
const abaDaLista = await abrirAba();
await enviar('Page.navigate', { url: `${endereco}/?listarPrints=${encodeURIComponent(idDaRotina)}` }, abaDaLista);
await esperarAte(abaDaLista, "!!document.getElementById('lista-de-prints')");
const listaDePrints = JSON.parse(await avaliar(abaDaLista, "document.getElementById('lista-de-prints').textContent") || '[]');
if (!listaDePrints.length) { console.error(`Nenhum print para "${idDaRotina}". O protótipo está servido e a rotina existe em visoes.jsx?`); chrome.kill(); process.exit(1); }

// A tela está montada quando o palco do print tem conteúdo e as fontes carregaram.
const TELA_MONTADA = "(() => { const palco = document.querySelector('.showcase-body--print'); return !!palco && palco.innerText.trim().length > 0 && document.fonts.status === 'loaded'; })()";

const tirarUmPrint = async (idDaSessao, { arquivo, endereco: enderecoDaCombinacao, largura, altura }) => {
  await enviar('Page.navigate', { url: 'about:blank' }, idDaSessao);
  await enviar('Emulation.setDeviceMetricsOverride', { width: largura, height: altura, deviceScaleFactor: 1, mobile: false }, idDaSessao);
  await enviar('Page.navigate', { url: `${endereco}/${enderecoDaCombinacao.replace('#', `&idioma=${idioma}#`)}` }, idDaSessao);
  const montou = await esperarAte(idDaSessao, TELA_MONTADA);
  await dormir(FOLGA_DEPOIS_DE_MONTAR_MS);
  const { result: { data: imagemEmBase64 } } = await enviar('Page.captureScreenshot', {
    format: 'png', clip: { x: 0, y: 0, width: largura, height: altura, scale: 1 },
  }, idDaSessao);
  const destino = join(pastaDeSaida, arquivo);
  mkdirSync(dirname(destino), { recursive: true });
  writeFileSync(destino, Buffer.from(imagemEmBase64, 'base64'));
  console.log(`${montou ? '' : '(sem montar) '}${destino}`);
};

const filaDePrints = [...listaDePrints];
const trabalhar = async () => {
  const idDaSessao = await abrirAba();
  while (filaDePrints.length) await tirarUmPrint(idDaSessao, filaDePrints.shift());
};
await Promise.all(Array.from({ length: quantidadeEmParalelo }, trabalhar));
console.error(`${listaDePrints.length} prints em ${pastaDeSaida}`);

conexao.close();
chrome.kill();
await dormir(300);
rmSync(perfilDoChrome, { recursive: true, force: true });
process.exit(0);
