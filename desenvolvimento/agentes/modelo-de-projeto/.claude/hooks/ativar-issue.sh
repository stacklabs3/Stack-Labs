#!/usr/bin/env bash
# Ativa uma issue para esta sessão. Quem valida e grava é o watchdog guarda-bash, que
# intercepta este comando ANTES de ele rodar: se você está lendo a saída abaixo, a issue passou.
# Uso: .claude/hooks/ativar-issue.sh <número>
set -euo pipefail
numero_issue="${1:?uso: ativar-issue.sh <número da issue>}"
numero_issue="${numero_issue#\#}"
echo "Issue #${numero_issue} ativa nesta sessão."
gh issue view "$numero_issue" --json title,labels,url --jq '"\(.title)\nLabels: \([.labels[].name] | join(", "))\n\(.url)"'
echo "Próximo passo: git switch -c issue/${numero_issue}-<slug> e então /implement com a issue (e o PRD pai)."
