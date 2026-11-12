//! Device status for the right rail: clock and battery.
//!
//! Battery comes from sysfs for now; UPower over D-Bus replaces it in a later step.

use std::path::Path;

/// Current local time as `HH:MM`.
pub fn clock_text() -> String {
    gtk4::glib::DateTime::now_local()
        .and_then(|t| t.format("%H:%M"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Charge of the first power supply of type `Battery`, e.g. `55%`, or `None` without one.
///
/// Matched by `type`, not by name: tablets often expose the battery as a fuel gauge
/// (`axp288_fuel_gauge` on the reference device) rather than `BAT0`.
pub fn battery_text() -> Option<String> {
    battery_text_in(Path::new("/sys/class/power_supply"))
}

fn battery_text_in(dir: &Path) -> Option<String> {
    let mut batteries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| std::fs::read_to_string(p.join("type")).is_ok_and(|t| t.trim() == "Battery"))
        .collect();
    batteries.sort();
    let raw = std::fs::read_to_string(batteries.first()?.join("capacity")).ok()?;
    format_capacity(&raw)
}

/// Formats a sysfs `capacity` value; rejects anything that is not 0–100.
pub fn format_capacity(raw: &str) -> Option<String> {
    let value: u8 = raw.trim().parse().ok()?;
    (value <= 100).then(|| format!("{value}%"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_valid_capacity() {
        assert_eq!(format_capacity("55\n").as_deref(), Some("55%"));
        assert_eq!(format_capacity("100").as_deref(), Some("100%"));
    }

    #[test]
    fn rejects_garbage_capacity() {
        assert_eq!(format_capacity(""), None);
        assert_eq!(format_capacity("abc"), None);
        assert_eq!(format_capacity("150"), None);
    }

    #[test]
    fn missing_power_supply_dir_is_none() {
        assert_eq!(battery_text_in(Path::new("/nonexistent/bayleaf")), None);
    }

    #[test]
    fn finds_battery_by_type_not_name() {
        let root = std::env::temp_dir().join(format!("bayleaf-ps-{}", std::process::id()));
        let supply = |name: &str, kind: &str, cap: Option<&str>| {
            let d = root.join(name);
            std::fs::create_dir_all(&d).ok();
            std::fs::write(d.join("type"), format!("{kind}\n")).ok();
            if let Some(c) = cap {
                std::fs::write(d.join("capacity"), c).ok();
            }
        };
        supply("axp288_charger", "USB", None);
        supply("axp288_fuel_gauge", "Battery", Some("91\n"));
        assert_eq!(battery_text_in(&root).as_deref(), Some("91%"));
        std::fs::remove_dir_all(&root).ok();
    }
}
