// Utilitários compartilhados pelos watchdogs do workflow wayfinder / Matt Pocock.
// Regras e motivação: docs/agents/watchdogs.md

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync, openSync, readSync, fstatSync, closeSync } from "node:fs";
import { join, relative, resolve, isAbsolute } from "node:path";

// Smart zone calibrada para modelos de contexto longo: no aviso, feche a etapa (o orquestrador de
// issues já passa a rodada adiante aqui); no limite duro, a próxima ação é /nova-sessao.
export const LIMITE_AVISO_CONTEXTO = 450_000;
export const LIMITE_DURO_CONTEXTO = 550_000;

// Labels de triagem que tornam uma issue implementável (docs/agents/triage-labels.md).
export const LABELS_IMPLEMENTAVEIS = ["ready-for-agent", "ready-for-human"];

// Branches descartáveis do próprio workflow: /prototype e tickets de pesquisa do wayfinder.
export const PREFIXOS_BRANCH_DESCARTAVEL = ["prototype/", "research/"];

// Mapas iniciais da casa (docs/agents/mapas-iniciais.md): label mapa-inicial:<tipo> + wayfinder:map.
export const MAPAS_INICIAIS = ["prototipo", "arquitetura", "fases", "regras-de-codigo", "infra"];

// Mapas iniciais que precisam estar fechados para uma issue de implementação ser ativada.
export const MAPAS_INICIAIS_FECHADOS_PARA_IMPLEMENTAR = ["prototipo", "arquitetura", "fases", "regras-de-codigo"];

// Tipos de ticket do wayfinder cuja resolução exige a seção "## Fontes" (pesquisa antes de recomendar).
export const TIPOS_QUE_EXIGEM_FONTES = ["grilling", "task", "research"];

// Skills que abrem uma sessão de grilling. Chamadas pelo Skill tool ou digitadas como /<nome>.
export const SKILLS_DE_GRILLING = ["grilling", "grill-me", "grill-with-docs", "wayfinder"];

// Linguagem das perguntas ao humano no grilling (AGENTS.md, regra 10). Injetada quando o grilling
// começa e a cada resposta do humano, porque a skill sozinha deixa a pergunta crescer.
export const REGRA_LINGUAGEM_GRILLING = [
  "🗣️ [watchdog linguagem-grilling] Perguntas do grilling em linguagem simples:",
  "- Curta: título de até 8 palavras e corpo de até 3 linhas. Sem parágrafo de contexto, sem repetir o que o humano já disse.",
  "- Simples: palavras do dia a dia. Termo técnico só se for inevitável, explicado em meia linha.",
  "- Com exemplo real do cenário deste projeto: uma tela, um usuário, um dado ou um caso que existe aqui, mostrando o que muda com cada opção. Nada de exemplo genérico (\"imagine um e-commerce\").",
  "- Opções em lista A/B/C, uma linha cada. Recomendação em uma frase, com o porquê e a fonte.",
  "Ex.: ❓ Q1 - Quem pode cancelar um pedido? A) Só o cliente, até o envio. B) Cliente e atendente. Ex.: a Ana pede às 10h, o pacote sai às 14h; às 15h ela quer cancelar. Em A, não pode; em B, o atendente cancela. ➡️ A: o fluxo de devolução já cobre depois do envio (docs/research/devolucao.md).",
].join("\n");

