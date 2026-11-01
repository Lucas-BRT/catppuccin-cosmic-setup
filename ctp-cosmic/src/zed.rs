//! Zed configuration editor.
//!
//! Uses `jsonc-parser` to parse and update `~/.config/zed/settings.json`
//! while preserving comments and existing structure.

use std::fs;
use std::path::{Path, PathBuf};

use jsonc_parser::cst::CstRootNode;
use jsonc_parser::{ParseOptions, json};

use crate::palette::Palette;

const USAGE: &str = "\
ctp-cosmic zed: configure Catppuccin theme and icon theme in Zed

Usage:
  ctp-cosmic zed apply <flavor> [--settings PATH]
  ctp-cosmic zed check <flavor> [--settings PATH]

Arguments:
  flavor            latte, frappe, macchiato, mocha
  --settings PATH   path to settings.json (default: ~/.config/zed/settings.json)
";

pub fn default_settings_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    Ok(PathBuf::from(home).join(".config/zed/settings.json"))
}

/// Applies the theme and icon theme to `settings.json`.
pub fn apply(flavor: &str, settings_path: &Path) -> Result<(), String> {
    let p = Palette::new(flavor).ok_or_else(|| format!("unknown flavor '{flavor}'"))?;
    let display_name = format!("Catppuccin {}", p.name());
    let is_dark = p.is_dark();

    let mut content = if settings_path.is_file() {
        fs::read_to_string(settings_path)
            .map_err(|e| format!("{}: cannot read settings: {e}", settings_path.display()))?
    } else {
        "{\n}\n".to_string()
    };

    if content.trim().is_empty() {
        content = "{\n}\n".to_string();
    }

    let parse_opts = ParseOptions::default();
    let root = CstRootNode::parse(&content, &parse_opts)
        .map_err(|e| format!("{}: JSON parse error: {e}", settings_path.display()))?;

    let root_obj = root.object_value_or_create().ok_or_else(|| {
        format!(
            "{}: settings root must be an object",
            settings_path.display()
        )
    })?;

    // 1. icon_theme: "Catppuccin <Name>"
    if let Some(prop) = root_obj.get("icon_theme") {
        prop.set_value(json!(display_name.clone()));
    } else {
        root_obj.append("icon_theme", json!(display_name.clone()));
    }

    // 2. theme: { "mode": "system", "light": ..., "dark": ... }
    let theme_obj = root_obj.object_value_or_set("theme");
    if theme_obj.get("mode").is_none() {
        theme_obj.append("mode", json!("system"));
    }
    if is_dark {
        if let Some(prop) = theme_obj.get("dark") {
            prop.set_value(json!(display_name.clone()));
        } else {
            theme_obj.append("dark", json!(display_name.clone()));
        }
        if theme_obj.get("light").is_none() {
            theme_obj.append("light", json!("One Light"));
        }
    } else {
        if let Some(prop) = theme_obj.get("light") {
            prop.set_value(json!(display_name.clone()));
        } else {
            theme_obj.append("light", json!(display_name.clone()));
        }
        if theme_obj.get("dark").is_none() {
            theme_obj.append("dark", json!("Catppuccin Mocha"));
        }
    }

    let updated = root.to_string();

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("{}: cannot create directory: {e}", parent.display()))?;
    }

    fs::write(settings_path, updated)
        .map_err(|e| format!("{}: cannot write settings: {e}", settings_path.display()))?;

    Ok(())
}

