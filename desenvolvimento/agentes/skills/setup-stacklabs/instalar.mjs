#!/usr/bin/env node
// Instala ou atualiza o método StackLabs num projeto: modelo-de-projeto, skills da casa e as skills
// publicadas do Matt Pocock (sempre da versão mais nova do repositório dele).
//
// Uso: node instalar.mjs [--destino <pasta>] [--simular] [--sem-labels] [--atualizar-watchdogs] [--adocao]
//   --destino              raiz do projeto (padrão: a raiz do git da pasta atual)
//   --simular              mostra o que faria, sem escrever nada
//   --sem-labels           não cria as labels do método no GitHub
//   --atualizar-watchdogs  sobrescreve watchdogs que já existem. SÓ HUMANO (o guarda-bash barra o agente)
//   --adocao               projeto que já existia: cria .claude/metodo.json na etapa de observação (/adotar-metodo-stack-labs)
// Fontes (opcional, para usar um clone local em vez de baixar):
//   STACKLABS_ORIGEM=<clone do Stack-Labs>   MATT_ORIGEM=<clone de mattpocock/skills>

import { execFileSync, spawnSync } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, relative, resolve } from "node:path";

// Arquivos que o projeto pode ter com conteúdo próprio: nunca sobrescreve, deixa o novo ao lado.
const ARQUIVOS_PARA_MESCLAR = ["AGENTS.md", "CLAUDE.md", ".gitignore", ".claude/settings.json"];
// Infraestrutura dos watchdogs: só se sobrescreve com --atualizar-watchdogs.
const CAMINHOS_DOS_WATCHDOGS = [".claude/hooks", ".claude/githooks", ".claude/agents", ".claude/settings.json", ".github/workflows/vigia-do-metodo.yml"];
const LABELS_DO_METODO = [
  "wayfinder:map", "wayfinder:research", "wayfinder:prototype", "wayfinder:grilling", "wayfinder:task",
  "mapa-inicial:prototipo", "mapa-inicial:arquitetura", "mapa-inicial:fases", "mapa-inicial:infra",
  "needs-triage", "needs-info", "ready-for-agent", "ready-for-human", "wontfix", "spec", "altera-watchdogs",
];

// ---------- utilitários ----------

function lerArgumentos(argumentosDaLinha) {
  const opcoes = { destino: null, simular: false, criarLabels: true, atualizarWatchdogs: false, adocao: false };
  for (let indice = 0; indice < argumentosDaLinha.length; indice++) {
    const argumento = argumentosDaLinha[indice];
    if (argumento === "--destino") opcoes.destino = argumentosDaLinha[++indice];
    else if (argumento === "--simular") opcoes.simular = true;
    else if (argumento === "--sem-labels") opcoes.criarLabels = false;
    else if (argumento === "--atualizar-watchdogs") opcoes.atualizarWatchdogs = true;
    else if (argumento === "--adocao") opcoes.adocao = true;
    else throw new Error(`opção desconhecida: ${argumento}`);
  }
  return opcoes;
}

