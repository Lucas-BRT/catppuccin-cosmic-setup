#!/usr/bin/env bash
# =============================================================================
# 10 · Tema do desktop COSMIC
#
# Gera o tema no formato do COSMIC 1.10 (ThemeBuilder v2) com
# themes/generate.py e aplica com `cosmic-settings appearance import`, o
# mesmo código do botão Importar em Aparência, sem precisar abrir a janela.
#
# Frosted Glass (transparência com desfoque), pela variável FROSTED:
#   FROSTED=auto    (padrão) mantém como está hoje: se já estiver ligado,
#                   repete a intensidade e os lugares onde ele vale
#   FROSTED=off     desliga
#   FROSTED=medium  liga com essa intensidade (extremely_low ... extremely_high_2)
# Onde ele vale, quando ligado, pela variável FROST_ON, separado por vírgula:
#   windows,system_interface,panel,applets,maximized_apps
#   (padrão: tudo menos maximized_apps)
# Opacidade: ALPHA_OFFSET=0.1 deixa mais opaco, -0.1 mais transparente.
#
# Também salva em ~/.local/share/catppuccin/themes/cosmic-settings/ todos
# os sabores e acentos, sólidos e com Frosted Glass, para importar à mão.
# =============================================================================
source "$(dirname "$0")/lib.sh"

builder="$COSMIC_DIR/com.system76.CosmicTheme.Dark.Builder/v2"
[ "$FLAVOR" = latte ] && builder="$COSMIC_DIR/com.system76.CosmicTheme.Light.Builder/v2"

# Lê o Frosted Glass atual. O COSMIC grava cada opção num arquivo:
# frosted = "Medium", frosted_windows = "true", ...
current_frost() {
    local on=() t
    for t in windows system_interface panel applets maximized_apps; do
        [ "$(cat "$builder/frosted_$t" 2>/dev/null)" = true ] && on+=("$t")
    done
    [ ${#on[@]} -gt 0 ] || return 1
    # "ExtremelyLow2" → "extremely_low_2", o nome que o gerador aceita.
    FROSTED=$(sed -E 's/([a-z])([A-Z0-9])/\1_\2/g' "$builder/frosted" 2>/dev/null | tr 'A-Z' 'a-z')
    FROSTED=${FROSTED:-medium}
    FROST_ON=$(IFS=,; echo "${on[*]}")
}

FROSTED="${FROSTED:-auto}"
if [ "$FROSTED" = auto ]; then
    current_frost || FROSTED=off
fi

args=(--flavor "$FLAVOR" --accent "$ACCENT" --alpha-offset "${ALPHA_OFFSET:-0}")
if [ "$FROSTED" = off ]; then
    label="sólido"
else
    args+=(--frosted "$FROSTED" --frost-on "${FROST_ON:-windows,system_interface,panel,applets}")
    label="Frosted Glass $FROSTED em ${FROST_ON:-windows,system_interface,panel,applets}"
fi

info "Tema COSMIC ($FLAVOR, acento $ACCENT, $label)"
gen="$REPO_DIR/themes/generate.py"
out="$DATA_DIR/themes/cosmic-settings"
mkdir -p "$out"
python3 -I "$gen" --all "$out" 2>/dev/null
theme="$out/catppuccin-$FLAVOR-$ACCENT+custom.ron"
python3 -I "$gen" "${args[@]}" -o "$theme"
ok "Temas gerados em $out"

if [ "${NO_APPLY:-0}" = 1 ]; then
    todo "NO_APPLY=1: para aplicar, rode  cosmic-settings appearance import $theme"
    exit 0
fi
cosmic-settings appearance import "$theme" >/dev/null
ok "Tema aplicado ($(basename "$theme"))"