/// Checks whether the theme and icon theme match the flavor in `settings.json`.
pub fn check(flavor: &str, settings_path: &Path) -> Result<bool, String> {
    let p = Palette::new(flavor).ok_or_else(|| format!("unknown flavor '{flavor}'"))?;
    let display_name = format!("Catppuccin {}", p.name());
    let is_dark = p.is_dark();

    if !settings_path.is_file() {
        return Ok(false);
    }
    let content = fs::read_to_string(settings_path)
        .map_err(|e| format!("{}: cannot read settings: {e}", settings_path.display()))?;

    let parse_opts = ParseOptions::default();
    let root = match CstRootNode::parse(&content, &parse_opts) {
        Ok(r) => r,
        Err(_) => return Ok(false),
    };
    let root_obj = match root.object_value() {
        Some(o) => o,
        None => return Ok(false),
    };

    // Check icon_theme
    let icon_ok = root_obj
        .get("icon_theme")
        .and_then(|p| p.value())
        .map(|v| v.to_string() == format!("\"{display_name}\""))
        .unwrap_or(false);

    // Check theme
    let theme_ok = if let Some(theme_obj) = root_obj.object_value("theme") {
        let key = if is_dark { "dark" } else { "light" };
        theme_obj
            .get(key)
            .and_then(|p| p.value())
            .map(|v| v.to_string() == format!("\"{display_name}\""))
            .unwrap_or(false)
    } else {
        false
    };

    Ok(icon_ok && theme_ok)
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{USAGE}");
        return Ok(());
    }

    let subcmd = &args[0];
    let mut flavor = None;
    let mut custom_path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--settings" => {
                i += 1;
                if i >= args.len() {
                    return Err("--settings requires a path".to_string());
                }
                custom_path = Some(PathBuf::from(&args[i]));
            }
            s if !s.starts_with('-') && flavor.is_none() => {
                flavor = Some(s.to_string());
            }
            other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
        }
        i += 1;
    }

    let flavor = flavor.ok_or_else(|| format!("missing flavor\n\n{USAGE}"))?;
    let path = match custom_path {
        Some(p) => p,
        None => default_settings_path()?,
    };

    match subcmd.as_str() {
        "apply" => {
            apply(&flavor, &path)?;
            let p = Palette::new(&flavor).unwrap();
            println!("Catppuccin {} applied to {}", p.name(), path.display());
            Ok(())
        }
        "check" => {
            let ok = check(&flavor, &path)?;
            if ok {
                let p = Palette::new(&flavor).unwrap();
                println!(
                    "Catppuccin {} is configured in {}",
                    p.name(),
                    path.display()
                );
                Ok(())
            } else {
                Err(format!(
                    "Catppuccin {} is not configured in {}",
                    flavor,
                    path.display()
                ))
            }
        }
        other => Err(format!("unknown zed subcommand '{other}'\n\n{USAGE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempFile(PathBuf);
    impl TempFile {
        fn new() -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let pid = std::process::id();
            let path = std::env::temp_dir().join(format!("zed-test-{pid}-{id}.json"));
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn applies_mocha_preserving_comments() {
        let original = r#"// Zed settings
// Documentation: https://zed.dev
{
  // User UI font size
  "ui_font_size": 16,
  "theme": {
    "mode": "system",
    "light": "One Light",
    "dark": "One Dark"
  }
}
"#;
        let tmp = TempFile::new();
        fs::write(tmp.path(), original).unwrap();

        apply("mocha", tmp.path()).unwrap();
        let updated = fs::read_to_string(tmp.path()).unwrap();

        assert!(updated.contains("// Zed settings"));
        assert!(updated.contains("// User UI font size"));
        assert!(updated.contains(r#""dark": "Catppuccin Mocha""#));
        assert!(updated.contains(r#""icon_theme": "Catppuccin Mocha""#));
        assert!(updated.contains(r#""ui_font_size": 16"#));

        assert!(check("mocha", tmp.path()).unwrap());
        assert!(!check("latte", tmp.path()).unwrap());
    }

    #[test]
    fn applies_latte_light_theme() {
        let original = r#"{
  "theme": {
    "mode": "system",
    "light": "One Light",
    "dark": "Catppuccin Mocha"
  }
}
"#;
        let tmp = TempFile::new();
        fs::write(tmp.path(), original).unwrap();

        apply("latte", tmp.path()).unwrap();
        let updated = fs::read_to_string(tmp.path()).unwrap();

        assert!(updated.contains(r#""light": "Catppuccin Latte""#));
        assert!(updated.contains(r#""icon_theme": "Catppuccin Latte""#));
        assert!(check("latte", tmp.path()).unwrap());
    }

    #[test]
    fn applies_to_empty_file() {
        let tmp = TempFile::new();
        fs::write(tmp.path(), "").unwrap();

        apply("frappe", tmp.path()).unwrap();
        let updated = fs::read_to_string(tmp.path()).unwrap();

        assert!(updated.contains(r#""dark": "Catppuccin Frappé""#));
        assert!(updated.contains(r#""icon_theme": "Catppuccin Frappé""#));
        assert!(check("frappe", tmp.path()).unwrap());
    }
}
