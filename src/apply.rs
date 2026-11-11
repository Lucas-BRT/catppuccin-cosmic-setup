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
    background, backup, folders, gtk, icons, prompt, styles, terminal, theme, ui, wallpaper, zed,
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
    /// With a dark `flavor`: the flavor for COSMIC's light mode (latte), and
    /// the desktop then switches by itself at sunrise and sunset.
    pub light_flavor: Option<String>,
    /// round, slightlyround or square.
    pub roundness: String,
    /// "OUTER,INNER" gaps between windows.
    pub gaps: Option<String>,
    pub active_hint: Option<i64>,
    /// Color of the active window outline (default: the accent).
    pub window_hint: Option<String>,
    /// "auto" keeps COSMIC's current Frosted Glass; "off"; or a blur strength.
    pub frosted: String,
    pub frost_on: Option<String>,
    pub alpha_offset: f64,
    /// A wallpaper style, or "all".
    pub style: String,
    /// Your own PNG, recolored with the flavor and used instead of a style.
    pub wallpaper_image: Option<String>,
    pub rotate: Option<String>,
    pub font: Option<String>,
    pub font_size: Option<u16>,
    /// Install this Nerd Font first and use it as the terminal font.
    pub install_font: Option<String>,
    pub steps: Vec<String>,
    #[serde(skip)]
    pub no_apply: bool,
    #[serde(skip)]
    pub no_backup: bool,
    #[serde(skip)]
    pub dry_run: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            light_flavor: None,
            roundness: "round".into(),
            gaps: None,
            active_hint: None,
            window_hint: None,
            frosted: "auto".into(),
            frost_on: None,
            alpha_offset: 0.0,
            style: "glow".into(),
            wallpaper_image: None,
            rotate: None,
            font: None,
            font_size: None,
            install_font: None,
            steps: STEPS
                .iter()
                .filter(|s| !s.optional)
                .map(|s| s.name.to_string())
                .collect(),
            no_apply: false,
            no_backup: false,
            dry_run: false,
        }
    }
}

pub struct Parsed {
    pub opts: Opts,
    /// Anything chosen on the command line or in the environment: no interface.
    pub explicit: bool,
    /// Steps were named: a part of the usual set, not a new selection.
    pub partial: bool,
}

/// (environment variable, flag name) of every option that takes a value.
const KEYS: [(&str, &str); 16] = [
    ("FLAVOR", "flavor"),
    ("ACCENT", "accent"),
    ("LIGHT_FLAVOR", "light-flavor"),
    ("FROSTED", "frosted"),
    ("FROST_ON", "frost-on"),
    ("ALPHA_OFFSET", "alpha-offset"),
    ("ROUNDNESS", "roundness"),
    ("GAPS", "gaps"),
    ("ACTIVE_HINT", "active-hint"),
    ("WINDOW_HINT", "window-hint"),
    ("WALLPAPER_STYLE", "wallpaper-style"),
    ("WALLPAPER_ROTATE", "wallpaper-rotate"),
    ("WALLPAPER_IMAGE", "wallpaper-image"),
    ("TERMINAL_FONT", "font"),
    ("TERMINAL_FONT_SIZE", "font-size"),
    ("INSTALL_FONT", "install-font"),
];

