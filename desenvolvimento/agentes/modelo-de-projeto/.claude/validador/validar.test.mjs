// Testes do validador: cada checagem roda num repositório git de verdade, numa pasta temporária.
// Rodar: node --test .claude/validador/   (o Opengrep entra se estiver no PATH ou em $OPENGREP)

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, copyFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { validar, checarCobertura, termosEvitadosDoGlossario, globParaRegex, PRINCIPIOS, ErroDeConfiguracao } from "./validar.mjs";

const pastaDoValidador = dirname(fileURLToPath(import.meta.url));
const opengrepDisponivel = spawnSync(process.env.OPENGREP || "opengrep", ["--version"]).status === 0;

function criarRepositorio(arquivos) {
  const raiz = mkdtempSync(join(tmpdir(), "validador-"));
  for (const [caminhoRelativo, conteudo] of Object.entries(arquivos)) {
    mkdirSync(dirname(join(raiz, caminhoRelativo)), { recursive: true });
    writeFileSync(join(raiz, caminhoRelativo), conteudo);
  }
  const git = (...argumentos) => spawnSync("git", argumentos, { cwd: raiz, encoding: "utf8" });
  git("init", "-q", "-b", "main");
  git("config", "user.email", "teste@stacklabs");
  git("config", "user.name", "Teste");
  git("add", "-A");
  git("commit", "-q", "-m", "chore: base");
  return { raiz, git };
}

function tabelaCom(linhasPorPrincipio) {
  const linhas = PRINCIPIOS.map((principio) => `| ${principio} | ${linhasPorPrincipio[principio] || "afastado | fora do escopo deste teste"} |`);
  return ["| Princípio | Decisão | Onde ou por quê |", "|---|---|---|", ...linhas].join("\n");
}

function rodarUmaRegra(arquivos, verificacao, principio = "E1") {
  const { raiz } = criarRepositorio(arquivos);
  const config = { versao: 1, projeto: { codigo: ["src/**"] }, regras: [{ id: "R1", principio, verificacao }] };
  const relatorio = validar({ raiz, config, textoMd: tabelaCom({ [principio]: "adotado | R1" }) });
  return relatorio.resultados.find((resultado) => resultado.id === "R1").violacoes;
}

test("glob: ** atravessa pastas, * fica numa só, {a,b} escolhe", () => {
  assert.ok(globParaRegex("src/**/*.ts").test("src/a/b/c.ts"));
  assert.ok(globParaRegex("src/**/*.ts").test("src/c.ts"));
  assert.ok(!globParaRegex("src/*.ts").test("src/a/c.ts"));
  assert.ok(globParaRegex("src/*.{ts,js}").test("src/c.js"));
});

test("cobertura: aprova os 22 decididos e ligados às regras", () => {
  const violacoes = checarCobertura({ textoMd: tabelaCom({ E1: "adotado | R1" }), regras: [{ id: "R1", principio: "E1" }] });
  assert.deepEqual(violacoes, []);
});

test("cobertura: reprova princípio faltando, decisão inválida, regra inexistente e regra órfã", () => {
  const textoMd = tabelaCom({ E1: "adotado | R1, R9", E2: "talvez | R2" }).replace(/\| D \|.*\n?/, "");
  const mensagens = checarCobertura({ textoMd, regras: [{ id: "R1", principio: "E1" }, { id: "R5", principio: "E3" }] }).map((violacao) => violacao.mensagem);
  assert.ok(mensagens.some((mensagem) => mensagem.startsWith("D sem linha")));
  assert.ok(mensagens.some((mensagem) => mensagem.includes('decisão "talvez"')));
  assert.ok(mensagens.some((mensagem) => mensagem.includes("R9, que não está")));
  assert.ok(mensagens.some((mensagem) => mensagem.startsWith("R5 está no .json")));
});

test("cobertura: reprova regra citada num princípio diferente do dela", () => {
  const mensagens = checarCobertura({ textoMd: tabelaCom({ E2: "adotado | R1" }), regras: [{ id: "R1", principio: "E1" }] }).map((violacao) => violacao.mensagem);
  assert.ok(mensagens.some((mensagem) => mensagem.includes("R1 nasce de E1")));
});

