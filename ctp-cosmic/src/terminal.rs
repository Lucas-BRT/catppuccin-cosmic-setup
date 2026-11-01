//! `ctp-cosmic terminal`: Catppuccin color schemes for COSMIC Terminal,
//! written straight into its cosmic-config (the same keys Color schemes… →
//! Import fills), so there is nothing to import by hand.
//!
//! The color mapping is the one in catppuccin/cosmic-desktop's cosmic-term
//! schemes. Dark flavors put surface1 in `black` and subtext1 in `white`;
//! Latte swaps those ends so black stays dark on a light background.
//! The scheme sets no `background`: the terminal follows the desktop theme.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::palette::{self, Palette};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Ansi {
    black: String,
    red: String,
    green: String,
    yellow: String,
    blue: String,
    magenta: String,
    cyan: String,
    white: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Scheme {
    name: String,
    foreground: String,
    cursor: String,
    bright_foreground: String,
    dim_foreground: String,
    normal: Ansi,
    bright: Ansi,
    dim: Ansi,
}

fn ansi(p: &Palette, black: &str, white: &str) -> Ansi {
    let h = |c: &str| format!("#{}", p.hex(c));
    Ansi {
        black: h(black),
        red: h("red"),
        green: h("green"),
        yellow: h("yellow"),
        blue: h("blue"),
        magenta: h("pink"),
        cyan: h("teal"),
        white: h(white),
    }
}

fn scheme(p: &Palette) -> Scheme {
    let h = |c: &str| format!("#{}", p.hex(c));
    let (n_black, n_white, b_black, b_white) = if p.is_dark() {
        ("surface1", "subtext1", "surface2", "subtext0")
    } else {
        ("subtext1", "surface2", "subtext0", "surface1")
    };
    Scheme {
        name: format!("Catppuccin {}", p.name()),
        foreground: h("text"),
        cursor: h("rosewater"),
        bright_foreground: h("text"),
        dim_foreground: h("overlay0"),
        normal: ansi(p, n_black, n_white),
        bright: ansi(p, b_black, b_white),
        dim: ansi(p, n_black, n_white),
    }
}

pub fn default_config_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME not set")?;
    Ok(PathBuf::from(home).join(".config/cosmic/com.system76.CosmicTerm/v1"))
}

/// The keys for the flavor's mode: (schemes, syntax theme).
fn keys(p: &Palette) -> (&'static str, &'static str) {
    if p.is_dark() {
        ("color_schemes_dark", "syntax_theme_dark")
    } else {
        ("color_schemes_light", "syntax_theme_light")
    }
}