fn set(o: &mut Opts, key: &str, v: &str) -> Result<(), String> {
    match key {
        "flavor" => o.flavor = v.into(),
        "accent" => o.accent = v.into(),
        "light-flavor" => o.light_flavor = Some(v.into()),
        "frosted" => o.frosted = v.into(),
        "frost-on" => o.frost_on = Some(v.into()),
        "roundness" => o.roundness = v.into(),
        "gaps" => o.gaps = Some(v.into()),
        "active-hint" => {
            o.active_hint = Some(
                v.parse()
                    .map_err(|_| format!("invalid active hint '{v}'"))?,
            )
        }
        "window-hint" => o.window_hint = Some(v.into()),
        "install-font" => o.install_font = Some(v.into()),
        "alpha-offset" => {
            o.alpha_offset = v
                .parse()
                .map_err(|_| format!("invalid alpha offset '{v}'"))?
        }
        "wallpaper-style" => o.style = v.into(),
        "wallpaper-image" => o.wallpaper_image = Some(v.into()),
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
    for (var, key) in KEYS {
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
            "--dry-run" => (o.dry_run, explicit) = (true, true),
            flag if flag.starts_with("--") => {
                let key = &flag[2..];
                if !KEYS.iter().any(|(_, k)| *k == key) {
                    return Err(format!("unknown option '{flag}' (see --help)"));
                }
                let v = it.next().ok_or(format!("{flag} needs a value"))?;
                set(&mut o, key, v)?;
                explicit = true;
            }
            step => named.push(step.to_string()),
        }
    }
    let partial = !named.is_empty();
    if partial {
        o.steps = named;
        explicit = true;
    }
    validate(&o)?;
    Ok(Parsed {
        opts: o,
        explicit,
        partial,
    })
}

