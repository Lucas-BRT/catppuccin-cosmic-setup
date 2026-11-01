//! `ctp-cosmic apply`: puts the chosen Catppuccin flavor and accent on the
//! desktop, terminal, prompt, Zed, icons and wallpaper. With no steps and no
//! options, in a terminal, it opens the interactive setup. Safe to run any
//! number of times: each step overwrites only what it installed itself.
//!
//! Options come from flags or from the environment (flags win); `--last`
//! starts from the choices of the last successful run.

use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::palette::{self, Palette};
use crate::steps::{self, STEPS};
use crate::{
    background, backup, folders, icons, prompt, styles, terminal, theme, ui, wallpaper, zed,
};

const PAPIRUS: (&str, &str) = (
    "PapirusDevelopmentTeam/papirus-icon-theme",
    "bf539287ef5dc18529424a02cccee76175920a6f",
);
const CTP_FOLDERS: (&str, &str) = (
    "catppuccin/papirus-folders",
    "f83671d17ea67e335b34f8028a7e6d78bca735d7",
);

/// What to apply. Saved after a successful run (without the run-only flags).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Opts {
    pub flavor: String,
    pub accent: String,
    /// "auto" keeps COSMIC's current Frosted Glass; "off"; or a blur strength.
    pub frosted: String,
    pub frost_on: Option<String>,
    pub alpha_offset: f64,
    /// A wallpaper style, or "all".
    pub style: String,
    pub rotate: Option<String>,
    pub font: Option<String>,
    pub font_size: Option<u16>,
    pub steps: Vec<String>,
    #[serde(skip)]
    pub no_apply: bool,
    #[serde(skip)]
    pub no_backup: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            frosted: "auto".into(),
            frost_on: None,
            alpha_offset: 0.0,
            style: "glow".into(),
            rotate: None,
            font: None,
            font_size: None,
            steps: STEPS
                .iter()
                .filter(|s| !s.optional)
                .map(|s| s.name.to_string())
                .collect(),
            no_apply: false,
            no_backup: false,
        }
    }
}

pub struct Parsed {
    pub opts: Opts,
    /// Anything chosen on the command line or in the environment: no interface.
    pub explicit: bool,
}

const ENV: [&str; 9] = [
    "FLAVOR",
    "ACCENT",
    "FROSTED",
    "FROST_ON",
    "ALPHA_OFFSET",
    "WALLPAPER_STYLE",
    "WALLPAPER_ROTATE",
    "TERMINAL_FONT",
    "TERMINAL_FONT_SIZE",
];

fn set(o: &mut Opts, key: &str, v: &str) -> Result<(), String> {
    match key {
        "flavor" => o.flavor = v.into(),
        "accent" => o.accent = v.into(),
        "frosted" => o.frosted = v.into(),
        "frost-on" => o.frost_on = Some(v.into()),
        "alpha-offset" => {
            o.alpha_offset = v
                .parse()
                .map_err(|_| format!("invalid alpha offset '{v}'"))?
        }
        "wallpaper-style" => o.style = v.into(),
        "wallpaper-rotate" => o.rotate = Some(v.into()),
        "font" => o.font = Some(v.into()),
        "font-size" => {
            o.font_size = Some(v.parse().map_err(|_| format!("invalid font size '{v}'"))?)
        }
        _ => unreachable!("{key}"),
    }
    Ok(())
}

/// Defaults, then `last`, then the environment, then the flags.
pub fn parse(
    args: &[String],
    env: &dyn Fn(&str) -> Option<String>,
    last: Option<Opts>,
) -> Result<Parsed, String> {
    let mut explicit = last.is_some();
    let mut o = last.unwrap_or_default();
    let pairs = ENV.iter().zip([
        "flavor",
        "accent",
        "frosted",
        "frost-on",
        "alpha-offset",
        "wallpaper-style",
        "wallpaper-rotate",
        "font",
        "font-size",
    ]);
    for (var, key) in pairs {
        if let Some(v) = env(var).filter(|v| !v.is_empty()) {
            set(&mut o, key, &v)?;
            explicit = true;
        }
    }
    for (var, flag) in [("NO_APPLY", 0), ("NO_BACKUP", 1)] {
        if env(var).as_deref() == Some("1") {
            explicit = true;
            if flag == 0 {
                o.no_apply = true
            } else {
                o.no_backup = true
            }
        }
    }

    let mut named = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--last" => {}
            "--no-apply" => (o.no_apply, explicit) = (true, true),
            "--no-backup" => (o.no_backup, explicit) = (true, true),
            flag if flag.starts_with("--") => {
                let key = &flag[2..];
                let v = it.next().ok_or(format!("{flag} needs a value"))?;
                if !matches!(
                    key,
                    "flavor"
                        | "accent"
                        | "frosted"
                        | "frost-on"
                        | "alpha-offset"
                        | "wallpaper-style"
                        | "wallpaper-rotate"
                        | "font"
                        | "font-size"
                ) {
                    return Err(format!("unknown option '{flag}' (see --help)"));
                }
                set(&mut o, key, v)?;
                explicit = true;
            }
            step => named.push(step.to_string()),
        }
    }
    if !named.is_empty() {
        o.steps = named;
        explicit = true;
    }
    validate(&o)?;
    Ok(Parsed { opts: o, explicit })
}

