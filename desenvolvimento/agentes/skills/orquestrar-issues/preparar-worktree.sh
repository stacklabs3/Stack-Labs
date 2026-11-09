#!/bin/sh
# Prepara a worktree de uma issue para o primeiro type-check ou teste. A worktree nasce só com o que o
# git versiona: sem dependências instaladas, sem os .env e sem o código gerado.
#
# uso (de dentro da worktree):  sh .claude/skills/orquestrar-issues/preparar-worktree.sh [<pasta> ...]
#
# Projeto com `.claude/preparar-worktree.sh` (do projeto, fora do que o /setup-stacklabs instala): este
# script só repassa os argumentos para ele, que sabe os passos do projeto (gerar contrato, ordem dos
# pacotes, l10n). Sem ele, o preparo genérico, na raiz e em cada pasta pedida (sem pastas: a raiz e as
# pastas de primeiro nível com manifesto):
#   - troca por instalação de verdade o node_modules que for symlink para o checkout principal: pelo
#     symlink, a worktree lê os pacotes locais do checkout principal, e a mudança feita na worktree some
#     do type-check e dos testes;
#   - instala pelo lockfile (pnpm, yarn, npm, bun), `dart pub get` ou `flutter pub get`;
#   - copia do checkout principal o `.env` que falta (molde: o `.env.example` da pasta).
# Instalação que compila (Flutter, pacotes nativos) é comando pesado: rode este script por
# `sh .claude/watchdogs/rodar pesado "sh .claude/skills/orquestrar-issues/preparar-worktree.sh"`.
set -eu

raiz_da_worktree=$(git rev-parse --show-toplevel)
raiz_do_checkout_principal=$(cd "$(git rev-parse --git-common-dir)/.." && pwd)

if [ "$raiz_da_worktree" = "$raiz_do_checkout_principal" ]; then
  echo "preparar-worktree: este é o checkout principal, nada a preparar."
  exit 0
fi

if [ -f "$raiz_da_worktree/.claude/preparar-worktree.sh" ]; then
  exec sh "$raiz_da_worktree/.claude/preparar-worktree.sh" "$@"
fi

tem_manifesto() {
  for manifesto in package.json pubspec.yaml; do
    [ -f "$1/$manifesto" ] && return 0
  done
  return 1
}

preparar_pasta() {
  pasta_relativa=$1
  pasta="$raiz_da_worktree/$pasta_relativa"
  [ -d "$pasta" ] || { echo "preparar-worktree: a pasta $pasta_relativa não existe." >&2; exit 1; }
  if [ -L "$pasta/node_modules" ]; then
    echo "preparar-worktree: $pasta_relativa/node_modules é symlink; trocando por instalação."
    rm "$pasta/node_modules"
  fi
  instalar=""
  if [ -f "$pasta/pnpm-lock.yaml" ]; then instalar="pnpm install --frozen-lockfile --prefer-offline"
  elif [ -f "$pasta/yarn.lock" ]; then instalar="yarn install --frozen-lockfile"
  elif [ -f "$pasta/package-lock.json" ]; then instalar="npm ci"
  elif [ -f "$pasta/bun.lockb" ] || [ -f "$pasta/bun.lock" ]; then instalar="bun install --frozen-lockfile"
  elif [ -f "$pasta/pubspec.yaml" ]; then
    if grep -q "sdk: flutter" "$pasta/pubspec.yaml"; then instalar="flutter pub get"; else instalar="dart pub get"; fi
  fi
  if [ -n "$instalar" ]; then
    echo "preparar-worktree: $instalar em $pasta_relativa"
    if ! (cd "$pasta" && $instalar > /dev/null); then
      echo "preparar-worktree: \"$instalar\" falhou em $pasta_relativa; rode à mão em $pasta para ver o erro." >&2
      exit 1
    fi
  fi
  env_do_principal="$raiz_do_checkout_principal/$pasta_relativa/.env"
  if [ ! -f "$pasta/.env" ] && [ -f "$env_do_principal" ]; then
    cp "$env_do_principal" "$pasta/.env"
    echo "preparar-worktree: $pasta_relativa/.env copiado do checkout principal."
  elif [ ! -f "$pasta/.env" ] && [ -f "$pasta/.env.example" ]; then
    echo "preparar-worktree: falta $pasta_relativa/.env, e o checkout principal também não tem (molde: .env.example)." >&2
  fi
}

if [ "$#" -gt 0 ]; then
  for pasta_pedida in "$@"; do preparar_pasta "$pasta_pedida"; done
else
  preparar_pasta "."
  for pasta in "$raiz_da_worktree"/*/; do
    nome_da_pasta=$(basename "$pasta")
    if tem_manifesto "$pasta" || [ -L "$pasta/node_modules" ]; then preparar_pasta "$nome_da_pasta"; fi
  done
fi
echo "preparar-worktree: pronto."
