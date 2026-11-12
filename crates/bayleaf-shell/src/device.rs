//! Brightness and volume backends for the right rail.
//!
//! First iteration: brightness is read from sysfs and set through logind's
//! `Session.SetBrightness` (no root, no udev rules); volume goes through WirePlumber's `wpctl`.
//! Both are run asynchronously via GIO so the UI never blocks. Native D-Bus / PipeWire clients
//! replace these later without changing the rail code.

use std::path::{Path, PathBuf};

use gtk4::gio;

/// Runs a command without waiting for it; GIO reaps the child.
fn spawn(argv: &[&str]) {
    let args: Vec<&std::ffi::OsStr> = argv.iter().map(std::ffi::OsStr::new).collect();
    if let Err(e) = gio::Subprocess::newv(&args, gio::SubprocessFlags::STDOUT_SILENCE) {
        tracing::warn!(error = %e, cmd = argv.first().copied().unwrap_or(""), "spawn failed");
    }
}

/// Runs a command and returns its stdout.
async fn output(argv: &[&str]) -> Option<String> {
    let args: Vec<&std::ffi::OsStr> = argv.iter().map(std::ffi::OsStr::new).collect();
    let p = gio::Subprocess::newv(&args, gio::SubprocessFlags::STDOUT_PIPE)
        .map_err(|e| tracing::warn!(error = %e, "spawn failed"))
        .ok()?;
    let (out, _) = p.communicate_utf8_future(None).await.ok()?;
    out.map(|s| s.to_string())
}

// --- brightness ---------------------------------------------------------------------------

/// A backlight device in `/sys/class/backlight`.
#[derive(Debug, Clone)]
pub struct Backlight {
    name: String,
    dir: PathBuf,
    max: u32,
}

impl Backlight {
    /// First backlight device, preferring firmware/platform ones over raw.
    pub fn find() -> Option<Self> {
        Self::find_in(Path::new("/sys/class/backlight"))
    }

    fn find_in(root: &Path) -> Option<Self> {
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .collect();
        dirs.sort();
        dirs.into_iter().find_map(|dir| {
            let max = read_u32(&dir.join("max_brightness"))?;
            let name = dir.file_name()?.to_string_lossy().into_owned();
            (max > 0).then_some(Self { name, dir, max })
        })
    }

    pub fn percent(&self) -> Option<f64> {
        read_u32(&self.dir.join("brightness")).map(|raw| raw_to_percent(raw, self.max))
    }

    pub fn set_percent(&self, percent: f64) {
        let raw = percent_to_raw(percent, self.max).to_string();
        spawn(&[
            "busctl",
            "call",
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto",
            "org.freedesktop.login1.Session",
            "SetBrightness",
            "ssu",
            "backlight",
            &self.name,
            &raw,
        ]);
    }
}

fn read_u32(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn raw_to_percent(raw: u32, max: u32) -> f64 {
    if max == 0 {
        return 0.0;
    }
    (f64::from(raw.min(max)) * 100.0 / f64::from(max)).round()
}

/// Percent to a raw value, never 0: a fully dark panel looks like a dead device.
pub fn percent_to_raw(percent: f64, max: u32) -> u32 {
    let p = percent.clamp(0.0, 100.0);
    let raw = (p * f64::from(max) / 100.0).round() as u32;
    raw.clamp(1, max.max(1))
}

// --- volume -------------------------------------------------------------------------------

const SINK: &str = "@DEFAULT_AUDIO_SINK@";

/// Default sink volume in percent (0–100), `None` if WirePlumber is unavailable.
pub async fn volume_percent() -> Option<f64> {
    let out = output(&["wpctl", "get-volume", SINK]).await?;
    parse_wpctl_volume(&out).map(|(v, _muted)| v)
}

pub fn set_volume_percent(percent: f64) {
    let fraction = format!("{:.2}", percent.clamp(0.0, 100.0) / 100.0);
    spawn(&["wpctl", "set-volume", SINK, &fraction]);
}

pub fn set_muted(muted: bool) {
    spawn(&["wpctl", "set-mute", SINK, if muted { "1" } else { "0" }]);
}

/// Whether moving the volume from `previous` to `next` crosses zero, i.e. needs a mute change.
pub fn mute_change(previous: Option<f64>, next: f64) -> Option<bool> {
    let now_muted = next <= 0.0;
    match previous {
        None => Some(now_muted),
        Some(p) if (p <= 0.0) != now_muted => Some(now_muted),
        Some(_) => None,
    }
}

/// Parses `wpctl get-volume` output, e.g. `Volume: 0.45 [MUTED]` → `(45.0, true)`.
pub fn parse_wpctl_volume(out: &str) -> Option<(f64, bool)> {
    let rest = out.trim().strip_prefix("Volume:")?.trim();
    let value: f64 = rest.split_whitespace().next()?.parse().ok()?;
    Some((
        (value * 100.0).round().clamp(0.0, 150.0),
        rest.contains("[MUTED]"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_raw_round_trip() {
        assert_eq!(raw_to_percent(50, 100), 50.0);
        assert_eq!(percent_to_raw(50.0, 100), 50);
        assert_eq!(raw_to_percent(937, 937), 100.0);
    }

    #[test]
    fn brightness_never_goes_fully_dark() {
        assert_eq!(percent_to_raw(0.0, 100), 1);
        assert_eq!(percent_to_raw(-5.0, 937), 1);
        assert_eq!(percent_to_raw(150.0, 937), 937);
    }

    #[test]
    fn mute_changes_only_when_crossing_zero() {
        assert_eq!(mute_change(None, 30.0), Some(false));
        assert_eq!(mute_change(Some(30.0), 40.0), None);
        assert_eq!(mute_change(Some(30.0), 0.0), Some(true));
        assert_eq!(mute_change(Some(0.0), 0.0), None);
        assert_eq!(mute_change(Some(0.0), 5.0), Some(false));
    }

    #[test]
    fn zero_max_is_safe() {
        assert_eq!(raw_to_percent(10, 0), 0.0);
        assert_eq!(percent_to_raw(50.0, 0), 1);
    }

    #[test]
    fn parses_wpctl_output() {
        assert_eq!(parse_wpctl_volume("Volume: 0.45\n"), Some((45.0, false)));
        assert_eq!(
            parse_wpctl_volume("Volume: 0.80 [MUTED]"),
            Some((80.0, true))
        );
        assert_eq!(parse_wpctl_volume("garbage"), None);
    }

    #[test]
    fn missing_backlight_dir_is_none() {
        assert!(Backlight::find_in(Path::new("/nonexistent/bayleaf")).is_none());
    }
}
