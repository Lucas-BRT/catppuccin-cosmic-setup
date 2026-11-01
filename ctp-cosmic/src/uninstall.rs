//! `ctp-cosmic uninstall`: undo what `apply` changed, and only that (see
//! `backup`). With --purge also delete what it downloaded or generated.

use std::fs;
use std::path::{Path, PathBuf};

use crate::apply::{cache_dir, home, state_dir};
use crate::steps::STEPS;
use crate::{backup, prompt, ui};

const USAGE: &str = "\
ctp-cosmic uninstall: undo the last apply

Usage:
  ctp-cosmic uninstall [BACKUP_DIR] [--only STEP[,STEP]] [--force] [--purge]

BACKUP_DIR defaults to the last backup. Only the files the install changed go
back to what they were, unless you changed them since (those are kept and
listed; --force restores them anyway). The prompt block is removed from
~/.bashrc.

--only restores just those steps (cosmic, terminal, bash, zed, icons,
wallpaper); run it again for the others. `--only icons` also deletes the
Papirus themes, and `--only fonts` deletes the Nerd Fonts that apply
installed (nothing is restored for it).

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

const ICON_THEMES: [&str; 4] = [
    "Papirus",
    "Papirus-Dark",
    "Papirus-Catppuccin",
    "Papirus-Dark-Catppuccin",
];

fn purge() -> Result<(), String> {
    let home = home()?;
    let icons = home.join(".local/share/icons");
    let mut doomed: Vec<PathBuf> = ICON_THEMES.iter().map(|t| icons.join(t)).collect();
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
    let (mut dir, mut force, mut purge_all, mut only) = (None, false, false, None::<String>);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "--force" => force = true,
            "--purge" => purge_all = true,
            "--only" => only = Some(it.next().ok_or("--only needs a step")?.clone()),
            d if !d.starts_with('-') && dir.is_none() => dir = Some(PathBuf::from(d)),
            other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
        }
    }
    let mut paths: Vec<&str> = Vec::new();
    let (mut fonts, mut icon_files) = (false, false);
    let mut with_prompt = only.is_none();
    if let Some(list) = &only {
        if purge_all {
            return Err("--only and --purge cannot be combined".into());
        }
        for name in list.split(',').map(str::trim) {
            if name == "fonts" {
                fonts = true;
                continue;
            }
            icon_files |= name == "icons";
            let step = STEPS
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| format!("unknown step '{name}' (see: ctp-cosmic apply --list)"))?;
            paths.extend(step.paths);
            with_prompt |= name == "bash";
        }
    }
    let only_paths = only.is_some().then_some(paths.as_slice());
    // `--only fonts` removes files only; there is nothing to restore.
    if only.is_none() || !paths.is_empty() {
        let (dir, restored, kept) = backup::restore(dir.as_deref(), force, only_paths)
            .map_err(|e| {
                format!(
                    "{e}\nPass the backup folder: ctp-cosmic uninstall ~/.config/catppuccin-backup-..."
                )
            })?;
        ui::info(&format!("Restoring {}", dir.display()));
        ui::ok(&format!("{restored} files restored"));
        for k in &kept {
            ui::warn(&format!("Kept (changed since the install): ~/{k}"));
        }
        if with_prompt {
            prompt::remove()?;
        }
        ui::ok("Settings restored");
    } else if with_prompt {
        prompt::remove()?;
    }
    let home = home()?;
    if fonts {
        remove(&home.join(".local/share/fonts/NerdFonts"));
        let _ = std::process::Command::new("fc-cache")
            .arg("-f")
            .stderr(std::process::Stdio::null())
            .status();
        ui::ok("Nerd Fonts removed");
    }
    if icon_files {
        for t in ICON_THEMES {
            remove(&home.join(".local/share/icons").join(t));
        }
        ui::ok("Papirus icons removed");
    }
    if purge_all {
        purge()?;
    }
    println!("Log out and back in for everything to return to normal.");
    Ok(())
}
