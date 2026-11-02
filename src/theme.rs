//! `ctp-cosmic theme`: Catppuccin desktop themes in the COSMIC 1.10 format
//! (ThemeBuilder v2), ready for `cosmic-settings appearance import`.
//!
//! Why not the official .ron files from catppuccin/cosmic-desktop: they target
//! ThemeBuilder v1. COSMIC 1.10 still imports them, but its `is_frosted` is
//! silently ignored and the new Frosted Glass fields (`frosted`,
//! `frosted_windows`, …, `alpha_map`) default to off, so importing them turns
//! Frosted Glass off. Colors, and which Catppuccin color goes into each field,
//! follow the official template (templates/cosmic-settings.tera there).
//!
//! The tests parse every generated theme with libcosmic's own ThemeBuilder.

use std::path::{Path, PathBuf};

use crate::palette::{self, Palette};

/// COSMIC palette field → Catppuccin color, in the order COSMIC 1.10 writes
/// them (CosmicPaletteInner in libcosmic).
const PALETTE_MAP: [(&str, &str); 32] = [
    ("bright_red", "red"),
    ("bright_green", "green"),
    ("bright_orange", "peach"),
    ("gray_1", "mantle"), // surfaces
    ("gray_2", "base"),
    ("neutral_0", "crust"), // neutral scale, darkest to lightest
    ("neutral_1", "mantle"),
    ("neutral_2", "base"),
    ("neutral_3", "surface0"),
    ("neutral_4", "surface1"),
    ("neutral_5", "surface2"),
    ("neutral_6", "overlay0"),
    ("neutral_7", "overlay1"),
    ("neutral_8", "overlay2"),
    ("neutral_9", "subtext0"),
    ("neutral_10", "subtext1"),
    ("accent_blue", "blue"), // the 9 accent swatches shown under Appearance
    ("accent_indigo", "mauve"),
    ("accent_purple", "lavender"),
    ("accent_pink", "pink"),
    ("accent_red", "red"),
    ("accent_orange", "peach"),
    ("accent_yellow", "yellow"),
    ("accent_green", "green"),
    ("accent_warm_grey", "overlay2"),
    ("ext_warm_grey", "overlay2"), // extra colors used by some apps
    ("ext_orange", "peach"),
    ("ext_yellow", "yellow"),
    ("ext_blue", "blue"),
    ("ext_purple", "lavender"),
    ("ext_pink", "pink"),
    ("ext_indigo", "mauve"),
];

/// Corner radii (radius_0, xs, s, m, l, xl) for each roundness.
const ROUNDNESS: [(&str, [u32; 6]); 3] = [
    ("round", [0, 4, 8, 16, 32, 160]), // COSMIC default
    ("slightlyround", [0, 2, 8, 8, 8, 8]),
    ("square", [0, 2, 2, 2, 2, 2]),
];

/// Blur strengths (libcosmic's BlurStrength enum), weakest to strongest, with
/// the default opacity for each one (AlphaMap::default).
const BLUR: [(&str, &str, f64); 14] = [
    ("ExtremelyLow", "extremely_low", 0.90),
    ("ExtremelyLow2", "extremely_low_2", 0.87692),
    ("VeryLow", "very_low", 0.85385),
    ("VeryLow2", "very_low_2", 0.83076),
    ("Low", "low", 0.80769),
    ("Low2", "low_2", 0.78461),
    ("Medium", "medium", 0.76154),
    ("Medium2", "medium_2", 0.73846),
    ("High", "high", 0.71538),
    ("High2", "high_2", 0.69231),
    ("VeryHigh", "very_high", 0.66023),
    ("VeryHigh2", "very_high_2", 0.64615),
    ("ExtremelyHigh", "extremely_high", 0.62308),
    ("ExtremelyHigh2", "extremely_high_2", 0.6),
];

/// Spacing for each interface density (libcosmic's `From<Density> for
/// Spacing`). The theme carries the spacing, so it must match the density
/// chosen under Appearance or the layout changes when the theme is imported.
const SPACING_KEYS: [&str; 10] = [
    "none", "xxxs", "xxs", "xs", "s", "m", "l", "xl", "xxl", "xxxl",
];
const DENSITY: [(&str, [u32; 10]); 3] = [
    ("compact", [0, 4, 4, 8, 8, 16, 24, 32, 48, 64]),
    ("standard", [0, 4, 8, 12, 16, 24, 32, 48, 64, 128]), // COSMIC default
    ("spacious", [4, 8, 12, 16, 24, 32, 48, 64, 128, 160]),
];

