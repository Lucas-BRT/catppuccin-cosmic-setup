//! `ctp-cosmic backup`: undo what the installer changed, and only that.
//!
//! `create` copies the files the installer may touch (COSMIC's config,
//! ~/.bashrc, Zed's settings.json) into `DIR/before/`. After the steps ran,
//! `seal` records what they changed in `DIR/manifest` (`M` modified, `A`
//! added, `D` deleted) and keeps the new content in `DIR/after/`. `restore`
//! puts back only those files, and only when they still hold what the
//! installer left: anything you or another app changed since is kept.

use std::fs;
use std::path::{Path, PathBuf};

const USAGE: &str = "\
ctp-cosmic backup: back up and restore what the installer changes

Usage:
  ctp-cosmic backup create DIR             copy the files the installer may touch
  ctp-cosmic backup seal [DIR]             record what the installer changed
  ctp-cosmic backup restore [DIR] [--force]  undo it; --force also restores files
                                           changed since
  ctp-cosmic backup path                   the last backup's folder

DIR defaults to the last backup made by `create`.
";

/// What the installer may touch, relative to $HOME.
const WATCHED: [&str; 3] = [".config/cosmic", ".bashrc", ".config/zed/settings.json"];

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME not set".to_string())
}

fn pointer(home: &Path) -> PathBuf {
    let cache = match std::env::var_os("XDG_CACHE_HOME") {
        Some(c) if !c.is_empty() => PathBuf::from(c),
        _ => home.join(".cache"),
    };
    cache.join("catppuccin-cosmic-setup/last-backup")
}

fn last(home: &Path) -> Result<PathBuf, String> {
    let file = pointer(home);
    let text =
        fs::read_to_string(&file).map_err(|_| "no backup found; pass its folder".to_string())?;
    Ok(PathBuf::from(text.trim()))
}

fn io<T>(r: std::io::Result<T>, path: &Path) -> Result<T, String> {
    r.map_err(|e| format!("{}: {e}", path.display()))
}