test("termo-evitado: lê os termos do glossário e acha o nome declarado que usa um", () => {
  assert.deepEqual(termosEvitadosDoGlossario("_Evitar_: Épico, roadmap, Tarefa (quando não é), ok\n"), ["Épico", "roadmap", "Tarefa"]);
  const violacoes = rodarUmaRegra({
    "GLOSSARY.md": "**Comanda**:\nO pedido da mesa.\n_Evitar_: Ticket, ordem de serviço\n",
    "src/comanda.ts": "export function abrirComanda() {}\nexport const ticketAberto = 1;\nclass OrdemDeServicoRepo {}\n// um ticket qualquer no comentário\n",
  }, { tipo: "termo-evitado" });
  assert.deepEqual(violacoes.map((violacao) => violacao.linha), [2, 3]);
});

test("todo-sem-issue: aceita TODO com issue e reprova sem", () => {
  const violacoes = rodarUmaRegra({ "src/a.ts": "// TODO #12 trocar\n// TODO trocar depois\n// FIXME ver https://x/issues/3\n" }, { tipo: "todo-sem-issue" }, "E6");
  assert.deepEqual(violacoes.map((violacao) => violacao.linha), [2]);
});

test("teste-de-contrato-por-port: reprova port sem contrato", () => {
  const violacoes = rodarUmaRegra({
    "src/pagamento/portas/cobranca.ts": "export interface Cobranca {}\n",
    "src/pagamento/portas/cobranca.contrato.test.ts": "",
    "src/pedido/portas/repositorio.ts": "export interface Repositorio {}\n",
  }, { tipo: "teste-de-contrato-por-port", ports: "src/**/portas/*.ts", contrato: "{dir}/{nome}.contrato.test.ts" }, "L");
  assert.deepEqual(violacoes.map((violacao) => violacao.arquivo), ["src/pedido/portas/repositorio.ts"]);
});

test("lint-cobre: reprova regra delegada ao lint que não está na configuração", () => {
  const violacoes = rodarUmaRegra({ "eslint.config.js": 'export default [{ rules: { "max-depth": ["error", 3] } }];\n' },
    { tipo: "lint-cobre", config: "eslint.config.js", regras: ["max-depth", "max-params"] }, "E2");
  assert.deepEqual(violacoes.map((violacao) => violacao.mensagem), ['a regra de lint "max-params" não está configurada']);
});

test("mensagem-de-commit: confere os commits desde a base", () => {
  const { raiz, git } = criarRepositorio({ "a.txt": "1" });
  git("switch", "-q", "-c", "issue/1-x");
  writeFileSync(join(raiz, "a.txt"), "2"); git("commit", "-qam", "feat: muda a");
  writeFileSync(join(raiz, "a.txt"), "3"); git("commit", "-qam", "ajustes");
  const config = { versao: 1, regras: [{ id: "R1", principio: "P9", verificacao: { tipo: "mensagem-de-commit", padrao: "^(feat|fix): .+" } }] };
  const relatorio = validar({ raiz, config, textoMd: tabelaCom({ P9: "adotado | R1" }), base: "main" });
  const violacoes = relatorio.resultados.find((resultado) => resultado.id === "R1").violacoes;
  assert.equal(violacoes.length, 1);
  assert.match(violacoes[0].mensagem, /"ajustes"/);
});

test("comando: saída 0 aprova, outra saída reprova com o final da saída", () => {
  assert.deepEqual(rodarUmaRegra({}, { tipo: "comando", executar: "true" }), []);
  const [violacao] = rodarUmaRegra({}, { tipo: "comando", executar: "echo duplicado em a.ts; exit 3" });
  assert.match(violacao.mensagem, /saiu com 3[\s\S]*duplicado em a\.ts/);
});

test("configuração: tipo fora do catálogo e princípio inexistente são erro de configuração", () => {
  const { raiz } = criarRepositorio({});
  assert.throws(() => validar({ raiz, config: { versao: 1, regras: [{ id: "R1", principio: "E1", verificacao: { tipo: "magia" } }] }, textoMd: "" }), ErroDeConfiguracao);
  assert.throws(() => validar({ raiz, config: { versao: 1, regras: [{ id: "R1", principio: "Z9", verificacao: { tipo: "revisor" } }] }, textoMd: "" }), ErroDeConfiguracao);
});

