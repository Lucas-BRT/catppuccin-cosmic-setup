//! `ctp-cosmic doctor` (is everything `apply` needs here?) and `ctp-cosmic
//! status` (is what the last `apply` chose still in place?).

use std::fs;
use std::path::Path;

use crate::apply::{Opts, home, load_state, which};
use crate::palette::Palette;
use crate::ui;
use crate::{icons, prompt, terminal, zed};

const DOCTOR_USAGE: &str = "\
ctp-cosmic doctor: check that what apply needs is on this machine

Usage:
  ctp-cosmic doctor

Exits 1 if something required is missing.
";

const STATUS_USAGE: &str = "\
ctp-cosmic status: is what the last apply chose still in place?

Usage:
  ctp-cosmic status [--fix]

Checks each step of your saved choices against what is on disk now (the
desktop theme, terminal colors, prompt, Zed, icons, wallpaper). Exits 1 if
something drifted. With --fix, reapplies just the steps that drifted.
";

pub fn doctor(args: Vec<String>) -> Result<u8, String> {
    if !args.is_empty() {
        print!("{DOCTOR_USAGE}");
        return Ok(u8::from(
            !matches!(args[..], [ref h] if h == "-h" || h == "--help"),
        ));
    }
    let home = home()?;
    // (found, required, what it is for, how to fix it)
    let mut checks: Vec<(String, bool, bool, &str, &str)> = Vec::new();
    for (bin, required, why, fix) in [
        (
            "cosmic-settings",
            true,
            "the cosmic step applies the desktop theme with it",
            "needs the COSMIC desktop (Pop!_OS 24.04)",
        ),
        (
            "date",
            true,
            "names the backup folders",
            "install coreutils",
        ),
        (
            "curl",
            false,
            "icons step and update download with it",
            "sudo apt install curl",
        ),
        (
            "tar",
            false,
            "icons step unpacks with it",
            "sudo apt install tar",
        ),
        (
            "sha256sum",
            false,
            "update checks downloads with it",
            "install coreutils",
        ),
        (
            "fc-list",
            false,
            "lists the fonts for the terminal font choice",
            "sudo apt install fontconfig",
        ),
        (
            "gtk-update-icon-cache",
            false,
            "speeds up icon lookups (optional)",
            "sudo apt install libgtk-3-bin",
        ),
        (
            "zed",
            false,
            "the zed step skips itself without Zed",
            "install Zed from zed.dev",
        ),
    ] {
        let bins: &[&str] = if bin == "zed" {
            &["zed", "zeditor"]
        } else {
            &[bin]
        };
        checks.push((bin.into(), which(&home, bins), required, why, fix));
    }
    checks.push((
        "~/.config/cosmic".into(),
        home.join(".config/cosmic").is_dir(),
        true,
        "COSMIC's configuration",
        "log in to a COSMIC session once",
    ));
    checks.push((
        "Cosmic icon theme".into(),
        icons::cosmic_controls().is_some(),
        false,
        "keeps COSMIC's window buttons with Papirus",
        "install the cosmic-icons package",
    ));
    let in_path = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .any(|d| d == home.join(".local/bin"));
    checks.push((
        "~/.local/bin in PATH".into(),
        in_path,
        false,
        "run ctp-cosmic by name",
        "add it to PATH in ~/.bashrc",
    ));

    let mut missing = false;
    for (name, found, required, why, fix) in &checks {
        match (found, required) {
            (true, _) => ui::ok(&format!("{name}: {why}")),
            (false, true) => {
                missing = true;
                println!("\x1b[38;2;243;139;168m ✗\x1b[0m {name}: {why} — {fix}");
            }
            (false, false) => ui::warn(&format!("{name} not found: {why} — {fix}")),
        }
    }
    Ok(u8::from(missing))
}

/// One line of `status`: what was checked and whether it holds.
pub struct Line {
    pub step: String,
    pub ok: bool,
    pub text: String,
}

fn line(ok: bool, text: impl Into<String>) -> Line {
    Line {
        step: String::new(),
        ok,
        text: text.into(),
    }
}