export function nomeDaSkillDeGrilling(textoOuNome) {
  const nomeLimpo = String(textoOuNome || "").trim().replace(/^\//, "").split(/\s/)[0].split(":").pop();
  return SKILLS_DE_GRILLING.includes(nomeLimpo) ? nomeLimpo : null;
}

// Caminhos que nenhum agente edita: são os próprios watchdogs e o estado deles.
export const CAMINHOS_PROTEGIDOS = [".claude/hooks/", ".claude/githooks/", ".claude/agents/", ".claude/metodo.json", ".claude/settings.json", ".claude/state/", ".github/workflows/vigia-do-metodo.yml"];

// Onde o git procura os hooks deste repo (pre-push que protege a main).
export const CAMINHO_GITHOOKS = ".claude/githooks";

// Caminhos de planejamento e documentação, editáveis sem issue ativa.
export const CAMINHOS_LIVRES_SEM_ISSUE = [
  "AGENTS.md",
  "CLAUDE.md",
  "GLOSSARY.md",
  "GLOSSARY-MAP.md",
  "CONTEXT.md",
  "CONTEXT-MAP.md",
  "README.md",
  ".gitignore",
  "docs/",
  ".scratch/",
  "prototipos/",
  ".claude/",
];

// Etapa de adoção do método (projeto que já existia antes dele; docs/agents/adocao.md), gravada em
// .claude/metodo.json. Sem o arquivo, o projeto nasceu no método: etapa "completo".
//   observacao   — só protegem a si mesmos e a main; o resto apenas avisa
//   planejamento — valem as regras do wayfinder, das fontes e da ativação de issue
//   completo     — tudo, inclusive código só com issue ativa
export const ETAPAS_DE_ADOCAO = ["observacao", "planejamento", "completo"];

export function lerMetodoDoProjeto(diretorioProjeto) {
  const caminhoMetodo = join(diretorioProjeto, ".claude", "metodo.json");
  if (!existsSync(caminhoMetodo)) return { etapaDeAdocao: "completo", adotadoEm: null };
  const metodoDoProjeto = JSON.parse(readFileSync(caminhoMetodo, "utf8"));
  const etapaValida = ETAPAS_DE_ADOCAO.includes(metodoDoProjeto.etapaDeAdocao) ? metodoDoProjeto.etapaDeAdocao : "completo";
  return { etapaDeAdocao: etapaValida, adotadoEm: metodoDoProjeto.adotadoEm || null };
}

export function etapaDeAdocao(diretorioProjeto) {
  return lerMetodoDoProjeto(diretorioProjeto).etapaDeAdocao;
}

export function watchdogsDesligados() {
  return process.env.STACKLABS_WATCHDOGS === "desligado";
}

export function lerEntradaDoHook() {
  const textoEntrada = readFileSync(0, "utf8");
  return textoEntrada.trim() ? JSON.parse(textoEntrada) : {};
}

export function diretorioDoProjeto(entradaHook) {
  return process.env.CLAUDE_PROJECT_DIR || entradaHook.cwd || process.cwd();
}

// Exit 2 em PreToolUse impede a ferramenta; o stderr volta para o agente como motivo.
export function bloquearFerramenta(nomeWatchdog, motivo) {
  process.stderr.write(`[watchdog ${nomeWatchdog}] BLOQUEADO — ${motivo}\n(Regras: AGENTS.md · Watchdogs: docs/agents/watchdogs.md)\n`);
  process.exit(2);
}

export function emitirContextoAdicional(nomeEvento, textoContexto) {
  process.stdout.write(JSON.stringify({
    hookSpecificOutput: { hookEventName: nomeEvento, additionalContext: textoContexto },
  }));
}

// ---------- estado por sessão ----------

function caminhoEstadoSessao(diretorioProjeto, idSessao) {
  return join(diretorioProjeto, ".claude", "state", "sessoes", `${idSessao}.json`);
}

export function lerEstadoSessao(diretorioProjeto, idSessao) {
  const caminhoEstado = caminhoEstadoSessao(diretorioProjeto, idSessao);
  if (!existsSync(caminhoEstado)) return { issuesAtivas: {}, ticketsWayfinderResolvidos: [] };
  const estadoSessao = JSON.parse(readFileSync(caminhoEstado, "utf8"));
  // Estado gravado antes da issue por agente: a issue única era da sessão principal.
  if (!estadoSessao.issuesAtivas) estadoSessao.issuesAtivas = estadoSessao.issueAtiva ? { principal: estadoSessao.issueAtiva } : {};
  delete estadoSessao.issueAtiva;
  return estadoSessao;
}

// Quem está agindo: um subagente (agent_id, na orquestração) ou a própria sessão. Cada um tem a sua
// issue ativa, para que o orquestrador dispare um implementador por issue na mesma sessão.
export function chaveDoAgente(entradaHook) {
  return entradaHook.agent_id || "principal";
}

export function gravarEstadoSessao(diretorioProjeto, idSessao, estadoSessao) {
  const caminhoEstado = caminhoEstadoSessao(diretorioProjeto, idSessao);
  mkdirSync(join(caminhoEstado, ".."), { recursive: true });
  writeFileSync(caminhoEstado, JSON.stringify(estadoSessao, null, 2));
}

// ---------- git ----------

function executarGit(diretorioProjeto, argumentosGit) {
  try {
    return execFileSync("git", argumentosGit, { cwd: diretorioProjeto, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return null;
  }
}

export function caminhoHooksDoGit(diretorioProjeto) {
  return executarGit(diretorioProjeto, ["config", "--get", "core.hooksPath"]);
}

export function branchAtual(diretorioProjeto) {
  return executarGit(diretorioProjeto, ["symbolic-ref", "--short", "HEAD"]);
}

export function repositorioTemCommits(diretorioProjeto) {
  return executarGit(diretorioProjeto, ["rev-parse", "--verify", "--quiet", "HEAD"]) !== null;
}

export function branchEhPrincipal(nomeBranch) {
  return nomeBranch === "main" || nomeBranch === "master";
}

export function branchEhDescartavel(nomeBranch) {
  return Boolean(nomeBranch) && PREFIXOS_BRANCH_DESCARTAVEL.some((prefixo) => nomeBranch.startsWith(prefixo));
}

// ---------- GitHub ----------

export function repositorioGithub(diretorioProjeto) {
  const urlRemoto = executarGit(diretorioProjeto, ["remote", "get-url", "origin"]) || "";
  const correspondencia = urlRemoto.match(/github\.com[:/]([^/]+)\/(.+?)(\.git)?$/);
  return correspondencia ? `${correspondencia[1]}/${correspondencia[2]}` : null;
}

export function executarGh(diretorioProjeto, argumentosGh) {
  return execFileSync("gh", argumentosGh, {
    cwd: diretorioProjeto,
    encoding: "utf8",
    timeout: 15_000,
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

// Retorna { numero, titulo, estado, labels[], responsaveis[], quantidadeComentarios } ou lança erro.
export function buscarIssue(diretorioProjeto, numeroIssue) {
  const nomeRepositorio = repositorioGithub(diretorioProjeto);
  const issueBruta = JSON.parse(executarGh(diretorioProjeto, ["api", `repos/${nomeRepositorio}/issues/${numeroIssue}`]));
  return {
    numero: issueBruta.number,
    titulo: issueBruta.title,
    estado: issueBruta.state,
    corpo: issueBruta.body || "",
    labels: issueBruta.labels.map((label) => label.name),
    responsaveis: issueBruta.assignees.map((responsavel) => responsavel.login),
    quantidadeComentarios: issueBruta.comments,
  };
}

export function tipoWayfinder(labelsIssue) {
  const labelWayfinder = labelsIssue.find((label) => label.startsWith("wayfinder:"));
  return labelWayfinder ? labelWayfinder.slice("wayfinder:".length) : null;
}

// Mapa dono de um ticket: sub-issue nativa primeiro, depois a convenção "Part of #N" do corpo.
export function numeroDoMapa(diretorioProjeto, ticketWayfinder) {
  const nomeRepositorio = repositorioGithub(diretorioProjeto);
  try {
    const numeroPai = executarGh(diretorioProjeto, ["api", `repos/${nomeRepositorio}/issues/${ticketWayfinder.numero}/parent`, "--jq", ".number"]);
    if (numeroPai) return Number(numeroPai);
  } catch { /* sub-issues indisponível: usa o corpo */ }
  const referenciaPai = ticketWayfinder.corpo.match(/Part of #(\d+)/i);
  return referenciaPai ? Number(referenciaPai[1]) : null;
}

// Corpos dos comentários de uma issue, do mais antigo ao mais novo.
export function buscarCorposDosComentarios(diretorioProjeto, numeroIssue) {
  const nomeRepositorio = repositorioGithub(diretorioProjeto);
  const corposJson = executarGh(diretorioProjeto, ["api", "--paginate", `repos/${nomeRepositorio}/issues/${numeroIssue}/comments`, "--jq", "[.[].body]"]);
  // --paginate emite um array JSON por página.
  return corposJson.split("\n").filter(Boolean).flatMap((paginaJson) => JSON.parse(paginaJson));
}

// Situação de cada mapa inicial: { tipo, numero, titulo, estado } ou { tipo, numero: null } quando falta.
export function buscarMapasIniciais(diretorioProjeto) {
  return MAPAS_INICIAIS.map((tipoMapa) => {
    const mapasEncontrados = JSON.parse(executarGh(diretorioProjeto, [
      "issue", "list", "--label", `mapa-inicial:${tipoMapa}`, "--label", "wayfinder:map", "--state", "all", "--json", "number,title,state",
    ]));
    const mapaDoTipo = mapasEncontrados[0];
    if (!mapaDoTipo) return { tipo: tipoMapa, numero: null };
    return { tipo: tipoMapa, numero: mapaDoTipo.number, titulo: mapaDoTipo.title, estado: mapaDoTipo.state.toLowerCase() };
  });
}

// "## Fontes" seguida de ao menos um link http(s) ou um caminho em docs/research/.
export function textoTemFontes(textoResolucao) {
  const textoAposFontes = (textoResolucao || "").split(/^#{2,3}\s*Fontes\b.*$/im)[1];
  if (textoAposFontes === undefined) return false;
  const secaoFontes = textoAposFontes.split(/^#{1,2}\s/m)[0];
  return /https?:\/\/\S+|docs\/research\/\S+/.test(secaoFontes);
}

// ---------- caminhos ----------

export function caminhoRelativoAoProjeto(diretorioProjeto, caminhoArquivo) {
  const caminhoAbsoluto = isAbsolute(caminhoArquivo) ? caminhoArquivo : resolve(diretorioProjeto, caminhoArquivo);
  const caminhoRelativo = relative(diretorioProjeto, caminhoAbsoluto);
  if (caminhoRelativo.startsWith("..") || isAbsolute(caminhoRelativo)) return null; // fora do repositório
  return caminhoRelativo.split("\\").join("/");
}

export function caminhoEhProtegido(caminhoRelativo) {
  return CAMINHOS_PROTEGIDOS.some((caminhoProtegido) => caminhoRelativo === caminhoProtegido.replace(/\/$/, "") || caminhoRelativo.startsWith(caminhoProtegido));
}

// Handoffs de sessão satélite (/voltar-a-principal): HANDOFF-<projeto>.md na raiz do repositório.
const PADRAO_HANDOFF_NA_RAIZ = /^HANDOFF-[^/]+\.md$/;

export function caminhoEhLivreSemIssue(caminhoRelativo) {
  if (PADRAO_HANDOFF_NA_RAIZ.test(caminhoRelativo)) return true;
  return CAMINHOS_LIVRES_SEM_ISSUE.some((caminhoLivre) => (caminhoLivre.endsWith("/") ? caminhoRelativo.startsWith(caminhoLivre) : caminhoRelativo === caminhoLivre));
}

// ---------- contexto ----------

// Tokens de entrada da última resposta do modelo, lidos do transcript (JSONL).
export function medirTokensDeContexto(caminhoTranscript) {
  if (!caminhoTranscript || !existsSync(caminhoTranscript)) return 0;
  const descritorArquivo = openSync(caminhoTranscript, "r");
  const tamanhoArquivo = fstatSync(descritorArquivo).size;
  const tamanhoLeitura = Math.min(tamanhoArquivo, 4 * 1024 * 1024);
  const bufferFinal = Buffer.alloc(tamanhoLeitura);
  readSync(descritorArquivo, bufferFinal, 0, tamanhoLeitura, tamanhoArquivo - tamanhoLeitura);
  closeSync(descritorArquivo);
  const linhasTranscript = bufferFinal.toString("utf8").split("\n").reverse();
  for (const linhaTranscript of linhasTranscript) {
    if (!linhaTranscript.includes('"usage"')) continue;
    try {
      const usoTokens = JSON.parse(linhaTranscript)?.message?.usage;
      if (!usoTokens) continue;
      return (usoTokens.input_tokens || 0) + (usoTokens.cache_creation_input_tokens || 0) + (usoTokens.cache_read_input_tokens || 0);
    } catch { /* linha truncada no início do buffer */ }
  }
  return 0;
}
