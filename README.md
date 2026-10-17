# Catppuccin Mocha no COSMIC

Scripts e configs para deixar o Pop!_OS 24.04 (desktop COSMIC) no tema
[Catppuccin](https://catppuccin.com) **Mocha**, com acento **Mauve**.

Todos os temas vêm dos repositórios oficiais, fixados em commits exatos.
Assim, rodar o instalador hoje ou daqui a um ano dá o mesmo resultado.

## Uso rápido

```bash
git clone <este-repo> ~/Projects/catppuccin-mocha-cosmic
cd ~/Projects/catppuccin-mocha-cosmic
./install.sh
```

Antes de mudar qualquer coisa, o instalador faz um backup. Ele pode ser
rodado de novo sem duplicar nada. Depois dele, faltam **duas importações
manuais**, explicadas em [Por que dois passos são manuais](#por-que-dois-passos-são-manuais).

Para rodar só alguns passos:

```bash
./install.sh --list            # lista os passos
./install.sh icons wallpaper   # roda só esses
ACCENT=lavender ./install.sh   # outro acento (blue, pink, peach, teal...)
```

| Variável    | Padrão  | Efeito                                         |
| ----------- | ------- | ---------------------------------------------- |
| `FLAVOR`    | `mocha` | Sabor do Catppuccin (`latte`, `frappe`, `macchiato`, `mocha`) |
| `ACCENT`    | `mauve` | Cor de destaque do desktop e das pastas        |
| `NO_GUI`    | `0`     | `1` não abre as Configurações no passo `cosmic` |
| `NO_BACKUP` | `0`     | `1` pula o backup                              |

O prompt do bash e o papel de parede usam cores fixas da paleta Mocha e não
mudam com `FLAVOR` nem com `ACCENT`.

## O que cada passo faz

| Passo       | Script                        | Muda                                                       |
| ----------- | ----------------------------- | ---------------------------------------------------------- |
| *(backup)*  | `scripts/00-backup.sh`        | Copia `~/.config/cosmic`, `~/.bashrc` e o `settings.json` do Zed |
| `cosmic`    | `scripts/10-cosmic-theme.sh`  | Baixa os temas para `~/.local/share/catppuccin/themes`, liga o modo escuro e abre Aparência |
| `terminal`  | `scripts/20-terminal.sh`      | Confere se o esquema do COSMIC Terminal foi importado      |
| `bash`      | `scripts/30-bash-prompt.sh`   | Instala `~/.config/catppuccin/catppuccin-prompt.bash` e um bloco no `~/.bashrc` |
| `zed`       | `scripts/40-zed.sh`           | Só confere as extensões e o `settings.json`, sem alterar nada |
| `icons`     | `scripts/50-icons.sh`         | Papirus-Dark em `~/.local/share/icons`, pastas `cat-mocha-mauve` |
| `wallpaper` | `scripts/60-wallpaper.sh`     | `~/Pictures/Wallpapers/catppuccin-mocha.png` como fundo    |

### Desktop COSMIC

- **Origem:** [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop),
  arquivo `themes/cosmic-settings/catppuccin-mocha-mauve+round.ron`.
- **Config:** `~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark = true`.
- Depois de importado, o COSMIC grava o tema em
  `~/.config/cosmic/com.system76.CosmicTheme.Dark.Builder/v1/` (o que você
  escolheu) e em `com.system76.CosmicTheme.Dark/v1/` (o tema calculado).

### COSMIC Terminal

- **Origem:** o mesmo repositório, `themes/cosmic-term/catppuccin-mocha.ron`.
- Texto `#cdd6f4`, cursor Rosewater `#f5e0dc` e as 16 cores ANSI do Mocha.

### Prompt do bash

- **Arquivo:** [`config/bash/catppuccin-prompt.bash`](config/bash/catppuccin-prompt.bash),
  com cada cor comentada.
- **Formato:** `usuário@máquina ~/caminho branch ❯`, nas cores
  blue, overlay0, teal, yellow, mauve e green.
- Também define `LS_COLORS` para pastas, links, executáveis e arquivos compactados.
- No `~/.bashrc` entra só este bloco:

  ```bash
  # >>> catppuccin-mocha >>>
  [ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
  # <<< catppuccin-mocha <<<
  ```

### Zed

- **Extensões:** `catppuccin` (tema) e `catppuccin-icons` (ícones), instaladas
  pelo próprio Zed (`Ctrl+Shift+X`).
- **Trecho de `~/.config/zed/settings.json`:**

  ```jsonc
  "icon_theme": "Catppuccin Mocha",
  "theme": { "mode": "system", "light": "One Light", "dark": "Catppuccin Mocha" }
  ```

- O script não reescreve o arquivo, porque ele tem comentários e outras
  preferências suas. Ele só avisa o que falta.

### Ícones

- **Base:** [Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme),
  copiado para `~/.local/share/icons` (sem `sudo`).
- **Pastas:** [catppuccin/papirus-folders](https://github.com/catppuccin/papirus-folders)
  com a cor trocada pelo script
  [papirus-folders](https://github.com/PapirusDevelopmentTeam/papirus-folders)
  (`-t Papirus-Dark -C cat-mocha-mauve`).
- **Config:** `~/.config/cosmic/com.system76.CosmicTk/v1/icon_theme = "Papirus-Dark"`.

### Papel de parede

- **Gerador:** [`wallpaper/generate.py`](wallpaper/generate.py), só com a
  biblioteca padrão do Python. Faz um degradê Crust→Base com brilhos Mauve,
  Blue e Pink e sempre gera a mesma imagem.
- **Imagem pronta:** `wallpaper/catppuccin-mocha.png` (2560×1440).
- **Config:** [`config/cosmic/background-all.ron`](config/cosmic/background-all.ron)
  vai para `~/.config/cosmic/com.system76.CosmicBackground/v1/all`.

Para gerar em outra resolução:

```bash
python3 wallpaper/generate.py ~/Pictures/Wallpapers/catppuccin-mocha.png 3840 2160
```

## Por que dois passos são manuais

O tema do desktop e o esquema do terminal só entram pela interface:

1. **Desktop:** Configurações → Desktop → Aparência → **Importar** →
   `~/.local/share/catppuccin/themes/cosmic-settings/catppuccin-mocha-mauve+round.ron`
2. **Terminal:** COSMIC Terminal → View → Color schemes… → **Import** →
   `~/.local/share/catppuccin/themes/cosmic-term/catppuccin-mocha.ron`

No desktop, o `.ron` guarda só as cores base. Ao importar, o
`cosmic-settings` calcula a partir delas o tema completo (botões,
contêineres, bordas). Copiar o `.ron` direto para a config não faz esse
cálculo. Os scripts mostram os caminhos e, ao rodar de novo, reconhecem que
a importação já foi feita.

## Desfazer

```bash
./uninstall.sh                 # volta ao último backup
./uninstall.sh ~/.config/catppuccin-backup-2026-10-17   # um backup específico
PURGE=1 ./uninstall.sh         # também apaga Papirus, temas e o papel de parede
```

## Atualizar as versões de origem

Os commits fixos ficam em [`scripts/lib.sh`](scripts/lib.sh)
(`CTP_COSMIC_REV`, `PAPIRUS_REV`...). Para atualizar, troque o hash e rode
o passo de novo. Os downloads ficam em `~/.cache/catppuccin-mocha-cosmic/`.

## Estrutura

```
install.sh / uninstall.sh      ponto de entrada
scripts/lib.sh                 caminhos, versões fixas e funções comuns
scripts/NN-*.sh                um passo por arquivo, cada um roda sozinho
config/bash/                   prompt do bash
config/cosmic/                 valores escritos no cosmic-config
wallpaper/                     gerador e imagem do papel de parede
```