/// Where Frosted Glass applies, in the order of the toggles under Appearance.
const FROST_TARGETS: [&str; 5] = [
    "windows",
    "system_interface",
    "panel",
    "applets",
    "maximized_apps",
];
const FROST_DEFAULT: [&str; 4] = ["windows", "system_interface", "panel", "applets"];

/// Everything that shapes one theme.
#[derive(Clone)]
pub struct Options {
    pub flavor: String,
    pub accent: String,
    pub roundness: String,
    pub window_hint: Option<String>,
    /// Blur strength key ("medium"); None means Frosted Glass off.
    pub frosted: Option<String>,
    pub frost_on: Vec<String>,
    pub alpha_offset: f64,
    pub gaps: (i64, i64),
    pub active_hint: i64,
    pub density: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            roundness: "round".into(),
            window_hint: None,
            frosted: None,
            frost_on: FROST_DEFAULT.iter().map(|s| s.to_string()).collect(),
            alpha_offset: 0.0,
            gaps: (0, 8),
            active_hint: 3,
            density: "standard".into(),
        }
    }
}

/// "#CBA6F7FF", the color format COSMIC writes.
fn hexa(p: Palette, color: &str) -> String {
    format!("#{}FF", p.hex(color).to_uppercase())
}

/// A float the way the original Python generator printed `round(x, 5)`:
/// shortest form, always with a decimal point ("0.9", "1.0").
fn py_round5(x: f64) -> String {
    let v: f64 = format!("{x:.5}").parse().unwrap();
    if v != 0.0 && v.abs() < 1e-4 {
        // Python switches to exponent notation below 1e-4 ("1e-05").
        return format!("{}e-05", (v * 1e5).round());
    }
    let s = format!("{v}");
    if s.contains('.') { s } else { format!("{s}.0") }
}

/// The RON text of a ThemeBuilder v2.
pub fn build(o: &Options) -> String {
    let p = Palette::new(&o.flavor).expect("validated flavor");
    let dark = p.is_dark();
    // Only the targets matter when Frosted Glass is on; the strength is
    // irrelevant when everything is off.
    let frost_on: &[String] = if o.frosted.is_some() {
        &o.frost_on
    } else {
        &[]
    };
    let blur_key = o.frosted.as_deref().unwrap_or("medium");
    let capitalize = |s: &str| s[..1].to_uppercase() + &s[1..];

    let mut l = vec!["(".to_string()];
    l.push(format!(
        "    palette: {}((",
        if dark { "Dark" } else { "Light" }
    ));
    l.push(format!(
        "        name: \"Catppuccin-{}-{}\",",
        capitalize(&o.flavor),
        capitalize(&o.accent)
    ));
    for (field, color) in PALETTE_MAP {
        l.push(format!("        {field}: \"{}\",", hexa(p, color)));
    }
    l.push("    )),".into());

    l.push("    spacing: (".into());
    let spacing = DENSITY
        .iter()
        .find(|(n, _)| *n == o.density)
        .expect("validated density")
        .1;
    for (k, v) in SPACING_KEYS.iter().zip(spacing) {
        l.push(format!("        space_{k}: {v},"));
    }
    l.push("    ),".into());

    l.push("    corner_radii: (".into());
    let radii = ROUNDNESS
        .iter()
        .find(|(n, _)| *n == o.roundness)
        .expect("validated roundness")
        .1;
    for (k, r) in ["0", "xs", "s", "m", "l", "xl"].iter().zip(radii) {
        l.push(format!("        radius_{k}: ({r}.0, {r}.0, {r}.0, {r}.0),"));
    }
    l.push("    ),".into());

    let opt = |l: &mut Vec<String>, name: &str, color: Option<&str>| {
        l.push(match color {
            Some(c) => format!("    {name}: Some(\"{}\"),", hexa(p, c)),
            None => format!("    {name}: None,"),
        });
    };
    // Containers and control tint only for dark flavors, as in the official
    // theme; for Latte COSMIC derives them from the palette.
    opt(&mut l, "neutral_tint", dark.then_some("overlay1"));
    opt(&mut l, "bg_color", Some("base")); // window background
    opt(&mut l, "primary_container_bg", dark.then_some("surface0"));
    opt(&mut l, "secondary_container_bg", dark.then_some("surface1"));
    opt(&mut l, "text_tint", Some("text"));
    opt(&mut l, "accent", Some(&o.accent));
    opt(&mut l, "success", Some("green"));
    opt(&mut l, "warning", Some("yellow"));
    opt(&mut l, "destructive", Some("red"));

    let blur_name = BLUR
        .iter()
        .find(|b| b.1 == blur_key)
        .expect("validated blur")
        .0;
    l.push(format!("    frosted: {blur_name},"));
    l.push(format!("    gaps: ({}, {}),", o.gaps.0, o.gaps.1));
    l.push(format!("    active_hint: {},", o.active_hint));
    // Active window outline.
    opt(
        &mut l,
        "window_hint",
        Some(o.window_hint.as_deref().unwrap_or(&o.accent)),
    );
    for t in FROST_TARGETS {
        let on = frost_on.iter().any(|f| f == t);
        l.push(format!("    frosted_{t}: {on},"));
    }

    // Opacity for each strength. The opacity slider under Appearance adds one
    // offset to every value; --alpha-offset does the same.
    l.push("    alpha_map: (".into());
    for (_, key, a) in BLUR {
        let v = (a + o.alpha_offset).clamp(0.0, 1.0);
        l.push(format!("        {key}: {},", py_round5(v)));
    }
    l.push("    ),".into());
    l.push(")".into());
    l.join("\n") + "\n"
}

