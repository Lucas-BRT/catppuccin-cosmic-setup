//! Nerd Fonts for the terminal: the default font lacks many of the icons the
//! interface and prompts use. Downloads a pinned release, checks its hash,
//! and installs the Mono variants (Regular, Bold, Italic, BoldItalic) for the
//! user only.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::apply::{cache_dir, home};

const VERSION: &str = "v3.4.0";

/// (name in the release, sha256 of its .tar.xz)
const FONTS: [(&str, &str); 3] = [
    (
        "JetBrainsMono",
        "ef552a3e638f25125c6ad4c51176a6adcdce295ab1d2ffacf0db060caf8c1582",
    ),
    (
        "FiraCode",
        "d83fb093e0e05a531cd6f19886a6ceb884a4fa5ea3b53cf099fc1f30c5b3e47d",
    ),
    (
        "Hack",
        "1d00a1435638084174516975840854368a45ac30bb0bad2c0c49db713b5925f0",
    ),
];

pub fn names() -> Vec<&'static str> {
    FONTS.iter().map(|(n, _)| *n).collect()
}

fn lookup(name: &str) -> Option<(&'static str, &'static str)> {
    FONTS
        .iter()
        .copied()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
}

/// The archive members to extract: `<Name>NerdFontMono-<Style>.ttf`, exactly
/// (a loose match would also take the JetBrainsMonoNL variants).
fn pick_members(listing: &str, name: &str) -> Vec<String> {
    let want: Vec<String> = ["Regular", "Bold", "Italic", "BoldItalic"]
        .iter()
        .map(|s| format!("{name}NerdFontMono-{s}.ttf"))
        .collect();
    listing
        .lines()
        .map(|l| l.trim().trim_start_matches("./"))
        .filter(|l| want.iter().any(|w| w == l))
        .map(String::from)
        .collect()
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    match cmd.stderr(Stdio::null()).status() {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("{what} failed ({s})")),
        Err(e) => Err(format!("{what}: {e}")),
    }
}

fn sha256(path: &Path) -> Result<String, String> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| format!("sha256sum: {e}"))?;
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .map(String::from)
        .ok_or_else(|| "sha256sum printed nothing".into())
}

/// Installs the font for this user and returns its family name, the one to
/// give `terminal font set`.
pub fn install(name: &str) -> Result<String, String> {
    let (name, want_hash) = lookup(name)
        .ok_or_else(|| format!("unknown font '{name}' (choose: {})", names().join(", ")))?;
    let cache = cache_dir()?.join("fonts");
    fs::create_dir_all(&cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let archive = cache.join(format!("{name}-{VERSION}.tar.xz"));
    if sha256(&archive).ok().as_deref() != Some(want_hash) {
        let url = format!(
            "https://github.com/ryanoasis/nerd-fonts/releases/download/{VERSION}/{name}.tar.xz"
        );
        let part = archive.with_extension("part");
        run(
            Command::new("curl")
                .args(["-fsSL", "--proto", "=https", "--tlsv1.2", "-o"])
                .arg(&part)
                .arg(&url),
            &format!("download of {url}"),
        )?;
        if sha256(&part)? != want_hash {
            let _ = fs::remove_file(&part);
            return Err(format!(
                "{name}.tar.xz does not match the pinned hash; refusing to install it"
            ));
        }
        fs::rename(&part, &archive).map_err(|e| e.to_string())?;
    }

    let listing = Command::new("tar")
        .arg("-tJf")
        .arg(&archive)
        .output()
        .map_err(|e| format!("tar: {e}"))?;
    let members = pick_members(&String::from_utf8_lossy(&listing.stdout), name);
    if members.is_empty() {
        return Err(format!(
            "no {name} Nerd Font Mono files in the archive (is xz installed?)"
        ));
    }
    let dest: PathBuf = home()?.join(".local/share/fonts/NerdFonts").join(name);
    fs::create_dir_all(&dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    run(
        Command::new("tar")
            .arg("-xJf")
            .arg(&archive)
            .arg("-C")
            .arg(&dest)
            .args(&members),
        "tar",
    )?;
    // Only the lookup cache; the font works after the next login without it.
    let _ = Command::new("fc-cache")
        .arg("-f")
        .arg(&dest)
        .stderr(Stdio::null())
        .status();

    let regular = dest.join(format!("{name}NerdFontMono-Regular.ttf"));
    let family = Command::new("fc-scan")
        .args(["--format", "%{family[0]}"])
        .arg(&regular)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|f| !f.is_empty());
    Ok(family.unwrap_or_else(|| format!("{name} Nerd Font Mono")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_ignores_case_and_unknown_names_fail() {
        assert_eq!(lookup("jetbrainsmono").map(|f| f.0), Some("JetBrainsMono"));
        assert!(lookup("Comic Sans").is_none());
        assert!(FONTS.iter().all(|(_, h)| h.len() == 64));
    }

    #[test]
    fn members_are_exact() {
        let listing = "JetBrainsMonoNerdFontMono-Regular.ttf\nJetBrainsMonoNLNerdFontMono-Regular.ttf\n\
                       JetBrainsMonoNerdFontMono-Bold.ttf\nJetBrainsMonoNerdFont-Regular.ttf\nOFL.txt\n";
        assert_eq!(
            pick_members(listing, "JetBrainsMono"),
            [
                "JetBrainsMonoNerdFontMono-Regular.ttf",
                "JetBrainsMonoNerdFontMono-Bold.ttf"
            ]
        );
    }
}
