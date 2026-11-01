# Usage

← [Back to README](../README.md)

## Running it

The [install command](../README.md#install) downloads the latest `ctp-cosmic`
into `~/.local/bin` and runs `ctp-cosmic apply`. From then on `ctp-cosmic` is
all you need:

```bash
ctp-cosmic                    # interactive setup (in a terminal, no options)
ctp-cosmic apply --last       # repeat the last successful choices
ctp-cosmic update             # install the latest release (--check, --apply)
ctp-cosmic uninstall          # undo the last apply
```

The interactive setup starts from your last choices: pick the flavor, accent,
Frosted Glass, wallpaper, terminal font and steps with a live preview, press
**Enter** to apply, and follow each step as it runs. Keys: **↑↓** move,
**←→** or **space** change, **enter** apply, **q** quit.

Passing any option or step skips the interface and runs directly, which is
also what happens without a terminal (`NO_TUI=1` forces it). Every option is
a flag or an environment variable (flags win), and steps can be named:

```bash
ctp-cosmic apply --list                       # list the steps
ctp-cosmic apply --flavor frappe bash         # one step with another flavor
FLAVOR=latte ACCENT=blue ctp-cosmic apply
curl -fsSL https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/latest/download/install.sh | bash -s -- bash wallpaper
```

### Profiles

```bash
ctp-cosmic apply --flavor latte --accent blue --save day     # save after a successful run
ctp-cosmic apply --profile day                               # use it (flags still win)
ctp-cosmic profile list                                      # also: show, delete
ctp-cosmic profile export day ~/dotfiles/day.ron             # and: profile import FILE [NAME]
```

Profiles live in `~/.config/catppuccin-cosmic-setup/profiles/`.

### Tab completion

```bash
ctp-cosmic completions install          # for your $SHELL (bash, zsh or fish)
```

It writes the script where the shell loads user completions from. Do not
redirect `completions bash` into `~/.bashrc`: the shell empties the file
before the program runs.

Commands, flags and their values (flavors, accents, styles, steps) complete.

### From a git clone

```bash
cargo run                       # interactive setup (same as ctp-cosmic)
cargo run -- apply --list
cargo run -- theme --help
cargo run -- apply --dry-run --flavor latte
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

`cargo run` runs the code in the clone, which is separate from the `ctp-cosmic`
in `~/.local/bin`; `update` refuses to run from a cargo build. Note that
`cargo run -- apply` changes your desktop like the installed one does: use
`--dry-run` or a throwaway `HOME` to try things.

## Options

| Variable | Flag | Default | |
| --- | --- | --- | --- |
| `FLAVOR` | `--flavor` | `mocha` | `latte`, `frappe`, `macchiato`, `mocha` |
| `ACCENT` | `--accent` | `mauve` | `rosewater`, `flamingo`, `pink`, `mauve`, `red`, `maroon`, `peach`, `yellow`, `green`, `teal`, `sky`, `sapphire`, `blue`, `lavender` |
| `LIGHT_FLAVOR` | `--light-flavor` | | with a dark `FLAVOR`, `latte` for COSMIC's light mode: both themes are installed and COSMIC's automatic switching (Appearance → Auto-switch) is turned on, so it follows sunrise and sunset |
| `FROSTED` | `--frosted` | `auto` | `auto` keeps your current Frosted Glass setting, `off` disables it, or a strength from `extremely_low` to `extremely_high_2` |
| `FROST_ON` | `--frost-on` | all but `maximized_apps` | where Frosted Glass applies: `windows`, `system_interface`, `panel`, `applets`, `maximized_apps` |
| `ALPHA_OFFSET` | `--alpha-offset` | `0` | Frosted Glass opacity tweak (`0.1` more opaque, `-0.1` more transparent) |
| `ROUNDNESS` | `--roundness` | `round` | corner style: `round`, `slightlyround`, `square`. Also a row in the interactive setup |
| `GAPS` | `--gaps` | `0,8` | gaps between windows, `OUTER,INNER` |
| `ACTIVE_HINT` | `--active-hint` | `3` | width of the active window outline |
| `WINDOW_HINT` | `--window-hint` | the accent | color of that outline: any accent name |
| `WALLPAPER_STYLE` | `--wallpaper-style` | `glow` | a wallpaper style, or `all` (see [Wallpapers](#wallpapers)) |
| `WALLPAPER_ROTATE` | `--wallpaper-rotate` | | slideshow interval for `WALLPAPER_STYLE=all`: `5m` `10m` `15m` `30m` `1h` `2h` |
| `WALLPAPER_IMAGE` | `--wallpaper-image` | | your own PNG, pulled toward the flavor's colors and set as the background instead of a style (not with style `all`) |
| `NO_APPLY` | `--no-apply` | `0` | `1` generates the desktop theme without applying it |
| `TERMINAL_FONT` | `--font` | | COSMIC Terminal font family, e.g. `JetBrainsMono Nerd Font`; list yours with `ctp-cosmic terminal font list`. The default font lacks many icons, so a [Nerd Font](https://www.nerdfonts.com) fits well (install it first). Also a row in the interactive setup |
| `INSTALL_FONT` | `--install-font` | | downloads a Nerd Font (`JetBrainsMono`, `FiraCode` or `Hack`; pinned release, checked hash, installed for you only in `~/.local/share/fonts`) and uses it as the terminal font. Also `ctp-cosmic terminal font install NAME --set` |
| `TERMINAL_FONT_SIZE` | `--font-size` | | size for `TERMINAL_FONT` (4 to 96) |
| `NO_BACKUP` | `--no-backup` | `0` | `1` skips the backup |
| `NO_TUI` | | `0` | `1` never opens the interactive setup |

Invalid values stop `apply` before anything is changed. Latte is the
only light flavor: it switches COSMIC to light mode (and, with the optional
icons step, uses the light Papirus icons).

## Steps

| Step | What it does |
| --- | --- |
| *(backup)* | copies `~/.config/cosmic`, `~/.bashrc` and Zed's `settings.json` to `~/.config/catppuccin-backup-<date>/`; when the steps end it records which of those files changed |
| `cosmic` | generates the desktop theme and applies it with `cosmic-settings appearance import`, keeping your interface density |
| `terminal` | adds the Catppuccin color scheme to COSMIC Terminal and selects it |
| `bash` | installs the prompt and adds one marked block to `~/.bashrc` |
| `zed` | applies Catppuccin theme and icons to Zed (`~/.config/zed/settings.json`) |
| `icons` | **optional**, see [Icons](#icons-optional) |
| `wallpaper` | renders a wallpaper and sets it as the background |
| `gtk` | **optional**, see [GTK and Flatpak apps](#gtk-and-flatpak-apps-optional) |

### Terminal

The scheme is written to COSMIC Terminal's own config, next to any schemes
you added yourself; open **View → Color schemes…** to pick another. The scheme has no
background of its own, so the terminal follows the desktop theme (and
Frosted Glass).

### GTK and Flatpak apps (optional)

COSMIC itself writes style sheets for GTK 3 and 4 apps from the desktop theme
(`~/.config/gtk-4.0/cosmic/dark.css` and `light.css`, linked from both
`gtk.css` files), so once the `cosmic` step has applied the Catppuccin theme,
regular GTK apps follow its colors and accent. The `gtk` step checks that, and
makes sure sandboxed Flatpak apps can read those sheets
(`flatpak override --user --filesystem=xdg-config/gtk-3.0:ro` and the same for
`gtk-4.0`). It does not install anything: for GTK 3 Flatpak apps it prints the
`flatpak install` line for the adw-gtk3 theme if that is missing.

```bash
ctp-cosmic apply gtk
ctp-cosmic uninstall --only gtk     # takes the Flatpak overrides back
```

One thing is COSMIC's, not ours: the sheet uses pure white for text, not
Catppuccin's Text color.

### Icons (optional)

COSMIC's own icons are left alone unless you ask for this step. It installs
Papirus (no `sudo`) with folders in the chosen accent from
[catppuccin/papirus-folders](https://github.com/catppuccin/papirus-folders),
keeps COSMIC's window buttons, and falls back to COSMIC's icons for anything
Papirus lacks. Name the step to run it:

```bash
ctp-cosmic apply icons
curl -fsSL https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/latest/download/install.sh | bash -s -- icons
```

In the interactive setup it starts unchecked. Uninstalling restores your
previous icon theme.

### Zed

The installer configures `~/.config/zed/settings.json` automatically, setting `theme` and `icon_theme` for your chosen flavor while preserving comments and existing preferences.

Make sure the `catppuccin` and `catppuccin-icons` extensions are installed in Zed (`Ctrl+Shift+X`).

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
ctp-cosmic apply --wallpaper-style sunset wallpaper
```

### All styles in COSMIC's wallpaper picker

```bash
ctp-cosmic apply --wallpaper-style all wallpaper
```

Renders every style into `~/Pictures/Wallpapers/catppuccin-<flavor>-<accent>/`
and adds that folder to **Settings → Desktop → Wallpaper**, where you can
pick one or turn on the slideshow. Your current background is left alone
unless `WALLPAPER_ROTATE` is set.

### Rendering by hand

```bash
ctp-cosmic wallpaper out.png --style dots --flavor latte --accent teal --size 3840x2160
ctp-cosmic wallpaper --all out-dir/ --flavor frappe --accent peach
ctp-cosmic wallpaper recolor photo.png out.png --flavor mocha   # your own PNG, in the flavor's colors
ctp-cosmic wallpaper --help
```

## Uninstalling

```bash
ctp-cosmic uninstall                   # undo the last apply that changed something
ctp-cosmic uninstall ~/.config/catppuccin-backup-<date>
ctp-cosmic uninstall --purge           # also delete icons, themes, wallpapers,
                                       # the cache and ctp-cosmic itself
```

Only the files the apply changed go back, and only if they still hold what
it left: anything you or another app changed since is kept and listed
(`--force` restores those too). An apply that changed nothing is not kept as
a backup. Backups made by versions before 0.4 are not supported.

Each backup also holds the previous desktop theme, which can be restored
on its own:
`cosmic-settings appearance import ~/.config/catppuccin-backup-<date>/previous-theme.ron`.
