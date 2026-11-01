//! GTK and Flatpak apps. COSMIC already writes GTK 3 and 4 style sheets from
//! the desktop theme (`~/.config/gtk-4.0/cosmic/{dark,light}.css`, linked from
//! both `gtk.css` files), so once the Catppuccin theme is applied, GTK apps
//! follow it. What is left is making sure that holds and that sandboxed
//! Flatpak apps can read those sheets.

use std::fs;
use std::path::Path;

use crate::palette::Palette;

/// Flatpak sandboxes only see what they are given; these hold the sheets.
pub const FLATPAK_SHARES: [&str; 2] = ["xdg-config/gtk-3.0:ro", "xdg-config/gtk-4.0:ro"];

#[derive(Debug, PartialEq)]
pub enum Css {
    Missing,
    /// There, but its window color is not this flavor's base.
    Other,
    Follows,
}

/// Whether COSMIC's generated sheet uses the flavor's Base as window color.
pub fn css_state(file: &Path, p: &Palette) -> Css {
    let Ok(text) = fs::read_to_string(file) else {
        return Css::Missing;
    };
    let [r, g, b] = p.get("base").map(|v| v as u8);
    let want = format!("@define-color window_bg_color rgba({r}, {g}, {b},");
    if text.contains(&want) {
        Css::Follows
    } else {
        Css::Other
    }
}

/// The sheet COSMIC points `gtk.css` at for this flavor's mode.
pub fn sheet(home: &Path, p: &Palette) -> std::path::PathBuf {
    let mode = if p.is_dark() { "dark" } else { "light" };
    home.join(format!(".config/gtk-4.0/cosmic/{mode}.css"))
}

/// Which of `FLATPAK_SHARES` the output of `flatpak override --user --show`
/// does not list yet.
pub fn missing_shares(shown: &str) -> Vec<&'static str> {
    FLATPAK_SHARES
        .iter()
        .copied()
        .filter(|s| !shown.contains(s.split(':').next().unwrap_or(s)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_is_judged_by_its_window_color() {
        let dir = std::env::temp_dir().join(format!("ctp-gtk-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mocha = Palette::new("mocha").unwrap();
        let file = dir.join("dark.css");
        assert_eq!(css_state(&file, &mocha), Css::Missing);
        std::fs::write(
            &file,
            "@define-color window_bg_color rgba(30, 30, 46, 1.00);\n",
        )
        .unwrap();
        assert_eq!(css_state(&file, &mocha), Css::Follows);
        std::fs::write(
            &file,
            "@define-color window_bg_color rgba(10, 10, 10, 1.00);\n",
        )
        .unwrap();
        assert_eq!(css_state(&file, &mocha), Css::Other);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn only_unlisted_shares_are_missing() {
        let shown = "[Context]\nfilesystems=xdg-config/gtk-4.0:ro;xdg-data/color-schemes:ro;\n";
        assert_eq!(missing_shares(shown), ["xdg-config/gtk-3.0:ro"]);
        assert!(missing_shares("xdg-config/gtk-3.0;xdg-config/gtk-4.0").is_empty());
        assert_eq!(missing_shares(""), FLATPAK_SHARES);
    }
}