fn read_schemes(file: &Path) -> Result<BTreeMap<u32, Scheme>, String> {
    match fs::read_to_string(file) {
        Ok(text) => ron::from_str(&text).map_err(|e| format!("{}: {e}", file.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(format!("{}: {e}", file.display())),
    }
}

fn write(file: &Path, text: &str) -> Result<(), String> {
    fs::write(file, text).map_err(|e| format!("{}: {e}", file.display()))
}

/// Adds (or refreshes) the flavor's scheme and selects it for syntax colors.
pub fn apply(flavor: &str, dir: &Path) -> Result<(), String> {
    let p = palette::check(flavor, None)?;
    let (schemes_key, syntax_key) = keys(&p);
    let file = dir.join(schemes_key);
    // An unreadable file is an error, not an empty map: never overwrite the
    // user's own schemes.
    let mut schemes = read_schemes(&file)?;
    let new = scheme(&p);
    let id = schemes
        .iter()
        .find(|(_, s)| s.name == new.name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| schemes.keys().next_back().map_or(0, |last| last + 1));
    let name = new.name.clone();
    schemes.insert(id, new);

    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let pretty = ron::ser::PrettyConfig::default();
    write(
        &file,
        &ron::ser::to_string_pretty(&schemes, pretty).map_err(|e| e.to_string())?,
    )?;
    write(&dir.join(syntax_key), &format!("\"{name}\""))
}

/// Whether the flavor's scheme is installed, up to date and selected.
pub fn check(flavor: &str, dir: &Path) -> Result<bool, String> {
    let p = palette::check(flavor, None)?;
    let (schemes_key, syntax_key) = keys(&p);
    let want = scheme(&p);
    // COSMIC's own import writes hex in capitals; the colors are what count.
    let key = |s: &Scheme| ron::to_string(s).map(|t| t.to_lowercase());
    let want_key = key(&want).map_err(|e| e.to_string())?;
    let installed = read_schemes(&dir.join(schemes_key))?
        .values()
        .any(|s| key(s).is_ok_and(|k| k == want_key));
    let selected = fs::read_to_string(dir.join(syntax_key))
        .is_ok_and(|t| t.trim().trim_matches('"') == want.name);
    Ok(installed && selected)
}

const USAGE: &str = "\
ctp-cosmic terminal: Catppuccin colors for COSMIC Terminal

Usage:
  ctp-cosmic terminal apply FLAVOR [--config DIR]
  ctp-cosmic terminal check FLAVOR [--config DIR]

--config DIR   cosmic-config folder (default: ~/.config/cosmic/com.system76.CosmicTerm/v1)
";

pub fn main(args: Vec<String>) -> Result<u8, String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (cmd, flavor, dir) = match args[..] {
        [cmd @ ("apply" | "check"), flavor] => (cmd, flavor, default_config_dir()?),
        [cmd @ ("apply" | "check"), flavor, "--config", dir] => (cmd, flavor, PathBuf::from(dir)),
        [] | ["-h" | "--help"] => {
            print!("{USAGE}");
            return Ok(0);
        }
        _ => return Err(format!("bad arguments\n\n{USAGE}")),
    };
    if cmd == "apply" {
        apply(flavor, &dir)?;
        Ok(0)
    } else {
        Ok(if check(flavor, &dir)? { 0 } else { 1 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ctp-term-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn mocha() -> Scheme {
        scheme(&Palette::new("mocha").unwrap())
    }

    #[test]
    fn mocha_matches_the_official_scheme() {
        let s = mocha();
        assert_eq!(s.name, "Catppuccin Mocha");
        assert_eq!(s.foreground, "#cdd6f4");
        assert_eq!(s.dim_foreground, "#6c7086");
        assert_eq!(s.normal.black, "#45475a");
        assert_eq!(s.normal.white, "#bac2de");
        assert_eq!(s.bright.black, "#585b70");
        assert_eq!(s.bright.white, "#a6adc8");
    }

    #[test]
    fn latte_swaps_the_dark_and_light_ends() {
        let s = scheme(&Palette::new("latte").unwrap());
        assert_eq!(s.normal.black, "#5c5f77");
        assert_eq!(s.normal.white, "#acb0be");
        assert_eq!(s.bright.black, "#6c6f85");
        assert_eq!(s.bright.white, "#bcc0cc");
    }

    #[test]
    fn check_ignores_hex_case() {
        let dir = tmp("case");
        fs::create_dir_all(&dir).unwrap();
        let upper = ron::to_string(&BTreeMap::from([(0u32, mocha())]))
            .unwrap()
            .replace("#cdd6f4", "#CDD6F4");
        write(&dir.join("color_schemes_dark"), &upper).unwrap();
        write(&dir.join("syntax_theme_dark"), "\"Catppuccin Mocha\"").unwrap();
        assert!(check("mocha", &dir).unwrap());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn apply_adds_once_and_keeps_other_schemes() {
        let dir = tmp("apply");
        fs::create_dir_all(&dir).unwrap();
        let mut other = BTreeMap::new();
        let mut mine = mocha();
        mine.name = "Mine".into();
        other.insert(3, mine);
        write(
            &dir.join("color_schemes_dark"),
            &ron::to_string(&other).unwrap(),
        )
        .unwrap();

        assert!(!check("mocha", &dir).unwrap());
        apply("mocha", &dir).unwrap();
        apply("mocha", &dir).unwrap();
        assert!(check("mocha", &dir).unwrap());
        assert!(!check("latte", &dir).unwrap());

        let all = read_schemes(&dir.join("color_schemes_dark")).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[&3].name, "Mine");
        assert_eq!(all[&4].name, "Catppuccin Mocha");
        assert_eq!(
            fs::read_to_string(dir.join("syntax_theme_dark")).unwrap(),
            "\"Catppuccin Mocha\""
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn apply_refuses_to_overwrite_unreadable_schemes() {
        let dir = tmp("bad");
        fs::create_dir_all(&dir).unwrap();
        write(&dir.join("color_schemes_light"), "not ron").unwrap();
        assert!(apply("latte", &dir).is_err());
        assert_eq!(
            fs::read_to_string(dir.join("color_schemes_light")).unwrap(),
            "not ron"
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
