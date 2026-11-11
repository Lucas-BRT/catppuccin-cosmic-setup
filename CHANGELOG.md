# Changelog

Format: [Keep a Changelog](https://keepachangelog.com). Releases after v0.7.0
also get notes generated from their commits on the releases page.

## [Unreleased]

### Added
- `ctp-cosmic-gui` is part of the release (x86_64 and aarch64, with its
  launcher entry and icon). `install.sh` installs it next to `ctp-cosmic`
  (`CATPPUCCIN_SETUP_GUI=0` skips it), `ctp-cosmic update` keeps it up to date
  and `ctp-cosmic update --gui` installs it for an existing install.
- CI builds, lints and tests the app.

## [0.9.0] - 2026-11-11

### Added
- `ctp-cosmic-gui`, a libcosmic app (`ctp-cosmic gui`, built from `gui/` with
  `just`): Dark and Light cards that show what COSMIC has now, a live preview
  of the desktop, flavor and accent swatches, and the 50 wallpaper styles
  rendered in your palette, grouped in five categories with a search. Apply
  saves the look as a COSMIC theme named `Catppuccin <Flavor> <Accent> ·
  <Style>` (a card under Settings → Appearance) and sets that mode's
  wallpaper. It is not part of the release binaries yet.
- `ctp-cosmic` is also a library, and the repository a workspace with the app.

### Changed
- Applying a theme keeps your gaps, window hint size, corner style, fonts, icon
  theme, density and Frosted Glass when COSMIC 1.11 (build of 2026-11-10 or
  newer) keeps them in the theme.
- The icons step runs before the desktop theme, so the theme COSMIC saves
  carries the icon theme.
- The tests parse themes with the libcosmic of COSMIC 1.11 and of 1.10.

### Fixed
- Importing a theme reset the interface font, monospace font and icon theme on
  COSMIC 1.11, which moved them from the toolkit settings into the theme.
- The wallpaper step changed nothing on COSMIC 1.11: it shows the wallpaper of
  the current mode (`CosmicBackground.Dark` or `.Light`). Both, and the saved
  theme's, are written now, `doctor` checks them and `uninstall` restores them.

## [0.8.0] - 2026-11-08

### Added
- Theme other apps from a declarative manifest (`ctp-cosmic apply NAME`,
  `ctp-cosmic app`): built-in Firefox support (interface and built-in pages,
  no root) and your own apps in `~/.config/catppuccin-cosmic-setup/apps/`.
  See docs/APPS.md. Apps are also in the interactive setup, and the Firefox
  step warns when a theme add-on is active, since it clashes with these colors.
- `install.sh` accepts `CATPPUCCIN_SETUP_SHA256` to pin the binary's hash and
  checks the release's signed provenance attestation when `gh` is installed
  (`CATPPUCCIN_SETUP_VERIFY=1` makes it mandatory). It refuses to run as root,
  retries downloads and honors `NO_COLOR`.
- `apply` and `uninstall` refuse to run as root and run one at a time.
- Installer tests (bats), dependency advisory checks and Dependabot in CI, a
  weekly check of the pinned Papirus commits, and signed provenance
  attestations plus generated notes for releases.

### Changed
- A Papirus you installed yourself is moved to `Papirus.user-backup` instead of
  being deleted, and `uninstall` puts it back; only themes this tool installed
  are removed.
- The prompt block is written to `~/.bashrc` atomically.
- A release is built only if its tests pass.
- Dependencies: `ron` 0.8 to 0.12, `jsonc-parser` 0.26 to 0.34, `crc32fast`
  1.5.2 to 1.5.4; new `toml` for app manifests.

### Fixed
- Firefox 147 and newer keep their profile in `~/.config/mozilla/firefox`; it
  is now found there (Flatpak and snap too, the newer layout not checked
  against a real install).
- Output is plain text without a terminal or with `NO_COLOR`.
- A `~/.bashrc` that is a symlink (dotfile managers) was not backed up, and
  `uninstall` stopped at "no backup found" before removing the prompt block.

## [0.7.0]
- 36 math wallpaper styles (50 in total).
- Light mode fixes, the profile name in the TUI, faster icons and wallpapers.

## [0.6.0]
- The gtk step: GTK and Flatpak apps.
- `uninstall --only fonts` and icon files; `status --fix` reapplies only what
  drifted; shell completions install; TUI profile picker and downloadable
  Nerd Fonts.

## [0.5.0]
- Named profiles, `doctor`, `status` and `apply --dry-run`.
- A Nerd Font for the terminal, shell completions, recolor your own wallpaper,
  dark and Latte pairing for COSMIC's automatic mode switching.
- Prune old backups; uninstall one step at a time.

## [0.4.0]
- `ctp-cosmic` is the whole installer: `apply`, `update` and `uninstall`.
- Undo only what the installer changed.

## [0.3.1]
- Keep COSMIC's window buttons and the user's interface density.

## [0.3.0]
- Interactive setup (ratatui).

## [0.2.0]
- Replace the remaining Python with ctp-cosmic (Rust).

## [0.1.0]
- One-line install from GitHub Releases; procedural wallpapers.

[Unreleased]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/tag/v0.1.0
