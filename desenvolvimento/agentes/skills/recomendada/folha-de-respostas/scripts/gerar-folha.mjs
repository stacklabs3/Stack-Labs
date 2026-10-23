#!/usr/bin/env node
// Gera a folha de respostas .html de uma rodada de perguntas a partir de um JSON.
// Uso: node gerar-folha.mjs <perguntas.json> <saida.html>
// O formato do JSON está no SKILL.md da skill folha-de-respostas.
import { readFileSync, writeFileSync } from 'node:fs';

const [caminhoDoJson, caminhoDaSaida] = process.argv.slice(2);
if (!caminhoDoJson || !caminhoDaSaida) {
  console.error('Uso: node gerar-folha.mjs <perguntas.json> <saida.html>');
  process.exit(1);
}

const rodada = JSON.parse(readFileSync(caminhoDoJson, 'utf8'));
const problemas = [];
for (const campoObrigatorio of ['titulo', 'chave', 'cabecalhoDaCopia', 'perguntas']) {
  if (!rodada[campoObrigatorio]) problemas.push(`falta o campo "${campoObrigatorio}"`);
}
const idsVistos = new Set();
for (const [posicao, pergunta] of (rodada.perguntas || []).entries()) {
  if (pergunta.bloco) continue;
  const onde = pergunta.id || `pergunta na posição ${posicao}`;
  if (!pergunta.id) problemas.push(`${onde}: falta "id"`);
  if (idsVistos.has(pergunta.id)) problemas.push(`${onde}: id repetido`);
  idsVistos.add(pergunta.id);
  if (!/^[A-Za-z][A-Za-z0-9_-]*$/.test(pergunta.id || '')) problemas.push(`${onde}: o id precisa começar com letra e ter só letras, números, _ ou -`);
  if (!pergunta.titulo) problemas.push(`${onde}: falta "titulo"`);
  if (!Array.isArray(pergunta.opcoes) || pergunta.opcoes.length < 2 || pergunta.opcoes.length > 5) problemas.push(`${onde}: "opcoes" precisa ter de 2 a 5 itens`);
  if (pergunta.rec !== undefined && (pergunta.rec < 0 || pergunta.rec >= (pergunta.opcoes || []).length)) problemas.push(`${onde}: "rec" aponta para uma opção que não existe`);
}
if (problemas.length) {
  console.error('A folha não foi gerada:\n- ' + problemas.join('\n- '));
  process.exit(1);
}

