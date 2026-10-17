# shellcheck shell=bash
# =============================================================================
# Prompt do bash no Catppuccin Mocha
# -----------------------------------------------------------------------------
# Carregado pelo ~/.bashrc através do bloco:
#   # >>> catppuccin-mocha >>>
#   [ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
#   # <<< catppuccin-mocha <<<
#
# Usa cores truecolor (24 bits, "\e[38;2;R;G;Bm"). O COSMIC Terminal suporta.
# Paleta oficial: https://catppuccin.com/palette
#
# Resultado:  usuario@maquina ~/caminho main ❯
#             (blue)  (overlay0)(teal) (yellow) (mauve) (green)
# =============================================================================

# Só vale para shells interativos.
[[ $- == *i* ]] || return 0

# Cores Mocha usadas no prompt (R;G;B).
__ctp_blue='137;180;250'     # #89b4fa  usuário
__ctp_overlay0='108;112;134' # #6c7086  "@"
__ctp_teal='148;226;213'     # #94e2d5  máquina
__ctp_yellow='249;226;175'   # #f9e2af  diretório atual
__ctp_mauve='203;166;247'    # #cba6f7  branch do git (acento do tema)
__ctp_green='166;227;161'    # #a6e3a1  símbolo do prompt

# Escape de cor para dentro do PS1. O \[ \] avisa ao readline que esses bytes
# não ocupam espaço na tela; sem isso a quebra de linha fica bagunçada.
__ctp() { printf '\\[\\e[38;2;%sm\\]' "$1"; }

# Branch atual do git, ou nada fora de um repositório.
# Aqui \001 e \002 fazem o papel de \[ \], porque o texto sai de uma função
# executada a cada prompt e não do PS1 literal.
__ctp_git() {
    local b
    b=$(git symbolic-ref --short HEAD 2>/dev/null) || return 0
    printf ' \001\e[38;2;%sm\002%s' "$__ctp_mauve" "$b"
}

PS1="$(__ctp "$__ctp_blue")\u$(__ctp "$__ctp_overlay0")@$(__ctp "$__ctp_teal")\h $(__ctp "$__ctp_yellow")\w\$(__ctp_git) $(__ctp "$__ctp_green")❯\[\e[0m\] "

# Cores do `ls` alinhadas à paleta:
#   di = diretórios (blue), ln = links (teal), ex = executáveis (green),
#   arquivos compactados (peach #fab387).
export LS_COLORS="di=38;2;137;180;250:ln=38;2;148;226;213:ex=38;2;166;227;161:*.tar=38;2;250;179;135:*.zip=38;2;250;179;135:*.gz=38;2;250;179;135"
