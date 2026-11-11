// SPDX-License-Identifier: MIT

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

/// What the app remembers between runs.
#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub flavor: String,
    pub accent: String,
    pub style: String,
    /// Latte for COSMIC's light mode, with the desktop switching by itself.
    pub light_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            style: "glow".into(),
            light_mode: false,
        }
    }
}
