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

/// Battery charge of the first `BAT*` power supply, e.g. `55%`, or `None` without a battery.
pub fn battery_text() -> Option<String> {
    battery_text_in(Path::new("/sys/class/power_supply"))
}

fn battery_text_in(dir: &Path) -> Option<String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("BAT"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    let raw = std::fs::read_to_string(entries.first()?.path().join("capacity")).ok()?;
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
}
