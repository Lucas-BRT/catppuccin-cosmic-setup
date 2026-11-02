//! Named sets of choices: `apply --save work`, `apply --profile work`, and
//! `ctp-cosmic profile ...` to list, show, export (for dotfiles) and import.

use std::fs;
use std::path::{Path, PathBuf};

use crate::apply::{self, Opts, state_dir};

const USAGE: &str = "\
ctp-cosmic profile: named sets of choices

Usage:
  ctp-cosmic profile list
  ctp-cosmic profile show NAME
  ctp-cosmic profile delete NAME
  ctp-cosmic profile export NAME FILE     copy a profile out (to keep in your dotfiles)
  ctp-cosmic profile import FILE [NAME]   add a profile from a file (NAME defaults to its file name)

Save one with `ctp-cosmic apply ... --save NAME` (after a successful run) and use
it with `ctp-cosmic apply --profile NAME`.
";

fn dir() -> Result<PathBuf, String> {
    Ok(state_dir()?.join("profiles"))
}

/// Letters, digits, `-`, `_` and `.`, not starting with a dot.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn path(name: &str) -> Result<PathBuf, String> {
    if !valid_name(name) {
        return Err(format!(
            "bad profile name '{name}' (letters, digits, - _ . only)"
        ));
    }
    Ok(dir()?.join(format!("{name}.ron")))
}

pub fn save(name: &str, o: &Opts) -> Result<(), String> {
    let file = path(name)?;
    fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
    let text = ron::ser::to_string_pretty(o, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))
}

fn parse(text: &str, from: &Path) -> Result<Opts, String> {
    let o: Opts = ron::from_str(text).map_err(|e| format!("{}: {e}", from.display()))?;
    apply::validate(&o).map_err(|e| format!("{}: {e}", from.display()))?;
    Ok(o)
}

pub fn load(name: &str) -> Result<Opts, String> {
    let file = path(name)?;
    let text = fs::read_to_string(&file)
        .map_err(|_| format!("no profile '{name}' (see: ctp-cosmic profile list)"))?;
    parse(&text, &file)
}

pub fn names() -> Result<Vec<String>, String> {
    let mut out: Vec<String> = match fs::read_dir(dir()?) {
        Ok(rd) => rd
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.strip_suffix(".ron").map(String::from)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort();
    Ok(out)
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["list"] => {
            for n in names()? {
                let o = load(&n)?;
                println!("{n}: {} {} ({})", o.flavor, o.accent, o.steps.join(","));
            }
        }
        ["show", name] => print!(
            "{}",
            fs::read_to_string(path(name)?).map_err(|_| format!("no profile '{name}'"))?
        ),
        ["delete", name] => {
            let file = path(name)?;
            fs::remove_file(&file).map_err(|_| format!("no profile '{name}'"))?;
        }
        ["export", name, file] => {
            load(name)?;
            fs::copy(path(name)?, file).map_err(|e| format!("{file}: {e}"))?;
        }
        ["import", file, ref rest @ ..] if rest.len() <= 1 => {
            let text = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
            let o = parse(&text, Path::new(file))?;
            let name = match rest.first() {
                Some(n) => n.to_string(),
                None => Path::new(file)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            };
            save(&name, &o)?;
            println!("imported as '{name}'");
        }
        [] | ["-h" | "--help"] => print!("{USAGE}"),
        _ => return Err(format!("bad arguments\n\n{USAGE}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_safe_file_names() {
        for ok in ["work", "dark-1", "a.b_c"] {
            assert!(valid_name(ok), "{ok}");
        }
        for bad in ["", ".hidden", "../x", "a/b", "sp ace", "é"] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn a_profile_must_still_validate() {
        let ok = Opts {
            flavor: "latte".into(),
            ..Opts::default()
        };
        let text = ron::to_string(&ok).unwrap();
        assert_eq!(parse(&text, Path::new("x")).unwrap().flavor, "latte");
        let bad = ron::to_string(&Opts {
            flavor: "mocca".into(),
            ..Opts::default()
        })
        .unwrap();
        assert!(parse(&bad, Path::new("x")).is_err());
    }
}
