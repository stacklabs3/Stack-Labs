#!/usr/bin/env node
// Watchdog UserPromptSubmit: mede a janela de contexto e cobra o /nova-sessao na smart zone.

import {
  lerEntradaDoHook, emitirContextoAdicional, watchdogsDesligados,
  medirTokensDeContexto, LIMITE_AVISO_CONTEXTO, LIMITE_DURO_CONTEXTO,
} from "./lib/comum.mjs";

const entradaHook = lerEntradaDoHook();
if (watchdogsDesligados()) process.exit(0);

const tokensDeContexto = medirTokensDeContexto(entradaHook.transcript_path);
const milharesDeTokens = Math.round(tokensDeContexto / 1000);

if (tokensDeContexto >= LIMITE_DURO_CONTEXTO) {
  emitirContextoAdicional("UserPromptSubmit", `🛑 [watchdog vigia-prompt] Contexto em ~${milharesDeTokens}k tokens: passou da smart zone (${LIMITE_DURO_CONTEXTO / 1000}k). REGRA DURA: a próxima ação é /nova-sessao, e o trabalho continua numa janela nova com o kickoff. Edições no repositório estão bloqueadas. Avise o usuário.`);
} else if (tokensDeContexto >= LIMITE_AVISO_CONTEXTO) {
  emitirContextoAdicional("UserPromptSubmit", `⚠️ [watchdog vigia-prompt] Contexto em ~${milharesDeTokens}k tokens, perto da smart zone (${LIMITE_DURO_CONTEXTO / 1000}k). Feche a etapa atual e planeje o /nova-sessao. Não comece uma fase nova nesta sessão.`);
}
