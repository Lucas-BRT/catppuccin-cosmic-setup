# Theming other apps

`ctp-cosmic apply firefox` themes an app from a small manifest. You can add
your own, and share it, without touching Rust.

```bash
ctp-cosmic app list                     # the apps it knows, and whether they are installed
ctp-cosmic apply firefox                # theme one (backed up like every other step)
ctp-cosmic app check firefox            # are its files up to date?
ctp-cosmic app remove firefox           # take out what it wrote
ctp-cosmic uninstall --only firefox     # or restore the files from the backup
```

Apps are never part of the default selection: they run only when named.

## Your own app

Make a folder `~/.config/catppuccin-cosmic-setup/apps/NAME/` with an
`app.toml` and the templates it names:

```toml
name = "btop"                       # lowercase letters, digits and '-'
label = "btop"                      # shown in messages (default: name)
detect = ["btop"]                   # programs that mean it is installed
detect_paths = ["~/.config/btop"]   # or ~/... paths (Flatpak, snap)

[[file]]
target = "~/.config/btop/themes/catppuccin.theme"
template = "catppuccin.theme"       # a file in the same folder
mode = "file"                       # the file is entirely ours

[[file]]
target = "~/.config/btop/btop.conf"
template = "btop.conf"
mode = "block"                      # a marked block; the rest of the file is yours
comment = "#"                       # "#", "//" or "/* */"
```

Without `detect` and `detect_paths` the app counts as installed. A same-named
app of yours replaces a built-in one.

### Modes

- **`file`**: ctp-cosmic owns the whole file. Use it for a theme file nothing
  else writes to.
- **`block`**: writes only between two marker lines
  (`# >>> catppuccin-cosmic-setup:NAME >>>` … `# <<< … <<<`), at the end of the
  file, and replaces just that block next time. Everything you wrote stays.
  Use it for config files.

### Placeholders in templates

`@base@` becomes `#1e1e2e` for Mocha. Any palette color works (`base`,
`text`, `surface0`, `red`, …), and `accent` is the one you chose.

| Placeholder | Gives (Mocha, Mauve) |
|---|---|
| `@base@`, `@accent@` | `#1e1e2e`, `#cba6f7` |
| `@base:bare@` | `1e1e2e` |
| `@base:rgb@` | `30, 30, 46` |
| `@accent_name@` | `mauve` |
| `@flavor@`, `@flavor_name@` | `mocha`, `Mocha` |
| `@scheme@` | `dark` (or `light` for Latte) |

An unknown name between two `@` is an error, so a typo is never written into
your config. Text that is not a placeholder (`@media`, an email address) is
left as it is.

### Where the files go

Every `target` is a `~/...` path inside your home. `..` and absolute paths
are refused. A manifest cannot run commands: it only says which files to
write. A manifest that is wrong is skipped (with the reason shown by
`ctp-cosmic app list`) and never stops the other apps.

For apps that keep files in a profile folder with a random name, `locate`
finds it, and `target` is then relative to the profile:

```toml
locate = "firefox"      # the default profile in profiles.ini
```

Only `firefox` exists so far.

## Firefox

Themes Firefox's own interface (`userChrome.css`) and its built-in pages,
such as the new tab and `about:preferences` (`userContent.css`), and turns on
the preference Firefox needs to read them (`user.js`). Needs no root and
installs no add-on.

- Firefox must have been started once, so a profile exists. Regular installs
  (`~/.config/mozilla/firefox`, or `~/.mozilla/firefox` before Firefox 147) are
  checked, and so are Flatpak and snap. The Flatpak and snap locations of the
  newer layout are guesses, not tested against a real install: if yours is
  not found, open an issue with the path of your `profiles.ini`.
- **Websites are not themed**: that takes an extension such as
  [Stylus](https://github.com/openstyles/stylus) with
  [catppuccin/userstyles](https://github.com/catppuccin/userstyles).
- Restart Firefox to see the change.
- A **theme add-on** that is active (for example a dark "Catppuccin" theme
  while you apply Latte) keeps its own text and icon colors and clashes with
  these. `apply` and `app check` warn when they see one; choose "System theme"
  under Add-ons > Themes.
- The colors are set through Firefox's own CSS variables, with the names
  Firefox 159 uses. Mozilla can rename them between versions: if part of the
  interface looks wrong after an update, open an issue with your Firefox
  version.
