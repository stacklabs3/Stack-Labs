#!/usr/bin/env node
// Watchdog PostToolUse (Skill): quando uma skill de grilling carrega, marca o grilling na sessão e
// injeta a regra de linguagem simples logo depois das instruções da skill.

import {
  lerEntradaDoHook, diretorioDoProjeto, emitirContextoAdicional, watchdogsDesligados,
  lerEstadoSessao, gravarEstadoSessao, nomeDaSkillDeGrilling, REGRA_LINGUAGEM_GRILLING,
} from "./lib/comum.mjs";

const entradaHook = lerEntradaDoHook();
if (watchdogsDesligados()) process.exit(0);
if (!nomeDaSkillDeGrilling(entradaHook.tool_input?.skill)) process.exit(0);

const diretorioProjeto = diretorioDoProjeto(entradaHook);
const estadoSessao = lerEstadoSessao(diretorioProjeto, entradaHook.session_id);
estadoSessao.grillingAtivo = true;
gravarEstadoSessao(diretorioProjeto, entradaHook.session_id, estadoSessao);

emitirContextoAdicional("PostToolUse", REGRA_LINGUAGEM_GRILLING);
