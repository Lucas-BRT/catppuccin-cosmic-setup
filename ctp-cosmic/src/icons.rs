//! `ctp-cosmic icons`: Papirus with Catppuccin folders, kept apart from
//! COSMIC's own icons (no sudo; everything lives in ~/.local/share/icons).
//!
//! 1. Copies Papirus and Papirus-Dark from the pinned Papirus sources and
//!    adds the colored folders from catppuccin/papirus-folders.
//! 2. Points the folder icons at `cat-<flavor>-<accent>`: for every
//!    `folder-<color>[-name].svg` and `user-<color>[-name].svg` in places/,
//!    a symlink without the color (what papirus-folders does).
//! 3. Creates `<Papirus theme>-Catppuccin`, which keeps COSMIC's own window
//!    controls (Papirus draws them small inside 22/24 px canvases, which
//!    COSMIC shrinks into its 16 px title bar buttons) and falls back to
//!    COSMIC's whole icon chain.
//! 4. Selects that theme in COSMIC (CosmicTk `icon_theme`).

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::palette;

const USAGE: &str = "\
ctp-cosmic icons: Papirus icons with Catppuccin folders

Usage:
  ctp-cosmic icons apply FLAVOR ACCENT --papirus DIR --folders DIR
                     [--icons-dir DIR] [--config DIR]

--papirus DIR     papirus-icon-theme sources (holds Papirus/ and Papirus-Dark/)
--folders DIR     catppuccin/papirus-folders sources (holds src/)
--icons-dir DIR   default: ~/.local/share/icons
--config DIR      COSMIC's config folder, default: ~/.config/cosmic
";

const SIZES: [&str; 5] = ["22x22", "24x24", "32x32", "48x48", "64x64"];
const CONTROLS: [&str; 4] = [
    "window-close",
    "window-minimize",
    "window-maximize",
    "window-restore",
];

fn io<T>(r: std::io::Result<T>, path: &Path) -> Result<T, String> {
    r.map_err(|e| format!("{}: {e}", path.display()))
}

/// Copies a tree, keeping symlinks as symlinks (Papirus-Dark is mostly links
/// into Papirus). Existing files are overwritten.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    let meta = io(fs::symlink_metadata(from), from)?;
    if meta.file_type().is_symlink() {
        let target = io(fs::read_link(from), from)?;
        if fs::symlink_metadata(to).is_ok() {
            io(fs::remove_file(to), to)?;
        }
        io(symlink(target, to), to)
    } else if meta.is_dir() {
        io(fs::create_dir_all(to), to)?;
        for entry in io(fs::read_dir(from), from)? {
            let entry = io(entry, from)?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        io(fs::copy(from, to), from).map(drop)
    }
}

/// Points the folders of `theme` at the color: `folder-<color>-x.svg` gets
/// the link `folder-x.svg`. Returns how many links were made.
fn set_folder_color(theme: &Path, color: &str) -> Result<usize, String> {
    let mut made = 0;
    for size in SIZES {
        let places = theme.join(size).join("places");
        let Ok(entries) = fs::read_dir(&places) else {
            continue;
        };
        for entry in entries {
            let entry = io(entry, &places)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let colored = ["folder", "user"].iter().any(|kind| {
                let prefix = format!("{kind}-{color}");
                name.strip_suffix(".svg")
                    .and_then(|n| n.strip_prefix(&prefix))
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
            });
            // Only real files: links made by an earlier run are replaced below.
            if !colored || !io(fs::symlink_metadata(entry.path()), &places)?.is_file() {
                continue;
            }
            let link = places.join(name.replacen(&format!("-{color}"), "", 1));
            if fs::symlink_metadata(&link).is_ok() {
                io(fs::remove_file(&link), &link)?;
            }
            io(symlink(&name, &link), &link)?;
            made += 1;
        }
    }
    Ok(made)
}

