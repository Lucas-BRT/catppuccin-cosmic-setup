//! The official Catppuccin palette (the `catppuccin` crate), looked up by the
//! lowercase identifiers used everywhere in this project, and the
//! `ctp-cosmic palette` subcommand the shell scripts call.

use std::io::Read;

use catppuccin::{Flavor, PALETTE};

/// An RGB color as floats in 0..255, which is what the styles compute with.
pub type Rgb = [f64; 3];

/// One flavor of the palette.
#[derive(Clone, Copy)]
pub struct Palette {
    flavor: &'static Flavor,
}

impl Palette {
    /// Look up a flavor by identifier ("latte", "frappe", "macchiato", "mocha").
    pub fn new(flavor: &str) -> Option<Self> {
        PALETTE
            .all_flavors()
            .into_iter()
            .find(|f| f.identifier() == flavor)
            .map(|flavor| Self { flavor })
    }

    pub fn is_dark(&self) -> bool {
        self.flavor.dark
    }

    /// Display name, as other Catppuccin ports use it ("Frappé").
    pub fn name(&self) -> String {
        self.flavor.name.to_string()
    }

    fn color(&self, name: &str) -> Option<&'static catppuccin::Color> {
        self.flavor.iter().find(|c| c.identifier() == name)
    }

    /// A color by identifier ("mauve", "base", ...). Panics on unknown names,
    /// which are always literals in this crate or validated user input.
    pub fn get(&self, name: &str) -> Rgb {
        let c = self
            .color(name)
            .unwrap_or_else(|| panic!("unknown Catppuccin color: {name}"));
        [f64::from(c.rgb.r), f64::from(c.rgb.g), f64::from(c.rgb.b)]
    }

    /// A color as six lowercase hex digits ("cba6f7").
    pub fn hex(&self, name: &str) -> String {
        let c = self
            .color(name)
            .unwrap_or_else(|| panic!("unknown Catppuccin color: {name}"));
        format!("{:02x}{:02x}{:02x}", c.rgb.r, c.rgb.g, c.rgb.b)
    }
}

/// Flavor identifiers in the official order.
pub fn flavors() -> Vec<&'static str> {
    PALETTE
        .all_flavors()
        .iter()
        .map(|f| f.identifier())
        .collect()
}

/// The 14 accent identifiers in the official order.
pub fn accents() -> Vec<&'static str> {
    PALETTE
        .mocha
        .iter()
        .filter(|c| c.accent)
        .map(|c| c.identifier())
        .collect()
}

/// Validate a flavor and, optionally, an accent.
pub fn check(flavor: &str, accent: Option<&str>) -> Result<Palette, String> {
    let p = Palette::new(flavor).ok_or_else(|| {
        format!(
            "unknown flavor '{flavor}' (choose: {})",
            flavors().join(", ")
        )
    })?;
    if let Some(accent) = accent
        && !accents().contains(&accent)
    {
        return Err(format!(
            "unknown accent '{accent}' (choose: {})",
            accents().join(", ")
        ));
    }
    Ok(p)
}

/// Every color of every flavor as "flavor color rrggbb" lines.
pub fn dump() -> String {
    let mut out = String::new();
    for flavor in flavors() {
        let p = Palette::new(flavor).unwrap();
        for c in p.flavor.iter() {
            out.push_str(&format!(
                "{flavor} {} {}\n",
                c.identifier(),
                p.hex(c.identifier())
            ));
        }
    }
    out
}

/// Replace every `@<color>@` placeholder with that color as "R;G;B" (for
/// 24-bit ANSI escapes), and `@accent@` with the chosen accent.
pub fn render(text: &str, p: Palette, accent: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let len = after
            .bytes()
            .take_while(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            .count();
        if len > 0 && after.as_bytes().get(len) == Some(&b'@') {
            let key = &after[..len];
            let name = if key == "accent" { accent } else { key };
            let c = p
                .color(name)
                .ok_or_else(|| format!("unknown placeholder: @{key}@"))?;
            out.push_str(&format!("{};{};{}", c.rgb.r, c.rgb.g, c.rgb.b));
            rest = &after[len + 1..];
        } else {
            out.push('@');
            rest = after;
        }
    }
    out.push_str(rest);
    Ok(out)
}

const USAGE: &str = "\
ctp-cosmic palette: the Catppuccin palette, for shell scripts

Usage:
  ctp-cosmic palette check FLAVOR [ACCENT]   exit 1 with a message if invalid
  ctp-cosmic palette name FLAVOR             display name (\"Frappé\")
  ctp-cosmic palette is-dark FLAVOR          exit 0 if dark, 1 if light
  ctp-cosmic palette rgb FLAVOR COLOR        \"137;180;250\", for ANSI escapes
  ctp-cosmic palette render FLAVOR ACCENT    stdin → stdout, replacing each
                                             @<color>@ with \"R;G;B\" and
                                             @accent@ with the accent
  ctp-cosmic palette dump                    every color as \"flavor color rrggbb\"
";

/// Returns the process exit code (is-dark answers through it).
pub fn main(args: Vec<String>) -> Result<u8, String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["dump"] => print!("{}", dump()),
        ["check", flavor] => drop(check(flavor, None)?),
        ["check", flavor, accent] => drop(check(flavor, Some(accent))?),
        ["name", flavor] => println!("{}", check(flavor, None)?.name()),
        ["is-dark", flavor] => return Ok(if check(flavor, None)?.is_dark() { 0 } else { 1 }),
        ["rgb", flavor, color] => {
            let p = check(flavor, None)?;
            let c = p
                .color(color)
                .ok_or_else(|| format!("unknown color '{color}'"))?;
            println!("{};{};{}", c.rgb.r, c.rgb.g, c.rgb.b);
        }
        ["render", flavor, accent] => {
            let p = check(flavor, Some(accent))?;
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|e| e.to_string())?;
            print!("{}", render(&text, p, accent)?);
        }
        ["-h" | "--help"] => print!("{USAGE}"),
        _ => return Err(format!("invalid arguments\n\n{USAGE}")),
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_has_official_shape() {
        assert_eq!(flavors(), ["latte", "frappe", "macchiato", "mocha"]);
        assert_eq!(accents().len(), 14);
        assert_eq!(dump().lines().count(), 4 * 26);
        assert_eq!(Palette::new("frappe").unwrap().name(), "Frappé");
        assert!(Palette::new("frappé").is_none());
    }

    #[test]
    fn render_fills_placeholders() {
        let p = Palette::new("mocha").unwrap();
        assert_eq!(
            render("a@blue@b @accent@ user@host @@ @x", p, "mauve").unwrap(),
            "a137;180;250b 203;166;247 user@host @@ @x"
        );
        assert!(render("@nope@", p, "mauve").is_err());
    }
}