pub fn filename(flavor: &str, accent: &str, roundness: &str, frosted: bool) -> String {
    format!(
        "catppuccin-{flavor}-{accent}+{roundness}{}.ron",
        if frosted { "+frosted" } else { "" }
    )
}

const USAGE: &str = "\
ctp-cosmic theme: Catppuccin desktop themes in the COSMIC 1.10 format

Usage:
  ctp-cosmic theme [OPTIONS] [-o FILE]   one theme (default: to stdout)
  ctp-cosmic theme [OPTIONS] --all DIR   every flavor, accent and roundness, solid
                                         and frosted, as catppuccin-<flavor>-<accent>+<roundness>[+frosted].ron

Apply one with: cosmic-settings appearance import FILE
Or let the installer do it, keeping your density and Frosted Glass:
  ctp-cosmic theme install --help

Options:
  --flavor NAME        latte, frappe, macchiato, mocha (default: mocha)
  --accent NAME        any of the 14 accents (default: mauve)
  --roundness NAME     round (default), slightlyround, square
  --window-hint NAME   active window outline color (default: the accent)
  --frosted STRENGTH   turn on Frosted Glass: extremely_low, extremely_low_2,
                       very_low, very_low_2, low, low_2, medium, medium_2, high,
                       high_2, very_high, very_high_2, extremely_high,
                       extremely_high_2
  --frost-on LIST      where Frosted Glass applies, comma-separated: windows,
                       system_interface, panel, applets, maximized_apps
                       (default: all but maximized_apps)
  --alpha-offset N     added to the opacity (0.1 more opaque, -0.1 more transparent)
  --gaps OUTER,INNER   gaps between windows (default: 0,8)
  --active-hint N      active window outline width (default: 3)
  --density NAME       spacing for the interface density set under Appearance:
                       compact, standard (default), spacious
  -o, --output FILE    write to FILE
  --all DIR            write every flavor and accent into DIR
  -h, --help           show this help
";

fn validate(o: &Options) -> Result<(), String> {
    palette::check(&o.flavor, Some(&o.accent))?;
    if let Some(h) = &o.window_hint {
        palette::check(&o.flavor, Some(h))?;
    }
    if !ROUNDNESS.iter().any(|(n, _)| *n == o.roundness) {
        return Err(format!(
            "unknown roundness '{}' (choose: round, slightlyround, square)",
            o.roundness
        ));
    }
    if !DENSITY.iter().any(|(n, _)| *n == o.density) {
        return Err(format!(
            "unknown density '{}' (choose: compact, standard, spacious)",
            o.density
        ));
    }
    if let Some(f) = &o.frosted
        && !BLUR.iter().any(|b| b.1 == f)
    {
        return Err(format!("unknown --frosted strength '{f}' (see --help)"));
    }
    let mut bad: Vec<_> = o
        .frost_on
        .iter()
        .filter(|t| !FROST_TARGETS.contains(&t.as_str()))
        .cloned()
        .collect();
    if !bad.is_empty() {
        bad.sort();
        bad.dedup();
        return Err(format!("unknown --frost-on value: {}", bad.join(", ")));
    }
    Ok(())
}

