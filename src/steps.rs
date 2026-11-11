//! The steps `apply` runs and the slideshow intervals, shared by `apply`
//! and the interactive setup.

/// One step of `apply`.
pub struct Step {
    pub name: &'static str,
    pub label: &'static str,
    /// Runs only when named explicitly; never part of the default selection.
    pub optional: bool,
    /// What the step changes among the backed-up files: path prefixes
    /// relative to $HOME (`uninstall --only` restores just these).
    pub paths: &'static [&'static str],
}

/// In installer order.
pub const STEPS: [Step; 7] = [
    Step {
        name: "cosmic",
        label: "Desktop theme",
        optional: false,
        paths: &[".config/cosmic/com.system76.CosmicTheme"],
    },
    Step {
        name: "terminal",
        label: "Terminal colors",
        optional: false,
        paths: &[".config/cosmic/com.system76.CosmicTerm/"],
    },
    Step {
        name: "bash",
        label: "Bash prompt",
        optional: false,
        paths: &[".bashrc"],
    },
    Step {
        name: "zed",
        label: "Zed theme",
        optional: false,
        paths: &[".config/zed/settings.json"],
    },
    Step {
        name: "wallpaper",
        label: "Wallpaper",
        optional: false,
        paths: &[
            ".config/cosmic/com.system76.CosmicBackground/",
            ".config/cosmic/com.system76.CosmicSettings.Wallpaper/",
        ],
    },
    Step {
        name: "gtk",
        label: "GTK and Flatpak apps (optional)",
        optional: true,
        paths: &[".local/share/flatpak/overrides/"],
    },
    // Last: the slowest step.
    Step {
        name: "icons",
        label: "Papirus icons (optional)",
        optional: true,
        paths: &[
            ".config/cosmic/com.system76.CosmicTk/v1/icon_theme",
            ".config/cosmic/com.system76.CosmicTheme.Dark/v2/icon_theme",
            ".config/cosmic/com.system76.CosmicTheme.Dark.Builder/v2/icon_theme",
            ".config/cosmic/com.system76.CosmicTheme.Light/v2/icon_theme",
            ".config/cosmic/com.system76.CosmicTheme.Light.Builder/v2/icon_theme",
        ],
    },
];

/// Intervals COSMIC Settings → Wallpaper offers: (label, seconds).
const ROTATE: [(&str, u32); 6] = [
    ("5m", 300),
    ("10m", 600),
    ("15m", 900),
    ("30m", 1800),
    ("1h", 3600),
    ("2h", 7200),
];

/// The slideshow interval labels, shortest first.
pub fn rotate_labels() -> Vec<&'static str> {
    ROTATE.iter().map(|(l, _)| *l).collect()
}

/// Seconds for a label ("30m") or the same number as text ("1800").
pub fn rotate_seconds(value: &str) -> Result<u32, String> {
    ROTATE
        .iter()
        .find(|(label, secs)| *label == value || secs.to_string() == value)
        .map(|(_, secs)| *secs)
        .ok_or_else(|| {
            let labels: Vec<_> = ROTATE.iter().map(|(l, _)| *l).collect();
            format!(
                "invalid WALLPAPER_ROTATE '{value}' (choose: {})",
                labels.join(", ")
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_accepts_labels_and_seconds_only() {
        assert_eq!(rotate_seconds("30m"), Ok(1800));
        assert_eq!(rotate_seconds("7200"), Ok(7200));
        assert!(rotate_seconds("1m").is_err());
    }
}