/// Rejects bad values before anything is changed.
pub fn validate(o: &Opts) -> Result<(), String> {
    palette::check(&o.flavor, Some(&o.accent))?;
    let names: Vec<_> = styles::list().iter().map(|(n, _)| *n).collect();
    if o.style != "all" && !names.contains(&o.style.as_str()) {
        return Err(format!(
            "unknown wallpaper style '{}' (choose: {}, all)",
            o.style,
            names.join(", ")
        ));
    }
    if let Some(r) = &o.rotate {
        steps::rotate_seconds(r)?;
        if o.style != "all" {
            return Err(
                "wallpaper rotation needs wallpaper style 'all' (a slideshow needs several images)"
                    .into(),
            );
        }
    }
    let frost_on: Option<Vec<String>> = o.frost_on.as_ref().map(|l| {
        l.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });
    theme::check_frost(&o.frosted, frost_on.as_deref())?;
    if let Some(unknown) = o
        .steps
        .iter()
        .find(|s| !STEPS.iter().any(|k| k.name == **s))
    {
        return Err(format!("unknown step '{unknown}' (see --list)"));
    }
    Ok(())
}

// --- paths ------------------------------------------------------------------

pub fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME not set".to_string())
}

pub fn cache_dir() -> Result<PathBuf, String> {
    let base = match std::env::var_os("XDG_CACHE_HOME") {
        Some(c) if !c.is_empty() => PathBuf::from(c),
        _ => home()?.join(".cache"),
    };
    Ok(base.join("catppuccin-cosmic-setup"))
}

/// Where the last successful choices are kept.
pub fn state_dir() -> Result<PathBuf, String> {
    Ok(home()?.join(".config/catppuccin-cosmic-setup"))
}

pub fn load_state() -> Result<Opts, String> {
    let file = state_dir()?.join("state.ron");
    let text = fs::read_to_string(&file)
        .map_err(|_| "no saved choices yet; run `ctp-cosmic apply` first".to_string())?;
    ron::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))
}

fn save_state(o: &Opts) -> Result<(), String> {
    let dir = state_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let text = ron::ser::to_string_pretty(o, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    let file = dir.join("state.ron");
    fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))
}

// --- downloads --------------------------------------------------------------

/// Downloads a GitHub repository at an exact commit into `dest`, unless it
/// already holds that commit.
fn fetch_pinned((slug, rev): (&str, &str), dest: &Path) -> Result<(), String> {
    let marker = dest.join(".pinned-rev");
    if fs::read_to_string(&marker).is_ok_and(|r| r.trim() == rev) {
        return Ok(());
    }
    let _ = fs::remove_dir_all(dest);
    fs::create_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    let url = format!("https://codeload.github.com/{slug}/tar.gz/{rev}");
    let mut curl = Command::new("curl")
        .args(["-fsSL", "--proto", "=https", "--tlsv1.2", &url])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("curl: {e}"))?;
    let tar = Command::new("tar")
        .args(["-xz", "-C"])
        .arg(dest)
        .arg("--strip-components=1")
        .stdin(curl.stdout.take().ok_or("curl: no output")?)
        .status()
        .map_err(|e| format!("tar: {e}"))?;
    let curl_ok = curl.wait().is_ok_and(|s| s.success());
    if !(tar.success() && curl_ok) {
        let _ = fs::remove_dir_all(dest);
        return Err(format!("could not download {url}"));
    }
    fs::write(&marker, format!("{rev}\n")).map_err(|e| format!("{}: {e}", marker.display()))
}

// --- steps ------------------------------------------------------------------