/// Rejects bad values before anything is changed.
pub fn validate(o: &Opts) -> Result<(), String> {
    palette::check(&o.flavor, Some(&o.accent))?;
    if let Some(light) = &o.light_flavor {
        let light_is_dark = palette::check(light, Some(&o.accent))?.is_dark();
        if light_is_dark || !Palette::new(&o.flavor).is_some_and(|p| p.is_dark()) {
            return Err("--light-flavor needs a light flavor (latte) with a dark --flavor".into());
        }
    }
    let names: Vec<_> = styles::list().iter().map(|(n, _)| *n).collect();
    if o.style != "all" && !names.contains(&o.style.as_str()) {
        return Err(format!(
            "unknown wallpaper style '{}' (choose: {}, all)",
            o.style,
            names.join(", ")
        ));
    }
    if let Some(img) = &o.wallpaper_image {
        if o.style == "all" {
            return Err("--wallpaper-image replaces a wallpaper style; it cannot be combined with style 'all'".into());
        }
        if !Path::new(img).is_file() {
            return Err(format!("{img}: no such image (a PNG is needed)"));
        }
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
    theme_opts(o, PathBuf::new(), PathBuf::new())?.check()?;
    if let Some(unknown) = o
        .steps
        .iter()
        .find(|s| !STEPS.iter().any(|k| k.name == **s) && !crate::apps::exists(s))
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

/// What to keep after a run. A run that named only some steps (`apply bash`,
/// `status --fix`) must not turn the saved selection into just those.
fn for_saving(o: &Opts, previous: Option<Opts>, partial: bool) -> Opts {
    let mut kept = o.clone();
    if let Some(prev) = previous.filter(|_| partial) {
        kept.steps = prev.steps;
    }
    kept
}

/// Remembers which steps the interactive setup selected, before it runs them.
pub fn remember_steps(steps: &[String]) -> Result<(), String> {
    let mut o = load_state().unwrap_or_default();
    o.steps = steps.to_vec();
    save_state(&o)
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
/// Downloads and unpacks the pinned revision. `only` limits what is unpacked
/// (tar patterns under the top folder); empty unpacks everything.
fn fetch_pinned((slug, rev): (&str, &str), dest: &Path, only: &[&str]) -> Result<(), String> {
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
        .arg("--wildcards")
        .args(only)
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

fn step_backup(extra: &[String]) -> Result<PathBuf, String> {
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
    backup::create(&dest, extra)?;
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

/// The desktop theme this run asks for.
fn theme_opts(o: &Opts, config: PathBuf, data: PathBuf) -> Result<theme::InstallOpts, String> {
    let mut t = theme::InstallOpts::new(&o.flavor, &o.accent);
    t.roundness = o.roundness.clone();
    if let Some(g) = &o.gaps {
        t.gaps = theme::parse_gaps(g)?;
    }
    if let Some(h) = o.active_hint {
        t.active_hint = h;
    }
    t.window_hint = o.window_hint.clone();
    t.alpha_offset = o.alpha_offset;
    t.frosted = o.frosted.clone();
    t.frost_on = o.frost_on.as_ref().map(|l| {
        l.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });
    t.apply = !o.no_apply;
    (t.config, t.data) = (config, data);
    Ok(t)
}

fn step_cosmic(o: &Opts) -> Result<(), String> {
    let home = home()?;
    ui::info(&format!("COSMIC theme ({}, {} accent)", o.flavor, o.accent));
    // Read before importing anything: importing a theme sets the mode.
    let mode_dir = home.join(".config/cosmic/com.system76.CosmicTheme.Mode/v1");
    // No file means COSMIC's default: dark.
    let was_dark = fs::read_to_string(mode_dir.join("is_dark"))
        .map(|v| v.trim() != "false")
        .unwrap_or(true);
    let (label, theme) = theme::install_theme(&theme_opts(
        o,
        home.join(".config/cosmic"),
        home.join(".local/share/catppuccin/themes/cosmic-settings"),
    )?)?;
    if let Some(light) = &o.light_flavor {
        // Importing a light theme switches COSMIC to light mode; put the mode
        // back, then let COSMIC follow sunrise and sunset.
        let mut t = theme_opts(
            o,
            home.join(".config/cosmic"),
            home.join(".local/share/catppuccin/themes/cosmic-settings"),
        )?;
        t.flavor = light.clone();
        let installed = theme::install_theme(&t);
        // Restore the mode even when the Latte install failed.
        if !o.no_apply {
            fs::create_dir_all(&mode_dir).map_err(|e| format!("{}: {e}", mode_dir.display()))?;
            for (key, value) in [("is_dark", was_dark), ("auto_switch", true)] {
                fs::write(mode_dir.join(key), value.to_string())
                    .map_err(|e| format!("{}: {e}", mode_dir.join(key).display()))?;
            }
        }
        installed?;
        ui::ok(&format!(
            "Light mode: Catppuccin {light}; switches at sunrise and sunset"
        ));
    }
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
    if let Some(light) = &o.light_flavor {
        terminal::apply(light, &terminal::default_config_dir()?)?;
        ui::ok(&format!(
            "Terminal colors in light mode: Catppuccin {light}"
        ));
    }
    let installed = o
        .install_font
        .as_deref()
        .map(crate::nerdfont::install)
        .transpose()?;
    if let Some(font) = installed.as_ref().or(o.font.as_ref()) {
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

/// Whether one of `bins` is a file in ~/.local/bin or on PATH.
pub fn which(home: &Path, bins: &[&str]) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs: Vec<PathBuf> = vec![home.join(".local/bin")];
    dirs.extend(std::env::split_paths(&path));
    dirs.iter()
        .any(|d| bins.iter().any(|b| d.join(b).is_file()))
}

fn zed_installed(home: &Path) -> bool {
    which(home, &["zed", "zeditor"])
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
    if let Some(light) = &o.light_flavor {
        zed::apply_light(light, &zed::default_settings_path()?)?;
        ui::ok(&format!("Editor theme in light mode: Catppuccin {light}"));
    }
    ui::ok(&format!("Editor icons: {name}"));
    Ok(())
}

fn step_gtk(o: &Opts) -> Result<(), String> {
    let home = home()?;
    let p = Palette::new(&o.flavor).ok_or("unknown flavor")?;
    let name = format!("Catppuccin {}", p.name());
    ui::info("GTK and Flatpak apps");

    // COSMIC writes the sheets and links gtk.css to them when the theme is
    // applied; only fill in a link that is missing altogether.
    let sheet = gtk::sheet(&home, &p);
    match gtk::css_state(&sheet, &p) {
        gtk::Css::Follows => ui::ok(&format!("GTK apps: COSMIC's style sheet follows {name}")),
        gtk::Css::Other => ui::warn(&format!(
            "COSMIC's GTK style sheet does not follow {name} yet; it is rewritten when the theme is applied (cosmic step)"
        )),
        gtk::Css::Missing => ui::warn(
            "COSMIC has not written its GTK style sheet yet; apply the desktop theme (cosmic step) first",
        ),
    }
    if sheet.is_file() {
        for v in ["gtk-3.0", "gtk-4.0"] {
            let link = home.join(format!(".config/{v}/gtk.css"));
            if fs::symlink_metadata(&link).is_err() {
                fs::create_dir_all(link.parent().unwrap()).map_err(|e| e.to_string())?;
                match std::os::unix::fs::symlink(&sheet, &link) {
                    Ok(()) => ui::ok(&format!("{v}/gtk.css now follows COSMIC's sheet")),
                    // COSMIC writes this link itself, right after the theme import.
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(format!("{}: {e}", link.display())),
                }
            }
        }
    }

    if !which(&home, &["flatpak"]) {
        ui::warn("flatpak not found, skipping Flatpak apps");
        return Ok(());
    }
    let flatpak = |args: &[&str]| {
        Command::new("flatpak")
            .args(args)
            .stderr(Stdio::null())
            .output()
            .map_err(|e| format!("flatpak: {e}"))
    };
    let shown = flatpak(&["override", "--user", "--show"])?;
    let shown = String::from_utf8_lossy(&shown.stdout);
    let missing = gtk::missing_shares(&shown);
    for share in gtk::FLATPAK_SHARES {
        if missing.contains(&share) {
            let out = flatpak(&["override", "--user", &format!("--filesystem={share}")])?;
            if !out.status.success() {
                return Err(format!("flatpak override for {share} failed"));
            }
            ui::ok(&format!("Flatpak apps can now read {share}"));
        } else {
            ui::ok(&format!("Flatpak apps already read {share}"));
        }
    }
    let runtimes = flatpak(&["list", "--runtime"])?;
    if !String::from_utf8_lossy(&runtimes.stdout).contains("org.gtk.Gtk3theme.adw-gtk3") {
        ui::todo("For GTK 3 Flatpak apps: flatpak install flathub org.gtk.Gtk3theme.adw-gtk3-dark");
    }
    Ok(())
}

fn step_icons(o: &Opts) -> Result<(), String> {
    let home = home()?;
    let cache = cache_dir()?;
    ui::info(&format!(
        "Papirus icons (cat-{}-{} folders)",
        o.flavor, o.accent
    ));
    // Only the two themes the step uses: the rest of the repo is most of its size.
    fetch_pinned(
        PAPIRUS,
        &cache.join("papirus"),
        &["*/Papirus", "*/Papirus-Dark"],
    )?;
    fetch_pinned(CTP_FOLDERS, &cache.join("ctp-folders"), &[])?;
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
    let dark = palette::check(&o.flavor, None)?.is_dark();
    let slug = format!("catppuccin-{}-{}", o.flavor, o.accent);
    if let Some(img) = &o.wallpaper_image {
        ui::info(&format!(
            "Wallpaper ({}, {} accent, your image)",
            o.flavor, o.accent
        ));
        let stem = Path::new(img)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let file = pictures.join(format!("catppuccin-{}-{}-{stem}.png", o.flavor, o.accent));
        fs::create_dir_all(&pictures).map_err(|e| format!("{}: {e}", pictures.display()))?;
        wallpaper::recolor(Path::new(img), &file, &o.flavor, 40.0)?;
        background::set(
            &config,
            &file.display().to_string(),
            None,
            dark,
            Some(&slug),
        )?;
        ui::ok(&format!("Background: {}", file.display()));
        return Ok(());
    }
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
        background::set(
            &config,
            &file.display().to_string(),
            None,
            dark,
            Some(&slug),
        )?;
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
                dark,
                Some(&slug),
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

/// What `apply` would do, step by step. A description, not a simulation:
/// nothing is read from COSMIC or downloaded to write it.
pub fn plan(o: &Opts) -> Vec<String> {
    let name = Palette::new(&o.flavor)
        .map(|p| p.name())
        .unwrap_or_default();
    let mut out = Vec::new();
    if !o.no_backup {
        out.push("back up ~/.config/cosmic, ~/.bashrc and Zed's settings.json to ~/.config/catppuccin-backup-<date>".into());
    }
    for s in STEPS.iter().filter(|s| o.steps.iter().any(|n| n == s.name)) {
        out.push(match s.name {
            "cosmic" => format!(
                "cosmic: write every theme to ~/.local/share/catppuccin/themes/cosmic-settings, then {} catppuccin-{}-{}+custom.ron (Frosted Glass: {}, keeping your density)",
                if o.no_apply { "only write" } else { "import with cosmic-settings" },
                o.flavor, o.accent, o.frosted
            ),
            "terminal" => format!(
                "terminal: add Catppuccin {name} to COSMIC Terminal's color schemes and select it{}",
                o.font.as_ref().map(|f| format!("; font {f}")).unwrap_or_default()
            ),
            "bash" => "bash: write ~/.config/catppuccin/catppuccin-prompt.bash and add a marked block to ~/.bashrc".into(),
            "zed" => format!("zed: set theme and icon_theme to Catppuccin {name} in ~/.config/zed/settings.json (skipped if Zed is not installed)"),
            "gtk" => "gtk: check that COSMIC's GTK style sheet follows the theme, and let Flatpak apps read it (flatpak override --user, if Flatpak is installed)".to_string(),
            "icons" => format!("icons: download Papirus and the Catppuccin folders (pinned), install them in ~/.local/share/icons with cat-{}-{} folders, select the theme in COSMIC", o.flavor, o.accent),
            "wallpaper" if o.style == "all" => format!("wallpaper: render every style into ~/Pictures/Wallpapers/catppuccin-{}-{}, add the folder to Settings → Wallpaper{}", o.flavor, o.accent, o.rotate.as_ref().map(|r| format!(", slideshow every {r}")).unwrap_or_default()),
            "wallpaper" => format!("wallpaper: render the {} style into ~/Pictures/Wallpapers and set it as the background", o.style),
            other => other.to_string(),
        });
    }
    for name in o.steps.iter().filter(|n| !is_builtin(n)) {
        out.push(format!(
            "{name}: write Catppuccin {name_} into the app's files, keeping your own content (skipped if the app is not installed)",
            name_ = name_of(o)
        ));
    }
    out
}

fn name_of(o: &Opts) -> String {
    Palette::new(&o.flavor)
        .map(|p| p.name())
        .unwrap_or_default()
}

fn is_builtin(name: &str) -> bool {
    STEPS.iter().any(|s| s.name == name)
}

fn step_app(o: &Opts, name: &str) -> Result<(), String> {
    let app = crate::apps::find(name)?.ok_or(format!("unknown step '{name}'"))?;
    ui::info(app.label());
    match app.apply(&o.flavor, &o.accent, &home()?)? {
        crate::apps::Outcome::Skipped(why) => ui::warn(&why),
        crate::apps::Outcome::Waiting(why) => {
            // A warning too, so the interactive setup does not show it as done.
            ui::warn(&format!("{} was not themed", app.label()));
            ui::todo(&why);
        }
        crate::apps::Outcome::Done(files) => {
            files
                .iter()
                .for_each(|f| ui::ok(&format!("Themed {}", f.display())));
            app.notices(&home()?).iter().for_each(|n| ui::warn(n));
        }
    }
    Ok(())
}

/// Runs the chosen steps in installer order. The backup is sealed even if a
/// step fails, so `uninstall` can still undo what was done.
pub fn run(o: &Opts, partial: bool) -> Result<(), String> {
    // Apps are steps by their own name; the backup must watch what they write.
    let apps: Vec<&String> = o.steps.iter().filter(|n| !is_builtin(n)).collect();
    let mut watched = Vec::new();
    for name in &apps {
        let app = crate::apps::find(name)?.ok_or(format!("unknown step '{name}'"))?;
        watched.extend(app.watch(&home()?)?);
    }
    let backup_dir = if o.no_backup {
        None
    } else {
        Some(step_backup(&watched)?)
    };
    // Icons go before the desktop theme: its import also fills the theme bundle.
    let mut order: Vec<_> = STEPS
        .iter()
        .filter(|s| o.steps.iter().any(|n| n == s.name))
        .collect();
    order.sort_by_key(|s| s.name != "icons");
    let result = order
        .into_iter()
        .try_for_each(|s| match s.name {
            "cosmic" => step_cosmic(o),
            "terminal" => step_terminal(o),
            "bash" => step_bash(o),
            "zed" => step_zed(o),
            "icons" => step_icons(o),
            "gtk" => step_gtk(o),
            "wallpaper" => step_wallpaper(o),
            other => Err(format!("unknown step '{other}'")),
        })
        .and_then(|()| apps.iter().try_for_each(|name| step_app(o, name)));
    if let Some(dir) = &backup_dir
        && let Err(e) = backup::seal(dir, &watched)
    {
        ui::warn(&format!(
            "Could not record the changes ({e}); uninstall cannot undo this run"
        ));
    }
    result?;
    save_state(&for_saving(o, load_state().ok(), partial))?;
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
  ctp-cosmic apply --profile NAME     start from a saved profile (see `ctp-cosmic profile`)
  ctp-cosmic apply ... --save NAME    save the choices as a profile after a successful run
  ctp-cosmic apply --list             list the steps

Steps: cosmic, terminal, bash, zed, wallpaper, and icons and gtk (only when named).
Apps (`ctp-cosmic app list`, only when named): firefox, and any of your own.

Options (each also an environment variable; flags win):
  --flavor NAME            FLAVOR            latte, frappe, macchiato, mocha (mocha)
  --accent NAME            ACCENT            any of the 14 accents (mauve)
  --light-flavor NAME      LIGHT_FLAVOR      with a dark --flavor: latte for COSMIC's light
                                             mode; the desktop then follows sunrise/sunset
  --frosted auto|off|STR   FROSTED           Frosted Glass: keep the current setting
                                             (auto), off, or a strength such as medium
  --frost-on LIST          FROST_ON          windows,system_interface,panel,applets,
                                             maximized_apps
  --alpha-offset N         ALPHA_OFFSET      0.1 more opaque, -0.1 more transparent
  --roundness NAME         ROUNDNESS         round (default), slightlyround, square
  --gaps OUTER,INNER       GAPS              gaps between windows (0,8)
  --active-hint N          ACTIVE_HINT       active window outline width (3)
  --window-hint NAME       WINDOW_HINT       its color: any accent (default: the accent)
  --wallpaper-style NAME   WALLPAPER_STYLE   glow, waves, ... or all (see `wallpaper --help`)
  --wallpaper-rotate T     WALLPAPER_ROTATE   with style all: 5m 10m 15m 30m 1h 2h
  --wallpaper-image FILE   WALLPAPER_IMAGE    your own PNG, recolored with the flavor,
                                             instead of a style
  --font NAME              TERMINAL_FONT      COSMIC Terminal font (`terminal font list`)
  --install-font NAME      INSTALL_FONT       download a Nerd Font (JetBrainsMono, FiraCode, Hack)
                                             and use it as the terminal font
  --font-size N            TERMINAL_FONT_SIZE
  --no-apply               NO_APPLY=1         only generate the desktop theme
  --no-backup              NO_BACKUP=1        skip the backup
  --dry-run                                   say what would be done, change nothing
                           NO_TUI=1           never open the interactive setup

What the run changed is recorded; `ctp-cosmic uninstall` undoes it.
";

fn interactive() -> bool {
    std::io::stdout().is_terminal() && fs::File::open("/dev/tty").is_ok()
}

/// Takes `flag VALUE` out of `args`.
fn take_value(args: &mut Vec<String>, flag: &str) -> Result<Option<String>, String> {
    let Some(i) = args.iter().position(|a| a == flag) else {
        return Ok(None);
    };
    if i + 1 >= args.len() {
        return Err(format!("{flag} needs a name"));
    }
    let value = args.remove(i + 1);
    args.remove(i);
    Ok(Some(value))
}

pub fn main(mut args: Vec<String>) -> Result<u8, String> {
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
        for (name, label) in crate::apps::catalog() {
            println!("{name}  ({label}) [app, run only when named]");
        }
        return Ok(0);
    }
    let save = take_value(&mut args, "--save")?;
    let profile = take_value(&mut args, "--profile")?;
    let wants_last = args.iter().any(|a| a == "--last");
    if wants_last && profile.is_some() {
        return Err("--last and --profile cannot be combined".into());
    }
    let last = match &profile {
        Some(name) => Some(crate::profiles::load(name)?),
        None => wants_last.then(load_state).transpose()?,
    };
    let env = |k: &str| std::env::var(k).ok();
    let parsed = parse(&args, &env, last)?;
    if parsed.opts.dry_run {
        println!(
            "ctp-cosmic would ({} {}):",
            parsed.opts.flavor, parsed.opts.accent
        );
        plan(&parsed.opts).iter().for_each(|l| println!("  - {l}"));
        return Ok(0);
    }
    if !parsed.explicit && env("NO_TUI").as_deref() != Some("1") && interactive() {
        return crate::tui::main(Vec::new());
    }
    let _lock = crate::safety::lock()?;
    run(&parsed.opts, parsed.partial)?;
    if let Some(name) = save {
        crate::profiles::save(&name, &parsed.opts)?;
        ui::ok(&format!(
            "Saved as profile '{name}' (ctp-cosmic apply --profile {name})"
        ));
    }
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
            args(&["--roundness", "blob"]),
            args(&["--gaps", "8"]),
            args(&["--active-hint", "thick"]),
            args(&["--window-hint", "teal2"]),
            args(&["nosuchstep"]),
            args(&["--wat", "1"]),
            args(&["--install-font"]),
            args(&["--wallpaper-image", "/nonexistent/x.png"]),
            args(&["--light-flavor", "mocha"]),
            args(&["--flavor", "latte", "--light-flavor", "latte"]),
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
    fn light_flavor_pairs_a_dark_one_with_latte() {
        let p = parse(
            &args(&["--flavor", "macchiato", "--light-flavor", "latte"]),
            &no_env,
            None,
        )
        .unwrap();
        assert_eq!(p.opts.light_flavor.as_deref(), Some("latte"));
        assert!(p.explicit);
    }

    #[test]
    fn a_partial_run_keeps_the_saved_selection() {
        let saved = Opts::default();
        let ran = Opts {
            flavor: "latte".into(),
            steps: vec!["bash".into()],
            ..Opts::default()
        };
        let kept = for_saving(&ran, Some(saved.clone()), true);
        assert_eq!(kept.steps, saved.steps);
        assert_eq!(kept.flavor, "latte", "the other choices still update");
        assert_eq!(for_saving(&ran, Some(saved), false).steps, ["bash"]);
        assert_eq!(for_saving(&ran, None, true).steps, ["bash"]);
    }

    #[test]
    fn theme_options_reach_the_theme() {
        let p = parse(
            &args(&[
                "--roundness",
                "square",
                "--gaps",
                "2,4",
                "--active-hint",
                "5",
                "--window-hint",
                "teal",
            ]),
            &no_env,
            None,
        )
        .unwrap();
        let t = theme_opts(&p.opts, PathBuf::new(), PathBuf::new()).unwrap();
        assert_eq!(
            (t.roundness.as_str(), t.gaps, t.active_hint),
            ("square", (2, 4), 5)
        );
        assert_eq!(t.window_hint.as_deref(), Some("teal"));
    }

    #[test]
    fn dry_run_plans_every_chosen_step_and_nothing_else() {
        let p = parse(
            &args(&["--dry-run", "--no-backup", "bash", "icons"]),
            &no_env,
            None,
        )
        .unwrap();
        assert!(p.opts.dry_run && p.explicit);
        let steps = plan(&p.opts);
        assert_eq!(steps.len(), 2);
        assert!(steps[0].starts_with("bash:") && steps[1].starts_with("icons:"));
        // Every default step has a line, plus the backup.
        let all = plan(&Opts::default());
        assert_eq!(all.len(), 6);
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
