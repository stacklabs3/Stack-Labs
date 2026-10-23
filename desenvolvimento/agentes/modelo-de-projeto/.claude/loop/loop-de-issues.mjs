#!/usr/bin/env node
// Loop de issues (padrão "Ralph" do Matt Pocock): implementa até N issues ready-for-agent em sequência.
// Cada issue roda numa sessão nova do Claude Code (`claude -p`), uma de cada vez, com os watchdogs do
// projeto ativos. Quando a sessão bate na smart zone, ela deixa um handoff e uma sessão nova continua
// a mesma issue. Cada issue termina numa PR; o merge é de um humano.
//
// Uso: node .claude/loop/loop-de-issues.mjs <N> [--orcamento <USD por sessão>] [--modelo <alias>] [--simular]
// Guia: docs/agents/loop-de-issues.md · Skill: /loop-de-issues

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const LABEL_DA_FILA = "ready-for-agent";
const LABEL_DE_DEVOLVIDA = "needs-triage";
const FALHAS_SEGUIDAS_PARA_PARAR = 2;
const SESSOES_POR_ISSUE = 4;
const ORCAMENTO_PADRAO_POR_SESSAO = "5";

// ---------- argumentos ----------

function lerArgumentos(argumentosDaLinha) {
  const opcoes = { quantidadeMaxima: Number(argumentosDaLinha[0]), orcamentoPorSessao: ORCAMENTO_PADRAO_POR_SESSAO, modelo: null, simular: false };
  if (!Number.isInteger(opcoes.quantidadeMaxima) || opcoes.quantidadeMaxima < 1) {
    throw new Error("uso: loop-de-issues.mjs <N> [--orcamento <USD>] [--modelo <alias>] [--simular]");
  }
  for (let indice = 1; indice < argumentosDaLinha.length; indice++) {
    const argumento = argumentosDaLinha[indice];
    if (argumento === "--orcamento") opcoes.orcamentoPorSessao = argumentosDaLinha[++indice];
    else if (argumento === "--modelo") opcoes.modelo = argumentosDaLinha[++indice];
    else if (argumento === "--simular") opcoes.simular = true;
    else throw new Error(`opção desconhecida: ${argumento}`);
  }
  return opcoes;
}

// ---------- git e gh ----------

