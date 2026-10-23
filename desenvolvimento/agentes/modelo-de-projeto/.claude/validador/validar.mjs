#!/usr/bin/env node
// Validador de regras de código (ADR 0010 do método): lê docs/regras-de-codigo.json, roda a
// verificação de cada regra e confere a cobertura dos 22 princípios em docs/regras-de-codigo.md.
// O motor é do método e fica protegido; o projeto refina só o JSON e os arquivos que ele aponta.
//
// Uso: node .claude/validador/validar.mjs [--config <json>] [--md <md>] [--base <ref>] [--so-cobertura] [--json]
// Saída: 0 aprovado, 1 violação, 2 configuração inválida. A última linha é "VALIDADOR: APROVADO" ou
// "VALIDADOR: REPROVADO".

import { readFileSync, existsSync } from "node:fs";
import { join, dirname, basename, extname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export const PRINCIPIOS = ["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "S", "O", "L", "I", "D"];
export const DECISOES = ["adotado", "adaptado", "afastado", "garantido pelo lint"];

export class ErroDeConfiguracao extends Error {}

// ---------- arquivos e globs ----------

export function globParaRegex(padraoGlob) {
  let expressao = "";
  for (let indice = 0; indice < padraoGlob.length; indice += 1) {
    const caractere = padraoGlob[indice];
    if (caractere === "*" && padraoGlob[indice + 1] === "*") {
      const seguidoDeBarra = padraoGlob[indice + 2] === "/";
      expressao += seguidoDeBarra ? "(?:.*/)?" : ".*";
      indice += seguidoDeBarra ? 2 : 1;
    } else if (caractere === "*") expressao += "[^/]*";
    else if (caractere === "?") expressao += "[^/]";
    else if (caractere === "{") {
      const fim = padraoGlob.indexOf("}", indice);
      expressao += `(?:${padraoGlob.slice(indice + 1, fim).split(",").map(escaparRegex).join("|")})`;
      indice = fim;
    } else expressao += escaparRegex(caractere);
  }
  return new RegExp(`^${expressao}$`);
}

function escaparRegex(texto) {
  return texto.replace(/[.+^${}()|[\]\\]/g, "\\$&");
}

export function casaAlgum(caminhoRelativo, padroesGlob = []) {
  return [].concat(padroesGlob).some((padraoGlob) => globParaRegex(padraoGlob).test(caminhoRelativo));
}

function listarArquivosDoRepositorio(raiz) {
  const resultado = spawnSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], { cwd: raiz, encoding: "utf8" });
  if (resultado.status !== 0) throw new ErroDeConfiguracao("o validador roda dentro de um repositório git (git ls-files falhou)");
  return resultado.stdout.split("\n").filter(Boolean).filter((caminhoRelativo) => existsSync(join(raiz, caminhoRelativo)));
}

function lerTexto(raiz, caminhoRelativo) {
  return readFileSync(join(raiz, caminhoRelativo), "utf8");
}

// ---------- cobertura: os 22 princípios decididos no .md e ligados às regras do .json ----------

export function lerTabelaDeCobertura(textoMd) {
  const linhasDaTabela = new Map();
  for (const linha of textoMd.split("\n")) {
    const celulas = linha.split("|").slice(1, -1).map((celula) => celula.trim());
    if (celulas.length < 3 || !PRINCIPIOS.includes(celulas[0])) continue;
    linhasDaTabela.set(celulas[0], { decisao: celulas[1].toLowerCase(), ondeOuPorque: celulas[2] });
  }
  return linhasDaTabela;
}

