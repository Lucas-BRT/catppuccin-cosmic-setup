# Catppuccin Mocha no COSMIC

Scripts e configs para deixar o Pop!_OS 24.04 (desktop COSMIC) no tema
[Catppuccin](https://catppuccin.com) **Mocha**, com acento **Mauve**.

O tema do desktop é gerado aqui, no formato do **COSMIC 1.10**, com
suporte a **Frosted Glass** (veja [Formato do COSMIC 1.10](#formato-do-cosmic-110)).
O resto vem dos repositórios oficiais, fixados em commits exatos. Assim,
rodar o instalador hoje ou daqui a um ano dá o mesmo resultado.

## Uso rápido

```bash
git clone <este-repo> ~/Projects/catppuccin-mocha-cosmic
cd ~/Projects/catppuccin-mocha-cosmic
./install.sh
```

Antes de mudar qualquer coisa, o instalador faz um backup. Ele pode ser
rodado de novo sem duplicar nada. Depois dele, só falta importar o
esquema do terminal pela interface (veja [COSMIC Terminal](#cosmic-terminal)).

Para rodar só alguns passos:

```bash
./install.sh --list            # lista os passos
./install.sh icons wallpaper   # roda só esses
ACCENT=lavender ./install.sh   # outro acento (blue, pink, peach, teal...)
FROSTED=high ./install.sh cosmic             # Frosted Glass mais forte
FROSTED=off ./install.sh cosmic              # sem transparência
FROST_ON=panel,applets ./install.sh cosmic   # só no painel e nos applets
```

| Variável    | Padrão  | Efeito                                         |
| ----------- | ------- | ---------------------------------------------- |
| `FLAVOR`    | `mocha` | Sabor do Catppuccin (`latte`, `frappe`, `macchiato`, `mocha`) |
| `ACCENT`    | `mauve` | Cor de destaque do desktop e das pastas        |
| `FROSTED`   | `auto`  | `auto` mantém o Frosted Glass como está, `off` desliga, ou uma intensidade: `extremely_low`, `extremely_low_2`, `very_low`, `very_low_2`, `low`, `low_2`, `medium`, `medium_2`, `high`, `high_2`, `very_high`, `very_high_2`, `extremely_high`, `extremely_high_2` |
| `FROST_ON`  | tudo menos `maximized_apps` | Onde o Frosted Glass vale: `windows`, `system_interface`, `panel`, `applets`, `maximized_apps` |
| `ALPHA_OFFSET` | `0`  | Soma à opacidade do Frosted Glass (`0.1` = mais opaco, `-0.1` = mais transparente) |
| `NO_APPLY`  | `0`     | `1` só gera o tema, sem aplicar                |
| `NO_BACKUP` | `0`     | `1` pula o backup                              |

O prompt do bash e o papel de parede usam cores fixas da paleta Mocha e não
mudam com `FLAVOR` nem com `ACCENT`.

## O que cada passo faz

| Passo       | Script                        | Muda                                                       |
| ----------- | ----------------------------- | ---------------------------------------------------------- |
| *(backup)*  | `scripts/00-backup.sh`        | Copia `~/.config/cosmic`, `~/.bashrc` e o `settings.json` do Zed, e exporta o tema atual como `tema-anterior.ron` |
| `cosmic`    | `scripts/10-cosmic-theme.sh`  | Gera o tema e aplica com `cosmic-settings appearance import` |
| `terminal`  | `scripts/20-terminal.sh`      | Baixa o esquema do terminal e confere se ele foi importado |
| `bash`      | `scripts/30-bash-prompt.sh`   | Instala `~/.config/catppuccin/catppuccin-prompt.bash` e um bloco no `~/.bashrc` |
| `zed`       | `scripts/40-zed.sh`           | Só confere as extensões e o `settings.json`, sem alterar nada |
| `icons`     | `scripts/50-icons.sh`         | Papirus-Dark em `~/.local/share/icons`, pastas `cat-mocha-mauve` |
| `wallpaper` | `scripts/60-wallpaper.sh`     | `~/Pictures/Wallpapers/catppuccin-mocha.png` como fundo    |

### Desktop COSMIC

- **Gerador:** [`themes/generate.py`](themes/generate.py). As cores e qual
  cor do Catppuccin vai em cada campo seguem o template oficial de
  [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop).
  Conferi campo a campo: as 42 cores batem com o `.ron` oficial do Mocha Mauve.
- **Prontos no repositório:** [`themes/cosmic-settings/`](themes/cosmic-settings)
  tem o Mocha em todos os 14 acentos, sólido (`+round.ron`) e com Frosted
  Glass médio (`+round+frosted.ron`). O instalador também gera Latte, Frappé
  e Macchiato em `~/.local/share/catppuccin/themes/cosmic-settings/`.
- **Aplicar à mão**, sem o instalador:

  ```bash
  cosmic-settings appearance import themes/cosmic-settings/catppuccin-mocha-mauve+round+frosted.ron
  ```

  Também dá pelo botão **Importar** em Configurações → Desktop → Aparência.
- **Tema sob medida:**

  ```bash
  themes/generate.py --accent lavender --frosted high --frost-on windows,panel \
      --alpha-offset 0.05 --roundness slightlyround -o meu-tema.ron
  themes/generate.py --help    # todas as opções
  ```

- **Onde o COSMIC grava:** `~/.config/cosmic/com.system76.CosmicTheme.Dark.Builder/v2/`
  (o que você escolheu) e `com.system76.CosmicTheme.Dark/v2/` (o tema calculado
  a partir disso: cores de botões, contêineres, bordas).

### COSMIC Terminal

- **Origem:** [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop),
  `themes/cosmic-term/catppuccin-mocha.ron`. O formato não mudou no 1.10.
- Texto `#cdd6f4`, cursor Rosewater `#f5e0dc` e as 16 cores ANSI do Mocha.
- Não define fundo próprio: o terminal usa o fundo do tema do desktop,
  então também fica translúcido com Frosted Glass.
- **Importar (manual, o terminal não tem comando para isso):** COSMIC Terminal →
  View → Color schemes… → **Import** →
  `~/.local/share/catppuccin/themes/cosmic-term/catppuccin-mocha.ron`

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

## Formato do COSMIC 1.10

O COSMIC 1.10 passou o tema para a versão 2 do formato (`ThemeBuilder` v2,
gravado em `.../v2/` em vez de `.../v1/`). Os `.ron` oficiais do Catppuccin
ainda estão no formato antigo. Eles importam sem erro, mas:

| Campo                           | Formato antigo (v1) | COSMIC 1.10 (v2)                                |
| ------------------------------- | ------------------- | ----------------------------------------------- |
| Transparência                   | `is_frosted: bool`  | removido, ignorado em silêncio                  |
| Intensidade do desfoque         | —                   | `frosted: Medium` (14 níveis, de `ExtremelyLow` a `ExtremelyHigh2`) |
| Onde há desfoque                | —                   | `frosted_windows`, `frosted_system_interface`, `frosted_panel`, `frosted_applets`, `frosted_maximized_apps` |
| Opacidade de cada intensidade   | —                   | `alpha_map: (extremely_low: 0.9, ..., extremely_high_2: 0.6)` |
| Paleta                          | tinha `blue`, `red`, `green`, `yellow`, `gray_3` | esses campos saíram (ignorados) |
| Cores                           | `(red: 0.79, green: 0.65, blue: 0.96)` | `"#CBA6F7FF"` (o antigo ainda é aceito) |

**Consequência:** como os campos `frosted_*` faltam no arquivo antigo, eles
ficam `false`. Ou seja, **importar o `.ron` oficial desliga o Frosted Glass**.
O gerador daqui escreve todos os campos novos. Com `FROSTED=auto`, o
instalador ainda repete o Frosted Glass que você já usava.

Como o Frosted Glass funciona: o fundo das janelas (Base `#1e1e2e`) fica
com a opacidade de `alpha_map` na intensidade escolhida (`medium` = 76%),
e o compositor desfoca o que está atrás. O controle de opacidade em
Aparência soma um mesmo valor a todo o `alpha_map`, igual ao `--alpha-offset`.

Para conferir, validei todos os 112 temas gerados com o parser real do
libcosmic, no mesmo commit usado pelo cosmic-settings 1.10 instalado
(`5a8bd94`), e importei o Mocha Mauve com `cosmic-settings appearance import`.

## Desfazer

```bash
./uninstall.sh                 # volta ao último backup
cosmic-settings appearance import ~/.config/catppuccin-backup-.../tema-anterior.ron   # só o tema
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
themes/generate.py             gerador de temas COSMIC 1.10 (com Frosted Glass)
themes/cosmic-settings/        temas Mocha prontos, sólidos e com Frosted Glass
wallpaper/                     gerador e imagem do papel de parede
```
