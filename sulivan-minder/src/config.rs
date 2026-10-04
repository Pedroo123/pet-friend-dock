// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct Config {
    /// CPU usage percentage above which the pet runs.
    pub cpu_threshold: f32,
    /// Saved reminders as `(due unix timestamp in seconds, message)`.
    pub reminders: Vec<(i64, String)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cpu_threshold: 20.0,
            reminders: Vec::new(),
        }
    }
}
