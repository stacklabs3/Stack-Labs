#!/usr/bin/env bash
# Instala ou atualiza o método StackLabs num projeto: modelo-de-projeto, skills da casa e as skills
# publicadas do Matt Pocock (sempre da versão mais nova do repositório dele).
#
# Uso: instalar.sh [--destino <pasta>] [--simular] [--sem-labels] [--atualizar-watchdogs]
#   --destino              raiz do projeto (padrão: a raiz do git da pasta atual)
#   --simular              mostra o que faria, sem escrever nada
#   --sem-labels           não cria as labels do método no GitHub
#   --atualizar-watchdogs  sobrescreve watchdogs que já existem. SÓ HUMANO (o guarda-bash barra o agente)
# Fontes (opcional, para usar um clone local em vez de baixar):
#   STACKLABS_ORIGEM=<clone do Stack-Labs>   MATT_ORIGEM=<clone de mattpocock/skills>
set -euo pipefail

destino="" ; simular=0 ; criar_labels=1 ; atualizar_watchdogs=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --destino) destino="$2"; shift 2 ;;
    --simular) simular=1; shift ;;
    --sem-labels) criar_labels=0; shift ;;
    --atualizar-watchdogs) atualizar_watchdogs=1; shift ;;
    *) echo "opção desconhecida: $1" >&2; exit 1 ;;
  esac
done
destino="${destino:-$(git rev-parse --show-toplevel)}"
destino="$(cd "$destino" && pwd)"

pasta_temporaria="$(mktemp -d)"
trap 'rm -rf "${pasta_temporaria:?}"' EXIT

buscar_origem() { # $1 variável com clone local, $2 url, $3 nome
  local clone_local="$1" url_repositorio="$2" nome_pasta="$3"
  if [[ -n "$clone_local" ]]; then echo "$clone_local"; return; fi
  git clone -q --depth 1 "$url_repositorio" "$pasta_temporaria/$nome_pasta" >&2
  echo "$pasta_temporaria/$nome_pasta"
}
origem_stacklabs="$(buscar_origem "${STACKLABS_ORIGEM:-}" https://github.com/stacklabs1/Stack-Labs stacklabs)"
origem_matt="$(buscar_origem "${MATT_ORIGEM:-}" https://github.com/mattpocock/skills matt)"
modelo="$origem_stacklabs/desenvolvimento/agentes/modelo-de-projeto"
skills_stacklabs="$origem_stacklabs/desenvolvimento/agentes/skills"
pasta_skills="$destino/.claude/skills"

relatorio=()
fazer() { # executa, ou só descreve no modo --simular
  if [[ $simular -eq 1 ]]; then echo "  [simular] $*"; else "$@"; fi
}

# ---------- 1. modelo-de-projeto ----------
# Arquivos que o projeto pode ter com conteúdo próprio: nunca sobrescreve, deixa o novo ao lado.
arquivos_para_mesclar=("AGENTS.md" "CLAUDE.md" ".gitignore" ".claude/settings.json")
# Infraestrutura dos watchdogs: só se sobrescreve com --atualizar-watchdogs.
caminhos_dos_watchdogs=(".claude/hooks" ".claude/githooks" ".claude/loop" ".claude/settings.json" ".github/workflows/vigia-do-metodo.yml")

ja_tem_watchdogs=0
[[ -d "$destino/.claude/hooks" ]] && ja_tem_watchdogs=1

