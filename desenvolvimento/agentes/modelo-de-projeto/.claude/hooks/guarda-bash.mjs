#!/usr/bin/env node
// Watchdog PreToolUse (Bash).
// W1 protege os watchdogs · W4 ativação de issue (com os mapas iniciais) · W5/W6 regras do /wayfinder
// · W7 nada de commit na main · W9 resolução cita as fontes (pesquisa antes de recomendar).

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  lerEntradaDoHook, diretorioDoProjeto, bloquearFerramenta, watchdogsDesligados,
  lerEstadoSessao, gravarEstadoSessao, buscarIssue, tipoWayfinder,
  branchAtual, branchEhPrincipal, repositorioTemCommits, LABELS_IMPLEMENTAVEIS,
  buscarMapasIniciais, buscarCorposDosComentarios, textoTemFontes,
  MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR, TIPOS_QUE_EXIGEM_FONTES,
} from "./lib/comum.mjs";

const NOME_WATCHDOG = "guarda-bash";

const entradaHook = lerEntradaDoHook();
if (watchdogsDesligados()) process.exit(0);

const diretorioProjeto = diretorioDoProjeto(entradaHook);
const idSessao = entradaHook.session_id;
const comando = entradaHook.tool_input?.command || "";
const trechosDoComando = comando.split(/&&|\|\||;|\n/);

function buscarIssueOuBloquear(numeroIssue) {
  try {
    return buscarIssue(diretorioProjeto, numeroIssue);
  } catch (erroGh) {
    bloquearFerramenta(NOME_WATCHDOG, `não consegui consultar a issue #${numeroIssue} no GitHub (${String(erroGh.message).split("\n")[0]}). Os watchdogs falham fechados: verifique o gh auth/rede.`);
  }
}