/// Every roundness, flavor and accent, solid and frosted, into `dir`. Returns how many.
fn write_all(dir: &Path, o: &Options) -> Result<usize, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let strength = o.frosted.clone().unwrap_or_else(|| "medium".into());
    let mut n = 0;
    for (roundness, _) in ROUNDNESS {
        for flavor in palette::flavors() {
            for accent in palette::accents() {
                for frosted in [None, Some(strength.clone())] {
                    let t = Options {
                        flavor: flavor.into(),
                        accent: accent.into(),
                        roundness: roundness.into(),
                        frosted: frosted.clone(),
                        ..o.clone()
                    };
                    let path = dir.join(filename(flavor, accent, roundness, frosted.is_some()));
                    std::fs::write(&path, build(&t))
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    n += 1;
                }
            }
        }
    }
    Ok(n)
}

/// Frosted Glass as COSMIC has it now: the strength key and the places it
/// applies to, or None when it is off everywhere. COSMIC keeps each option in
/// its own file: `frosted` = "ExtremelyLow2", `frosted_windows` = "true", ...
fn current_frost(builder: &Path) -> Option<(String, Vec<String>)> {
    let read = |key: &str| std::fs::read_to_string(builder.join(key)).ok();
    let on: Vec<String> = FROST_TARGETS
        .iter()
        .filter(|t| read(&format!("frosted_{t}")).is_some_and(|v| v.trim() == "true"))
        .map(|t| t.to_string())
        .collect();
    if on.is_empty() {
        return None;
    }
    let strength = read("frosted")
        .and_then(|v| {
            BLUR.iter()
                .find(|b| b.0 == v.trim().trim_matches('"'))
                .map(|b| b.1)
        })
        .unwrap_or("medium");
    Some((strength.to_string(), on))
}

/// The interface density set under Appearance (the theme carries its spacing).
fn current_density(config: &Path) -> String {
    std::fs::read_to_string(config.join("com.system76.CosmicTk/v1/interface_density"))
        .map(|v| v.trim().trim_matches('"').to_lowercase())
        .ok()
        .filter(|v| DENSITY.iter().any(|(n, _)| n == v))
        .unwrap_or_else(|| "standard".into())
}

const INSTALL_USAGE: &str = "\
ctp-cosmic theme install: generate the theme and apply it

Usage:
  ctp-cosmic theme install [--flavor NAME] [--accent NAME] [--alpha-offset N]
                           [--roundness NAME] [--gaps OUTER,INNER] [--active-hint N]
                           [--window-hint NAME]
                           [--frosted auto|off|STRENGTH] [--frost-on LIST]
                           [--no-apply] [--config DIR] [--data-dir DIR]

Keeps your interface density. --frosted auto (default) keeps the current
Frosted Glass setting (strength and places); off turns it off; a strength
(see `theme --help`) turns it on in the --frost-on places. Writes every theme
into --data-dir (default ~/.local/share/catppuccin/themes/cosmic-settings),
then imports the chosen one with `cosmic-settings appearance import` unless
--no-apply. Prints what Frosted Glass is set to, then the theme file.
";

/// The corner styles and the Frosted Glass strengths, for completions.
pub fn roundness_names() -> Vec<&'static str> {
    ROUNDNESS.iter().map(|(n, _)| *n).collect()
}
pub fn blur_names() -> Vec<&'static str> {
    BLUR.iter().map(|b| b.1).collect()
}

/// "OUTER,INNER" as two numbers.
pub fn parse_gaps(v: &str) -> Result<(i64, i64), String> {
    let parts: Vec<_> = v.split(',').map(|s| s.trim().parse::<i64>()).collect();
    match parts.as_slice() {
        [Ok(a), Ok(b)] => Ok((*a, *b)),
        _ => Err(format!("invalid gaps '{v}' (expected OUTER,INNER)")),
    }
}

