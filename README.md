# catppuccin-cosmic-setup

[![CI](https://img.shields.io/github/actions/workflow/status/Lucas-BRT/catppuccin-cosmic-setup/ci.yml?branch=main&style=for-the-badge&label=CI&labelColor=363a4f)](https://github.com/Lucas-BRT/catppuccin-cosmic-setup/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Lucas-BRT/catppuccin-cosmic-setup?style=for-the-badge&label=ctp-wallpaper&labelColor=363a4f&color=c6a0f6)](https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases)
[![License](https://img.shields.io/github/license/Lucas-BRT/catppuccin-cosmic-setup?style=for-the-badge&labelColor=363a4f&color=a6da95)](LICENSE)
[![COSMIC](https://img.shields.io/badge/COSMIC-1.10-8aadf4?style=for-the-badge&labelColor=363a4f)](https://system76.com/cosmic)
[![Rust](https://img.shields.io/badge/Rust-f5a97f?style=for-the-badge&logo=rust&logoColor=24273a)](wallpaper)
[![Bash](https://img.shields.io/badge/Bash-a6da95?style=for-the-badge&logo=gnubash&logoColor=24273a)](scripts)
[![Python](https://img.shields.io/badge/Python-eed49f?style=for-the-badge&logo=python&logoColor=24273a)](themes)

[Catppuccin](https://catppuccin.com) for Pop!_OS 24.04 with the COSMIC
desktop, in one command: desktop theme, terminal, bash prompt, icons and
wallpapers, in any of the 4 flavors and 14 accents.

![Wallpaper styles in every flavor](wallpaper/previews.png)

## Install

```bash
git clone https://github.com/Lucas-BRT/catppuccin-cosmic-setup
cd catppuccin-cosmic-setup
./install.sh                            # Mocha with the Mauve accent
FLAVOR=latte ACCENT=blue ./install.sh   # or any other combination
```

Your settings are backed up first, and `./uninstall.sh` restores them.
The only manual step left is importing the [terminal colors](docs/USAGE.md#terminal).

## What it sets up

- **Desktop theme** in the COSMIC 1.10 format, keeping Frosted Glass on
- **COSMIC Terminal** color scheme
- **Bash prompt** with git branch, in the flavor's colors
- **Papirus icons** with Catppuccin folders
- **Wallpapers** in 10 styles, also available in COSMIC's wallpaper picker
- **Zed** check, telling you what to set

## Docs

- [Usage](docs/USAGE.md): options, steps, wallpapers, uninstalling
- [Development](docs/DEVELOPMENT.md): layout, CI, releases, COSMIC 1.10 notes

## Credits

This is a community project, not an official Catppuccin port; see
[catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop)
for that. Palette by [Catppuccin](https://github.com/catppuccin/catppuccin),
icons by [Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme)
and [catppuccin/papirus-folders](https://github.com/catppuccin/papirus-folders),
downloaded at install time.

## License

[MIT](LICENSE)