/// COSMIC's own window controls, from the cosmic-icons package.
pub fn cosmic_controls() -> Option<PathBuf> {
    let dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let dirs = if dirs.is_empty() {
        "/usr/local/share:/usr/share".to_string()
    } else {
        dirs
    };
    dirs.split(':')
        .map(|d| Path::new(d).join("icons/Cosmic/scalable/actions"))
        .find(|d| d.join("window-close-symbolic.svg").is_file())
}

fn index_theme(name: &str, base: &str, parent: &str) -> String {
    // COSMIC follows Inherits only one level deep (then tries Cosmic, hicolor
    // and gnome), so the whole chain is listed: Papirus first, then COSMIC's
    // own (Cosmic → Pop → Pop-Extra, pop-os-branding, Adwaita) for the icons
    // Papirus lacks, then Papirus' own parent. Without it, icons that only
    // Pop ships (e.g. Repoman) vanish.
    format!(
        "[Icon Theme]\nName={name}\nComment={base} with COSMIC's window controls \
         (catppuccin-cosmic-setup)\nInherits={base},Cosmic,Pop,Pop-Extra,pop-os-branding,\
         Adwaita,{parent},hicolor\nDirectories=scalable/actions\n\n[scalable/actions]\n\
         Context=Actions\nSize=16\nMinSize=8\nMaxSize=512\nType=Scalable\n"
    )
}

pub struct Paths {
    pub papirus: PathBuf,
    pub folders: PathBuf,
    pub icons: PathBuf,
    pub config: PathBuf,
}

