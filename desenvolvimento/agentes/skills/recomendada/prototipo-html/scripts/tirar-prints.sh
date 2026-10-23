#!/usr/bin/env bash
# Tira os prints da conferência de uma rotina (ou de todas) de um protótipo servido.
#
# Uso: tirar-prints.sh <endereco-do-prototipo> <id-da-rotina|todas> <pasta-de-saida> [idioma] [paralelo]
#   ex.: tirar-prints.sh http://localhost:8765 turno-do-caixa prototipos/appetito-v2/prints
# O idioma padrão é pt-BR; passe outro locale de i18n.jsx quando o projeto for de outra língua.
#
# A lista de combinações vem do próprio protótipo (?listarPrints=<rotina>), montada a
# partir de visoes.jsx. O trabalho é do tirar-prints.mjs (Node 22+): ele abre o Chrome sem
# janela pelo protocolo de depuração, com a área de tela exata de cada combinação, e espera
# a tela montar. O --screenshot do Chrome, usado antes, saía com a janela 87 px mais baixa e
# no mínimo 500 px de largura (moldura cortada e deslocada) e quebrava com "Argument list
# too long" em rotinas grandes.
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 5 ]; then
  echo "Uso: $0 <endereco-do-prototipo> <id-da-rotina|todas> <pasta-de-saida> [idioma] [paralelo]" >&2
  exit 2
fi

if ! command -v node >/dev/null 2>&1; then
  echo "Os prints precisam do Node 22 ou mais novo (WebSocket nativo)." >&2
  exit 1
fi

pasta_do_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec node "$pasta_do_script/tirar-prints.mjs" "$@"