function executar(programa, argumentos, opcoesExtras = {}) {
  return execFileSync(programa, argumentos, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"], ...opcoesExtras }).trim();
}

const git = (...argumentos) => executar("git", argumentos);
const gh = (...argumentos) => executar("gh", argumentos);
const ghJson = (...argumentos) => JSON.parse(gh(...argumentos));

function workingTreeLimpo() {
  return git("status", "--porcelain") === "";
}

// ---------- fronteira ----------

// Uma issue está bloqueada se tem dependência nativa aberta ou um "Blocked by: #n" com issue aberta.
function issueTemBloqueioAberto(repositorio, numeroIssue) {
  const issueCompleta = ghJson("api", `repos/${repositorio}/issues/${numeroIssue}`);
  if ((issueCompleta.issue_dependencies_summary?.blocked_by ?? 0) > 0) return true;
  const linhaDeBloqueio = (issueCompleta.body || "").match(/Blocked by:.*/i)?.[0] || "";
  const numerosBloqueadores = [...linhaDeBloqueio.matchAll(/#(\d+)/g)].map((correspondencia) => correspondencia[1]);
  return numerosBloqueadores.some((numeroBloqueador) => ghJson("issue", "view", numeroBloqueador, "--json", "state").state === "OPEN");
}

// Issues abertas da fila, sem responsável e sem bloqueio aberto, da mais antiga para a mais nova.
function proximaIssueDaFronteira(repositorio, numerosJaTentados) {
  const issuesDaFila = ghJson("issue", "list", "--label", LABEL_DA_FILA, "--state", "open", "--limit", "200", "--json", "number,title,assignees,labels");
  const candidatas = issuesDaFila
    .filter((issue) => issue.assignees.length === 0)
    .filter((issue) => !issue.labels.some((label) => label.name.startsWith("wayfinder:")))
    .filter((issue) => !numerosJaTentados.includes(issue.number))
    .sort((issueA, issueB) => issueA.number - issueB.number);
  return candidatas.find((issue) => !issueTemBloqueioAberto(repositorio, issue.number)) || null;
}

// ---------- sessão ----------

function montarPrompt(modeloDoPrompt, numeroIssue, handoffDeEntrada, handoffDeSaida) {
  const continuacao = handoffDeEntrada
    ? `\nEsta sessão continua o trabalho de uma sessão anterior que bateu na smart zone. Antes de tudo,\nleia o handoff em ${handoffDeEntrada} e siga dali. A branch da issue já existe, com o trabalho commitado.\n`
    : "";
  return modeloDoPrompt
    .replaceAll("{{ISSUE}}", String(numeroIssue))
    .replaceAll("{{CONTINUACAO}}", continuacao)
    .replaceAll("{{HANDOFF_SAIDA}}", handoffDeSaida);
}

// A última linha "RESULTADO: ..." da sessão: { tipo: "pr" | "handoff" | "bloqueado" | "sem-resultado", detalhe }.
export function lerResultadoDaSessao(textoDoRegistro) {
  let textoDoResultado = "";
  try {
    textoDoResultado = JSON.parse(textoDoRegistro).result || "";
  } catch { /* sessão sem JSON válido: sem resultado */ }
  const linhaDeResultado = textoDoResultado.split("\n").filter((linha) => linha.startsWith("RESULTADO: ")).pop();
  if (!linhaDeResultado) return { tipo: "sem-resultado", detalhe: "" };
  const conteudo = linhaDeResultado.slice("RESULTADO: ".length).trim();
  if (conteudo.startsWith("PR ")) return { tipo: "pr", detalhe: conteudo.slice(3).trim() };
  if (conteudo === "HANDOFF") return { tipo: "handoff", detalhe: "" };
  if (conteudo.startsWith("BLOQUEADO")) return { tipo: "bloqueado", detalhe: conteudo.slice("BLOQUEADO".length).trim() };
  return { tipo: "sem-resultado", detalhe: conteudo };
}

function rodarSessao({ prompt, opcoes, caminhoDoRegistro }) {
  const argumentosDoClaude = ["-p", prompt, "--permission-mode", "auto", "--permission-prompts", "none",
    "--max-budget-usd", opcoes.orcamentoPorSessao, "--output-format", "json"];
  if (opcoes.modelo) argumentosDoClaude.push("--model", opcoes.modelo);
  const execucaoDoClaude = spawnSync("claude", argumentosDoClaude, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  writeFileSync(caminhoDoRegistro, execucaoDoClaude.stdout || "");
  writeFileSync(caminhoDoRegistro.replace(/\.json$/, ".stderr"), execucaoDoClaude.stderr || "");
  return lerResultadoDaSessao(execucaoDoClaude.stdout || "");
}

function devolverIssue(numeroIssue, motivo) {
  gh("issue", "comment", String(numeroIssue), "--body", `Loop de issues: não terminou esta issue. Motivo: ${motivo}`);
  gh("issue", "edit", String(numeroIssue), "--remove-assignee", "@me", "--remove-label", LABEL_DA_FILA, "--add-label", LABEL_DE_DEVOLVIDA);
}

// ---------- rodada ----------

function rodarLoop(opcoes) {
  const raizDoProjeto = git("rev-parse", "--show-toplevel");
  process.chdir(raizDoProjeto);

  if (!workingTreeLimpo()) throw new Error("O working tree tem mudanças. O loop começa cada issue de uma main limpa: faça commit ou stash antes.");
  if (spawnSync("gh", ["auth", "status"]).status !== 0) throw new Error("gh não está autenticado.");
  if (spawnSync("claude", ["--version"]).status !== 0) throw new Error("claude não está no PATH.");

  const repositorio = gh("repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner");
  const modeloDoPrompt = readFileSync(join(raizDoProjeto, ".claude", "loop", "prompt-da-issue.md"), "utf8");
  const nomeDaRodada = new Date().toISOString().replace(/[:.]/g, "-");
  const pastaDeRegistros = join(raizDoProjeto, ".claude", "state", "loop", nomeDaRodada);
  // Os handoffs ficam fora do repositório: na smart zone, o watchdog barra edições dentro dele.
  const pastaDeHandoffs = join(tmpdir(), `loop-de-issues-${nomeDaRodada}`);
  mkdirSync(pastaDeRegistros, { recursive: true });
  mkdirSync(pastaDeHandoffs, { recursive: true });

  const numerosJaTentados = [];
  const resumo = [];
  let falhasSeguidas = 0;

  for (let volta = 1; volta <= opcoes.quantidadeMaxima; volta++) {
    const issueDaVez = proximaIssueDaFronteira(repositorio, numerosJaTentados);
    if (!issueDaVez) {
      resumo.push(`A fronteira esvaziou: nenhuma issue ${LABEL_DA_FILA} livre e desbloqueada.`);
      break;
    }
    numerosJaTentados.push(issueDaVez.number);
    const nomeDaIssue = `"${issueDaVez.title}" (#${issueDaVez.number})`;
    console.log(`[${volta}/${opcoes.quantidadeMaxima}] ${nomeDaIssue}`);

    if (opcoes.simular) {
      resumo.push(`SIMULADA  ${nomeDaIssue}`);
      continue;
    }

    // Reivindica antes de começar: outra pessoa ou outro loop pula esta issue.
    gh("issue", "edit", String(issueDaVez.number), "--add-assignee", "@me");
    git("fetch", "-q", "origin", "main");
    git("switch", "-q", "main");
    git("merge", "-q", "--ff-only", "origin/main");

    let handoffDeEntrada = null;
    let resultadoDaIssue = { tipo: "sem-resultado", detalhe: "" };
    let sessao = 1;
    let caminhoDoRegistro = "";
    for (; sessao <= SESSOES_POR_ISSUE; sessao++) {
      const handoffDeSaida = join(pastaDeHandoffs, `issue-${issueDaVez.number}-sessao-${sessao}.md`);
      caminhoDoRegistro = join(pastaDeRegistros, `issue-${issueDaVez.number}-sessao-${sessao}.json`);
      console.log(`  sessão ${sessao}`);
      resultadoDaIssue = rodarSessao({
        prompt: montarPrompt(modeloDoPrompt, issueDaVez.number, handoffDeEntrada, handoffDeSaida),
        opcoes,
        caminhoDoRegistro,
      });
      const handoffEscrito = existsSync(handoffDeSaida) && statSync(handoffDeSaida).size > 0;
      if (resultadoDaIssue.tipo === "handoff" && handoffEscrito) {
        handoffDeEntrada = handoffDeSaida;
        continue;
      }
      break;
    }

    if (resultadoDaIssue.tipo === "pr") {
      resumo.push(`PR        ${nomeDaIssue} em ${sessao} sessão(ões): ${resultadoDaIssue.detalhe}`);
      falhasSeguidas = 0;
    } else {
      const motivoDaFalha = {
        bloqueado: resultadoDaIssue.detalhe,
        handoff: `passou de ${SESSOES_POR_ISSUE} sessões sem terminar; a issue provavelmente precisa ser fatiada`,
        "sem-resultado": `a sessão terminou sem a linha RESULTADO (registro: ${caminhoDoRegistro})`,
      }[resultadoDaIssue.tipo];
      devolverIssue(issueDaVez.number, motivoDaFalha);
      resumo.push(`DEVOLVIDA ${nomeDaIssue}: ${motivoDaFalha}`);
      falhasSeguidas++;
    }

    // Trabalho não commitado nunca se descarta: o loop para e deixa para um humano.
    if (!workingTreeLimpo()) {
      resumo.push(`Parei: a sessão deixou mudanças sem commit em ${git("branch", "--show-current")}. Confira antes de rodar de novo.`);
      break;
    }
    git("switch", "-q", "main");
    if (falhasSeguidas >= FALHAS_SEGUIDAS_PARA_PARAR) {
      resumo.push(`Parei: ${FALHAS_SEGUIDAS_PARA_PARAR} issues seguidas sem PR. Algo comum a elas precisa de um humano.`);
      break;
    }
  }

  writeFileSync(join(pastaDeRegistros, "resumo.txt"), resumo.join("\n") + "\n");
  console.log(resumo.join("\n"));
  console.log(`Registros: ${pastaDeRegistros}`);
  console.log(`Handoffs: ${pastaDeHandoffs}`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    rodarLoop(lerArgumentos(process.argv.slice(2)));
  } catch (erro) {
    console.error(erro.message);
    process.exit(1);
  }
}
