# Usage

← [Back to README](../README.md)

## Running it

The [install command](../README.md#install) downloads the latest release
into `~/.local/share/catppuccin-cosmic-setup/current/` and runs
`install.sh` from there.

In a terminal, with no options, it opens an interactive setup: pick the
flavor, accent, Frosted Glass, wallpaper and steps with a live preview,
press **Enter** to install, and follow each step as it runs. Keys:
**↑↓** move, **←→** or **space** change, **enter** install, **q** quit.

Passing any option or step skips the interface and runs directly, which is
also what happens without a terminal (`NO_TUI=1` forces it). Options are
environment variables, and steps can be run on their own:

```bash
curl -fsSL https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/latest/download/get.sh | bash -s -- icons wallpaper

cd ~/.local/share/catppuccin-cosmic-setup/current
./install.sh --list              # list the steps
FLAVOR=frappe ./install.sh bash  # rerun one step with another flavor
```

From a git clone, `./install.sh` works the same; `ctp-cosmic` (the Rust tool
that generates themes and wallpapers) is then downloaded from the pinned
release, or built with `cargo`.

## Options

| Variable | Default | |
| --- | --- | --- |
| `FLAVOR` | `mocha` | `latte`, `frappe`, `macchiato`, `mocha` |
| `ACCENT` | `mauve` | `rosewater`, `flamingo`, `pink`, `mauve`, `red`, `maroon`, `peach`, `yellow`, `green`, `teal`, `sky`, `sapphire`, `blue`, `lavender` |
| `FROSTED` | `auto` | `auto` keeps your current Frosted Glass setting, `off` disables it, or a strength from `extremely_low` to `extremely_high_2` |
| `FROST_ON` | all but `maximized_apps` | where Frosted Glass applies: `windows`, `system_interface`, `panel`, `applets`, `maximized_apps` |
| `ALPHA_OFFSET` | `0` | Frosted Glass opacity tweak (`0.1` more opaque, `-0.1` more transparent) |
| `WALLPAPER_STYLE` | `glow` | a wallpaper style, or `all` (see [Wallpapers](#wallpapers)) |
| `WALLPAPER_ROTATE` | | slideshow interval for `WALLPAPER_STYLE=all`: `5m` `10m` `15m` `30m` `1h` `2h` |
| `NO_APPLY` | `0` | `1` generates the desktop theme without applying it |
| `NO_BACKUP` | `0` | `1` skips the backup |
| `BUILD_FROM_SOURCE` | `0` | `1` builds `ctp-cosmic` with `cargo` instead of downloading it |
| `NO_TUI` | `0` | `1` never opens the interactive setup |

Invalid values stop the installer before anything is changed. Latte is the
only light flavor: it switches COSMIC to light mode and uses the light
Papirus icons.

## Steps

| Step | What it does |
| --- | --- |
| *(backup)* | copies `~/.config/cosmic`, `~/.bashrc` and Zed's `settings.json` to `~/.config/catppuccin-backup-<date>/` |
| `cosmic` | generates the desktop theme and applies it with `cosmic-settings appearance import` |
| `terminal` | downloads the COSMIC Terminal scheme (import it by hand, see below) |
| `bash` | installs the prompt and adds one marked block to `~/.bashrc` |
| `zed` | only checks your Zed setup and tells you what is missing |
| `icons` | Papirus in `~/.local/share/icons` with `cat-<flavor>-<accent>` folders, no `sudo` |
| `wallpaper` | renders a wallpaper and sets it as the background |

### Terminal

COSMIC Terminal has no command to import color schemes, so this one is
manual: **View → Color schemes… → Import**, then pick
`~/.local/share/catppuccin/themes/cosmic-term/catppuccin-<flavor>.ron`.
The scheme has no background of its own, so the terminal follows the
desktop theme (and Frosted Glass).

### Zed

Install the `catppuccin` and `catppuccin-icons` extensions from Zed
(`Ctrl+Shift+X`), then in `~/.config/zed/settings.json`:

```jsonc
"icon_theme": "Catppuccin Mocha",
"theme": { "mode": "system", "light": "One Light", "dark": "Catppuccin Mocha" }
```

Use `"light"` for Latte, and note the accent in `"Catppuccin Frappé"`.

### Desktop theme by hand

Ready-made Mocha themes (every accent, solid and frosted) are in
[`themes/cosmic-settings/`](../themes/cosmic-settings):

```bash
cosmic-settings appearance import themes/cosmic-settings/catppuccin-mocha-mauve+round+frosted.ron
```

Or generate your own with `ctp-cosmic`, found in
`~/.local/share/catppuccin-cosmic-setup/current/bin/` after installing:

```bash
ctp-cosmic theme --flavor macchiato --accent teal --frosted high -o my-theme.ron
ctp-cosmic theme --help
```

## Wallpapers

Rows in the [preview](previews.png) follow this order:

| Style | |
| --- | --- |
| `glow` | soft glows over a dark gradient (default) |
| `waves` | layered hills fading into the accent |
| `stripes` | a diagonal ribbon of the accent and its neighbors |
| `dots` | halftone dots growing toward a corner |
| `triangles` | low-poly triangles tinted by the accent |
| `mesh` | a soft mesh gradient |
| `aurora` | curtains of light over a night sky |
| `sunset` | a retro sun over a perspective grid |
| `topography` | contour lines like a topographic map |
| `mountains` | mountain ridges under a moon |

```bash
WALLPAPER_STYLE=sunset ./install.sh wallpaper
```

### All styles in COSMIC's wallpaper picker

```bash
WALLPAPER_STYLE=all ./install.sh wallpaper
```

Renders every style into `~/Pictures/Wallpapers/catppuccin-<flavor>-<accent>/`
and adds that folder to **Settings → Desktop → Wallpaper**, where you can
pick one or turn on the slideshow. Your current background is left alone
unless `WALLPAPER_ROTATE` is set.

### Rendering by hand

```bash
ctp-cosmic wallpaper out.png --style dots --flavor latte --accent teal --size 3840x2160
ctp-cosmic wallpaper --all out-dir/ --flavor frappe --accent peach
ctp-cosmic wallpaper --help
```

## Uninstalling

From `~/.local/share/catppuccin-cosmic-setup/current/` (or your clone):

```bash
./uninstall.sh                         # restore the last backup
./uninstall.sh ~/.config/catppuccin-backup-<date>
PURGE=1 ./uninstall.sh                 # also delete icons, themes, wallpapers and this setup
```

Each backup also holds the previous desktop theme, which can be restored
on its own:
`cosmic-settings appearance import ~/.config/catppuccin-backup-<date>/previous-theme.ron`.
