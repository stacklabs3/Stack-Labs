#!/usr/bin/env bash
# Loop de issues (padrão "Ralph" do Matt Pocock): implementa até N issues ready-for-agent em sequência.
# Cada issue roda numa sessão nova do Claude Code (`claude -p`), uma de cada vez, com os watchdogs do
# projeto ativos. Quando a sessão bate na smart zone, ela deixa um handoff e uma sessão nova continua
# a mesma issue. Cada issue termina numa PR; o merge é de um humano.
#
# Uso: .claude/loop/loop-de-issues.sh <N> [--orcamento <USD por sessão>] [--modelo <alias>] [--simular]
# Guia: docs/agents/loop-de-issues.md · Skill: /loop-de-issues
set -euo pipefail

quantidade_maxima="${1:?uso: loop-de-issues.sh <N> [--orcamento <USD>] [--modelo <alias>] [--simular]}"
shift
orcamento_por_sessao="5" ; modelo="" ; simular=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --orcamento) orcamento_por_sessao="$2"; shift 2 ;;
    --modelo) modelo="$2"; shift 2 ;;
    --simular) simular=1; shift ;;
    *) echo "opção desconhecida: $1" >&2; exit 1 ;;
  esac
done

LABEL_DA_FILA="ready-for-agent"
LABEL_DE_DEVOLVIDA="needs-triage"
FALHAS_SEGUIDAS_PARA_PARAR=2
SESSOES_POR_ISSUE=4

raiz_do_projeto="$(git rev-parse --show-toplevel)"
cd "$raiz_do_projeto"
repositorio="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
modelo_do_prompt="$raiz_do_projeto/.claude/loop/prompt-da-issue.md"
nome_da_rodada="$(date +%Y-%m-%dT%H-%M-%S)"
pasta_de_registros="$raiz_do_projeto/.claude/state/loop/$nome_da_rodada"
# Os handoffs ficam fora do repositório: na smart zone, o watchdog barra edições dentro dele.
pasta_de_handoffs="${TMPDIR:-/tmp}/loop-de-issues-$nome_da_rodada"

# ---------- pré-requisitos ----------
if [[ -n "$(git status --porcelain)" ]]; then
  echo "O working tree tem mudanças. O loop começa cada issue de uma main limpa: faça commit ou stash antes." >&2
  exit 1
fi
gh auth status >/dev/null 2>&1 || { echo "gh não está autenticado." >&2; exit 1; }
command -v claude >/dev/null || { echo "claude não está no PATH." >&2; exit 1; }

# ---------- fronteira ----------
# Issues abertas com a label da fila, sem responsável e sem bloqueio aberto, da mais antiga para a mais nova.
issue_tem_bloqueio_aberto() {
  local numero_issue="$1" bloqueios_abertos corpo_issue numero_bloqueio
  bloqueios_abertos="$(gh api "repos/$repositorio/issues/$numero_issue" --jq '.issue_dependencies_summary.blocked_by // 0' 2>/dev/null || echo 0)"
  [[ "$bloqueios_abertos" -gt 0 ]] && return 0
  corpo_issue="$(gh issue view "$numero_issue" --json body --jq .body)"
  for numero_bloqueio in $(grep -oiE 'Blocked by:.*' <<<"$corpo_issue" | grep -oE '#[0-9]+' | tr -d '#'); do
    [[ "$(gh issue view "$numero_bloqueio" --json state --jq .state)" == "OPEN" ]] && return 0
  done
  return 1
}

proxima_issue_da_fronteira() {
  local numero_issue
  for numero_issue in $(gh issue list --label "$LABEL_DA_FILA" --state open --limit 200 --json number,assignees,labels \
      --jq '[.[] | select((.assignees | length) == 0) | select([.labels[].name | startswith("wayfinder:")] | any | not)] | sort_by(.number) | .[].number'); do
    [[ " ${issues_tentadas[*]} " == *" $numero_issue "* ]] && continue
    issue_tem_bloqueio_aberto "$numero_issue" && continue
    echo "$numero_issue"
    return
  done
}

montar_prompt() { # $1 issue, $2 handoff de entrada (ou vazio), $3 handoff de saída
  node -e '
    const [modelo, issue, handoffEntrada, handoffSaida] = process.argv.slice(1);
    const continuacao = handoffEntrada
      ? `\nEsta sessão continua o trabalho de uma sessão anterior que bateu na smart zone. Antes de tudo,\nleia o handoff em ${handoffEntrada} e siga dali. A branch da issue já existe, com o trabalho commitado.\n`
      : "";
    process.stdout.write(require("fs").readFileSync(modelo, "utf8")
      .replaceAll("{{ISSUE}}", issue).replaceAll("{{CONTINUACAO}}", continuacao).replaceAll("{{HANDOFF_SAIDA}}", handoffSaida));
  ' "$modelo_do_prompt" "$1" "$2" "$3"
}