/// What `theme install` needs; `frosted` is "auto", "off" or a strength.
#[derive(Clone)]
pub struct InstallOpts {
    pub flavor: String,
    pub accent: String,
    pub roundness: String,
    pub gaps: (i64, i64),
    pub active_hint: i64,
    /// Color of the active window outline (default: the accent).
    pub window_hint: Option<String>,
    pub alpha_offset: f64,
    pub frosted: String,
    pub frost_on: Option<Vec<String>>,
    pub apply: bool,
    pub config: PathBuf,
    pub data: PathBuf,
}

impl InstallOpts {
    pub fn new(flavor: &str, accent: &str) -> Self {
        let d = Options::default();
        Self {
            flavor: flavor.into(),
            accent: accent.into(),
            roundness: d.roundness,
            gaps: d.gaps,
            active_hint: d.active_hint,
            window_hint: None,
            alpha_offset: 0.0,
            frosted: "auto".into(),
            frost_on: None,
            apply: true,
            config: PathBuf::new(),
            data: PathBuf::new(),
        }
    }

    /// Fails on a bad value before anything is changed.
    pub fn check(&self) -> Result<(), String> {
        let probe = Self {
            config: PathBuf::from("/nonexistent"),
            ..self.clone()
        };
        probe.resolve().map(drop)
    }

    /// The theme options this resolves to, given COSMIC's current Frosted
    /// Glass and density.
    fn resolve(&self) -> Result<Options, String> {
        let mut o = Options {
            flavor: self.flavor.clone(),
            accent: self.accent.clone(),
            alpha_offset: self.alpha_offset,
            roundness: self.roundness.clone(),
            gaps: self.gaps,
            active_hint: self.active_hint,
            window_hint: self.window_hint.clone(),
            ..Options::default()
        };
        let dark = palette::check(&o.flavor, Some(&o.accent))?.is_dark();
        let builder = |dark: bool| {
            self.config.join(format!(
                "com.system76.CosmicTheme.{}.Builder/v2",
                if dark { "Dark" } else { "Light" }
            ))
        };
        // The other mode's setting stands in until this mode has been themed.
        let current = current_frost(&builder(dark)).or_else(|| current_frost(&builder(!dark)));
        let default_on = || FROST_DEFAULT.iter().map(|s| s.to_string()).collect();
        (o.frosted, o.frost_on) = match (self.frosted.as_str(), current) {
            ("off", _) | ("auto", None) => (None, default_on()),
            ("auto", Some((strength, on))) => (Some(strength), on),
            (strength, _) => (
                Some(strength.to_string()),
                self.frost_on.clone().unwrap_or_else(default_on),
            ),
        };
        o.density = current_density(&self.config);
        validate(&o)?;
        Ok(o)
    }
}

/// Writes every theme plus the chosen one and (unless `apply` is off)
/// imports it. Returns the Frosted Glass label and the theme file.
pub fn install_theme(i: &InstallOpts) -> Result<(String, PathBuf), String> {
    let o = i.resolve()?;
    // The generic set for manual import; its own defaults, not this choice.
    write_all(&i.data, &Options::default())?;
    let theme = i
        .data
        .join(format!("catppuccin-{}-{}+custom.ron", o.flavor, o.accent));
    std::fs::write(&theme, build(&o)).map_err(|e| format!("{}: {e}", theme.display()))?;
    let label = match &o.frosted {
        Some(f) => format!("Frosted Glass {f} on {}", o.frost_on.join(",")),
        None => "solid".to_string(),
    };
    if i.apply {
        let status = std::process::Command::new("cosmic-settings")
            .args(["appearance", "import"])
            .arg(&theme)
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("cosmic-settings: {e}"))?;
        if !status.success() {
            return Err(format!(
                "cosmic-settings appearance import failed ({status})"
            ));
        }
    }
    Ok((label, theme))
}

