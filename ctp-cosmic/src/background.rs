//! `ctp-cosmic background`: set COSMIC's wallpaper for every display, with
//! the same keys COSMIC Settings → Wallpaper writes.

use std::fs;
use std::path::{Path, PathBuf};

use crate::folders::quote;

const USAGE: &str = "\
ctp-cosmic background: set the wallpaper of every display

Usage:
  ctp-cosmic background set SOURCE [--rotation SECONDS] [--config DIR]

SOURCE is an image, or a folder for a slideshow. --rotation is the seconds
between images (folders only; default 3600) and is also kept in sync in
Settings → Wallpaper. --config is COSMIC's config folder
(default: ~/.config/cosmic).
";

fn write(file: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(file, text).map_err(|e| format!("{}: {e}", file.display()))
}

/// The `all` entry of com.system76.CosmicBackground: one config for every
/// output. The filter, scaling and sampling values are COSMIC's defaults.
fn entry(source: &str, rotation: u32) -> String {
    format!(
        "(\n    output: \"all\",\n    source: Path({}),\n    filter_by_theme: true,\n    \
         rotation_frequency: {rotation},\n    filter_method: Lanczos,\n    \
         scaling_mode: Zoom,\n    sampling_method: Alphanumeric,\n)\n",
        quote(source)
    )
}

pub fn set(config: &Path, source: &str, rotation: Option<u32>) -> Result<(), String> {
    let bg = config.join("com.system76.CosmicBackground/v1");
    write(&bg.join("same-on-all"), "true")?;
    write(&bg.join("all"), &entry(source, rotation.unwrap_or(3600)))?;
    if let Some(secs) = rotation {
        let settings = config.join("com.system76.CosmicSettings.Wallpaper/v1");
        write(&settings.join("rotation-frequency"), &secs.to_string())?;
    }
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let Some((&"set", rest)) = args.split_first() else {
        if matches!(args[..], [] | ["-h" | "--help"]) {
            print!("{USAGE}");
            return Ok(());
        }
        return Err(format!("bad arguments\n\n{USAGE}"));
    };
    let (mut source, mut rotation, mut config) = (None, None, None);
    let mut it = rest.iter();
    while let Some(&a) = it.next() {
        match a {
            "--rotation" => {
                let v = it.next().ok_or("--rotation needs a value")?;
                rotation = Some(
                    v.parse::<u32>()
                        .map_err(|_| format!("bad --rotation '{v}'"))?,
                );
            }
            "--config" => config = Some(PathBuf::from(it.next().ok_or("--config needs a folder")?)),
            _ if source.is_none() && !a.starts_with("--") => source = Some(a),
            _ => return Err(format!("unexpected argument '{a}'\n\n{USAGE}")),
        }
    }
    let source = source.ok_or_else(|| format!("missing SOURCE\n\n{USAGE}"))?;
    let config = match config {
        Some(c) => c,
        None => {
            PathBuf::from(std::env::var_os("HOME").ok_or("HOME not set")?).join(".config/cosmic")
        }
    };
    set(&config, source, rotation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_the_keys_cosmic_reads() {
        let dir = std::env::temp_dir().join(format!("ctp-bg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        set(&dir, "/pics/a \"b\".png", None).unwrap();
        let bg = dir.join("com.system76.CosmicBackground/v1");
        assert_eq!(fs::read_to_string(bg.join("same-on-all")).unwrap(), "true");
        let all = fs::read_to_string(bg.join("all")).unwrap();
        assert!(all.contains(r#"source: Path("/pics/a \"b\".png"),"#));
        assert!(all.contains("rotation_frequency: 3600,"));
        assert!(!dir.join("com.system76.CosmicSettings.Wallpaper").exists());

        set(&dir, "/pics", Some(1800)).unwrap();
        assert!(
            fs::read_to_string(bg.join("all"))
                .unwrap()
                .contains("rotation_frequency: 1800,")
        );
        let freq = dir.join("com.system76.CosmicSettings.Wallpaper/v1/rotation-frequency");
        assert_eq!(fs::read_to_string(freq).unwrap(), "1800");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn matches_the_old_template_output() {
        // What the removed config/cosmic/background-all.ron produced.
        assert_eq!(
            entry("/x.png", 3600),
            "(\n    output: \"all\",\n    source: Path(\"/x.png\"),\n    filter_by_theme: true,\n    rotation_frequency: 3600,\n    filter_method: Lanczos,\n    scaling_mode: Zoom,\n    sampling_method: Alphanumeric,\n)\n"
        );
    }
}
