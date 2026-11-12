//! Shared protocol types between the Bayleaf compositor, shell and tools.
//!
//! The wire format and transport are not decided yet (see `docs/architecture.md`);
//! for now this crate only holds the core state types every component agrees on.

use serde::{Deserialize, Serialize};

/// Whether the device is used as a tablet or as a laptop (keyboard dock attached).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Mode {
    /// No keyboard: one maximized window at a time, touch-first UI.
    #[default]
    Tablet,
    /// Keyboard dock attached: floating windows, pointer-first UI.
    Laptop,
}

/// Screen orientation, matching the accelerometer readings of iio-sensor-proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Orientation {
    #[default]
    Normal,
    LeftUp,
    BottomUp,
    RightUp,
}

impl Orientation {
    /// Clockwise rotation in degrees to apply to the output so content stays upright.
    pub fn transform_degrees(self) -> u16 {
        match self {
            Orientation::Normal => 0,
            Orientation::RightUp => 90,
            Orientation::BottomUp => 180,
            Orientation::LeftUp => 270,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_tablet() {
        assert_eq!(Mode::default(), Mode::Tablet);
    }

    #[test]
    fn transforms_cover_all_quarter_turns() {
        let mut degs: Vec<u16> = [
            Orientation::Normal,
            Orientation::LeftUp,
            Orientation::BottomUp,
            Orientation::RightUp,
        ]
        .iter()
        .map(|o| o.transform_degrees())
        .collect();
        degs.sort_unstable();
        assert_eq!(degs, vec![0, 90, 180, 270]);
    }
}
