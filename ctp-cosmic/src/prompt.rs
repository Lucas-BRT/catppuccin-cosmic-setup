//! `ctp-cosmic prompt`: the bash prompt and the marked block in ~/.bashrc.

use std::fs;
use std::path::PathBuf;

use crate::palette;

const TEMPLATE: &str = include_str!("../../config/bash/catppuccin-prompt.bash.in");
const START: &str = "# >>> catppuccin-cosmic-setup >>>";
const END: &str = "# <<< catppuccin-cosmic-setup <<<";
const BLOCK: &str = "[ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash";

const USAGE: &str = "\
ctp-cosmic prompt: the Catppuccin bash prompt

Usage:
  ctp-cosmic prompt install FLAVOR ACCENT   write ~/.config/catppuccin/catppuccin-prompt.bash
                                            and add the block to ~/.bashrc
  ctp-cosmic prompt remove                  take the block out of ~/.bashrc
";

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME not set".to_string())
}

/// `text` without the marked block (and the lines inside it).
fn without_block(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        match line.trim_end_matches('\n') {
            START => inside = true,
            END if inside => inside = false,
            _ if !inside => out.push_str(line),
            _ => {}
        }
    }
    out
}

/// `text` with the block replaced by a fresh one at the end.
fn with_block(text: &str) -> String {
    let mut out = without_block(text);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("{START}\n{BLOCK}\n{END}\n"));
    out
}

fn read_bashrc(path: &PathBuf) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(t) => Ok(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

pub fn install(flavor: &str, accent: &str) -> Result<(), String> {
    let p = palette::check(flavor, Some(accent))?;
    let home = home()?;
    let dir = home.join(".config/catppuccin");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let file = dir.join("catppuccin-prompt.bash");
    fs::write(&file, palette::render(TEMPLATE, p, accent)?)
        .map_err(|e| format!("{}: {e}", file.display()))?;

    let bashrc = home.join(".bashrc");
    let text = with_block(&read_bashrc(&bashrc)?);
    fs::write(&bashrc, text).map_err(|e| format!("{}: {e}", bashrc.display()))
}

pub fn remove() -> Result<(), String> {
    let bashrc = home()?.join(".bashrc");
    let text = read_bashrc(&bashrc)?;
    if text.contains(START) {
        fs::write(&bashrc, without_block(&text))
            .map_err(|e| format!("{}: {e}", bashrc.display()))?;
    }
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["install", flavor, accent] => install(flavor, accent),
        ["remove"] => remove(),
        [] | ["-h" | "--help"] => {
            print!("{USAGE}");
            Ok(())
        }
        _ => Err(format!("bad arguments\n\n{USAGE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_is_replaced_not_duplicated() {
        let once = with_block("export A=1");
        assert_eq!(with_block(&once), once);
        assert!(once.starts_with("export A=1\n"));
        assert_eq!(without_block(&once), "export A=1\n");
    }

    #[test]
    fn rest_of_bashrc_is_kept() {
        let text = format!("a\n{START}\nold\n{END}\nb\n");
        assert_eq!(without_block(&text), "a\nb\n");
        assert_eq!(without_block("a\nb"), "a\nb");
    }

    #[test]
    fn template_renders_for_every_flavor() {
        for f in palette::flavors() {
            let p = palette::Palette::new(f).unwrap();
            let out = palette::render(TEMPLATE, p, "mauve").unwrap();
            assert!(!out.contains("@blue@"));
        }
    }
}
