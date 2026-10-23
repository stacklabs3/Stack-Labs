#!/usr/bin/env node
// Watchdog Stop: o turno só termina quando cada ticket do wayfinder resolvido nesta sessão
// tem seu ponteiro em "Decisions so far" no mapa.

import {
  lerEntradaDoHook, diretorioDoProjeto, watchdogsDesligados,
  lerEstadoSessao, gravarEstadoSessao, buscarIssue, numeroDoMapa,
} from "./lib/comum.mjs";

const entradaHook = lerEntradaDoHook();
if (watchdogsDesligados() || entradaHook.stop_hook_active) process.exit(0);

const diretorioProjeto = diretorioDoProjeto(entradaHook);
const estadoSessao = lerEstadoSessao(diretorioProjeto, entradaHook.session_id);
const ticketsPendentes = estadoSessao.ticketsWayfinderResolvidos.filter((ticketResolvido) => !ticketResolvido.ponteiroNoMapaVerificado);
if (ticketsPendentes.length === 0) process.exit(0);

const pendenciasEncontradas = [];
for (const ticketPendente of ticketsPendentes) {
  try {
    const ticketCompleto = buscarIssue(diretorioProjeto, ticketPendente.numero);
    if (ticketCompleto.estado !== "closed") continue; // fechamento falhou ou ainda não aconteceu
    const numeroMapa = numeroDoMapa(diretorioProjeto, ticketCompleto);
    if (!numeroMapa) {
      pendenciasEncontradas.push(`"${ticketPendente.titulo}" (#${ticketPendente.numero}) não está ligado a nenhum mapa (sub-issue ou "Part of #N").`);
      continue;
    }
    const mapaDoTicket = buscarIssue(diretorioProjeto, numeroMapa);
    const secaoDecisoes = (mapaDoTicket.corpo.split(/##\s*Decisions so far/i)[1] || "").split(/\n##\s/)[0];
    const ponteiroPresente = new RegExp(`(issues/${ticketPendente.numero}\\b|#${ticketPendente.numero}\\b)`).test(secaoDecisoes);
    if (ponteiroPresente) {
      ticketPendente.ponteiroNoMapaVerificado = true;
    } else {
      pendenciasEncontradas.push(`o mapa "${mapaDoTicket.titulo}" (#${numeroMapa}) não tem o ponteiro de "${ticketPendente.titulo}" (#${ticketPendente.numero}) em "Decisions so far". Acrescente: - [${ticketPendente.titulo}](link) — <gist de uma linha>`);
    }
  } catch {
    pendenciasEncontradas.push(`não consegui verificar "${ticketPendente.titulo}" (#${ticketPendente.numero}) no GitHub. Confira o mapa manualmente.`);
  }
}
gravarEstadoSessao(diretorioProjeto, entradaHook.session_id, estadoSessao);

if (pendenciasEncontradas.length) {
  process.stdout.write(JSON.stringify({
    decision: "block",
    reason: `[watchdog vigia-parada] Resolução do wayfinder incompleta:\n- ${pendenciasEncontradas.join("\n- ")}\nDepois, gradue a névoa que a resposta esclareceu e atualize "Not yet specified".`,
  }));
}