function numerosAposSubcomando(padraoSubcomando) {
  const numerosEncontrados = [];
  for (const correspondencia of comando.matchAll(padraoSubcomando)) {
    const numeroNoFinal = correspondencia[1].match(/(\d+)["']?$/);
    if (numeroNoFinal) numerosEncontrados.push(Number(numeroNoFinal[1]));
  }
  return numerosEncontrados;
}

function comandoReivindicaNoMesmoPasso(numeroTicket) {
  const padraoReivindicacao = new RegExp(`gh\\s+issue\\s+edit\\s+#?${numeroTicket}\\b[^;&|\\n]*--add-assignee`);
  return padraoReivindicacao.test(comando);
}

// ---------- W1: os watchdogs não se desligam por Bash ----------
const comandoSemRedirecionamentoInofensivo = comando.replace(/\d*>&\d+/g, "").replace(/\d*>\s*\/dev\/null/g, "");
const padraoCaminhoProtegido = /\.claude\/(hooks|githooks|loop|settings\.json|state)|\.github\/workflows\/vigia-do-metodo\.yml/;
const verboMutante = /(>|\brm\b|\bmv\b|\bcp\b|sed\s+-i|perl\s+-[a-z]*i|\btee\b|\bchmod\b|\btruncate\b|\bln\b|\bunlink\b|git\s+(checkout|restore|rm|stash)\b)/;
const trechoMutaCaminhoProtegido = comandoSemRedirecionamentoInofensivo
  .split(/&&|\|\||;|\n/)
  .some((trechoComando) => padraoCaminhoProtegido.test(trechoComando) && verboMutante.test(trechoComando));
if (trechoMutaCaminhoProtegido) {
  bloquearFerramenta(NOME_WATCHDOG, "o comando altera .claude/hooks, .claude/githooks, .claude/loop, .claude/settings.json, .claude/state ou o vigia-do-metodo do CI. Só um humano altera a infraestrutura dos watchdogs.");
}

// ---------- W8: o pre-push que protege a main não se desliga ----------
if (/\bgit\b[^;&|\n]*--no-verify\b/.test(comando)) {
  bloquearFerramenta(NOME_WATCHDOG, "--no-verify pula os hooks do git (inclusive o pre-push que protege a main). Proibido para o agente.");
}
if (/core\.hooksPath/i.test(comando) && !/git\s+config\s+(--get\s+)?core\.hooksPath\s*($|[;&|\n])/i.test(comando)) {
  bloquearFerramenta(NOME_WATCHDOG, "alterar core.hooksPath desliga o pre-push que protege a main. Só um humano altera.");
}
if (/--atualizar-watchdogs\b/.test(comando)) {
  bloquearFerramenta(NOME_WATCHDOG, "--atualizar-watchdogs sobrescreve os watchdogs do projeto. Só um humano roda o setup-stacklabs com essa opção.");
}
if (/STACKLABS_PERMITIR_PUSH_MAIN/.test(comando)) {
  bloquearFerramenta(NOME_WATCHDOG, "STACKLABS_PERMITIR_PUSH_MAIN é o escape humano do pre-push. O agente não usa.");
}

// ---------- W4: ativação de issue (uma issue por sessão) ----------
const numerosAtivados = [...new Set([...comando.matchAll(/ativar-issue(?:\.mjs)?\s+#?(\d+)/g)].map((correspondencia) => Number(correspondencia[1])))];
if (numerosAtivados.length > 1) {
  bloquearFerramenta(NOME_WATCHDOG, `o comando ativa mais de uma issue (${numerosAtivados.map((numero) => `#${numero}`).join(", ")}). Uma sessão trabalha uma issue só.`);
}
if (numerosAtivados.length === 1) {
  const numeroIssue = numerosAtivados[0];
  const estadoSessao = lerEstadoSessao(diretorioProjeto, idSessao);
  if (estadoSessao.issueAtiva && estadoSessao.issueAtiva.numero !== numeroIssue) {
    bloquearFerramenta(NOME_WATCHDOG, `esta sessão já trabalha na issue #${estadoSessao.issueAtiva.numero}. Contexto limpo por issue: termine, rode /nova-sessao e cole o kickoff numa janela nova para a #${numeroIssue}.`);
  }
  const issueCandidata = buscarIssueOuBloquear(numeroIssue);
  if (issueCandidata.estado !== "open") {
    bloquearFerramenta(NOME_WATCHDOG, `a issue #${numeroIssue} está fechada.`);
  }
  if (tipoWayfinder(issueCandidata.labels)) {
    bloquearFerramenta(NOME_WATCHDOG, `#${numeroIssue} é um ticket do wayfinder: decide, não implementa. Feche o mapa e passe por /to-spec → /to-tickets.`);
  }
  if (!issueCandidata.labels.some((label) => LABELS_IMPLEMENTAVEIS.includes(label))) {
    bloquearFerramenta(NOME_WATCHDOG, `a issue #${numeroIssue} não tem ${LABELS_IMPLEMENTAVEIS.join(" nem ")}. Ela precisa sair do /to-tickets ou passar pelo /triage antes.`);
  }
  // W4b — mapas iniciais: os quatro existem, e os que decidem stack e fases estão fechados.
  let mapasIniciais;
  try {
    mapasIniciais = buscarMapasIniciais(diretorioProjeto);
  } catch (erroGh) {
    bloquearFerramenta(NOME_WATCHDOG, `não consegui consultar os mapas iniciais no GitHub (${String(erroGh.message).split("\n")[0]}). Os watchdogs falham fechados.`);
  }
  const mapasFaltando = mapasIniciais.filter((mapaInicial) => !mapaInicial.numero);
  if (mapasFaltando.length) {
    bloquearFerramenta(NOME_WATCHDOG, `faltam mapas iniciais: ${mapasFaltando.map((mapaInicial) => mapaInicial.tipo).join(", ")}. Todo projeto abre os quatro antes de implementar (docs/agents/mapas-iniciais.md). Abra com /wayfinder e as labels wayfinder:map + mapa-inicial:<tipo>.`);
  }
  const mapasAindaAbertos = mapasIniciais.filter((mapaInicial) => MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR.includes(mapaInicial.tipo) && mapaInicial.estado !== "closed");
  if (mapasAindaAbertos.length) {
    bloquearFerramenta(NOME_WATCHDOG, `os mapas iniciais ${mapasAindaAbertos.map((mapaInicial) => `"${mapaInicial.titulo}"`).join(" e ")} ainda estão abertos. Implementação só começa com o desenho inicial do protótipo aprovado e a arquitetura e as fases decididas (o mapa de Infra não trava).`);
  }

  estadoSessao.issueAtiva = { numero: issueCandidata.numero, titulo: issueCandidata.titulo, ativadaEm: new Date().toISOString() };
  gravarEstadoSessao(diretorioProjeto, idSessao, estadoSessao);
}

// ---------- W6: reivindicar o ticket antes de trabalhar nele ----------
for (const numeroTicket of numerosAposSubcomando(/gh\s+issue\s+comment\s+([^\s;&|]+)/g)) {
  const ticketComentado = buscarIssueOuBloquear(numeroTicket);
  const tipoTicket = tipoWayfinder(ticketComentado.labels);
  if (!tipoTicket || tipoTicket === "map") continue;
  if (ticketComentado.responsaveis.length === 0 && !comandoReivindicaNoMesmoPasso(numeroTicket)) {
    bloquearFerramenta(NOME_WATCHDOG, `o ticket "${ticketComentado.titulo}" (#${numeroTicket}) não foi reivindicado. Primeira escrita da sessão: gh issue edit ${numeroTicket} --add-assignee @me`);
  }
}

// ---------- W5: fechar ticket do wayfinder ----------
const numerosFechados = [
  ...numerosAposSubcomando(/gh\s+issue\s+close\s+([^\s;&|]+)/g),
  ...[...comando.matchAll(/issues\/(\d+)\b[^;&|\n]*state[=:]\s*["']?closed/g)].map((correspondencia) => Number(correspondencia[1])),
];

for (const numeroTicket of numerosFechados) {
  const ticketFechado = buscarIssueOuBloquear(numeroTicket);
  const tipoTicket = tipoWayfinder(ticketFechado.labels);
  if (!tipoTicket || tipoTicket === "map") continue;

  const trechoDoFechamento = trechosDoComando.find((trecho) => new RegExp(`gh\\s+issue\\s+close\\s+\\S*${numeroTicket}\\b`).test(trecho)) || "";
  const foraDeEscopo = /(--reason|-r)\s+["']?not[ _]planned/i.test(trechoDoFechamento);

  if (ticketFechado.responsaveis.length === 0 && !comandoReivindicaNoMesmoPasso(numeroTicket)) {
    bloquearFerramenta(NOME_WATCHDOG, `o ticket "${ticketFechado.titulo}" (#${numeroTicket}) nunca foi reivindicado (sem assignee). Reivindique antes de resolver.`);
  }
  if (foraDeEscopo) continue; // fora de escopo: não é passo da rota, não conta no limite

  const temComentarioDeResolucao = ticketFechado.quantidadeComentarios > 0
    || /(--comment|-c)\s/.test(trechoDoFechamento)
    || new RegExp(`gh\\s+issue\\s+comment\\s+#?${numeroTicket}\\b`).test(comando);
  if (!temComentarioDeResolucao) {
    bloquearFerramenta(NOME_WATCHDOG, `o ticket "${ticketFechado.titulo}" (#${numeroTicket}) seria fechado sem comentário de resolução. Poste a resposta antes (gh issue comment ou --comment).`);
  }

  // W9 — pesquisa antes de recomendar: a resolução termina com "## Fontes" e ao menos um link.
  if (TIPOS_QUE_EXIGEM_FONTES.includes(tipoTicket)) {
    let corposDosComentarios = [];
    try {
      corposDosComentarios = buscarCorposDosComentarios(diretorioProjeto, numeroTicket);
    } catch (erroGh) {
      bloquearFerramenta(NOME_WATCHDOG, `não consegui ler os comentários de "${ticketFechado.titulo}" (#${numeroTicket}) no GitHub (${String(erroGh.message).split("\n")[0]}). Os watchdogs falham fechados.`);
    }
    const textosDosArquivosDeCorpo = [...comando.matchAll(/--body-file\s+("([^"]+)"|'([^']+)'|(\S+))/g)]
      .map((correspondencia) => correspondencia[2] || correspondencia[3] || correspondencia[4])
      .filter((caminhoCorpo) => caminhoCorpo !== "-" && existsSync(resolve(entradaHook.cwd || diretorioProjeto, caminhoCorpo)))
      .map((caminhoCorpo) => readFileSync(resolve(entradaHook.cwd || diretorioProjeto, caminhoCorpo), "utf8"));
    const resolucaoTemFontes = textoTemFontes(comando)
      || textosDosArquivosDeCorpo.some((textoCorpo) => textoTemFontes(textoCorpo)) || corposDosComentarios.some((corpoComentario) => textoTemFontes(corpoComentario));
    if (!resolucaoTemFontes) {
      bloquearFerramenta(NOME_WATCHDOG, `o ticket "${ticketFechado.titulo}" (#${numeroTicket}) seria fechado sem "## Fontes" no comentário de resolução. Nenhuma recomendação sem pesquisa: acrescente a seção com ao menos um link para docs/research/ ou para a fonte primária (docs/agents/pesquisa-antes-de-recomendar.md).`);
    }
  }

  const estadoSessao = lerEstadoSessao(diretorioProjeto, idSessao);
  const jaRegistrado = estadoSessao.ticketsWayfinderResolvidos.some((ticketResolvido) => ticketResolvido.numero === numeroTicket);
  const decisoesJaResolvidas = estadoSessao.ticketsWayfinderResolvidos.filter((ticketResolvido) =>
  ticketResolvido.tipo !== "research" && !(tipoTicket === "prototype" && ticketResolvido.tipo === "prototype"));
  if (!jaRegistrado && tipoTicket !== "research" && decisoesJaResolvidas.length >= 1) {
    const ticketAnterior = decisoesJaResolvidas[0];
    bloquearFerramenta(NOME_WATCHDOG, `esta sessão já resolveu "${ticketAnterior.titulo}" (#${ticketAnterior.numero}). O /wayfinder resolve UM ticket por sessão (pesquisa é a exceção). Abra uma sessão nova para "${ticketFechado.titulo}".`);
  }
  if (!jaRegistrado) {
    estadoSessao.ticketsWayfinderResolvidos.push({ numero: numeroTicket, titulo: ticketFechado.titulo, tipo: tipoTicket, ponteiroNoMapaVerificado: false });
    gravarEstadoSessao(diretorioProjeto, idSessao, estadoSessao);
  }
}

// ---------- W7: main só recebe merge ----------
// A branch é a da pasta onde o commit roda (worktree de agente, `git -C`, `cd`), não a do checkout principal.
function diretorioDoCommit() {
  const caminhoGitC = comando.match(/\bgit\s+-C\s+("([^"]+)"|'([^']+)'|(\S+))\s+commit\b/);
  if (caminhoGitC) return caminhoGitC[2] || caminhoGitC[3] || caminhoGitC[4];
  const caminhoCd = comando.match(/^\s*cd\s+("([^"]+)"|'([^']+)'|(\S+))\s*(?:&&|;)/);
  if (caminhoCd) return caminhoCd[2] || caminhoCd[3] || caminhoCd[4];
  return entradaHook.cwd || diretorioProjeto;
}
if (/\bgit\s+(?:-C\s+\S+\s+)?commit\b/.test(comando)) {
  const pastaDoCommit = diretorioDoCommit();
  const nomeBranch = branchAtual(pastaDoCommit);
  if (branchEhPrincipal(nomeBranch) && repositorioTemCommits(pastaDoCommit)) {
    bloquearFerramenta(NOME_WATCHDOG, `commit direto na "${nomeBranch}" é proibido. Use a branch da issue (issue/<n>-<slug>), prototype/<nome> ou research/<nome>.`);
  }
}
if (/\bgit\s+push\b[^;&|\n]*(--force\b|--force-with-lease\b|\s-f\b)/.test(comando)) {
  bloquearFerramenta(NOME_WATCHDOG, "push forçado é proibido para o agente. Peça ao usuário.");
}

process.exit(0);