fn install(args: Vec<String>) -> Result<(), String> {
    let mut i = InstallOpts::new("mocha", "mauve");
    let (mut config, mut data) = (None::<PathBuf>, None::<PathBuf>);
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{INSTALL_USAGE}");
                return Ok(());
            }
            "--flavor" => i.flavor = value("--flavor")?,
            "--accent" => i.accent = value("--accent")?,
            "--roundness" => i.roundness = value("--roundness")?,
            "--gaps" => i.gaps = parse_gaps(&value("--gaps")?)?,
            "--active-hint" => {
                let v = value("--active-hint")?;
                i.active_hint = v
                    .trim()
                    .parse()
                    .map_err(|_| format!("invalid --active-hint '{v}'"))?;
            }
            "--window-hint" => i.window_hint = Some(value("--window-hint")?),
            "--alpha-offset" => {
                let v = value("--alpha-offset")?;
                i.alpha_offset = v
                    .parse()
                    .map_err(|_| format!("invalid --alpha-offset '{v}'"))?;
            }
            "--frosted" => i.frosted = value("--frosted")?,
            "--frost-on" => {
                i.frost_on = Some(
                    value("--frost-on")?
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            }
            "--no-apply" => i.apply = false,
            "--config" => config = Some(value("--config")?.into()),
            "--data-dir" => data = Some(value("--data-dir")?.into()),
            s => return Err(format!("unexpected argument '{s}'\n\n{INSTALL_USAGE}")),
        }
    }
    let home = || {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("HOME not set".to_string())
    };
    i.config = match config {
        Some(c) => c,
        None => home()?.join(".config/cosmic"),
    };
    i.data = match data {
        Some(d) => d,
        None => home()?.join(".local/share/catppuccin/themes/cosmic-settings"),
    };
    let (label, theme) = install_theme(&i)?;
    println!("{label}");
    println!("{}", theme.display());
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) == Some("install") {
        return install(args[1..].to_vec());
    }
    let mut o = Options::default();
    let (mut output, mut all): (Option<PathBuf>, Option<PathBuf>) = (None, None);
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "--flavor" => o.flavor = value("--flavor")?,
            "--accent" => o.accent = value("--accent")?,
            "--roundness" => o.roundness = value("--roundness")?,
            "--window-hint" => o.window_hint = Some(value("--window-hint")?),
            "--frosted" => o.frosted = Some(value("--frosted")?),
            "--frost-on" => {
                o.frost_on = value("--frost-on")?
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            }
            "--alpha-offset" => {
                let v = value("--alpha-offset")?;
                o.alpha_offset = v
                    .parse()
                    .map_err(|_| format!("invalid --alpha-offset '{v}'"))?;
            }
            "--gaps" => {
                o.gaps = parse_gaps(&value("--gaps")?)?;
            }
            "--active-hint" => {
                let v = value("--active-hint")?;
                o.active_hint = v
                    .trim()
                    .parse()
                    .map_err(|_| format!("invalid --active-hint '{v}'"))?;
            }
            "--density" => o.density = value("--density")?,
            "-o" | "--output" => output = Some(value("--output")?.into()),
            "--all" => all = Some(value("--all")?.into()),
            s => return Err(format!("unexpected argument '{s}' (see --help)")),
        }
    }

    validate(&o)?;

    if let Some(dir) = all {
        let n = write_all(&dir, &o)?;
        eprintln!("{n} themes in {}", dir.display());
        return Ok(());
    }

    let text = build(&o);
    match output {
        Some(path) => std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display())),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_theme::ThemeBuilder;

    #[test]
    fn frost_and_density_are_read_from_cosmic_config() {
        let dir = std::env::temp_dir().join(format!("ctp-frost-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let builder = dir.join("builder");
        std::fs::create_dir_all(&builder).unwrap();
        assert_eq!(current_frost(&builder), None);
        std::fs::write(builder.join("frosted"), "ExtremelyLow2").unwrap();
        assert_eq!(current_frost(&builder), None); // no place is on
        std::fs::write(builder.join("frosted_panel"), "true").unwrap();
        std::fs::write(builder.join("frosted_windows"), "false").unwrap();
        assert_eq!(
            current_frost(&builder),
            Some(("extremely_low_2".into(), vec!["panel".into()]))
        );

        assert_eq!(current_density(&dir), "standard");
        let tk = dir.join("com.system76.CosmicTk/v1");
        std::fs::create_dir_all(&tk).unwrap();
        std::fs::write(tk.join("interface_density"), "\"Compact\"").unwrap();
        assert_eq!(current_density(&dir), "compact");
        std::fs::write(tk.join("interface_density"), "huge").unwrap();
        assert_eq!(current_density(&dir), "standard");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_keeps_frost_and_density_and_skips_apply() {
        let dir = std::env::temp_dir().join(format!("ctp-inst-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let builder = dir.join("cfg/com.system76.CosmicTheme.Dark.Builder/v2");
        std::fs::create_dir_all(&builder).unwrap();
        std::fs::write(builder.join("frosted"), "High").unwrap();
        std::fs::write(builder.join("frosted_applets"), "true").unwrap();
        let args = |extra: &[&str]| -> Vec<String> {
            [
                "--flavor",
                "macchiato",
                "--accent",
                "peach",
                "--no-apply",
                "--config",
            ]
            .into_iter()
            .map(String::from)
            .chain([dir.join("cfg").display().to_string()])
            .chain(["--data-dir".into(), dir.join("out").display().to_string()])
            .chain(extra.iter().map(|s| s.to_string()))
            .collect()
        };
        install(args(&[])).unwrap();
        let theme =
            std::fs::read_to_string(dir.join("out/catppuccin-macchiato-peach+custom.ron")).unwrap();
        assert!(theme.contains("frosted_applets: true"));
        assert!(theme.contains("frosted_panel: false"));
        // Sets: 4 flavors x 14 accents x (solid, frosted), plus the custom one.
        assert_eq!(
            std::fs::read_dir(dir.join("out")).unwrap().count(),
            4 * 14 * 2 * 3 + 1
        );

        install(args(&["--frosted", "off"])).unwrap();
        let theme =
            std::fs::read_to_string(dir.join("out/catppuccin-macchiato-peach+custom.ron")).unwrap();
        assert!(theme.contains("frosted_applets: false"));
        assert!(install(args(&["--frosted", "bogus"])).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn all_variants() -> Vec<Options> {
        let mut v = Vec::new();
        for flavor in palette::flavors() {
            for accent in palette::accents() {
                for frosted in [None, Some("medium"), Some("extremely_high_2")] {
                    v.push(Options {
                        flavor: flavor.into(),
                        accent: accent.into(),
                        frosted: frosted.map(String::from),
                        ..Options::default()
                    });
                }
            }
        }
        for roundness in ["slightlyround", "square"] {
            v.push(Options {
                roundness: roundness.into(),
                window_hint: Some("peach".into()),
                frosted: Some("low".into()),
                frost_on: vec!["panel".into(), "maximized_apps".into()],
                alpha_offset: 0.15,
                gaps: (4, 12),
                active_hint: 1,
                density: if roundness == "square" {
                    "compact"
                } else {
                    "spacious"
                }
                .into(),
                ..Options::default()
            });
        }
        v
    }

    /// Every theme must parse as COSMIC's own ThemeBuilder (the libcosmic
    /// commit used by cosmic-settings 1.10) and keep what we put in it.
    #[test]
    fn cosmic_parses_every_theme() {
        for o in all_variants() {
            let text = build(&o);
            let b: ThemeBuilder = ron::de::from_str(&text)
                .unwrap_or_else(|e| panic!("{}/{}: {e}", o.flavor, o.accent));
            assert_eq!(b.palette.is_dark(), o.flavor != "latte");
            assert_eq!(
                b.frosted_windows,
                o.frosted.is_some() && o.frost_on.iter().any(|t| t == "windows")
            );
            assert_eq!(
                b.frosted_panel,
                o.frosted.is_some() && o.frost_on.iter().any(|t| t == "panel")
            );
            assert_eq!(b.gaps, (o.gaps.0 as u32, o.gaps.1 as u32));
            // COSMIC must read the spacing back as the same density.
            let density = match cosmic_theme::Density::from(b.spacing) {
                cosmic_theme::Density::Compact => "compact",
                cosmic_theme::Density::Standard => "standard",
                cosmic_theme::Density::Spacious => "spacious",
            };
            assert_eq!(density, o.density);
            let theme = b.build();
            let accent = Palette::new(&o.flavor).unwrap().get(&o.accent);
            let got = theme.accent_color();
            for (i, ch) in [got.red, got.green, got.blue].into_iter().enumerate() {
                assert!(
                    (f64::from(ch) * 255.0 - accent[i]).abs() < 0.6,
                    "{}/{}",
                    o.flavor,
                    o.accent
                );
            }
        }
    }

    #[test]
    fn python_float_format() {
        assert_eq!(py_round5(0.9), "0.9");
        assert_eq!(py_round5(1.0), "1.0");
        assert_eq!(py_round5(0.0), "0.0");
        assert_eq!(py_round5(0.76154 + 0.1), "0.86154");
        assert_eq!(py_round5(0.00001), "1e-05");
    }
}