export function checarCobertura({ textoMd, regras }) {
  const violacoes = [];
  const tabela = lerTabelaDeCobertura(textoMd);
  const idsDasRegras = new Set(regras.map((regra) => regra.id));
  const regrasCitadas = new Set();
  for (const principio of PRINCIPIOS) {
    const linhaDoPrincipio = tabela.get(principio);
    if (!linhaDoPrincipio) { violacoes.push({ mensagem: `${principio} sem linha na tabela de cobertura` }); continue; }
    if (!DECISOES.includes(linhaDoPrincipio.decisao)) {
      violacoes.push({ mensagem: `${principio}: decisão "${linhaDoPrincipio.decisao}" não é uma de: ${DECISOES.join(", ")}` });
      continue;
    }
    if (linhaDoPrincipio.decisao === "afastado") {
      if (!linhaDoPrincipio.ondeOuPorque) violacoes.push({ mensagem: `${principio} afastado sem o porquê` });
      continue;
    }
    const idsNaLinha = linhaDoPrincipio.ondeOuPorque.match(/\bR\d+\b/g) || [];
    if (!idsNaLinha.length) violacoes.push({ mensagem: `${principio} ${linhaDoPrincipio.decisao} sem regra (R<n>) na coluna "Onde"` });
    for (const idDaRegra of idsNaLinha) {
      regrasCitadas.add(idDaRegra);
      const regraCitada = regras.find((regra) => regra.id === idDaRegra);
      if (!idsDasRegras.has(idDaRegra)) violacoes.push({ mensagem: `${principio} cita ${idDaRegra}, que não está em regras-de-codigo.json` });
      else if (regraCitada.principio !== principio) violacoes.push({ mensagem: `${idDaRegra} nasce de ${regraCitada.principio} no .json, mas a tabela a cita em ${principio}` });
    }
  }
  for (const regra of regras) {
    if (!regrasCitadas.has(regra.id)) violacoes.push({ mensagem: `${regra.id} está no .json e nenhuma linha da tabela de cobertura a cita` });
  }
  return violacoes;
}

// ---------- catálogo de checagens ----------
// Toda checagem recebe (parametros, contexto) e devolve [{ arquivo?, linha?, mensagem }].

function exigir(parametros, nomes, tipo) {
  for (const nome of nomes) {
    if (parametros[nome] === undefined) throw new ErroDeConfiguracao(`verificação "${tipo}" exige o parâmetro "${nome}"`);
  }
}

function arquivosDeCodigo(contexto, parametros) {
  const padroesDeCodigo = parametros.arquivos || contexto.projeto.codigo || ["**"];
  const padroesIgnorados = [...(contexto.projeto.ignorar || []), ...(parametros.ignorar || [])];
  return contexto.arquivos.filter((caminhoRelativo) => casaAlgum(caminhoRelativo, padroesDeCodigo) && !casaAlgum(caminhoRelativo, padroesIgnorados));
}