const escaparHtml = (texto) => String(texto).replace(/[&<>"]/g, (caractere) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[caractere]));
// O JSON vai dentro de <script>: "</" não pode fechar a tag antes da hora.
const dadosNoScript = JSON.stringify({ perguntas: rodada.perguntas, chave: rodada.chave, cabecalhoDaCopia: rodada.cabecalhoDaCopia }).replace(/<\//g, '<\\/');
const paragrafosDaIntro = (rodada.intro || []).map((paragrafo) => `<p class="intro">${escaparHtml(paragrafo)}</p>`).join('\n');
const totalDePerguntas = rodada.perguntas.filter((pergunta) => !pergunta.bloco).length;

const html = `<!doctype html>
<!-- Folha de respostas descartável, gerada pela skill folha-de-respostas. Abre com dois cliques.
     Marque, escreva a observação e use "Copiar respostas". -->
<html lang="pt-BR">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escaparHtml(rodada.tituloDaAba || rodada.titulo)}</title>
<style>
:root{--bg:#FAF7F4;--surface:#fff;--text:#1C1917;--muted:#57534E;--line:#E7E0D8;--brand:#C2410C;--brand-soft:#FFEDD5;--brand-text:#9A3412;--ok:#15803D;--ok-soft:#DCFCE7;--hover:#F5F0EA;--sec:#F3EEE8;--campo:#FFFDFB;--marcada:#FDBA74;--feita:#86EFAC}
@media (prefers-color-scheme: dark){:root{--bg:#131010;--surface:#1D1917;--text:#F5F0EB;--muted:#B9AEA5;--line:#3A322D;--brand:#FB923C;--brand-soft:#3A2116;--brand-text:#FDBA74;--ok:#4ADE80;--ok-soft:#14321F;--hover:#262019;--sec:#2A231F;--campo:#171311;--marcada:#9A3412;--feita:#166534}}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--text);font:15px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif;-webkit-font-smoothing:antialiased}
main{max-width:760px;margin:0 auto;padding:24px 16px 140px}
h1{font-size:22px;margin:0 0 8px;text-wrap:balance}
.intro{color:var(--muted);margin:0 0 12px}
h2{font-size:13px;text-transform:uppercase;letter-spacing:.06em;color:var(--muted);margin:28px 0 8px}
.q{background:var(--surface);border:1px solid var(--line);border-radius:14px;padding:16px;margin-bottom:12px}
.q.feita{border-color:var(--feita)}
.q h3{font-size:16px;margin:0 0 4px}
.q h3 b{color:var(--brand-text)}
.cena{color:var(--muted);margin:0 0 10px;font-size:14px}
label.op{display:flex;gap:10px;align-items:flex-start;padding:9px 10px;border-radius:10px;cursor:pointer;border:1px solid transparent}
label.op:hover{background:var(--hover)}
label.op:has(input:checked){background:var(--brand-soft);border-color:var(--marcada)}
label.op input{margin-top:4px;accent-color:var(--brand)}
.rec{font-size:11px;font-weight:700;color:var(--ok);background:var(--ok-soft);border-radius:999px;padding:1px 7px;margin-left:6px;white-space:nowrap}
textarea{width:100%;margin-top:8px;min-height:44px;border:1px solid var(--line);border-radius:10px;padding:8px 10px;font:inherit;color:inherit;resize:vertical;background:var(--campo)}
textarea:focus{outline:2px solid var(--brand);outline-offset:1px}
.barra{position:fixed;left:0;right:0;bottom:0;background:var(--surface);border-top:1px solid var(--line);padding:12px 16px;display:flex;gap:10px;justify-content:center;align-items:center;flex-wrap:wrap}
.barra button{border:0;border-radius:10px;padding:11px 18px;font:inherit;font-weight:700;cursor:pointer;transition:transform 160ms ease-out}
.barra button:active{transform:scale(.97)}
.principal{background:var(--brand);color:#fff}
.sec{background:var(--sec);color:var(--text)}
#status{color:var(--ok);font-weight:700;min-width:140px}
#conta{color:var(--muted);font-size:14px}
</style>
</head>
<body>
<main>
<h1>${escaparHtml(rodada.titulo)}</h1>
${paragrafosDaIntro}
<p class="intro">Marque uma opção em cada pergunta (o botão de baixo marca a recomendação nas que faltam), escreva a observação se quiser e use “Copiar respostas” para colar no chat.</p>
<div id="lista"></div>
</main>
<div class="barra">
  <span id="conta">0 de ${totalDePerguntas} marcadas</span>
  <button class="sec" id="tudoRec">Marcar a recomendação nas que faltam</button>
  <button class="principal" id="copiar">Copiar respostas</button>
  <span id="status" role="status" aria-live="polite"></span>
</div>
<script>
const RODADA = ${dadosNoScript};
const LETRA = (indice) => String.fromCharCode(65 + indice);
let salvo = {};
try { salvo = JSON.parse(localStorage.getItem(RODADA.chave)) || {}; } catch (erro) {}
const lista = document.getElementById('lista');
const criar = (tag, classe, texto) => { const elemento = document.createElement(tag); if (classe) elemento.className = classe; if (texto !== undefined) elemento.textContent = texto; return elemento; };
for (const pergunta of RODADA.perguntas) {
  if (pergunta.bloco) { lista.append(criar('h2', '', pergunta.bloco)); continue; }
  const cartao = criar('section', 'q'); cartao.id = 'pergunta-' + pergunta.id;
  const titulo = criar('h3'); titulo.append(criar('b', '', pergunta.id), ' · ' + pergunta.titulo); cartao.append(titulo);
  if (pergunta.cena) cartao.append(criar('p', 'cena', pergunta.cena));
  pergunta.opcoes.forEach((textoDaOpcao, indice) => {
    const rotulo = criar('label', 'op');
    const escolha = document.createElement('input'); escolha.type = 'radio'; escolha.name = pergunta.id; escolha.value = indice;
    if (salvo[pergunta.id]?.opcao === indice) escolha.checked = true;
    const texto = criar('span'); texto.append(criar('b', '', LETRA(indice) + ')'), ' ' + textoDaOpcao);
    if (indice === pergunta.rec) texto.append(criar('span', 'rec', 'recomendado'));
    rotulo.append(escolha, texto); cartao.append(rotulo);
  });
  const observacao = criar('textarea'); observacao.placeholder = 'Observação (opcional)'; observacao.setAttribute('aria-label', 'Observação da ' + pergunta.id);
  observacao.value = salvo[pergunta.id]?.obs || ''; cartao.append(observacao);
  lista.append(cartao);
}
const perguntasReais = RODADA.perguntas.filter((pergunta) => !pergunta.bloco);
function lerRespostas() {
  return perguntasReais.map((pergunta) => {
    const marcada = document.querySelector('input[name="' + pergunta.id + '"]:checked');
    const observacao = document.querySelector('#pergunta-' + pergunta.id + ' textarea').value.trim();
    return { pergunta, opcao: marcada ? Number(marcada.value) : null, obs: observacao };
  });
}
function atualizar() {
  const respostas = lerRespostas(); const paraGuardar = {};
  for (const resposta of respostas) {
    document.getElementById('pergunta-' + resposta.pergunta.id).classList.toggle('feita', resposta.opcao !== null || !!resposta.obs);
    paraGuardar[resposta.pergunta.id] = { opcao: resposta.opcao, obs: resposta.obs };
  }
  try { localStorage.setItem(RODADA.chave, JSON.stringify(paraGuardar)); } catch (erro) {}
  const marcadas = respostas.filter((resposta) => resposta.opcao !== null).length;
  document.getElementById('conta').textContent = marcadas + ' de ' + respostas.length + ' marcadas';
}
document.addEventListener('input', atualizar);
document.getElementById('tudoRec').onclick = () => {
  for (const pergunta of perguntasReais) {
    if (pergunta.rec === undefined || document.querySelector('input[name="' + pergunta.id + '"]:checked')) continue;
    document.querySelector('input[name="' + pergunta.id + '"][value="' + pergunta.rec + '"]').checked = true;
  }
  atualizar();
};
document.getElementById('copiar').onclick = async () => {
  const linhas = lerRespostas().map((resposta) => {
    const letra = resposta.opcao === null ? '—' : LETRA(resposta.opcao);
    const marcaDaRecomendacao = resposta.opcao !== null && resposta.opcao === resposta.pergunta.rec ? ' (recomendado)' : '';
    return resposta.pergunta.id + ' = ' + letra + marcaDaRecomendacao + (resposta.obs ? ' · Obs: ' + resposta.obs : '');
  });
  const texto = RODADA.cabecalhoDaCopia + '\\n' + linhas.join('\\n');
  const status = document.getElementById('status');
  try { await navigator.clipboard.writeText(texto); }
  catch (erro) { const area = document.createElement('textarea'); area.value = texto; document.body.append(area); area.select(); document.execCommand('copy'); area.remove(); }
  status.textContent = 'Copiado ✓';
  setTimeout(() => { status.textContent = ''; }, 2500);
};
atualizar();
</script>
</body>
</html>
`;

writeFileSync(caminhoDaSaida, html);
console.log(`Folha gerada: ${caminhoDaSaida} (${totalDePerguntas} perguntas)`);
