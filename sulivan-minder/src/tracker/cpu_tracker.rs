// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Mutex};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

/// Samples the global CPU usage. Cheap to clone; clones share the same sampler.
#[derive(Clone)]
pub struct CpuTracker {
    system: Arc<Mutex<System>>,
}

impl Default for CpuTracker {
    fn default() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        // The first reading is meaningless; a second refresh yields a real delta.
        system.refresh_cpu_usage();
        Self {
            system: Arc::new(Mutex::new(system)),
        }
    }
}

impl CpuTracker {
    /// Refreshes and returns the global CPU usage (0.0..=100.0).
    /// This touches /proc, so call it from a blocking task.
    pub fn sample(&self) -> f32 {
        let mut system = self
            .system
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        system.refresh_cpu_usage();
        system.global_cpu_usage()
    }
}

/// Whether the pet should be running for the given usage and threshold.
pub fn is_over_threshold(usage: f32, threshold: f32) -> bool {
    usage > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_comparison() {
        assert!(is_over_threshold(60.0, 50.0));
        assert!(!is_over_threshold(50.0, 50.0));
        assert!(!is_over_threshold(10.0, 50.0));
    }
}