function git(...argumentos) {
  return execFileSync("git", argumentos, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
}

function gitOuNulo(...argumentos) {
  try {
    return git(...argumentos);
  } catch {
    return null;
  }
}

// Caminhos relativos de todos os arquivos de uma pasta, em ordem, com "/" como separador.
function listarArquivos(pastaRaiz, pastaAtual = pastaRaiz) {
  return readdirSync(pastaAtual, { withFileTypes: true })
    .flatMap((entrada) => {
      const caminhoDaEntrada = join(pastaAtual, entrada.name);
      return entrada.isDirectory() ? listarArquivos(pastaRaiz, caminhoDaEntrada) : [relative(pastaRaiz, caminhoDaEntrada).split("\\").join("/")];
    })
    .sort();
}

function arquivosIguais(caminhoA, caminhoB) {
  return existsSync(caminhoB) && statSync(caminhoB).isFile() && readFileSync(caminhoA).equals(readFileSync(caminhoB));
}

function pastasIguais(pastaA, pastaB) {
  if (!existsSync(pastaB)) return false;
  const arquivosDeA = listarArquivos(pastaA);
  const arquivosDeB = listarArquivos(pastaB);
  return arquivosDeA.length === arquivosDeB.length
    && arquivosDeA.every((caminhoRelativo, indice) => caminhoRelativo === arquivosDeB[indice] && arquivosIguais(join(pastaA, caminhoRelativo), join(pastaB, caminhoRelativo)));
}

function caminhoEhWatchdog(caminhoRelativo) {
  return CAMINHOS_DOS_WATCHDOGS.some((caminhoWatchdog) => caminhoRelativo === caminhoWatchdog || caminhoRelativo.startsWith(`${caminhoWatchdog}/`));
}

// ---------- instalação ----------

function instalar(opcoes) {
  const destino = resolve(opcoes.destino || git("rev-parse", "--show-toplevel"));
  const pastaTemporaria = mkdtempSync(join(tmpdir(), "setup-stacklabs-"));
  const relatorio = [];

  try {
    const buscarOrigem = (cloneLocal, urlDoRepositorio, nomeDaPasta) => {
      if (cloneLocal) return resolve(cloneLocal);
      const pastaDoClone = join(pastaTemporaria, nomeDaPasta);
      git("clone", "-q", "--depth", "1", urlDoRepositorio, pastaDoClone);
      return pastaDoClone;
    };
    const origemStacklabs = buscarOrigem(process.env.STACKLABS_ORIGEM, "https://github.com/stacklabs1/Stack-Labs", "stacklabs");
    const origemMatt = buscarOrigem(process.env.MATT_ORIGEM, "https://github.com/mattpocock/skills", "matt");
    const pastaDoModelo = join(origemStacklabs, "desenvolvimento", "agentes", "modelo-de-projeto");
    const pastaDasSkillsDaCasa = join(origemStacklabs, "desenvolvimento", "agentes", "skills");
    const pastaDasSkillsDoProjeto = join(destino, ".claude", "skills");
    const fazer = (acao) => { if (!opcoes.simular) acao(); };

    // ---------- 0. adoção: a etapa vem antes dos watchdogs, para eles já nascerem nela ----------
    const arquivoDoMetodo = join(destino, ".claude", "metodo.json");
    if (opcoes.adocao && !existsSync(arquivoDoMetodo)) {
      fazer(() => {
        mkdirSync(dirname(arquivoDoMetodo), { recursive: true });
        writeFileSync(arquivoDoMetodo, JSON.stringify({ etapaDeAdocao: "observacao", adotadoEm: new Date().toISOString() }, null, 2) + "\n");
      });
      relatorio.push("ADOÇÃO    .claude/metodo.json criado na etapa observacao");
    }

    // ---------- 1. modelo-de-projeto ----------
    const jaTemWatchdogs = existsSync(join(destino, ".claude", "hooks"));
    for (const caminhoRelativo of listarArquivos(pastaDoModelo)) {
      const arquivoDoModelo = join(pastaDoModelo, caminhoRelativo);
      const arquivoDoDestino = join(destino, caminhoRelativo);
      if (arquivosIguais(arquivoDoModelo, arquivoDoDestino)) continue;

      if (caminhoEhWatchdog(caminhoRelativo) && jaTemWatchdogs && !opcoes.atualizarWatchdogs) {
        relatorio.push(`PULADO    ${caminhoRelativo} (watchdog ausente ou desatualizado: um humano roda com --atualizar-watchdogs)`);
        continue;
      }
      if (ARQUIVOS_PARA_MESCLAR.includes(caminhoRelativo) && existsSync(arquivoDoDestino)) {
        fazer(() => copyFileSync(arquivoDoModelo, `${arquivoDoDestino}.stacklabs-novo`));
        relatorio.push(`MESCLAR   ${caminhoRelativo} (modelo novo em ${caminhoRelativo}.stacklabs-novo)`);
        continue;
      }
      fazer(() => {
        mkdirSync(dirname(arquivoDoDestino), { recursive: true });
        cpSync(arquivoDoModelo, arquivoDoDestino, { preserveTimestamps: true }); // mantém o modo (executável)
      });
      relatorio.push(`COPIADO   ${caminhoRelativo}`);
    }
    fazer(() => {
      mkdirSync(join(destino, "docs", "adr"), { recursive: true });
      mkdirSync(join(destino, "docs", "research"), { recursive: true });
    });

    // ---------- 2. skills ----------
    const pluginDoMatt = JSON.parse(readFileSync(join(origemMatt, ".claude-plugin", "plugin.json"), "utf8"));
    const pastasDoMatt = pluginDoMatt.skills.map((caminhoSkill) => join(origemMatt, caminhoSkill));
    const nomesDoMatt = pastasDoMatt.map((pastaSkill) => basename(pastaSkill));

    // Da casa: as recomendadas que não são do Matt e as skills próprias do método (as demais pastas de skills/).
    const subpastas = (pasta) => readdirSync(pasta, { withFileTypes: true }).filter((entrada) => entrada.isDirectory()).map((entrada) => join(pasta, entrada.name));
    const pastasDaCasa = [...subpastas(join(pastaDasSkillsDaCasa, "recomendada")), ...subpastas(pastaDasSkillsDaCasa)]
      .filter((pastaSkill) => basename(pastaSkill) !== "recomendada")
      .filter((pastaSkill) => !nomesDoMatt.includes(basename(pastaSkill)));
    const nomesDaCasa = pastasDaCasa.map((pastaSkill) => basename(pastaSkill));

    // Remove o que a instalação anterior trouxe e a fonte não tem mais.
    const arquivoDeOrigem = join(pastaDasSkillsDoProjeto, "ORIGEM.json");
    if (existsSync(arquivoDeOrigem)) {
      const origemAnterior = JSON.parse(readFileSync(arquivoDeOrigem, "utf8"));
      for (const nomeAnterior of [...origemAnterior.skillsDoMatt, ...origemAnterior.skillsDaCasa]) {
        const pastaAnterior = join(pastaDasSkillsDoProjeto, nomeAnterior);
        if (![...nomesDoMatt, ...nomesDaCasa].includes(nomeAnterior) && existsSync(pastaAnterior)) {
          fazer(() => rmSync(pastaAnterior, { recursive: true, force: true }));
          relatorio.push(`REMOVIDA  skill ${nomeAnterior} (saiu da fonte)`);
        }
      }
    }

    const copiarSkill = (pastaDeOrigem, rotulo) => {
      const pastaDoDestino = join(pastaDasSkillsDoProjeto, basename(pastaDeOrigem));
      if (pastasIguais(pastaDeOrigem, pastaDoDestino)) return;
      fazer(() => {
        rmSync(pastaDoDestino, { recursive: true, force: true });
        cpSync(pastaDeOrigem, pastaDoDestino, { recursive: true });
      });
      relatorio.push(`SKILL     ${basename(pastaDeOrigem)} (${rotulo})`);
    };
    pastasDoMatt.forEach((pastaSkill) => copiarSkill(pastaSkill, "Matt"));
    pastasDaCasa.forEach((pastaSkill) => copiarSkill(pastaSkill, "casa"));

    const commitStacklabs = gitOuNulo("-C", origemStacklabs, "rev-parse", "--short", "HEAD") || "desconhecido";
    const commitMatt = gitOuNulo("-C", origemMatt, "rev-parse", "--short", "HEAD") || "desconhecido";
    fazer(() => writeFileSync(arquivoDeOrigem, JSON.stringify({
      instaladoEm: new Date().toISOString(),
      stacklabs: { repositorio: "stacklabs1/Stack-Labs", commit: commitStacklabs },
      matt: { repositorio: "mattpocock/skills", commit: commitMatt, versao: pluginDoMatt.version },
      skillsDoMatt: nomesDoMatt,
      skillsDaCasa: nomesDaCasa,
    }, null, 2) + "\n"));

    // ---------- 3. pre-push e labels ----------
    if (gitOuNulo("-C", destino, "config", "--get", "core.hooksPath") !== ".claude/githooks") {
      if (!jaTemWatchdogs || opcoes.atualizarWatchdogs) {
        fazer(() => git("-C", destino, "config", "core.hooksPath", ".claude/githooks"));
        relatorio.push("PRE-PUSH  ativado (core.hooksPath)");
      } else {
        relatorio.push("PRE-PUSH  inativo: um humano roda git config core.hooksPath .claude/githooks");
      }
    }

    if (opcoes.criarLabels) {
      fazer(() => {
        for (const label of LABELS_DO_METODO) {
          const criacaoDaLabel = spawnSync("gh", ["label", "create", label, "--force"], { cwd: destino, encoding: "utf8" });
          if (criacaoDaLabel.status !== 0) throw new Error(`gh label create ${label} falhou: ${criacaoDaLabel.stderr.trim()}`);
        }
      });
      relatorio.push(`LABELS    as ${LABELS_DO_METODO.length} do método conferidas no GitHub`);
    }

    // ---------- relatório ----------
    console.log(`\nMétodo StackLabs em ${destino}`);
    console.log(`Fontes: Stack-Labs ${commitStacklabs} · mattpocock/skills ${commitMatt} (v${pluginDoMatt.version})`);
    if (opcoes.simular) console.log("(simulação: nada foi escrito)");
    console.log(relatorio.length ? relatorio.join("\n") : "Nada a fazer: o projeto já está em dia.");
  } finally {
    rmSync(pastaTemporaria, { recursive: true, force: true });
  }
}

try {
  instalar(lerArgumentos(process.argv.slice(2)));
} catch (erro) {
  console.error(erro.message);
  process.exit(1);
}
