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
use std::process::Command;

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

/// Monospace families installed on this machine, from fontconfig: the
/// fixed-width ones, plus Nerd Fonts (their icons are wider than a cell in
/// the non-"Mono" variants, but they are what the TUI's glyphs need).
pub fn installed_fonts() -> Vec<String> {
    let out = Command::new("fc-list")
        .args([":", "family", "spacing"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    parse_fonts(&out)
}

/// `fc-list : family spacing` lines look like `Fira Mono,Fira Mono Medium:spacing=100`.
fn parse_fonts(fc_list: &str) -> Vec<String> {
    let mut fonts: Vec<String> = fc_list
        .lines()
        .filter_map(|line| {
            let (families, rest) = line.split_once(':').unwrap_or((line, ""));
            let family = families.split(',').next()?.trim();
            let mono = rest.contains("spacing=100") || family.contains("Nerd Font");
            let junk = family.contains("Emoji") || family.contains("SignWriting");
            (mono && !junk && !family.is_empty()).then(|| family.to_string())
        })
        .collect();
    fonts.sort();
    fonts.dedup();
    fonts
}

/// Sets COSMIC Terminal's font (`font_name`, and `font_size` when given).
pub fn set_font(dir: &Path, name: &str, size: Option<u16>) -> Result<(), String> {
    if name.is_empty() || name.chars().any(char::is_control) {
        return Err("font name is empty or has control characters".into());
    }
    // Skipped when fontconfig is missing: then nothing can be checked.
    let installed = installed_fonts();
    if !installed.is_empty() && !installed.iter().any(|f| f == name) {
        return Err(format!(
            "font '{name}' is not installed (see: ctp-cosmic terminal font list)"
        ));
    }
    if size.is_some_and(|s| !(4..=96).contains(&s)) {
        return Err("font size must be between 4 and 96".into());
    }
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    write(&dir.join("font_name"), &crate::folders::quote(name))?;
    match size {
        Some(s) => write(&dir.join("font_size"), &s.to_string()),
        None => Ok(()),
    }
}

const USAGE: &str = "\
ctp-cosmic terminal: Catppuccin colors and font for COSMIC Terminal

Usage:
  ctp-cosmic terminal apply FLAVOR [--config DIR]
  ctp-cosmic terminal check FLAVOR [--config DIR]
  ctp-cosmic terminal font list                      installed monospace and Nerd Fonts
  ctp-cosmic terminal font set NAME [--size N] [--config DIR]
  ctp-cosmic terminal font install NAME [--set]      JetBrainsMono, FiraCode or Hack
                                                     (Nerd Font Mono, for you only)

--config DIR   cosmic-config folder (default: ~/.config/cosmic/com.system76.CosmicTerm/v1)
";

pub fn main(args: Vec<String>) -> Result<u8, String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["font", "list"] => {
            installed_fonts().iter().for_each(|f| println!("{f}"));
            return Ok(0);
        }
        ["font", "set", name, ref rest @ ..] => return font_set(name, rest),
        ["font", "install", name, ref rest @ ..] => {
            let family = crate::nerdfont::install(name)?;
            println!("{family}");
            if rest.contains(&"--set") {
                let dir = default_config_dir()?;
                set_font(&dir, &family, None)?;
            }
            return Ok(0);
        }
        _ => {}
    }
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

fn font_set(name: &str, rest: &[&str]) -> Result<u8, String> {
    let (mut size, mut dir) = (None, None);
    let mut it = rest.iter();
    while let Some(&flag) = it.next() {
        match (flag, it.next()) {
            ("--size", Some(v)) => {
                size = Some(v.parse::<u16>().map_err(|_| format!("bad --size '{v}'"))?)
            }
            ("--config", Some(d)) => dir = Some(PathBuf::from(d)),
            _ => return Err(format!("bad arguments\n\n{USAGE}")),
        }
    }
    let dir = match dir {
        Some(d) => d,
        None => default_config_dir()?,
    };
    set_font(&dir, name, size).map(|()| 0)
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
    fn fonts_are_parsed_from_fc_list() {
        let out = "Fira Mono,Fira Mono Medium:spacing=100\n\
                   DejaVu Sans:spacing=0\n\
                   Noto Color Emoji:spacing=100\n\
                   JetBrainsMono Nerd Font:spacing=90\n\
                   Fira Mono:spacing=100\n";
        assert_eq!(parse_fonts(out), ["Fira Mono", "JetBrainsMono Nerd Font"]);
    }

    #[test]
    fn font_is_written_as_cosmic_reads_it() {
        let dir = tmp("font");
        // An unknown font is refused whenever fontconfig can list fonts.
        if installed_fonts().is_empty() {
            set_font(&dir, "Anything", Some(13)).unwrap();
            assert_eq!(
                fs::read_to_string(dir.join("font_name")).unwrap(),
                "\"Anything\""
            );
            assert_eq!(fs::read_to_string(dir.join("font_size")).unwrap(), "13");
        } else {
            assert!(set_font(&dir, "No Such Font 123", None).is_err());
            let first = installed_fonts().remove(0);
            set_font(&dir, &first, None).unwrap();
            assert!(!dir.join("font_size").exists());
        }
        assert!(set_font(&dir, "bad\nname", None).is_err());
        assert!(set_font(&dir, "x", Some(2)).is_err());
        let _ = fs::remove_dir_all(&dir);
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
