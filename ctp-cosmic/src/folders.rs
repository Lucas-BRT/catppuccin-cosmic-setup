//! `ctp-cosmic register-wallpaper-folder`: add a folder to COSMIC Settings →
//! Wallpaper the way the Settings app does it.
//!
//! In `com.system76.CosmicSettings.Wallpaper/v1/`, `recent-folders` keeps at
//! most five folders (oldest dropped first) and `current-folder` is the one
//! the picker opens on.

use std::path::Path;

const USAGE: &str = "\
ctp-cosmic register-wallpaper-folder: add a folder to COSMIC's wallpaper picker

Usage:
  ctp-cosmic register-wallpaper-folder SETTINGS_DIR FOLDER

SETTINGS_DIR is ~/.config/cosmic/com.system76.CosmicSettings.Wallpaper/v1.
";

/// A RON string literal.
fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The new `recent-folders` list with `folder` added, given the old one.
pub fn add_recent(mut folders: Vec<String>, folder: &str) -> Vec<String> {
    if !folders.iter().any(|f| f == folder) {
        while folders.len() > 4 {
            folders.remove(0);
        }
        folders.push(folder.to_string());
    }
    folders
}

pub fn register(settings: &Path, folder: &str) -> Result<(), String> {
    let recent = settings.join("recent-folders");
    let folders: Vec<String> = match std::fs::read_to_string(&recent) {
        Ok(text) => ron::de::from_str(&text).map_err(|e| format!("{}: {e}", recent.display()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(format!("{}: {e}", recent.display())),
    };
    let folders = add_recent(folders, folder);

    std::fs::create_dir_all(settings).map_err(|e| format!("{}: {e}", settings.display()))?;
    let list: String = folders
        .iter()
        .map(|f| format!("    {},\n", quote(f)))
        .collect();
    std::fs::write(&recent, format!("[\n{list}]"))
        .map_err(|e| format!("{}: {e}", recent.display()))?;
    let current = settings.join("current-folder");
    std::fs::write(&current, format!("Some({})", quote(folder)))
        .map_err(|e| format!("{}: {e}", current.display()))
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [h] if h == "-h" || h == "--help" => {
            print!("{USAGE}");
            Ok(())
        }
        [settings, folder] => register(Path::new(settings), folder),
        _ => Err(format!("invalid arguments\n\n{USAGE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_five_folders_and_no_duplicates() {
        let five: Vec<String> = ["/a", "/b", "/c", "/d", "/e"].map(String::from).to_vec();
        assert_eq!(
            add_recent(five.clone(), "/f"),
            ["/b", "/c", "/d", "/e", "/f"]
        );
        assert_eq!(add_recent(five.clone(), "/c"), five);
        assert_eq!(add_recent(Vec::new(), "/x"), ["/x"]);
    }
}
