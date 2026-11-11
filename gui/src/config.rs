// SPDX-License-Identifier: MIT

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

/// What the app remembers between runs: one look per COSMIC mode. The light
/// look is always Latte, the only light flavor.
#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 2]
pub struct Config {
    pub dark_flavor: String,
    pub dark_accent: String,
    pub dark_style: String,
    pub light_accent: String,
    pub light_style: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dark_flavor: "mocha".into(),
            dark_accent: "mauve".into(),
            dark_style: "glow".into(),
            light_accent: "mauve".into(),
            light_style: "glow".into(),
        }
    }
}