/// Every regular file under the watched paths, relative to `root`.
fn files(root: &Path, watched: &[&str]) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, rel: PathBuf, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let path = root.join(&rel);
        let Ok(meta) = fs::symlink_metadata(&path) else {
            return Ok(());
        };
        if meta.is_dir() {
            for entry in io(fs::read_dir(&path), &path)? {
                let entry = io(entry, &path)?;
                walk(root, rel.join(entry.file_name()), out)?;
            }
        } else if meta.is_file() {
            out.push(rel);
        }
        Ok(())
    }
    let mut out = Vec::new();
    for w in watched {
        walk(root, PathBuf::from(w), &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn copy(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        io(fs::create_dir_all(parent), parent)?;
    }
    io(fs::copy(from, to), from).map(drop)
}

fn same(a: &Path, b: &Path) -> bool {
    matches!((fs::read(a), fs::read(b)), (Ok(x), Ok(y)) if x == y)
}

fn create_in(home: &Path, dir: &Path, watched: &[&str]) -> Result<usize, String> {
    let before = dir.join("before");
    io(fs::create_dir_all(&before), &before)?;
    let all = files(home, watched)?;
    for rel in &all {
        copy(&home.join(rel), &before.join(rel))?;
    }
    Ok(all.len())
}

fn seal_in(home: &Path, dir: &Path, watched: &[&str]) -> Result<Vec<String>, String> {
    let (before, after) = (dir.join("before"), dir.join("after"));
    if !before.is_dir() {
        return Err(format!("{}: not a backup made by `create`", dir.display()));
    }
    let _ = fs::remove_dir_all(&after);
    let now = files(home, watched)?;
    let was = files(&before, watched)?;
    let mut lines = Vec::new();
    for rel in &now {
        let cur = home.join(rel);
        let tag = if !was.contains(rel) {
            "A"
        } else if !same(&cur, &before.join(rel)) {
            "M"
        } else {
            continue;
        };
        copy(&cur, &after.join(rel))?;
        lines.push(format!("{tag} {}", rel.display()));
    }
    for rel in was.iter().filter(|r| !now.contains(r)) {
        lines.push(format!("D {}", rel.display()));
    }
    let manifest = dir.join("manifest");
    io(fs::write(&manifest, lines.join("\n") + "\n"), &manifest)?;
    Ok(lines)
}

/// Returns (restored, kept because changed since).
fn restore_in(home: &Path, dir: &Path, force: bool) -> Result<(usize, Vec<String>), String> {
    let manifest = dir.join("manifest");
    let text = fs::read_to_string(&manifest).map_err(|_| {
        format!(
            "{}: no manifest (the installation did not finish, or this is an old backup)",
            dir.display()
        )
    })?;
    let (before, after) = (dir.join("before"), dir.join("after"));
    let (mut restored, mut kept) = (0, Vec::new());
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (tag, rel) = line.split_once(' ').ok_or("bad manifest")?;
        let cur = home.join(rel);
        // Still what the installer left (or gone), so nothing of yours is lost.
        let untouched = match tag {
            "D" => !cur.exists(),
            _ => !cur.exists() || same(&cur, &after.join(rel)),
        };
        if !untouched && !force {
            kept.push(rel.to_string());
            continue;
        }
        match tag {
            "A" => {
                if cur.exists() {
                    io(fs::remove_file(&cur), &cur)?;
                }
            }
            "M" | "D" => copy(&before.join(rel), &cur)?,
            _ => return Err(format!("bad manifest line: {line}")),
        }
        restored += 1;
    }
    Ok((restored, kept))
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let home = home()?;
    match args[..] {
        ["create", dir] => {
            let n = create_in(&home, Path::new(dir), &WATCHED)?;
            let ptr = pointer(&home);
            if let Some(parent) = ptr.parent() {
                io(fs::create_dir_all(parent), parent)?;
            }
            io(fs::write(&ptr, format!("{dir}\n")), &ptr)?;
            println!("{n} files backed up to {dir}");
        }
        ["seal"] | ["seal", _] => {
            let dir = match args.get(1) {
                Some(d) => PathBuf::from(d),
                None => last(&home)?,
            };
            let lines = seal_in(&home, &dir, &WATCHED)?;
            println!("{} files changed", lines.len());
        }
        ["restore", ref rest @ ..] => {
            let force = rest.contains(&"--force");
            let dir = match rest.iter().find(|a| **a != "--force") {
                Some(d) => PathBuf::from(d),
                None => last(&home)?,
            };
            let (restored, kept) = restore_in(&home, &dir, force)?;
            println!("{restored} files restored from {}", dir.display());
            for k in &kept {
                println!("kept (changed since the install): ~/{k}");
            }
        }
        ["path"] => println!("{}", last(&home)?.display()),
        [] | ["-h" | "--help"] => print!("{USAGE}"),
        _ => return Err(format!("bad arguments\n\n{USAGE}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: [&str; 2] = ["cfg", "rc"];

    fn setup(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("ctp-bk-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let home = root.join("home");
        fs::create_dir_all(home.join("cfg")).unwrap();
        fs::write(home.join("cfg/a"), "a0").unwrap();
        fs::write(home.join("cfg/b"), "b0").unwrap();
        fs::write(home.join("rc"), "rc0").unwrap();
        (home, root.join("backup"))
    }

    fn read(home: &Path, rel: &str) -> Option<String> {
        fs::read_to_string(home.join(rel)).ok()
    }

    /// Back up, then modify a, delete rc, add c (what an install does).
    fn install(home: &Path, dir: &Path) {
        create_in(home, dir, &W).unwrap();
        fs::write(home.join("cfg/a"), "a1").unwrap();
        fs::remove_file(home.join("rc")).unwrap();
        fs::write(home.join("cfg/c"), "c1").unwrap();
        let lines = seal_in(home, dir, &W).unwrap();
        assert_eq!(lines, ["M cfg/a", "A cfg/c", "D rc"]);
    }

    #[test]
    fn restore_undoes_only_the_install() {
        let (home, dir) = setup("undo");
        install(&home, &dir);
        // Changed by something else after the install: not in the manifest.
        fs::write(home.join("cfg/b"), "b-later").unwrap();

        let (n, kept) = restore_in(&home, &dir, false).unwrap();
        assert_eq!((n, kept.len()), (3, 0));
        assert_eq!(read(&home, "cfg/a").as_deref(), Some("a0"));
        assert_eq!(read(&home, "cfg/c"), None);
        assert_eq!(read(&home, "rc").as_deref(), Some("rc0"));
        assert_eq!(read(&home, "cfg/b").as_deref(), Some("b-later"));
        fs::remove_dir_all(home.parent().unwrap()).unwrap();
    }

    #[test]
    fn files_changed_since_are_kept_unless_forced() {
        let (home, dir) = setup("kept");
        install(&home, &dir);
        fs::write(home.join("cfg/a"), "mine").unwrap();

        let (n, kept) = restore_in(&home, &dir, false).unwrap();
        assert_eq!((n, kept), (2, vec!["cfg/a".to_string()]));
        assert_eq!(read(&home, "cfg/a").as_deref(), Some("mine"));

        let (n, kept) = restore_in(&home, &dir, true).unwrap();
        assert_eq!((n, kept.len()), (3, 0));
        assert_eq!(read(&home, "cfg/a").as_deref(), Some("a0"));
        fs::remove_dir_all(home.parent().unwrap()).unwrap();
    }

    #[test]
    fn restore_without_seal_is_an_error() {
        let (home, dir) = setup("unsealed");
        create_in(&home, &dir, &W).unwrap();
        assert!(restore_in(&home, &dir, false).is_err());
        fs::remove_dir_all(home.parent().unwrap()).unwrap();
    }
}
