#!/usr/bin/env node
// Watchdog UserPromptSubmit: mede a janela de contexto e cobra o /nova-sessao na smart zone; em
// sessão de grilling, relembra a linguagem simples a cada resposta do humano.

import {
  lerEntradaDoHook, emitirContextoAdicional, watchdogsDesligados, diretorioDoProjeto,
  medirTokensDeContexto, LIMITE_AVISO_CONTEXTO, LIMITE_DURO_CONTEXTO,
  lerEstadoSessao, gravarEstadoSessao, nomeDaSkillDeGrilling, REGRA_LINGUAGEM_GRILLING,
} from "./lib/comum.mjs";

const entradaHook = lerEntradaDoHook();
if (watchdogsDesligados()) process.exit(0);

const avisosDoPrompt = [];

const tokensDeContexto = medirTokensDeContexto(entradaHook.transcript_path);
const milharesDeTokens = Math.round(tokensDeContexto / 1000);

if (tokensDeContexto >= LIMITE_DURO_CONTEXTO) {
  avisosDoPrompt.push(`🛑 [watchdog vigia-prompt] Contexto em ~${milharesDeTokens}k tokens: passou da smart zone (${LIMITE_DURO_CONTEXTO / 1000}k). REGRA DURA: a próxima ação é /nova-sessao, e o trabalho continua numa janela nova com o kickoff. Edições no repositório estão bloqueadas. Avise o usuário.`);
} else if (tokensDeContexto >= LIMITE_AVISO_CONTEXTO) {
  avisosDoPrompt.push(`⚠️ [watchdog vigia-prompt] Contexto em ~${milharesDeTokens}k tokens, perto da smart zone (${LIMITE_DURO_CONTEXTO / 1000}k). Feche a etapa atual e planeje o /nova-sessao. Não comece uma fase nova nesta sessão.`);
}

// Grilling digitado como /grill-me, /wayfinder etc. não passa pelo Skill tool: marca aqui.
const diretorioProjeto = diretorioDoProjeto(entradaHook);
const estadoSessao = lerEstadoSessao(diretorioProjeto, entradaHook.session_id);
if (!estadoSessao.grillingAtivo && nomeDaSkillDeGrilling((entradaHook.prompt || "").match(/^\s*(\/\S+)/)?.[1])) {
  estadoSessao.grillingAtivo = true;
  gravarEstadoSessao(diretorioProjeto, entradaHook.session_id, estadoSessao);
}
if (estadoSessao.grillingAtivo) avisosDoPrompt.push(REGRA_LINGUAGEM_GRILLING);

if (avisosDoPrompt.length) emitirContextoAdicional("UserPromptSubmit", avisosDoPrompt.join("\n\n"));