fn step_backup() -> Result<PathBuf, String> {
    let stamp = Command::new("date")
        .arg("+%F-%H%M%S")
        .output()
        .map_err(|e| format!("date: {e}"))?;
    let base = home()?.join(format!(
        ".config/catppuccin-backup-{}",
        String::from_utf8_lossy(&stamp.stdout).trim()
    ));
    // Two runs in the same second must not share a folder.
    let mut dest = base.clone();
    for n in 2.. {
        if !dest.exists() {
            break;
        }
        dest = PathBuf::from(format!("{}-{n}", base.display()));
    }
    ui::info(&format!("Backing up to {}", dest.display()));
    backup::create(&dest)?;
    // The current theme as a standalone .ron that can be re-imported:
    //   cosmic-settings appearance import <backup>/previous-theme.ron
    let _ = Command::new("cosmic-settings")
        .args(["appearance", "export"])
        .arg(dest.join("previous-theme.ron"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    ui::ok("Backup saved");
    Ok(dest)
}

fn step_cosmic(o: &Opts) -> Result<(), String> {
    let home = home()?;
    ui::info(&format!("COSMIC theme ({}, {} accent)", o.flavor, o.accent));
    let (label, theme) = theme::install_theme(&theme::InstallOpts {
        flavor: o.flavor.clone(),
        accent: o.accent.clone(),
        alpha_offset: o.alpha_offset,
        frosted: o.frosted.clone(),
        frost_on: o.frost_on.as_ref().map(|l| {
            l.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }),
        apply: !o.no_apply,
        config: home.join(".config/cosmic"),
        data: home.join(".local/share/catppuccin/themes/cosmic-settings"),
    })?;
    ui::ok(&format!("Style: {label}"));
    ui::ok(&format!(
        "Themes generated in {}",
        theme.parent().unwrap_or(Path::new("")).display()
    ));
    if o.no_apply {
        ui::todo(&format!(
            "no-apply: to apply, run  cosmic-settings appearance import {}",
            theme.display()
        ));
    } else {
        ui::ok(&format!(
            "Theme applied ({})",
            theme.file_name().unwrap_or_default().to_string_lossy()
        ));
    }
    Ok(())
}

fn step_terminal(o: &Opts) -> Result<(), String> {
    ui::info("COSMIC Terminal");
    terminal::apply(&o.flavor, &terminal::default_config_dir()?)?;
    let p = Palette::new(&o.flavor).ok_or("unknown flavor")?;
    ui::ok(&format!("Terminal colors: Catppuccin {}", p.name()));
    if let Some(font) = &o.font {
        terminal::set_font(&terminal::default_config_dir()?, font, o.font_size)?;
        let size = o.font_size.map(|s| format!(" ({s})")).unwrap_or_default();
        ui::ok(&format!("Terminal font: {font}{size}"));
    }
    Ok(())
}

fn step_bash(o: &Opts) -> Result<(), String> {
    ui::info(&format!("Bash prompt ({}, {} accent)", o.flavor, o.accent));
    prompt::install(&o.flavor, &o.accent)?;
    ui::ok("Prompt installed (open a new terminal)");
    Ok(())
}

fn zed_installed(home: &Path) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs: Vec<PathBuf> = vec![home.join(".local/bin")];
    dirs.extend(std::env::split_paths(&path));
    dirs.iter()
        .any(|d| ["zed", "zeditor"].iter().any(|b| d.join(b).is_file()))
}

fn step_zed(o: &Opts) -> Result<(), String> {
    let home = home()?;
    ui::info("Zed");
    if !zed_installed(&home) {
        ui::warn("Zed not found, skipping");
        return Ok(());
    }
    // Extensions can only be installed from Zed itself (Ctrl+Shift+X).
    let ext = home.join(".local/share/zed/extensions/installed");
    for e in ["catppuccin", "catppuccin-icons"] {
        if ext.join(e).is_dir() {
            ui::ok(&format!("Extension {e} installed"));
        } else {
            ui::todo(&format!(
                "Install the \"{e}\" extension in Zed (Ctrl+Shift+X)"
            ));
        }
    }
    zed::apply(&o.flavor, &zed::default_settings_path()?)?;
    let name = format!(
        "Catppuccin {}",
        Palette::new(&o.flavor).ok_or("unknown flavor")?.name()
    );
    ui::ok(&format!("Editor theme: {name}"));
    ui::ok(&format!("Editor icons: {name}"));
    Ok(())
}

fn step_icons(o: &Opts) -> Result<(), String> {
    let home = home()?;
    let cache = cache_dir()?;
    ui::info(&format!(
        "Papirus icons (cat-{}-{} folders)",
        o.flavor, o.accent
    ));
    fetch_pinned(PAPIRUS, &cache.join("papirus"))?;
    fetch_pinned(CTP_FOLDERS, &cache.join("ctp-folders"))?;
    let theme = icons::apply(
        &o.flavor,
        &o.accent,
        &icons::Paths {
            papirus: cache.join("papirus"),
            folders: cache.join("ctp-folders"),
            icons: home.join(".local/share/icons"),
            config: home.join(".config/cosmic"),
        },
    )?;
    ui::ok(&format!("Icons applied ({theme})"));
    Ok(())
}

fn step_wallpaper(o: &Opts) -> Result<(), String> {
    let home = home()?;
    let config = home.join(".config/cosmic");
    let pictures = home.join("Pictures/Wallpapers");
    if o.style != "all" {
        ui::info(&format!(
            "Wallpaper ({}, {} accent, {})",
            o.flavor, o.accent, o.style
        ));
        // The default "glow" keeps the plain catppuccin-<flavor>-<accent>.png name.
        let suffix = if o.style == "glow" {
            String::new()
        } else {
            format!("-{}", o.style)
        };
        let file = pictures.join(format!("catppuccin-{}-{}{suffix}.png", o.flavor, o.accent));
        fs::create_dir_all(&pictures).map_err(|e| format!("{}: {e}", pictures.display()))?;
        wallpaper::render_to(&file, &o.style, &o.flavor, &o.accent)?;
        background::set(&config, &file.display().to_string(), None)?;
        ui::ok(&format!("Background: {}", file.display()));
        return Ok(());
    }

    ui::info(&format!(
        "Wallpapers: every style ({}, {} accent)",
        o.flavor, o.accent
    ));
    let dir = pictures.join(format!("catppuccin-{}-{}", o.flavor, o.accent));
    wallpaper::render_all(&dir, &o.flavor, &o.accent, (2560, 1440))?;
    ui::ok(&format!(
        "{} wallpapers in {}",
        styles::list().len(),
        dir.display()
    ));
    // The same keys COSMIC Settings writes: recent-folders (at most five) and
    // current-folder, the one the picker opens on.
    folders::register(
        &config.join("com.system76.CosmicSettings.Wallpaper/v1"),
        &dir.display().to_string(),
    )?;
    ui::ok("Folder added to Settings → Wallpaper");
    match &o.rotate {
        Some(r) => {
            background::set(
                &config,
                &dir.display().to_string(),
                Some(steps::rotate_seconds(r)?),
            )?;
            ui::ok(&format!(
                "Background: slideshow of {} every {r}",
                dir.display()
            ));
        }
        None => {
            ui::todo("Background unchanged. Pick an image or turn on the slideshow in");
            ui::todo("Settings → Wallpaper, or rerun with --wallpaper-rotate 30m (5m ... 2h).");
        }
    }
    Ok(())
}

/// Runs the chosen steps in installer order. The backup is sealed even if a
/// step fails, so `uninstall` can still undo what was done.
pub fn run(o: &Opts) -> Result<(), String> {
    let backup_dir = if o.no_backup {
        None
    } else {
        Some(step_backup()?)
    };
    let result = STEPS
        .iter()
        .filter(|s| o.steps.iter().any(|n| n == s.name))
        .try_for_each(|s| match s.name {
            "cosmic" => step_cosmic(o),
            "terminal" => step_terminal(o),
            "bash" => step_bash(o),
            "zed" => step_zed(o),
            "icons" => step_icons(o),
            "wallpaper" => step_wallpaper(o),
            other => Err(format!("unknown step '{other}'")),
        });
    if let Some(dir) = &backup_dir
        && let Err(e) = backup::seal(dir)
    {
        ui::warn(&format!(
            "Could not record the changes ({e}); uninstall cannot undo this run"
        ));
    }
    result?;
    save_state(o)?;
    println!();
    println!("Done. If icons or the wallpaper don't change right away, log out and back in.");
    Ok(())
}

const USAGE: &str = "\
ctp-cosmic apply: put Catppuccin on the desktop, terminal, prompt, Zed,
icons and wallpaper

Usage:
  ctp-cosmic [apply]                  in a terminal: the interactive setup
  ctp-cosmic apply [STEP...] [OPTIONS]   no questions; every step but icons by default
  ctp-cosmic apply --last             repeat the last successful choices
  ctp-cosmic apply --list             list the steps

Steps: cosmic, terminal, bash, zed, wallpaper, and icons (only when named).

Options (each also an environment variable; flags win):
  --flavor NAME            FLAVOR            latte, frappe, macchiato, mocha (mocha)
  --accent NAME            ACCENT            any of the 14 accents (mauve)
  --frosted auto|off|STR   FROSTED           Frosted Glass: keep the current setting
                                             (auto), off, or a strength such as medium
  --frost-on LIST          FROST_ON          windows,system_interface,panel,applets,
                                             maximized_apps
  --alpha-offset N         ALPHA_OFFSET      0.1 more opaque, -0.1 more transparent
  --wallpaper-style NAME   WALLPAPER_STYLE   glow, waves, ... or all (see `wallpaper --help`)
  --wallpaper-rotate T     WALLPAPER_ROTATE   with style all: 5m 10m 15m 30m 1h 2h
  --font NAME              TERMINAL_FONT      COSMIC Terminal font (`terminal font list`)
  --font-size N            TERMINAL_FONT_SIZE
  --no-apply               NO_APPLY=1         only generate the desktop theme
  --no-backup              NO_BACKUP=1        skip the backup
                           NO_TUI=1           never open the interactive setup

What the run changed is recorded; `ctp-cosmic uninstall` undoes it.
";

fn interactive() -> bool {
    std::io::stdout().is_terminal() && fs::File::open("/dev/tty").is_ok()
}

pub fn main(args: Vec<String>) -> Result<u8, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return Ok(0);
    }
    if args.iter().any(|a| a == "--list") {
        for s in &STEPS {
            let note = if s.optional {
                "  [optional, run only when named]"
            } else {
                ""
            };
            println!("{}  ({}){note}", s.name, s.label);
        }
        return Ok(0);
    }
    let last = args
        .iter()
        .any(|a| a == "--last")
        .then(load_state)
        .transpose()?;
    let env = |k: &str| std::env::var(k).ok();
    let parsed = parse(&args, &env, last)?;
    if !parsed.explicit && env("NO_TUI").as_deref() != Some("1") && interactive() {
        return crate::tui::main(Vec::new());
    }
    run(&parsed.opts)?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }
    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn nothing_given_is_not_explicit_and_skips_icons() {
        let p = parse(&[], &no_env, None).unwrap();
        assert!(!p.explicit);
        assert_eq!(
            p.opts.steps,
            ["cosmic", "terminal", "bash", "zed", "wallpaper"]
        );
        assert_eq!(
            (p.opts.flavor.as_str(), p.opts.accent.as_str()),
            ("mocha", "mauve")
        );
    }

    #[test]
    fn flags_beat_environment_beat_last() {
        let last = Opts {
            flavor: "latte".into(),
            accent: "blue".into(),
            ..Opts::default()
        };
        let env = |k: &str| (k == "ACCENT").then(|| "red".to_string());
        let p = parse(&args(&["--flavor", "frappe"]), &env, Some(last.clone())).unwrap();
        assert_eq!(p.opts.flavor, "frappe");
        assert_eq!(p.opts.accent, "red");
        assert!(p.explicit);
        // --last alone reuses everything and is explicit (no interface).
        let p = parse(&args(&["--last"]), &no_env, Some(last.clone())).unwrap();
        assert_eq!(p.opts, last);
        assert!(p.explicit);
    }

    #[test]
    fn steps_named_run_only_those_and_icons_by_name() {
        let p = parse(&args(&["bash", "icons"]), &no_env, None).unwrap();
        assert_eq!(p.opts.steps, ["bash", "icons"]);
        assert!(p.explicit);
    }

    #[test]
    fn bad_values_are_rejected_up_front() {
        for bad in [
            args(&["--flavor", "mocca"]),
            args(&["--accent", "pink2"]),
            args(&["--wallpaper-style", "nope"]),
            args(&["--wallpaper-rotate", "30m"]), // needs style all
            args(&["--frosted", "bogus"]),
            args(&["--frost-on", "sidebar", "--frosted", "medium"]),
            args(&["nosuchstep"]),
            args(&["--wat", "1"]),
            args(&["--flavor"]),
        ] {
            assert!(parse(&bad, &no_env, None).is_err(), "{bad:?}");
        }
        assert!(
            parse(
                &args(&["--wallpaper-style", "all", "--wallpaper-rotate", "30m"]),
                &no_env,
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn state_round_trips_without_run_only_flags() {
        let o = Opts {
            flavor: "macchiato".into(),
            font: Some("Fira Mono".into()),
            font_size: Some(13),
            no_apply: true,
            ..Opts::default()
        };
        let text = ron::ser::to_string_pretty(&o, ron::ser::PrettyConfig::default()).unwrap();
        let back: Opts = ron::from_str(&text).unwrap();
        assert_eq!(
            back,
            Opts {
                no_apply: false,
                ..o
            }
        );
        // An older or partial file still loads.
        let partial: Opts = ron::from_str("(flavor: \"latte\")").unwrap();
        assert_eq!(partial.flavor, "latte");
        assert_eq!(partial.accent, "mauve");
    }
}
