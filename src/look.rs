//! A look: the Catppuccin theme and wallpaper of one COSMIC mode, saved as a
//! theme of its own (a card under Settings → Appearance).
//!
//! The name carries the wallpaper style, so each look keeps its own wallpaper
//! instead of replacing the card of the same flavor and accent.

use std::path::{Path, PathBuf};

use crate::theme::{self, InstallOpts};
use crate::{background, palette, wallpaper};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Look {
    pub flavor: String,
    pub accent: String,
    pub style: String,
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

impl Look {
    /// "Catppuccin Mocha Teal · Lissajous".
    pub fn name(&self) -> String {
        format!(
            "Catppuccin {} {} · {}",
            capitalize(&self.flavor),
            capitalize(&self.accent),
            capitalize(&self.style)
        )
    }

    pub fn is_dark(&self) -> bool {
        palette::check(&self.flavor, None).is_ok_and(|p| p.is_dark())
    }

    /// The wallpaper file, named as `apply` names its wallpapers.
    fn wallpaper(&self, pictures: &Path) -> PathBuf {
        let suffix = if self.style == "glow" {
            String::new()
        } else {
            format!("-{}", self.style)
        };
        pictures.join(format!(
            "catppuccin-{}-{}{suffix}.png",
            self.flavor, self.accent
        ))
    }
}

/// Where a look is written.
#[derive(Clone)]
pub struct Dirs {
    /// COSMIC's config folder.
    pub config: PathBuf,
    pub pictures: PathBuf,
    /// Where the generated theme files go.
    pub data: PathBuf,
}

impl Dirs {
    pub fn home() -> Result<Self, String> {
        let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME not set")?);
        Ok(Self {
            config: home.join(".config/cosmic"),
            pictures: home.join("Pictures/Wallpapers"),
            data: home.join(".local/share/catppuccin/themes/cosmic-settings"),
        })
    }
}

/// The slug of the saved theme named `name`: COSMIC derives it from the name
/// and may shorten it or add a number, so it is looked up.
pub fn bundle_slug(config: &Path, name: &str) -> Option<String> {
    let want = format!("{name:?}");
    std::fs::read_dir(config)
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|id| {
            let slug = id
                .strip_prefix("com.system76.CosmicThemeBundle.")?
                .strip_suffix(".Theme")?;
            Some(slug.to_string())
        })
        .find(|slug| {
            let file = config.join(format!(
                "com.system76.CosmicThemeBundle.{slug}.Theme/v2/name"
            ));
            std::fs::read_to_string(file).is_ok_and(|n| n.trim() == want)
        })
}

/// Fails on a bad flavor, accent or style before anything is changed.
pub fn check(look: &Look) -> Result<(), String> {
    palette::check(&look.flavor, Some(&look.accent))?;
    if !crate::styles::list().iter().any(|(n, _)| *n == look.style) {
        return Err(format!("unknown style '{}'", look.style));
    }
    Ok(())
}

/// Step 1 of applying: renders the wallpaper file.
pub fn render_wallpaper(look: &Look, dirs: &Dirs) -> Result<PathBuf, String> {
    check(look)?;
    std::fs::create_dir_all(&dirs.pictures)
        .map_err(|e| format!("{}: {e}", dirs.pictures.display()))?;
    let png = look.wallpaper(&dirs.pictures);
    wallpaper::render_to(&png, &look.style, &look.flavor, &look.accent)?;
    Ok(png)
}

/// Step 2: imports the theme under the look's name. COSMIC saves it as a
/// theme and makes it the live one for its mode. Its own gaps, corner style,
/// fonts, icons, density and Frosted Glass are kept. `import` off only writes
/// the files.
pub fn import_theme(look: &Look, dirs: &Dirs, import: bool) -> Result<(), String> {
    check(look)?;
    let mut t = InstallOpts::new(&look.flavor, &look.accent);
    t.keep_layout = true;
    t.name = Some(look.name());
    t.apply = import;
    t.config.clone_from(&dirs.config);
    t.data.clone_from(&dirs.data);
    theme::install_theme(&t).map(drop)
}

/// Step 3: sets the wallpaper for the look's mode, in the saved theme too.
pub fn set_wallpaper(look: &Look, dirs: &Dirs, png: &Path) -> Result<(), String> {
    let slug = bundle_slug(&dirs.config, &look.name());
    background::set(
        &dirs.config,
        &png.display().to_string(),
        None,
        look.is_dark(),
        slug.as_deref(),
    )
}

/// All three steps.
pub fn apply(look: &Look, dirs: &Dirs, import: bool) -> Result<(), String> {
    let png = render_wallpaper(look, dirs)?;
    import_theme(look, dirs, import)?;
    set_wallpaper(look, dirs, &png)
}

/// The name of the theme live in a mode, if COSMIC has one.
pub fn active_name(config: &Path, dark: bool) -> Option<String> {
    let file = config.join(format!(
        "com.system76.CosmicTheme.{}/v2/name",
        if dark { "Dark" } else { "Light" }
    ));
    let text = std::fs::read_to_string(file).ok()?;
    Some(text.trim().trim_matches('"').to_string())
}

/// Whether COSMIC is in dark mode (its default without a setting).
pub fn is_dark_now(config: &Path) -> bool {
    std::fs::read_to_string(config.join("com.system76.CosmicTheme.Mode/v1/is_dark"))
        .map(|v| v.trim() != "false")
        .unwrap_or(true)
}

pub fn auto_switch(config: &Path) -> bool {
    std::fs::read_to_string(config.join("com.system76.CosmicTheme.Mode/v1/auto_switch"))
        .is_ok_and(|v| v.trim() == "true")
}

