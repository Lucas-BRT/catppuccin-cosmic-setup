# Catppuccin Mocha for COSMIC

Scripts and configs that theme Pop!_OS 24.04 (COSMIC desktop) with
[Catppuccin](https://catppuccin.com) **Mocha** and the **Mauve** accent.

The desktop theme is generated here in the **COSMIC 1.10** format, with
**Frosted Glass** support (see [The COSMIC 1.10 format](#the-cosmic-110-format)).
Everything else comes from the official repositories, pinned to exact
commits, so running the installer today or a year from now gives the same
result.

## Quick start

```bash
git clone <this-repo> ~/Projects/catppuccin-mocha-cosmic
cd ~/Projects/catppuccin-mocha-cosmic
./install.sh
```

The installer backs everything up before changing anything, and it can be
run again without duplicating anything. Afterwards, the only manual step
is importing the terminal color scheme (see [COSMIC Terminal](#cosmic-terminal)).

To run only some steps or change options:

```bash
./install.sh --list            # list the steps
./install.sh icons wallpaper   # run only these
ACCENT=lavender ./install.sh   # another accent (blue, pink, peach, teal...)
FROSTED=high ./install.sh cosmic             # stronger Frosted Glass
FROSTED=off ./install.sh cosmic              # no transparency
FROST_ON=panel,applets ./install.sh cosmic   # only on the panel and applets
```

| Variable       | Default | Effect                                         |
| -------------- | ------- | ---------------------------------------------- |
| `FLAVOR`       | `mocha` | Catppuccin flavor (`latte`, `frappe`, `macchiato`, `mocha`) |
| `ACCENT`       | `mauve` | Accent color for the desktop and the folders   |
| `FROSTED`      | `auto`  | `auto` keeps the current Frosted Glass setting, `off` disables it, or a strength: `extremely_low`, `extremely_low_2`, `very_low`, `very_low_2`, `low`, `low_2`, `medium`, `medium_2`, `high`, `high_2`, `very_high`, `very_high_2`, `extremely_high`, `extremely_high_2` |
| `FROST_ON`     | all but `maximized_apps` | Where Frosted Glass applies: `windows`, `system_interface`, `panel`, `applets`, `maximized_apps` |
| `ALPHA_OFFSET` | `0`     | Added to the Frosted Glass opacity (`0.1` = more opaque, `-0.1` = more transparent) |
| `NO_APPLY`     | `0`     | `1` only generates the theme, without applying it |
| `NO_BACKUP`    | `0`     | `1` skips the backup                           |

The bash prompt and the wallpaper use fixed Mocha colors and do not change
with `FLAVOR` or `ACCENT`.

## What each step does

| Step        | Script                        | Changes                                                    |
| ----------- | ----------------------------- | ---------------------------------------------------------- |
| *(backup)*  | `scripts/00-backup.sh`        | Copies `~/.config/cosmic`, `~/.bashrc` and Zed's `settings.json`, and exports the current theme as `previous-theme.ron` |
| `cosmic`    | `scripts/10-cosmic-theme.sh`  | Generates the theme and applies it with `cosmic-settings appearance import` |
| `terminal`  | `scripts/20-terminal.sh`      | Downloads the terminal scheme and checks whether it was imported |
| `bash`      | `scripts/30-bash-prompt.sh`   | Installs `~/.config/catppuccin/catppuccin-prompt.bash` and a block in `~/.bashrc` |
| `zed`       | `scripts/40-zed.sh`           | Only checks the extensions and `settings.json`, changes nothing |
| `icons`     | `scripts/50-icons.sh`         | Papirus-Dark in `~/.local/share/icons`, `cat-mocha-mauve` folders |
| `wallpaper` | `scripts/60-wallpaper.sh`     | Sets `~/Pictures/Wallpapers/catppuccin-mocha.png` as the background |

### COSMIC desktop

- **Generator:** [`themes/generate.py`](themes/generate.py). Colors, and
  which Catppuccin color goes into each field, follow the official template
  from [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop).
  All 42 colors match the official Mocha Mauve `.ron` field by field.
- **Ready-made files:** [`themes/cosmic-settings/`](themes/cosmic-settings)
  has Mocha in all 14 accents, solid (`+round.ron`) and with medium Frosted
  Glass (`+round+frosted.ron`). The installer also generates Latte, Frappé
  and Macchiato in `~/.local/share/catppuccin/themes/cosmic-settings/`.
- **Applying by hand**, without the installer:

  ```bash
  cosmic-settings appearance import themes/cosmic-settings/catppuccin-mocha-mauve+round+frosted.ron
  ```

  The **Import** button under Settings → Desktop → Appearance works too.
- **Custom theme:**

  ```bash
  themes/generate.py --accent lavender --frosted high --frost-on windows,panel \
      --alpha-offset 0.05 --roundness slightlyround -o my-theme.ron
  themes/generate.py --help    # every option
  ```

- **Where COSMIC stores it:** `~/.config/cosmic/com.system76.CosmicTheme.Dark.Builder/v2/`
  (what you picked) and `com.system76.CosmicTheme.Dark/v2/` (the full theme
  computed from it: button, container and border colors).

### COSMIC Terminal

- **Source:** [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop),
  `themes/cosmic-term/catppuccin-mocha.ron`. The format did not change in 1.10.
- Text `#cdd6f4`, Rosewater cursor `#f5e0dc` and the 16 Mocha ANSI colors.
- It sets no background of its own: the terminal uses the desktop theme's
  background, so it turns translucent with Frosted Glass too.
- **Import (manual, the terminal has no command for it):** COSMIC Terminal →
  View → Color schemes… → **Import** →
  `~/.local/share/catppuccin/themes/cosmic-term/catppuccin-mocha.ron`

### Bash prompt

- **File:** [`config/bash/catppuccin-prompt.bash`](config/bash/catppuccin-prompt.bash),
  with every color commented.
- **Layout:** `user@host ~/path branch ❯`, in blue, overlay0, teal,
  yellow, mauve and green.
- Also sets `LS_COLORS` for directories, symlinks, executables and archives.
- Only this block is added to `~/.bashrc`:

  ```bash
  # >>> catppuccin-mocha >>>
  [ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
  # <<< catppuccin-mocha <<<
  ```

### Zed

- **Extensions:** `catppuccin` (theme) and `catppuccin-icons` (icons),
  installed from Zed itself (`Ctrl+Shift+X`).
- **Snippet for `~/.config/zed/settings.json`:**

  ```jsonc
  "icon_theme": "Catppuccin Mocha",
  "theme": { "mode": "system", "light": "One Light", "dark": "Catppuccin Mocha" }
  ```

- The script never rewrites the file, since it holds comments and your
  other preferences. It only reports what is missing.

### Icons

- **Base:** [Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme),
  copied to `~/.local/share/icons` (no `sudo`).
- **Folders:** [catppuccin/papirus-folders](https://github.com/catppuccin/papirus-folders),
  recolored with the
  [papirus-folders](https://github.com/PapirusDevelopmentTeam/papirus-folders)
  script (`-t Papirus-Dark -C cat-mocha-mauve`).
- **Config:** `~/.config/cosmic/com.system76.CosmicTk/v1/icon_theme = "Papirus-Dark"`.

### Wallpaper

- **Generator:** [`wallpaper/generate.py`](wallpaper/generate.py), Python
  standard library only. Draws a Crust→Base gradient with Mauve, Blue and
  Pink glows, and always produces the same image.
- **Pre-rendered image:** `wallpaper/catppuccin-mocha.png` (2560×1440).
- **Config:** [`config/cosmic/background-all.ron`](config/cosmic/background-all.ron)
  goes to `~/.config/cosmic/com.system76.CosmicBackground/v1/all`.

To render at another resolution:

```bash
python3 wallpaper/generate.py ~/Pictures/Wallpapers/catppuccin-mocha.png 3840 2160
```

## The COSMIC 1.10 format

COSMIC 1.10 moved the theme to version 2 of its format (`ThemeBuilder` v2,
stored under `.../v2/` instead of `.../v1/`). The official Catppuccin `.ron`
files still use the old format. They import without errors, but:

| Field                     | Old format (v1)     | COSMIC 1.10 (v2)                                |
| ------------------------- | ------------------- | ----------------------------------------------- |
| Transparency              | `is_frosted: bool`  | removed, silently ignored                       |
| Blur strength             | —                   | `frosted: Medium` (14 levels, `ExtremelyLow` to `ExtremelyHigh2`) |
| Where blur applies        | —                   | `frosted_windows`, `frosted_system_interface`, `frosted_panel`, `frosted_applets`, `frosted_maximized_apps` |
| Opacity per strength      | —                   | `alpha_map: (extremely_low: 0.9, ..., extremely_high_2: 0.6)` |
| Palette                   | had `blue`, `red`, `green`, `yellow`, `gray_3` | those fields are gone (ignored) |
| Colors                    | `(red: 0.79, green: 0.65, blue: 0.96)` | `"#CBA6F7FF"` (the old form is still accepted) |

**Consequence:** since the `frosted_*` fields are missing from the old
file, they default to `false`. In other words, **importing the official
`.ron` turns Frosted Glass off**. This repo's generator writes every new
field, and with `FROSTED=auto` the installer also carries over the Frosted
Glass setting you already had.

How Frosted Glass works: the window background (Base `#1e1e2e`) gets the
`alpha_map` opacity for the chosen strength (`medium` = 76%), and the
compositor blurs whatever is behind it. The opacity slider under Appearance
adds the same value to the whole `alpha_map`, just like `--alpha-offset`.

All 112 generated themes were validated with libcosmic's own parser at the
commit used by cosmic-settings 1.10 (`5a8bd94`), and Mocha Mauve was
imported with `cosmic-settings appearance import`.

## Uninstalling

```bash
./uninstall.sh                 # restore the last backup
./uninstall.sh ~/.config/catppuccin-backup-YYYY-MM-DD-HHMMSS   # a specific backup
PURGE=1 ./uninstall.sh         # also delete Papirus, themes and the wallpaper
cosmic-settings appearance import ~/.config/catppuccin-backup-.../previous-theme.ron   # theme only
```

## Updating upstream versions

The pinned commits live in [`scripts/lib.sh`](scripts/lib.sh)
(`CTP_COSMIC_REV`, `PAPIRUS_REV`...). To update, change the hash and run
the step again. Downloads are cached in `~/.cache/catppuccin-mocha-cosmic/`.

## Layout

```
install.sh / uninstall.sh      entry points
scripts/lib.sh                 paths, pinned versions and shared functions
scripts/NN-*.sh                one step per file, each runs on its own
config/bash/                   bash prompt
config/cosmic/                 values written to cosmic-config
themes/generate.py             COSMIC 1.10 theme generator (with Frosted Glass)
themes/cosmic-settings/        ready-made Mocha themes, solid and frosted
wallpaper/                     wallpaper generator and image
```

## Credits

- Palette and color mapping: [Catppuccin](https://github.com/catppuccin/catppuccin)
  and [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop) (MIT).
- Icons: [Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme) (GPL-3.0)
  and [catppuccin/papirus-folders](https://github.com/catppuccin/papirus-folders) (MIT),
  downloaded at install time, not bundled here.

## License

[MIT](LICENSE)
