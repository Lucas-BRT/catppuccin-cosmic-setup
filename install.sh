#!/usr/bin/env bash
# =============================================================================
# Instala o Catppuccin Mocha no Pop!_OS 24.04 com COSMIC.
#
# Uso:
#   ./install.sh                 # todos os passos (com backup antes)
#   ./install.sh icons wallpaper # só os passos citados
#   ./install.sh --list          # mostra os passos
#
# Variáveis opcionais:
#   FLAVOR=mocha   ACCENT=mauve  # outro acento: ACCENT=lavender ./install.sh
#   NO_GUI=1                     # não abre as Configurações no passo 10
#   NO_BACKUP=1                  # pula o backup
#
# Pode rodar quantas vezes quiser: cada passo sobrescreve o que ele mesmo
# instalou e não duplica nada.
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/scripts"

declare -A STEPS=(
    [cosmic]=10-cosmic-theme.sh
    [terminal]=20-terminal.sh
    [bash]=30-bash-prompt.sh
    [zed]=40-zed.sh
    [icons]=50-icons.sh
    [wallpaper]=60-wallpaper.sh
)
ORDER=(cosmic terminal bash zed icons wallpaper)

if [ "${1:-}" = --list ]; then
    for s in "${ORDER[@]}"; do echo "$s  (scripts/${STEPS[$s]})"; done
    exit 0
fi

if [ $# -eq 0 ]; then selected=("${ORDER[@]}"); else selected=("$@"); fi
for s in "${selected[@]}"; do
    [ -n "${STEPS[$s]:-}" ] || { echo "Passo desconhecido: $s (veja --list)" >&2; exit 1; }
done

[ "${NO_BACKUP:-0}" = 1 ] || ./00-backup.sh
for s in "${ORDER[@]}"; do
    [[ " ${selected[*]} " == *" $s "* ]] && "./${STEPS[$s]}"
done

echo
echo "Pronto. Se ícones ou fundo não mudarem na hora, saia e entre na sessão."
