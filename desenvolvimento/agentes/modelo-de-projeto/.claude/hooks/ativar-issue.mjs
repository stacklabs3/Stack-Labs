#!/usr/bin/env node
// Ativa uma issue para esta sessão. Quem valida e grava é o watchdog guarda-bash, que intercepta
// este comando ANTES de ele rodar: se você está lendo a saída abaixo, a issue passou.
// Uso: node .claude/hooks/ativar-issue.mjs <número>

import { execFileSync } from "node:child_process";

const numeroIssue = (process.argv[2] || "").replace(/^#/, "");
if (!/^\d+$/.test(numeroIssue)) {
  console.error("uso: node .claude/hooks/ativar-issue.mjs <número da issue>");
  process.exit(1);
}

const issueAtivada = JSON.parse(execFileSync("gh", ["issue", "view", numeroIssue, "--json", "title,labels,url"], { encoding: "utf8" }));
console.log(`Issue #${numeroIssue} ativa nesta sessão.`);
console.log(issueAtivada.title);
console.log(`Labels: ${issueAtivada.labels.map((label) => label.name).join(", ")}`);
console.log(issueAtivada.url);
console.log(`Próximo passo: git switch -c issue/${numeroIssue}-<slug> e então /implement com a issue (e o spec pai).`);