devolver_issue() { # $1 issue, $2 motivo
  gh issue comment "$1" --body "Loop de issues: não terminou esta issue. Motivo: $2" >/dev/null
  gh issue edit "$1" --remove-assignee @me --remove-label "$LABEL_DA_FILA" --add-label "$LABEL_DE_DEVOLVIDA" >/dev/null
}

# ---------- loop ----------
issues_tentadas=()
resumo=()
falhas_seguidas=0
mkdir -p "$pasta_de_registros" "$pasta_de_handoffs"

for ((volta = 1; volta <= quantidade_maxima; volta++)); do
  numero_issue="$(proxima_issue_da_fronteira)"
  if [[ -z "$numero_issue" ]]; then
    resumo+=("A fronteira esvaziou: nenhuma issue $LABEL_DA_FILA livre e desbloqueada.")
    break
  fi
  issues_tentadas+=("$numero_issue")
  titulo_issue="$(gh issue view "$numero_issue" --json title --jq .title)"
  echo "[$volta/$quantidade_maxima] \"$titulo_issue\" (#$numero_issue)"

  if [[ $simular -eq 1 ]]; then
    resumo+=("SIMULADA  \"$titulo_issue\" (#$numero_issue)")
    continue
  fi

  # Reivindica antes de começar: outra pessoa ou outro loop pula esta issue.
  gh issue edit "$numero_issue" --add-assignee @me >/dev/null
  git fetch -q origin main
  git switch -q main
  git merge -q --ff-only origin/main

  handoff_de_entrada=""
  resultado_da_issue=""
  for ((sessao = 1; sessao <= SESSOES_POR_ISSUE; sessao++)); do
    handoff_de_saida="$pasta_de_handoffs/issue-$numero_issue-sessao-$sessao.md"
    registro_da_sessao="$pasta_de_registros/issue-$numero_issue-sessao-$sessao.json"
    argumentos_do_claude=(-p "$(montar_prompt "$numero_issue" "$handoff_de_entrada" "$handoff_de_saida")"
      --permission-mode auto --permission-prompts none
      --max-budget-usd "$orcamento_por_sessao" --output-format json)
    [[ -n "$modelo" ]] && argumentos_do_claude+=(--model "$modelo")
    echo "  sessão $sessao"
    claude "${argumentos_do_claude[@]}" >"$registro_da_sessao" 2>"${registro_da_sessao%.json}.stderr" || true

    resultado_da_sessao="$(node -e 'try { console.log(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).result || "") } catch { console.log("") }' "$registro_da_sessao")"
    resultado_da_issue="$(grep -E '^RESULTADO: ' <<<"$resultado_da_sessao" | tail -1 || true)"

    if [[ "$resultado_da_issue" == "RESULTADO: HANDOFF" && -s "$handoff_de_saida" ]]; then
      handoff_de_entrada="$handoff_de_saida"
      continue
    fi
    break
  done

  if [[ "$resultado_da_issue" == "RESULTADO: PR "* ]]; then
    resumo+=("PR        \"$titulo_issue\" (#$numero_issue) em $sessao sessão(ões): ${resultado_da_issue#RESULTADO: PR }")
    falhas_seguidas=0
  else
    case "$resultado_da_issue" in
      "RESULTADO: BLOQUEADO "*) motivo_da_falha="${resultado_da_issue#RESULTADO: BLOQUEADO }" ;;
      "RESULTADO: HANDOFF") motivo_da_falha="passou de $SESSOES_POR_ISSUE sessões sem terminar; a issue provavelmente precisa ser fatiada" ;;
      *) motivo_da_falha="a sessão terminou sem a linha RESULTADO (registro: $registro_da_sessao)" ;;
    esac
    devolver_issue "$numero_issue" "$motivo_da_falha"
    resumo+=("DEVOLVIDA \"$titulo_issue\" (#$numero_issue): $motivo_da_falha")
    falhas_seguidas=$((falhas_seguidas + 1))
  fi

  # Trabalho não commitado nunca se descarta: o loop para e deixa para um humano.
  if [[ -n "$(git status --porcelain)" ]]; then
    resumo+=("Parei: a sessão deixou mudanças sem commit em $(git branch --show-current). Confira antes de rodar de novo.")
    break
  fi
  git switch -q main
  if [[ $falhas_seguidas -ge $FALHAS_SEGUIDAS_PARA_PARAR ]]; then
    resumo+=("Parei: $FALHAS_SEGUIDAS_PARA_PARAR issues seguidas sem PR. Algo comum a elas precisa de um humano.")
    break
  fi
done

printf '%s\n' "${resumo[@]}" | tee "$pasta_de_registros/resumo.txt"
echo "Registros: $pasta_de_registros"
echo "Handoffs: $pasta_de_handoffs"