/// Checks `o`'s choices against the files under `home`. Only steps in
/// `o.steps` are checked.
pub fn check(o: &Opts, home: &Path) -> Vec<Line> {
    let Some(p) = Palette::new(&o.flavor) else {
        return vec![line(false, format!("unknown flavor '{}'", o.flavor))];
    };
    let name = format!("Catppuccin {}", p.name());
    let cosmic = home.join(".config/cosmic");
    let read = |path: &Path| fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::new();
    for step in o.steps.iter().map(String::as_str) {
        let mut checked = match step {
            "cosmic" => {
                // One mode's theme: its accent and its base color are in place.
                let themed = |flavor: &str| {
                    let Some(p) = Palette::new(flavor) else {
                        return false;
                    };
                    let mode = if p.is_dark() { "Dark" } else { "Light" };
                    let builder =
                        cosmic.join(format!("com.system76.CosmicTheme.{mode}.Builder/v2"));
                    let accent = format!("#{}FF", p.hex(&o.accent).to_uppercase());
                    let base = format!("#{}FF", p.hex("base").to_uppercase());
                    read(&builder.join("accent")).contains(&accent)
                        && read(&builder.join("palette")).contains(&base)
                };
                let mut what = format!("{name}, {} accent", o.accent);
                let mut ok = themed(&o.flavor);
                if let Some(light) = &o.light_flavor {
                    ok &= themed(light);
                    what.push_str(&format!(" + Catppuccin {light} in light mode"));
                }
                line(ok, format!("desktop theme: {what}"))
            }
            "terminal" => line(
                terminal::check(&o.flavor, &cosmic.join("com.system76.CosmicTerm/v1"))
                    .unwrap_or(false),
                format!("terminal colors: {name}"),
            ),
            "bash" => line(
                prompt::installed(home, &o.accent, &p),
                format!("bash prompt: {name}, {} accent", o.accent),
            ),
            "zed" => {
                if which(home, &["zed", "zeditor"]) {
                    line(
                        zed::check(&o.flavor, &home.join(".config/zed/settings.json"))
                            .unwrap_or(false),
                        format!("zed theme and icons: {name}"),
                    )
                } else {
                    line(true, "zed: not installed (skipped)")
                }
            }
            "icons" => {
                let want = if p.is_dark() {
                    "Papirus-Dark-Catppuccin"
                } else {
                    "Papirus-Catppuccin"
                };
                let now = read(&cosmic.join("com.system76.CosmicTk/v1/icon_theme"));
                // Without COSMIC's own icons the step selects plain Papirus.
                let ok = now.contains(want) || now.contains(want.trim_end_matches("-Catppuccin"));
                line(ok, format!("icons: {want}"))
            }
            "wallpaper" => {
                let all = read(&cosmic.join("com.system76.CosmicBackground/v1/all"));
                line(
                    all.contains(&format!("catppuccin-{}-{}", o.flavor, o.accent)),
                    format!("wallpaper: catppuccin-{}-{}", o.flavor, o.accent),
                )
            }
            other => line(false, format!("unknown step '{other}'")),
        };
        checked.step = step.to_string();
        out.push(checked);
    }
    out
}

pub fn status(args: Vec<String>) -> Result<u8, String> {
    let fix = args.iter().any(|a| a == "--fix");
    if args.iter().any(|a| a != "--fix") {
        print!("{STATUS_USAGE}");
        return Ok(u8::from(
            !matches!(args[..], [ref h] if h == "-h" || h == "--help"),
        ));
    }
    let o = load_state()?;
    println!("last apply: {} {}", o.flavor, o.accent);
    let lines = check(&o, &home()?);
    let drifted: Vec<String> = lines
        .iter()
        .filter(|l| !l.ok)
        .map(|l| l.step.clone())
        .collect();
    for l in &lines {
        if l.ok {
            ui::ok(&l.text);
        } else {
            println!(
                "\x1b[38;2;243;139;168m ✗\x1b[0m {} (changed or not applied)",
                l.text
            );
        }
    }
    if drifted.is_empty() {
        return Ok(0);
    }
    if !fix {
        println!(
            "Reapply with: ctp-cosmic apply --last   (or just these: ctp-cosmic status --fix)"
        );
        return Ok(1);
    }
    println!();
    println!("Reapplying: {}", drifted.join(", "));
    let partial = Opts {
        steps: drifted,
        ..o
    };
    crate::apply::run(&partial, true)?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_sees_what_apply_wrote_and_what_drifted() {
        let home = std::env::temp_dir().join(format!("ctp-health-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let o = Opts {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            steps: vec!["terminal".into(), "wallpaper".into(), "bash".into()],
            ..Opts::default()
        };
        assert!(
            check(&o, &home).iter().all(|l| !l.ok),
            "nothing applied yet"
        );

        terminal::apply(
            "mocha",
            &home.join(".config/cosmic/com.system76.CosmicTerm/v1"),
        )
        .unwrap();
        crate::background::set(
            &home.join(".config/cosmic"),
            &home
                .join("Pictures/Wallpapers/catppuccin-mocha-mauve.png")
                .display()
                .to_string(),
            None,
        )
        .unwrap();
        let lines = check(&o, &home);
        assert!(
            lines[0].ok && lines[1].ok && !lines[2].ok,
            "prompt still missing"
        );

        // Another flavor is a drift.
        let latte = Opts {
            flavor: "latte".into(),
            ..o
        };
        assert!(!check(&latte, &home)[0].ok);
        fs::remove_dir_all(&home).unwrap();
    }
}