/// Follow sunrise and sunset (or not), as the toggle under Appearance does.
pub fn set_auto_switch(config: &Path, on: bool) -> Result<(), String> {
    let dir = config.join("com.system76.CosmicTheme.Mode/v1");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let file = dir.join("auto_switch");
    std::fs::write(&file, on.to_string()).map_err(|e| format!("{}: {e}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(style: &str) -> Look {
        Look {
            flavor: "mocha".into(),
            accent: "teal".into(),
            style: style.into(),
        }
    }

    #[test]
    fn the_name_carries_the_style() {
        assert_eq!(
            look("lissajous").name(),
            "Catppuccin Mocha Teal · Lissajous"
        );
        assert_ne!(look("glow").name(), look("mesh").name());
    }

    #[test]
    fn apply_writes_the_theme_and_the_wallpaper_of_its_mode() {
        let dir = std::env::temp_dir().join(format!("ctp-look-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let dirs = Dirs {
            config: dir.join("cfg"),
            pictures: dir.join("pics"),
            data: dir.join("data"),
        };
        // A saved theme COSMIC made for this name, and a layout of its own.
        let part = |p: &str| {
            dirs.config.join(format!(
                "com.system76.CosmicThemeBundle.catppuccin-mocha-teal-mesh.{p}"
            ))
        };
        std::fs::create_dir_all(part("Theme").join("v2")).unwrap();
        std::fs::write(
            part("Theme").join("v2/name"),
            "\"Catppuccin Mocha Teal · Mesh\"",
        )
        .unwrap();
        std::fs::create_dir_all(part("Background").join("v1")).unwrap();
        let builder = dirs.config.join("com.system76.CosmicTheme.Dark.Builder/v2");
        std::fs::create_dir_all(&builder).unwrap();
        std::fs::write(builder.join("gaps"), "(0, 12)").unwrap();
        std::fs::write(builder.join("active_hint"), "2").unwrap();

        apply(&look("mesh"), &dirs, false).unwrap();

        let ron =
            std::fs::read_to_string(dirs.data.join("catppuccin-mocha-teal+custom.ron")).unwrap();
        assert!(ron.contains("name: \"Catppuccin Mocha Teal · Mesh\","));
        assert!(ron.contains("gaps: (0, 12),") && ron.contains("active_hint: 2,"));
        assert!(
            dirs.pictures
                .join("catppuccin-mocha-teal-mesh.png")
                .is_file()
        );
        let live = std::fs::read_to_string(
            dirs.config
                .join("com.system76.CosmicBackground.Dark/v1/all"),
        )
        .unwrap();
        assert!(live.contains("catppuccin-mocha-teal-mesh.png"));
        let saved = std::fs::read_to_string(part("Background").join("v1/all")).unwrap();
        assert!(saved.contains("catppuccin-mocha-teal-mesh.png"));
        assert!(
            !dirs
                .config
                .join("com.system76.CosmicBackground.Light")
                .exists()
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Drives the real `cosmic-settings appearance import`. Run it with HOME
    /// and XDG_CONFIG_HOME pointing at a throwaway copy of ~/.config/cosmic,
    /// whose path contains "ctp-sandbox":
    /// `cargo test -- --ignored real_cosmic_import`.
    #[test]
    #[ignore = "needs cosmic-settings and a sandbox HOME"]
    fn real_cosmic_import() {
        let dirs = Dirs::home().unwrap();
        assert!(
            dirs.config.to_string_lossy().contains("ctp-sandbox"),
            "refusing to touch {}",
            dirs.config.display()
        );
        let read = |p: &str| std::fs::read_to_string(dirs.config.join(p)).unwrap_or_default();
        let dark = look("mesh");
        apply(&dark, &dirs, true).unwrap();
        assert_eq!(
            active_name(&dirs.config, true).as_deref(),
            Some(dark.name().as_str())
        );
        let slug = bundle_slug(&dirs.config, &dark.name()).expect("COSMIC saved a theme");
        let png = "catppuccin-mocha-teal-mesh.png";
        assert!(read("com.system76.CosmicBackground.Dark/v1/all").contains(png));
        assert!(
            read(&format!(
                "com.system76.CosmicThemeBundle.{slug}.Background/v1/all"
            ))
            .contains(png),
            "the saved theme keeps the wallpaper"
        );
        // Layout and fonts the sandbox already had must survive the import.
        assert_eq!(
            read("com.system76.CosmicTheme.Dark/v2/gaps").trim(),
            "(0, 12)"
        );
        assert_eq!(
            read("com.system76.CosmicTheme.Dark/v2/icon_theme").trim(),
            "\"Papirus-Dark\""
        );

        // The same look again saves no second theme.
        let count = || {
            std::fs::read_dir(&dirs.config)
                .unwrap()
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().ends_with(".Theme"))
                .count()
        };
        let before = count();
        apply(&dark, &dirs, true).unwrap();
        assert_eq!(count(), before);

        // A light look sets the light wallpaper and the light mode.
        let light = Look {
            flavor: "latte".into(),
            accent: "blue".into(),
            style: "dots".into(),
        };
        apply(&light, &dirs, true).unwrap();
        assert_eq!(
            active_name(&dirs.config, false).as_deref(),
            Some(light.name().as_str())
        );
        assert!(
            read("com.system76.CosmicBackground.Light/v1/all")
                .contains("catppuccin-latte-blue-dots.png")
        );
        assert!(
            read("com.system76.CosmicBackground.Dark/v1/all").contains(png),
            "dark kept its own"
        );
        assert!(!is_dark_now(&dirs.config));
    }
}
