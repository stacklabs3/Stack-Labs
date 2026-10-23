#!/usr/bin/env node
// Watchdog SessionStart: verifica o setup e reancora a constituição no início de toda sessão
// (inclusive depois de /clear e /compact, quando as regras somem da memória de trabalho).

import { existsSync } from "node:fs";
import { join } from "node:path";
import {
  lerEntradaDoHook, diretorioDoProjeto, emitirContextoAdicional, watchdogsDesligados,
  lerEstadoSessao, branchAtual, executarGh, caminhoHooksDoGit, CAMINHO_GITHOOKS,
  buscarMapasIniciais, MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR,
} from "./lib/comum.mjs";

const entradaHook = lerEntradaDoHook();
const diretorioProjeto = diretorioDoProjeto(entradaHook);

if (watchdogsDesligados()) {
  emitirContextoAdicional("SessionStart", "⚠️ Watchdogs DESLIGADOS (STACKLABS_WATCHDOGS=desligado). As regras de AGENTS.md continuam valendo; só não há bloqueio automático.");
  process.exit(0);
}

const arquivosDoSetup = [
  "AGENTS.md", "docs/agents/issue-tracker.md", "docs/agents/triage-labels.md", "docs/agents/domain.md",
  "docs/agents/mapas-iniciais.md", "docs/agents/pesquisa-antes-de-recomendar.md", "docs/agents/roteiro-de-prototipo.md",
];
const arquivosFaltando = arquivosDoSetup.filter((arquivoSetup) => !existsSync(join(diretorioProjeto, arquivoSetup)));

let mapasAbertos = "não consultado";
try {
  mapasAbertos = executarGh(diretorioProjeto, ["issue", "list", "--label", "wayfinder:map", "--state", "open", "--json", "number,title", "--jq", '.[] | "- \\(.title) (#\\(.number))"']) || "nenhum";
} catch {
  mapasAbertos = "falha ao consultar o GitHub (gh auth/rede)";
}

let situacaoDosMapasIniciais;
try {
  situacaoDosMapasIniciais = buscarMapasIniciais(diretorioProjeto).map((mapaInicial) => {
    if (!mapaInicial.numero) return `- ❌ ${mapaInicial.tipo}: NÃO EXISTE. Abra com /wayfinder (docs/agents/mapas-iniciais.md).`;
    const precisaFechar = MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR.includes(mapaInicial.tipo) && mapaInicial.estado !== "closed";
    return `- ${precisaFechar ? "⏳" : "✅"} ${mapaInicial.tipo}: "${mapaInicial.titulo}" (#${mapaInicial.numero}, ${mapaInicial.estado === "closed" ? "fechado" : "aberto"})`;
  }).join("\n");
} catch {
  situacaoDosMapasIniciais = "falha ao consultar o GitHub (gh auth/rede)";
}

const estadoSessao = lerEstadoSessao(diretorioProjeto, entradaHook.session_id);
const linhasContexto = [
  "## Watchdogs do método StackLabs (wayfinder / Matt Pocock) ativos",
  "AGENTS.md é a constituição deste repositório: regras DURAS, não sugestões. Releia antes de agir.",
  arquivosFaltando.length ? `❌ Setup incompleto, faltam: ${arquivosFaltando.join(", ")}. Copie o modelo-de-projeto do repositório Stack-Labs (desenvolvimento/agentes/) antes de qualquer fluxo.` : "✅ Setup do issue tracker (GitHub) presente.",
  caminhoHooksDoGit(diretorioProjeto) === CAMINHO_GITHOOKS ? "✅ pre-push da main ativo." : `❌ pre-push da main INATIVO neste clone. Peça ao usuário para rodar: git config core.hooksPath ${CAMINHO_GITHOOKS}`,
  `Branch atual: ${branchAtual(diretorioProjeto) || "(desconhecida)"}`,
  `Issue ativa nesta sessão: ${estadoSessao.issueAtiva ? `#${estadoSessao.issueAtiva.numero} ${estadoSessao.issueAtiva.titulo}` : "nenhuma (editar código de produção está bloqueado)"}`,
  `Mapas iniciais (ativar issue exige os quatro e ${MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR.join(" + ")} fechados):\n${situacaoDosMapasIniciais}`,
  `Mapas do wayfinder abertos:\n${mapasAbertos}`,
  "Grilling em todo ticket de decisão: o humano responde. Nenhuma recomendação sem pesquisa: cite docs/research/ ou a fonte primária; sem fonte, rode /research antes.",
  "Rota: ideia grande → /wayfinder (1 ticket por sessão) → /to-spec → /to-tickets → 1 sessão limpa por issue com /implement + /tdd. Janela cheia → /handoff.",
];
emitirContextoAdicional("SessionStart", linhasContexto.join("\n"));
