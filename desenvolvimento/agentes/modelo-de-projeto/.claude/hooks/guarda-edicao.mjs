#!/usr/bin/env node
// Watchdog PreToolUse (Write | Edit | NotebookEdit).
// W1 protege os próprios watchdogs · W2 smart zone · W3 código só com issue ativa, fora da main.

import {
  lerEntradaDoHook, diretorioDoProjeto, bloquearFerramenta, watchdogsDesligados,
  lerEstadoSessao, chaveDoAgente, caminhoRelativoAoProjeto, caminhoEhProtegido, caminhoEhLivreSemIssue,
  branchAtual, branchEhPrincipal, branchEhDescartavel, repositorioTemCommits,
  medirTokensDeContexto, LIMITE_DURO_CONTEXTO, etapaDeAdocao,
} from "./lib/comum.mjs";

const NOME_WATCHDOG = "guarda-edicao";

const entradaHook = lerEntradaDoHook();
if (watchdogsDesligados()) process.exit(0);

const diretorioProjeto = diretorioDoProjeto(entradaHook);
const parametrosFerramenta = entradaHook.tool_input || {};
const caminhoArquivo = parametrosFerramenta.file_path || parametrosFerramenta.notebook_path;
if (!caminhoArquivo) process.exit(0);

const caminhoRelativo = caminhoRelativoAoProjeto(diretorioProjeto, caminhoArquivo);
if (caminhoRelativo === null) process.exit(0); // fora do repo (ex.: documento de handoff no tmp)

// W1 — os watchdogs não podem ser desligados pelo agente que eles vigiam.
if (caminhoEhProtegido(caminhoRelativo)) {
  bloquearFerramenta(NOME_WATCHDOG, `"${caminhoRelativo}" é infraestrutura dos watchdogs. Só um humano altera esse caminho. Peça ao usuário.`);
}

// W2 — passou da smart zone: nada de seguir editando com o raciocínio degradado.
const tokensDeContexto = medirTokensDeContexto(entradaHook.transcript_path);
if (tokensDeContexto >= LIMITE_DURO_CONTEXTO) {
  bloquearFerramenta(NOME_WATCHDOG, `contexto em ~${Math.round(tokensDeContexto / 1000)}k tokens, acima da smart zone (${LIMITE_DURO_CONTEXTO / 1000}k). Rode /nova-sessao (se precisar de documento, ele vai para o tmp do SO, fora do repositório) e continue numa janela nova com o kickoff.`);
}

// Na adoção do método, código sem issue ativa só passa a ser barrado na etapa "completo".
if (etapaDeAdocao(diretorioProjeto) !== "completo") process.exit(0);

if (caminhoEhLivreSemIssue(caminhoRelativo)) process.exit(0);

const nomeBranch = branchAtual(diretorioProjeto);
if (branchEhDescartavel(nomeBranch)) process.exit(0); // prototype/* e research/* são descartáveis

// W3 — código de produção nasce de uma issue (to-tickets / triage) numa sessão limpa.
const estadoSessao = lerEstadoSessao(diretorioProjeto, entradaHook.session_id);
const issueAtiva = estadoSessao.issuesAtivas[chaveDoAgente(entradaHook)];
if (!issueAtiva) {
  bloquearFerramenta(NOME_WATCHDOG, [
    `editar "${caminhoRelativo}" é código de produção e exige uma issue ativa nesta sessão.`,
    `Ative com: node .claude/hooks/ativar-issue.mjs <número>  (issue aberta com label ready-for-agent ou ready-for-human).`,
    `Ainda não há issue? O caminho é /wayfinder → /to-spec → /to-tickets (ou /triage para pedidos externos).`,
  ].join("\n"));
}

if (branchEhPrincipal(nomeBranch) && repositorioTemCommits(diretorioProjeto)) {
  const numeroIssue = issueAtiva.numero;
  bloquearFerramenta(NOME_WATCHDOG, `issue #${numeroIssue} ativa, mas a branch é "${nomeBranch}". Crie a branch da issue antes: git switch -c issue/${numeroIssue}-<slug>`);
}

process.exit(0);
