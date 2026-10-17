# shellcheck shell=bash
# =============================================================================
# Funções e variáveis comuns a todos os passos. Não é executado sozinho:
# cada scripts/NN-*.sh faz `source` deste arquivo.
# =============================================================================
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COSMIC_DIR="$HOME/.config/cosmic"
DATA_DIR="$HOME/.local/share/catppuccin"   # temas .ron baixados
ICONS_DIR="$HOME/.local/share/icons"       # Papirus vai aqui (sem sudo)
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/catppuccin-mocha-cosmic"
WALLPAPER="$HOME/Pictures/Wallpapers/catppuccin-mocha.png"

# Sabor e acento do Catppuccin. Podem ser trocados por variável de ambiente:
#   ACCENT=lavender ./install.sh
FLAVOR="${FLAVOR:-mocha}"
ACCENT="${ACCENT:-mauve}"

# Versões fixas dos repositórios de origem, para a instalação ser
# reproduzível. Para atualizar, troque o hash pelo commit desejado.
CTP_COSMIC_REPO=https://github.com/catppuccin/cosmic-desktop.git
CTP_COSMIC_REV=95e81098042dd2102f0b258f6990f886c5759692
CTP_FOLDERS_REPO=https://github.com/catppuccin/papirus-folders.git
CTP_FOLDERS_REV=f83671d17ea67e335b34f8028a7e6d78bca735d7
PAPIRUS_REPO=https://github.com/PapirusDevelopmentTeam/papirus-icon-theme.git
PAPIRUS_REV=bf539287ef5dc18529424a02cccee76175920a6f
PAPIRUS_FOLDERS_REPO=https://github.com/PapirusDevelopmentTeam/papirus-folders.git
PAPIRUS_FOLDERS_REV=0f838ee5679229e3a3e97e3b333c222c9e9615b4

# Mensagens coloridas com a própria paleta.
info() { printf '\e[38;2;137;180;250m==>\e[0m %s\n' "$*"; }   # blue
ok()   { printf '\e[38;2;166;227;161m ✓\e[0m %s\n' "$*"; }    # green
warn() { printf '\e[38;2;249;226;175m !\e[0m %s\n' "$*"; }    # yellow
todo() { printf '\e[38;2;203;166;247m ➜\e[0m %s\n' "$*"; }    # mauve

# Clona um repositório num commit exato, baixando só esse commit.
# Se já estiver no cache naquele commit, não baixa de novo.
fetch_pinned() {
    local url=$1 rev=$2 dest=$3
    if [ "$(git -C "$dest" rev-parse HEAD 2>/dev/null)" = "$rev" ]; then
        return 0
    fi
    rm -rf "$dest"
    mkdir -p "$dest"
    git -C "$dest" init -q
    git -C "$dest" fetch -q --depth 1 "$url" "$rev"
    git -C "$dest" -c advice.detachedHead=false checkout -q FETCH_HEAD
}

# Escreve uma chave do cosmic-config. Cada chave é um arquivo com um valor RON.
cosmic_set() {
    local component=$1 key=$2 value=$3
    mkdir -p "$COSMIC_DIR/$component/v1"
    printf '%s' "$value" > "$COSMIC_DIR/$component/v1/$key"
}

# Remove comentários // de um arquivo RON do repositório antes de instalar.
strip_ron_comments() { sed -e 's#[[:space:]]*//.*$##' -e '/^[[:space:]]*$/d' "$1"; }
