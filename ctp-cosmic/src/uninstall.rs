//! `ctp-cosmic uninstall`: undo what `apply` changed, and only that (see
//! `backup`). With --purge also delete what it downloaded or generated.

use std::fs;
use std::path::{Path, PathBuf};

use crate::apply::{cache_dir, home, state_dir};
use crate::{backup, prompt, ui};

const USAGE: &str = "\
ctp-cosmic uninstall: undo the last apply

Usage:
  ctp-cosmic uninstall [BACKUP_DIR] [--force] [--purge]

BACKUP_DIR defaults to the last backup. Only the files the install changed go
back to what they were, unless you changed them since (those are kept and
listed; --force restores them anyway). The prompt block is removed from
~/.bashrc.

--purge also deletes the Papirus icons, generated themes, wallpapers named
catppuccin-*, the cache, the saved choices and ctp-cosmic itself.
";

fn remove(path: &Path) {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => drop(fs::remove_dir_all(path)),
        Ok(_) => drop(fs::remove_file(path)),
        Err(_) => {}
    }
}

fn purge() -> Result<(), String> {
    let home = home()?;
    let icons = home.join(".local/share/icons");
    let mut doomed: Vec<PathBuf> = [
        "Papirus",
        "Papirus-Dark",
        "Papirus-Catppuccin",
        "Papirus-Dark-Catppuccin",
    ]
    .iter()
    .map(|t| icons.join(t))
    .collect();
    doomed.extend([
        home.join(".local/share/catppuccin"),
        home.join(".config/catppuccin"),
        // The copy that earlier versions of install.sh unpacked.
        home.join(".local/share/catppuccin-cosmic-setup"),
        cache_dir()?,
        state_dir()?,
        home.join(".local/bin/ctp-cosmic"),
    ]);
    if let Ok(pics) = fs::read_dir(home.join("Pictures/Wallpapers")) {
        doomed.extend(
            pics.flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with("catppuccin-"))
                .map(|e| e.path()),
        );
    }
    doomed.iter().for_each(|p| remove(p));
    ui::ok("Downloaded and generated files removed");
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    let (mut dir, mut force, mut purge_all) = (None, false, false);
    for a in &args {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "--force" => force = true,
            "--purge" => purge_all = true,
            d if !d.starts_with('-') && dir.is_none() => dir = Some(PathBuf::from(d)),
            other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
        }
    }
    let (dir, restored, kept) = backup::restore(dir.as_deref(), force).map_err(|e| {
        format!("{e}\nPass the backup folder: ctp-cosmic uninstall ~/.config/catppuccin-backup-...")
    })?;
    ui::info(&format!("Restoring {}", dir.display()));
    ui::ok(&format!("{restored} files restored"));
    for k in &kept {
        ui::warn(&format!("Kept (changed since the install): ~/{k}"));
    }
    prompt::remove()?;
    ui::ok("Settings restored");
    if purge_all {
        purge()?;
    }
    println!("Log out and back in for everything to return to normal.");
    Ok(())
}