test("modelo de regras-de-codigo.json é válido para o motor", () => {
  const modelo = JSON.parse(readFileSync(join(pastaDoValidador, "modelos", "regras-de-codigo.json"), "utf8"));
  const { raiz } = criarRepositorio({});
  const linhas = Object.fromEntries(modelo.regras.map((regra) => [regra.principio, `adotado | ${regra.id}`]));
  const relatorio = validar({ raiz, config: modelo, textoMd: tabelaCom(linhas), soCobertura: true });
  assert.ok(relatorio.aprovado, JSON.stringify(relatorio.resultados[0].violacoes));
});

test("opengrep: os modelos acham import de infra no domínio, catch vazio e ambiente fora da montagem", { skip: !opengrepDisponivel && "opengrep não instalado" }, () => {
  const preencher = (modelo, trocas) => Object.entries(trocas).reduce((texto, [marcador, valor]) => texto.replaceAll(marcador, valor), readFileSync(join(pastaDoValidador, "modelos", "opengrep", modelo), "utf8"));
  const { raiz } = criarRepositorio({
    "regras/dominio.yml": preencher("dominio-sem-infra.yml", { "<PASTA_DO_DOMINIO>": "src/dominio", "<PACOTES_DE_INFRA>": "prisma|@prisma/|express" }),
    "regras/catch.yml": preencher("catch-vazio.yml", {}),
    "regras/ambiente.yml": preencher("ambiente-so-na-montagem.yml", { "<ARQUIVOS_DA_MONTAGEM>": "src/montagem/**" }),
    "src/dominio/pedido.ts": 'import { PrismaClient } from "@prisma/client";\nimport { somar } from "./dinheiro";\nexport const x = 1;\n',
    "src/infra/repo.ts": 'import { PrismaClient } from "@prisma/client";\nexport function salvar() { try { gravar(); } catch (erro) { } }\n',
    "src/montagem/index.ts": "export const chave = process.env.CHAVE;\n",
    "src/dominio/config.ts": "export const chave = process.env.CHAVE;\n",
  });
  const config = { versao: 1, regras: [
    { id: "R1", principio: "D", verificacao: { tipo: "opengrep", regras: "regras/dominio.yml" } },
    { id: "R2", principio: "E4", verificacao: { tipo: "opengrep", regras: "regras/catch.yml" } },
    { id: "R3", principio: "P3", verificacao: { tipo: "opengrep", regras: "regras/ambiente.yml" } },
  ] };
  const relatorio = validar({ raiz, config, textoMd: tabelaCom({ D: "adotado | R1", E4: "adotado | R2", P3: "adotado | R3" }) });
  const achadosDe = (id) => relatorio.resultados.find((resultado) => resultado.id === id).violacoes.map((violacao) => `${violacao.arquivo}:${violacao.linha}`);
  assert.deepEqual(achadosDe("R1"), ["src/dominio/pedido.ts:1"]);
  assert.deepEqual(achadosDe("R2"), ["src/infra/repo.ts:2"]);
  assert.deepEqual(achadosDe("R3"), ["src/dominio/config.ts:1"]);
});

test("linha de comando: imprime o parecer e sai com 1 quando reprova", () => {
  const { raiz } = criarRepositorio({ "docs/regras-de-codigo.md": tabelaCom({ E6: "adotado | R1" }), "src/a.ts": "// TODO sem issue\n" });
  mkdirSync(join(raiz, ".claude", "validador"), { recursive: true });
  copyFileSync(join(pastaDoValidador, "validar.mjs"), join(raiz, ".claude", "validador", "validar.mjs"));
  writeFileSync(join(raiz, "docs", "regras-de-codigo.json"), JSON.stringify({ versao: 1, projeto: { codigo: ["src/**"] }, regras: [{ id: "R1", principio: "E6", verificacao: { tipo: "todo-sem-issue" } }] }));
  const resultado = spawnSync("node", [".claude/validador/validar.mjs"], { cwd: raiz, encoding: "utf8" });
  assert.equal(resultado.status, 1);
  assert.match(resultado.stdout, /❌ R1 \(E6\) todo-sem-issue[\s\S]*src\/a\.ts:1[\s\S]*VALIDADOR: REPROVADO$/m);
});
