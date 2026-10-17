# Development

← [Back to README](../README.md)

## Layout

```
install.sh, uninstall.sh   entry points
scripts/lib.sh             paths, pinned upstream versions, shared functions
scripts/NN-*.sh            one step per file, each runs on its own
scripts/dev/               maintainer helpers
lib/catppuccin.py          palette for the shell scripts and theme generator
themes/generate.py         COSMIC 1.10 theme generator
themes/cosmic-settings/    generated Mocha themes (kept in sync by CI)
config/                    bash prompt template, cosmic-config values
wallpaper/                 ctp-wallpaper (Rust), styles.txt, pinned release
```

Colors come from the official palette (1.8.0): `lib/catppuccin.py` for the
scripts, the [`catppuccin`](https://crates.io/crates/catppuccin) crate for
the wallpaper generator. CI checks that both agree.

## CI

[`ci.yml`](../.github/workflows/ci.yml) runs on every push and pull request:
`cargo fmt`, Clippy, the Rust tests, ShellCheck, and checks that the
palettes, the style list in `wallpaper/styles.txt` and the committed themes
are in sync.

## Releasing ctp-wallpaper

The installer downloads a prebuilt `ctp-wallpaper` and runs it only if its
SHA-256 matches [`wallpaper/SHA256SUMS`](../wallpaper/SHA256SUMS). Pushing a
`wallpaper-v*` tag runs [`release.yml`](../.github/workflows/release.yml),
which builds static binaries for x86_64 and aarch64 and publishes them with
a `SHA256SUMS` file.

```bash
# bump version in wallpaper/Cargo.toml and commit, then
git tag wallpaper-v0.2.0 && git push origin wallpaper-v0.2.0
# when the workflow is done
scripts/dev/pin-wallpaper-release.sh wallpaper-v0.2.0
git commit -am "Pin ctp-wallpaper 0.2.0" && git push
```

Users keep the previously pinned release until the pin commit lands.

## Upstream versions

Papirus, papirus-folders and catppuccin/cosmic-desktop are fetched at the
commits pinned in [`scripts/lib.sh`](../scripts/lib.sh). Change a hash and
rerun the step to update. Downloads are cached in
`~/.cache/catppuccin-cosmic-setup/`.

## Notes on COSMIC 1.10

COSMIC 1.10 moved the desktop theme to a new format (`ThemeBuilder` v2,
stored under `com.system76.CosmicTheme.Dark.Builder/v2/`). The `.ron` files
in catppuccin/cosmic-desktop still use v1: they import fine, but v1's
`is_frosted` is ignored and the new fields default to off, so importing
them turns Frosted Glass off. The new fields are:

- `frosted`: blur strength, `ExtremelyLow` … `ExtremelyHigh2`
- `frosted_windows`, `frosted_system_interface`, `frosted_panel`,
  `frosted_applets`, `frosted_maximized_apps`
- `alpha_map`: background opacity for each strength (`medium` = 0.76)

`themes/generate.py` writes all of them. Its output was checked against
libcosmic's parser at the commit used by cosmic-settings 1.10.

For the wallpaper picker, the installer writes the same keys COSMIC
Settings does: `recent-folders` (at most 5), `current-folder` and, with
`WALLPAPER_ROTATE`, `rotation-frequency`, all under
`com.system76.CosmicSettings.Wallpaper/v1/`.
