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

use std::path::PathBuf;

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
    let spacing = [
        ("none", 0),
        ("xxxs", 4),
        ("xxs", 8),
        ("xs", 12),
        ("s", 16),
        ("m", 24),
        ("l", 32),
        ("xl", 48),
        ("xxl", 64),
        ("xxxl", 128),
    ];
    for (k, v) in spacing {
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
  ctp-cosmic theme [OPTIONS] --all DIR   every flavor and accent, solid and
                                         frosted, as catppuccin-<flavor>-<accent>+<roundness>[+frosted].ron

Apply one with: cosmic-settings appearance import FILE

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
  -o, --output FILE    write to FILE
  --all DIR            write every flavor and accent into DIR
  -h, --help           show this help
";

pub fn main(args: Vec<String>) -> Result<(), String> {
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
                let v = value("--gaps")?;
                let parts: Vec<_> = v.split(',').map(|s| s.trim().parse::<i64>()).collect();
                o.gaps = match parts.as_slice() {
                    [Ok(a), Ok(b)] => (*a, *b),
                    _ => return Err(format!("invalid --gaps '{v}' (expected OUTER,INNER)")),
                };
            }
            "--active-hint" => {
                let v = value("--active-hint")?;
                o.active_hint = v
                    .trim()
                    .parse()
                    .map_err(|_| format!("invalid --active-hint '{v}'"))?;
            }
            "-o" | "--output" => output = Some(value("--output")?.into()),
            "--all" => all = Some(value("--all")?.into()),
            s => return Err(format!("unexpected argument '{s}' (see --help)")),
        }
    }

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

    if let Some(dir) = all {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let strength = o.frosted.clone().unwrap_or_else(|| "medium".into());
        let mut n = 0;
        for flavor in palette::flavors() {
            for accent in palette::accents() {
                for frosted in [None, Some(strength.clone())] {
                    let t = Options {
                        flavor: flavor.into(),
                        accent: accent.into(),
                        frosted: frosted.clone(),
                        ..o.clone()
                    };
                    let path = dir.join(filename(flavor, accent, &o.roundness, frosted.is_some()));
                    std::fs::write(&path, build(&t))
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    n += 1;
                }
            }
        }
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