while IFS= read -r arquivo_modelo; do
  caminho_relativo="${arquivo_modelo#"$modelo"/}"
  arquivo_destino="$destino/$caminho_relativo"
  eh_watchdog=0
  for caminho_watchdog in "${caminhos_dos_watchdogs[@]}"; do
    [[ "$caminho_relativo" == "$caminho_watchdog" || "$caminho_relativo" == "$caminho_watchdog"/* ]] && eh_watchdog=1
  done
  if [[ -f "$arquivo_destino" ]] && cmp -s "$arquivo_modelo" "$arquivo_destino"; then continue; fi

  if [[ $eh_watchdog -eq 1 && $ja_tem_watchdogs -eq 1 && $atualizar_watchdogs -eq 0 ]]; then
    relatorio+=("PULADO    $caminho_relativo (watchdog ausente ou desatualizado: um humano roda com --atualizar-watchdogs)")
    continue
  fi
  if [[ " ${arquivos_para_mesclar[*]} " == *" $caminho_relativo "* && -f "$arquivo_destino" ]]; then
    fazer cp "$arquivo_modelo" "$arquivo_destino.stacklabs-novo"
    relatorio+=("MESCLAR   $caminho_relativo (modelo novo em $caminho_relativo.stacklabs-novo)")
    continue
  fi
  fazer mkdir -p "$(dirname "$arquivo_destino")"
  fazer cp -p "$arquivo_modelo" "$arquivo_destino"
  relatorio+=("COPIADO   $caminho_relativo")
done < <(find "$modelo" -type f | sort)
fazer mkdir -p "$destino/docs/adr" "$destino/docs/research"

# ---------- 2. skills ----------
skills_do_matt=()
while IFS= read -r caminho_skill; do skills_do_matt+=("$caminho_skill"); done < <(
  node -e 'for (const caminho of require(process.argv[1]).skills) console.log(caminho.replace(/^\.\//, ""))' "$origem_matt/.claude-plugin/plugin.json"
)
nomes_do_matt=()
for caminho_skill in "${skills_do_matt[@]}"; do nomes_do_matt+=("$(basename "$caminho_skill")"); done

# Da casa: as recomendadas que não são do Matt e as skills próprias do método (as demais pastas de skills/).
skills_da_casa=()
for pasta_skill in "$skills_stacklabs"/recomendada/*/ "$skills_stacklabs"/*/; do
  nome_skill="$(basename "$pasta_skill")"
  [[ "$nome_skill" == "recomendada" ]] && continue
  [[ " ${nomes_do_matt[*]} " == *" $nome_skill "* ]] && continue
  skills_da_casa+=("$pasta_skill")
done

# Remove o que a instalação anterior trouxe e a fonte não tem mais.
arquivo_origem="$pasta_skills/ORIGEM.json"
nomes_novos=("${nomes_do_matt[@]}")
for pasta_skill in "${skills_da_casa[@]}"; do nomes_novos+=("$(basename "$pasta_skill")"); done
if [[ -f "$arquivo_origem" ]]; then
  while IFS= read -r nome_anterior; do
    if [[ " ${nomes_novos[*]} " != *" $nome_anterior "* && -d "$pasta_skills/$nome_anterior" ]]; then
      fazer rm -rf "${pasta_skills:?}/${nome_anterior:?}"
      relatorio+=("REMOVIDA  skill $nome_anterior (saiu da fonte)")
    fi
  done < <(node -e 'const origem = require(process.argv[1]); for (const nome of [...origem.skillsDoMatt, ...origem.skillsDaCasa]) console.log(nome)' "$arquivo_origem")
fi

copiar_skill() { # $1 pasta de origem, $2 rótulo
  local pasta_origem="${1%/}" nome_skill
  nome_skill="$(basename "$pasta_origem")"
  if [[ -d "$pasta_skills/$nome_skill" ]] && diff -rq "$pasta_origem" "$pasta_skills/$nome_skill" >/dev/null; then return; fi
  fazer rm -rf "${pasta_skills:?}/${nome_skill:?}"
  fazer mkdir -p "$pasta_skills"
  fazer cp -R "$pasta_origem" "$pasta_skills/$nome_skill"
  relatorio+=("SKILL     $nome_skill ($2)")
}
for caminho_skill in "${skills_do_matt[@]}"; do copiar_skill "$origem_matt/$caminho_skill" "Matt"; done
for pasta_skill in "${skills_da_casa[@]}"; do copiar_skill "$pasta_skill" "casa"; done

commit_stacklabs="$(git -C "$origem_stacklabs" rev-parse --short HEAD 2>/dev/null || echo desconhecido)"
commit_matt="$(git -C "$origem_matt" rev-parse --short HEAD 2>/dev/null || echo desconhecido)"
versao_matt="$(node -e 'console.log(require(process.argv[1]).version)' "$origem_matt/.claude-plugin/plugin.json")"
if [[ $simular -eq 0 ]]; then
  nomes_da_casa=()
  for pasta_skill in "${skills_da_casa[@]}"; do nomes_da_casa+=("$(basename "$pasta_skill")"); done
  node -e '
    const [arquivo, commitStacklabs, commitMatt, versaoMatt, nomesMatt, nomesCasa] = process.argv.slice(1);
    require("fs").writeFileSync(arquivo, JSON.stringify({
      instaladoEm: new Date().toISOString(),
      stacklabs: { repositorio: "stacklabs1/Stack-Labs", commit: commitStacklabs },
      matt: { repositorio: "mattpocock/skills", commit: commitMatt, versao: versaoMatt },
      skillsDoMatt: nomesMatt.split(" ").filter(Boolean),
      skillsDaCasa: nomesCasa.split(" ").filter(Boolean),
    }, null, 2) + "\n");
  ' "$arquivo_origem" "$commit_stacklabs" "$commit_matt" "$versao_matt" "${nomes_do_matt[*]}" "${nomes_da_casa[*]}"
fi

# ---------- 3. pre-push e labels ----------
if [[ "$(git -C "$destino" config --get core.hooksPath || true)" != ".claude/githooks" ]]; then
  if [[ $ja_tem_watchdogs -eq 0 || $atualizar_watchdogs -eq 1 ]]; then
    fazer git -C "$destino" config core.hooksPath .claude/githooks
    relatorio+=("PRE-PUSH  ativado (core.hooksPath)")
  else
    relatorio+=("PRE-PUSH  inativo: um humano roda git config core.hooksPath .claude/githooks")
  fi
fi

if [[ $criar_labels -eq 1 ]]; then
  for label in wayfinder:map wayfinder:research wayfinder:prototype wayfinder:grilling wayfinder:task \
    mapa-inicial:prototipo mapa-inicial:arquitetura mapa-inicial:fases mapa-inicial:infra \
    needs-triage needs-info ready-for-agent ready-for-human wontfix altera-watchdogs; do
    if [[ $simular -eq 1 ]]; then echo "  [simular] gh label create $label --force"; else (cd "$destino" && gh label create "$label" --force >/dev/null); fi
  done
  relatorio+=("LABELS    as 15 do método conferidas no GitHub")
fi

# ---------- relatório ----------
echo
echo "Método StackLabs em $destino"
echo "Fontes: Stack-Labs $commit_stacklabs · mattpocock/skills $commit_matt (v$versao_matt)"
[[ $simular -eq 1 ]] && echo "(simulação: nada foi escrito)"
if [[ ${#relatorio[@]} -eq 0 ]]; then
  echo "Nada a fazer: o projeto já está em dia."
else
  printf '%s\n' "${relatorio[@]}"
fi
