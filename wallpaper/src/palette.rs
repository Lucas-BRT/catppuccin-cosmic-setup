//! Access to the official Catppuccin palette (the `catppuccin` crate) by the
//! lowercase identifiers used everywhere else in this project.

use catppuccin::{Flavor, FlavorName, PALETTE};

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
        let name: FlavorName = flavor.parse().ok()?;
        Some(Self {
            flavor: &PALETTE[name],
        })
    }

    pub fn is_dark(&self) -> bool {
        self.flavor.dark
    }

    /// A color by identifier ("mauve", "base", ...). Panics on unknown names,
    /// which are always literals in this crate or validated user input.
    pub fn get(&self, name: &str) -> Rgb {
        let c = self
            .flavor
            .iter()
            .find(|c| c.identifier() == name)
            .unwrap_or_else(|| panic!("unknown Catppuccin color: {name}"));
        [f64::from(c.rgb.r), f64::from(c.rgb.g), f64::from(c.rgb.b)]
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

/// Every color of every flavor as "flavor color rrggbb" lines, used by CI to
/// check that this palette and lib/catppuccin.py agree.
pub fn dump() -> String {
    let mut out = String::new();
    for flavor in PALETTE.all_flavors() {
        for c in flavor.iter() {
            out.push_str(&format!(
                "{} {} {:02x}{:02x}{:02x}\n",
                flavor.identifier(),
                c.identifier(),
                c.rgb.r,
                c.rgb.g,
                c.rgb.b
            ));
        }
    }
    out
}