/// Returns the name of the theme selected in COSMIC.
pub fn apply(flavor: &str, accent: &str, p: &Paths) -> Result<String, String> {
    let pal = palette::check(flavor, Some(accent))?;
    let (base, parent) = if pal.is_dark() {
        ("Papirus-Dark", "breeze-dark")
    } else {
        ("Papirus", "breeze")
    };
    let color = format!("cat-{flavor}-{accent}");

    io(fs::create_dir_all(&p.icons), &p.icons)?;
    for theme in ["Papirus", "Papirus-Dark"] {
        let dest = p.icons.join(theme);
        if fs::symlink_metadata(&dest).is_ok() {
            io(fs::remove_dir_all(&dest), &dest)?;
        }
        copy_tree(&p.papirus.join(theme), &dest)?;
    }
    for entry in io(fs::read_dir(p.folders.join("src")), &p.folders)? {
        let entry = io(entry, &p.folders)?;
        copy_tree(
            &entry.path(),
            &p.icons.join("Papirus").join(entry.file_name()),
        )?;
    }
    if set_folder_color(&p.icons.join(base), &color)? == 0 {
        return Err(format!("no '{color}' folders found in {base}"));
    }
    for theme in ["Papirus", "Papirus-Dark"] {
        // Only a cache for faster lookups: fine if the tool is missing.
        let _ = Command::new("gtk-update-icon-cache")
            .arg("-q")
            .arg(p.icons.join(theme))
            .stderr(Stdio::null())
            .status();
    }

    let theme = match cosmic_controls() {
        Some(native) => {
            let name = format!("{base}-Catppuccin");
            let dir = p.icons.join(&name);
            if dir.exists() {
                io(fs::remove_dir_all(&dir), &dir)?;
            }
            let actions = dir.join("scalable/actions");
            io(fs::create_dir_all(&actions), &actions)?;
            for icon in CONTROLS {
                let file = format!("{icon}-symbolic.svg");
                if native.join(&file).is_file() {
                    io(fs::copy(native.join(&file), actions.join(&file)), &native)?;
                }
            }
            let index = dir.join("index.theme");
            io(fs::write(&index, index_theme(&name, base, parent)), &index)?;
            name
        }
        None => {
            eprintln!("COSMIC icons not found; window controls will use {base}'s");
            base.to_string()
        }
    };

    let key = p.config.join("com.system76.CosmicTk/v1/icon_theme");
    if let Some(parent) = key.parent() {
        io(fs::create_dir_all(parent), parent)?;
    }
    io(fs::write(&key, format!("\"{theme}\"")), &key)?;
    Ok(theme)
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let ["apply", flavor, accent, rest @ ..] = args.as_slice() else {
        if matches!(args[..], [] | ["-h" | "--help"]) {
            print!("{USAGE}");
            return Ok(());
        }
        return Err(format!("bad arguments\n\n{USAGE}"));
    };
    let home = || {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("HOME not set".to_string())
    };
    let (mut papirus, mut folders, mut icons, mut config) = (None, None, None, None);
    let mut it = rest.iter();
    while let Some(&flag) = it.next() {
        let slot = match flag {
            "--papirus" => &mut papirus,
            "--folders" => &mut folders,
            "--icons-dir" => &mut icons,
            "--config" => &mut config,
            _ => return Err(format!("unexpected argument '{flag}'\n\n{USAGE}")),
        };
        *slot = Some(PathBuf::from(
            it.next().ok_or_else(|| format!("{flag} needs a folder"))?,
        ));
    }
    let paths = Paths {
        papirus: papirus.ok_or_else(|| format!("missing --papirus\n\n{USAGE}"))?,
        folders: folders.ok_or_else(|| format!("missing --folders\n\n{USAGE}"))?,
        icons: match icons {
            Some(i) => i,
            None => match std::env::var_os("XDG_DATA_HOME") {
                Some(d) if !d.is_empty() => PathBuf::from(d).join("icons"),
                _ => home()?.join(".local/share/icons"),
            },
        },
        config: match config {
            Some(c) => c,
            None => home()?.join(".config/cosmic"),
        },
    };
    let theme = apply(flavor, accent, &paths)?;
    println!("{theme}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(p: &Path, text: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }

    #[test]
    fn folder_color_links_strip_the_color() {
        let dir = std::env::temp_dir().join(format!("ctp-icons-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let places = dir.join("48x48/places");
        touch(&places.join("folder-cat-mocha-mauve.svg"), "plain");
        touch(&places.join("folder-cat-mocha-mauve-documents.svg"), "docs");
        touch(&places.join("user-cat-mocha-mauve-desktop.svg"), "desk");
        touch(&places.join("folder-cat-mocha-blue.svg"), "other accent");
        touch(&places.join("folder-cat-mocha-mauvex.svg"), "prefix only");
        touch(&places.join("folder-documents.svg"), "old default");

        assert_eq!(set_folder_color(&dir, "cat-mocha-mauve").unwrap(), 3);
        assert_eq!(
            fs::read_to_string(places.join("folder.svg")).unwrap(),
            "plain"
        );
        // A plain file was replaced by the link.
        assert_eq!(
            fs::read_link(places.join("folder-documents.svg")).unwrap(),
            Path::new("folder-cat-mocha-mauve-documents.svg")
        );
        assert!(places.join("user-desktop.svg").exists());
        assert!(!places.join("folder-mauvex.svg").exists());
        // Running again (links exist now) changes nothing.
        assert_eq!(set_folder_color(&dir, "cat-mocha-mauve").unwrap(), 3);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn copy_keeps_symlinks() {
        let dir = std::env::temp_dir().join(format!("ctp-copy-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        touch(&dir.join("a/real.svg"), "x");
        symlink("real.svg", dir.join("a/link.svg")).unwrap();
        copy_tree(&dir.join("a"), &dir.join("b")).unwrap();
        assert_eq!(
            fs::read_link(dir.join("b/link.svg")).unwrap(),
            Path::new("real.svg")
        );
        copy_tree(&dir.join("a"), &dir.join("b")).unwrap();
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn index_lists_the_whole_chain() {
        let t = index_theme("Papirus-Dark-Catppuccin", "Papirus-Dark", "breeze-dark");
        assert!(t.contains(
            "Inherits=Papirus-Dark,Cosmic,Pop,Pop-Extra,pop-os-branding,Adwaita,breeze-dark,hicolor\n"
        ));
    }
}
