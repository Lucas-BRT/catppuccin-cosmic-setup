# shellcheck shell=bash
# =============================================================================
# Catppuccin Mocha bash prompt
# -----------------------------------------------------------------------------
# Sourced from ~/.bashrc through this block:
#   # >>> catppuccin-mocha >>>
#   [ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
#   # <<< catppuccin-mocha <<<
#
# Uses truecolor (24-bit, "\e[38;2;R;G;Bm"), which COSMIC Terminal supports.
# Official palette: https://catppuccin.com/palette
#
# Result:  user@host ~/path main ❯
#          (blue)(overlay0)(teal) (yellow) (mauve) (green)
# =============================================================================

# Interactive shells only.
[[ $- == *i* ]] || return 0

# Mocha colors used by the prompt (R;G;B).
__ctp_blue='137;180;250'     # #89b4fa  user
__ctp_overlay0='108;112;134' # #6c7086  "@"
__ctp_teal='148;226;213'     # #94e2d5  host
__ctp_yellow='249;226;175'   # #f9e2af  working directory
__ctp_mauve='203;166;247'    # #cba6f7  git branch (theme accent)
__ctp_green='166;227;161'    # #a6e3a1  prompt symbol

# Color escape for use inside PS1. \[ \] tells readline these bytes take no
# space on screen; without it, line wrapping gets garbled.
__ctp() { printf '\\[\\e[38;2;%sm\\]' "$1"; }

# Current git branch, or nothing outside a repository.
# \001 and \002 do the job of \[ \] here, because this text comes from a
# function run at every prompt rather than from the literal PS1.
__ctp_git() {
    local b
    b=$(git symbolic-ref --short HEAD 2>/dev/null) || return 0
    printf ' \001\e[38;2;%sm\002%s' "$__ctp_mauve" "$b"
}

PS1="$(__ctp "$__ctp_blue")\u$(__ctp "$__ctp_overlay0")@$(__ctp "$__ctp_teal")\h $(__ctp "$__ctp_yellow")\w\$(__ctp_git) $(__ctp "$__ctp_green")❯\[\e[0m\] "

# `ls` colors matched to the palette:
#   di = directories (blue), ln = symlinks (teal), ex = executables (green),
#   archives (peach #fab387).
export LS_COLORS="di=38;2;137;180;250:ln=38;2;148;226;213:ex=38;2;166;227;161:*.tar=38;2;250;179;135:*.zip=38;2;250;179;135:*.gz=38;2;250;179;135"