function semAcento(texto) {
  return texto.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

function quebrarIdentificador(identificador) {
  return identificador.replace(/([a-z0-9])([A-Z])/g, "$1 $2").split(/[\s_\-]+/).filter(Boolean).map(semAcento);
}

export function termosEvitadosDoGlossario(textoGlossario) {
  const termos = [];
  for (const linha of textoGlossario.split("\n")) {
    const encontrado = linha.match(/^_(?:Evitar|Avoid)_:\s*(.+)$/);
    if (!encontrado) continue;
    for (const termoBruto of encontrado[1].split(",")) {
      const termo = termoBruto.replace(/\(.*?\)/g, "").replace(/["“”]/g, "").trim();
      if (termo.length >= 3) termos.push(termo);
    }
  }
  return termos;
}

const DECLARACAO = /\b(?:function|class|const|let|var|def|type|interface|enum|struct|fn|func|record|trait|object)\s+([A-Za-zÀ-ÿ_][\wÀ-ÿ]*)/g;

export const CHECAGENS = {
  // E1: nenhum nome declarado usa um termo que o GLOSSARY.md manda evitar.
  "termo-evitado"(parametros, contexto) {
    const caminhoDoGlossario = parametros.glossario || "GLOSSARY.md";
    if (!existsSync(join(contexto.raiz, caminhoDoGlossario))) return [{ mensagem: `glossário ${caminhoDoGlossario} não existe` }];
    const ignorados = new Set((parametros.ignorarTermos || []).map(semAcento));
    const termos = termosEvitadosDoGlossario(lerTexto(contexto.raiz, caminhoDoGlossario))
      .map((termo) => ({ termo, partes: quebrarIdentificador(termo) }))
      .filter(({ termo }) => !ignorados.has(semAcento(termo)));
    const violacoes = [];
    for (const caminhoRelativo of arquivosDeCodigo(contexto, parametros)) {
      const nomesDoArquivo = [{ nome: basename(caminhoRelativo, extname(caminhoRelativo)), linha: undefined }];
      lerTexto(contexto.raiz, caminhoRelativo).split("\n").forEach((textoDaLinha, indice) => {
        for (const declaracao of textoDaLinha.matchAll(DECLARACAO)) nomesDoArquivo.push({ nome: declaracao[1], linha: indice + 1 });
      });
      for (const { nome, linha } of nomesDoArquivo) {
        const partesDoNome = quebrarIdentificador(nome);
        for (const { termo, partes } of termos) {
          const contem = partesDoNome.some((_, inicio) => partes.every((parte, deslocamento) => partesDoNome[inicio + deslocamento] === parte));
          if (contem) violacoes.push({ arquivo: caminhoRelativo, linha, mensagem: `"${nome}" usa "${termo}", que o glossário manda evitar` });
        }
      }
    }
    return violacoes;
  },

  // L: toda port tem um teste de contrato. "{dir}" e "{nome}" vêm do arquivo da port; arquivos de
  // teste na pasta das ports não contam como port.
  "teste-de-contrato-por-port"(parametros, contexto) {
    exigir(parametros, ["ports", "contrato"], "teste-de-contrato-por-port");
    const naoSaoPorts = parametros.ignorar || ["**/*.test.*", "**/*.spec.*"];
    const ports = contexto.arquivos.filter((caminhoRelativo) => casaAlgum(caminhoRelativo, parametros.ports) && !casaAlgum(caminhoRelativo, naoSaoPorts));
    return ports.flatMap((caminhoDaPort) => {
      const nomeDaPort = basename(caminhoDaPort, extname(caminhoDaPort));
      const caminhoDoContrato = parametros.contrato.replaceAll("{dir}", dirname(caminhoDaPort)).replaceAll("{nome}", nomeDaPort);
      return existsSync(join(contexto.raiz, caminhoDoContrato)) ? [] : [{ arquivo: caminhoDaPort, mensagem: `port sem teste de contrato em ${caminhoDoContrato}` }];
    });
  },

  // "garantido pelo lint": a configuração do lint tem as regras que o projeto delegou a ele.
  "lint-cobre"(parametros, contexto) {
    exigir(parametros, ["config", "regras"], "lint-cobre");
    const caminhosDeConfig = [].concat(parametros.config);
    const existentes = caminhosDeConfig.filter((caminhoRelativo) => existsSync(join(contexto.raiz, caminhoRelativo)));
    if (!existentes.length) return [{ mensagem: `nenhuma configuração de lint encontrada em ${caminhosDeConfig.join(", ")}` }];
    const textoDasConfigs = existentes.map((caminhoRelativo) => lerTexto(contexto.raiz, caminhoRelativo)).join("\n");
    return parametros.regras.filter((nomeDaRegra) => !textoDasConfigs.includes(nomeDaRegra))
      .map((nomeDaRegra) => ({ arquivo: existentes[0], mensagem: `a regra de lint "${nomeDaRegra}" não está configurada` }));
  },

  // E6: TODO e FIXME só com a issue que vai resolvê-los.
  "todo-sem-issue"(parametros, contexto) {
    const marcadores = parametros.marcadores || ["TODO", "FIXME"];
    const padraoDeMarcador = new RegExp(`\\b(?:${marcadores.map(escaparRegex).join("|")})\\b`);
    const padraoDeIssue = new RegExp(parametros.issue || "#\\d+|/issues/\\d+");
    const violacoes = [];
    for (const caminhoRelativo of arquivosDeCodigo(contexto, parametros)) {
      lerTexto(contexto.raiz, caminhoRelativo).split("\n").forEach((textoDaLinha, indice) => {
        if (padraoDeMarcador.test(textoDaLinha) && !padraoDeIssue.test(textoDaLinha)) {
          violacoes.push({ arquivo: caminhoRelativo, linha: indice + 1, mensagem: "marcador sem a issue que o resolve" });
        }
      });
    }
    return violacoes;
  },

  // P9: o título de cada commit desde a base segue o formato escolhido.
  "mensagem-de-commit"(parametros, contexto) {
    exigir(parametros, ["padrao"], "mensagem-de-commit");
    const base = contexto.base || parametros.base || "origin/main";
    const baseExiste = spawnSync("git", ["rev-parse", "--verify", "--quiet", base], { cwd: contexto.raiz }).status === 0;
    if (!baseExiste) { contexto.avisos.push(`mensagem-de-commit: a base ${base} não existe; commits não conferidos`); return []; }
    const historico = spawnSync("git", ["log", "--no-merges", "--format=%h%x09%s", `${base}..HEAD`], { cwd: contexto.raiz, encoding: "utf8" });
    const padraoDoTitulo = new RegExp(parametros.padrao);
    return historico.stdout.split("\n").filter(Boolean).map((linha) => linha.split("\t"))
      .filter(([, titulo]) => !padraoDoTitulo.test(titulo))
      .map(([hash, titulo]) => ({ mensagem: `commit ${hash} "${titulo}" fora do formato ${parametros.padrao}` }));
  },

  // Qualquer ferramenta (lint, dependency-cruiser, jscpd, typecheck): saída diferente de 0 é violação.
  comando(parametros, contexto) {
    exigir(parametros, ["executar"], "comando");
    const resultado = spawnSync(parametros.executar, { cwd: contexto.raiz, shell: true, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
    if (resultado.status === 0) return [];
    if (resultado.status === 127) return [{ mensagem: `ferramenta não encontrada: ${parametros.executar}` }];
    const finalDaSaida = `${resultado.stdout || ""}${resultado.stderr || ""}`.trim().split("\n").slice(-15).join("\n");
    return [{ mensagem: `"${parametros.executar}" saiu com ${resultado.status}:\n${finalDaSaida}` }];
  },

  // Padrões de código em 30+ linguagens, com regras que o projeto escreve a partir dos modelos.
  opengrep(parametros, contexto) {
    exigir(parametros, ["regras"], "opengrep");
    const binario = parametros.binario || process.env.OPENGREP || "opengrep";
    const argumentos = ["scan", "--json", "--quiet", ...[].concat(parametros.regras).flatMap((regra) => ["--config", regra]), ...[].concat(parametros.alvos || ".")];
    const resultado = spawnSync(binario, argumentos, { cwd: contexto.raiz, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
    if (resultado.error?.code === "ENOENT") return [{ mensagem: `opengrep não encontrado (${binario}); instale-o no CI (github.com/opengrep/opengrep)` }];
    let saida;
    try { saida = JSON.parse(resultado.stdout); } catch { return [{ mensagem: `opengrep falhou: ${(resultado.stderr || "").trim().split("\n").slice(-5).join(" ")}` }]; }
    const erros = (saida.errors || []).filter((erro) => erro.level !== "warn");
    return [
      ...erros.map((erro) => ({ mensagem: `opengrep: ${erro.message || erro.type}` })),
      ...(saida.results || []).map((achado) => ({ arquivo: achado.path, linha: achado.start?.line, mensagem: (achado.extra?.message || achado.check_id).trim() })),
    ];
  },

  // Princípios de julgamento: o validador só os lista; quem decide é o revisor da orquestração.
  revisor() {
    return [];
  },
};

// ---------- execução ----------

export function validar({ raiz, config, textoMd, base, soCobertura = false }) {
  if (config.versao !== 1) throw new ErroDeConfiguracao(`versão ${config.versao} não suportada (esperado 1)`);
  if (!Array.isArray(config.regras)) throw new ErroDeConfiguracao(`"regras" precisa ser uma lista`);
  for (const regra of config.regras) {
    if (!/^R\d+$/.test(regra.id || "")) throw new ErroDeConfiguracao(`regra sem id no formato R<n>: ${JSON.stringify(regra).slice(0, 80)}`);
    if (!PRINCIPIOS.includes(regra.principio)) throw new ErroDeConfiguracao(`${regra.id}: princípio "${regra.principio}" não existe`);
    if (!CHECAGENS[regra.verificacao?.tipo]) throw new ErroDeConfiguracao(`${regra.id}: tipo de verificação "${regra.verificacao?.tipo}" não existe no catálogo (${Object.keys(CHECAGENS).join(", ")})`);
  }
  const contexto = { raiz, projeto: config.projeto || {}, base, avisos: [], arquivos: soCobertura ? [] : listarArquivosDoRepositorio(raiz) };
  const resultados = [{ id: "cobertura", principio: "todos", tipo: "cobertura", violacoes: checarCobertura({ textoMd, regras: config.regras }) }];
  if (!soCobertura) {
    for (const regra of config.regras) {
      const { tipo, ...parametros } = regra.verificacao;
      resultados.push({ id: regra.id, principio: regra.principio, tipo, violacoes: CHECAGENS[tipo](parametros, contexto) });
    }
  }
  const aprovado = resultados.every((resultado) => resultado.violacoes.length === 0);
  return { aprovado, resultados, avisos: contexto.avisos };
}

function formatar({ aprovado, resultados, avisos }) {
  const linhas = [];
  for (const { id, principio, tipo, violacoes } of resultados) {
    const rotulo = `${id} (${principio}) ${tipo}`;
    if (tipo === "revisor") { linhas.push(`👁  ${rotulo}: fica com o revisor`); continue; }
    linhas.push(`${violacoes.length ? "❌" : "✅"} ${rotulo}${violacoes.length ? `: ${violacoes.length} violação(ões)` : ""}`);
    for (const { arquivo, linha, mensagem } of violacoes) {
      const local = arquivo ? `${arquivo}${linha ? `:${linha}` : ""}  ` : "";
      linhas.push(`   ${local}${mensagem.replace(/\n/g, "\n      ")}`);
    }
  }
  for (const aviso of avisos) linhas.push(`⚠️  ${aviso}`);
  linhas.push(`VALIDADOR: ${aprovado ? "APROVADO" : "REPROVADO"}`);
  return linhas.join("\n");
}

function lerArgumentos(argv) {
  const opcoes = { config: "docs/regras-de-codigo.json", md: "docs/regras-de-codigo.md", base: undefined, soCobertura: false, json: false };
  for (let indice = 0; indice < argv.length; indice += 1) {
    const argumento = argv[indice];
    if (argumento === "--config") opcoes.config = argv[++indice];
    else if (argumento === "--md") opcoes.md = argv[++indice];
    else if (argumento === "--base") opcoes.base = argv[++indice];
    else if (argumento === "--so-cobertura") opcoes.soCobertura = true;
    else if (argumento === "--json") opcoes.json = true;
    else throw new ErroDeConfiguracao(`argumento desconhecido: ${argumento}`);
  }
  return opcoes;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    const opcoes = lerArgumentos(process.argv.slice(2));
    const raiz = process.cwd();
    for (const caminhoRelativo of [opcoes.config, opcoes.md]) {
      if (!existsSync(join(raiz, caminhoRelativo))) throw new ErroDeConfiguracao(`${caminhoRelativo} não existe (o mapa de Regras de código gera os dois arquivos)`);
    }
    let config;
    try { config = JSON.parse(lerTexto(raiz, opcoes.config)); } catch (erro) { throw new ErroDeConfiguracao(`${opcoes.config} não é JSON válido: ${erro.message}`); }
    const relatorio = validar({ raiz, config, textoMd: lerTexto(raiz, opcoes.md), base: opcoes.base, soCobertura: opcoes.soCobertura });
    console.log(opcoes.json ? JSON.stringify(relatorio, null, 2) : formatar(relatorio));
    process.exit(relatorio.aprovado ? 0 : 1);
  } catch (erro) {
    if (!(erro instanceof ErroDeConfiguracao)) throw erro;
    console.error(`Configuração inválida: ${erro.message}`);
    console.log("VALIDADOR: REPROVADO");
    process.exit(2);
  }
}
