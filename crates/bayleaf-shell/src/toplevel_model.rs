//! Pure model of the compositor's toplevel windows, fed by protocol events.
//!
//! Kept free of Wayland and GTK types so it can be unit-tested.

/// Stable identifier of a toplevel for the lifetime of its protocol object.
pub type ToplevelId = u32;

/// Snapshot of one toplevel, sent when the compositor finishes a batch of changes (`done`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Toplevel {
    pub id: ToplevelId,
    pub app_id: String,
    pub title: String,
    pub activated: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
}

/// Change reported by the protocol client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToplevelEvent {
    Updated(Toplevel),
    Closed(ToplevelId),
    /// The compositor does not implement wlr-foreign-toplevel-management.
    Unsupported,
}

/// Window states of `zwlr_foreign_toplevel_handle_v1.state`, decoded from the raw array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct States {
    pub maximized: bool,
    pub minimized: bool,
    pub activated: bool,
    pub fullscreen: bool,
}

/// Decodes the `state` array: native-endian u32 values (0 maximized, 1 minimized,
/// 2 activated, 3 fullscreen). Unknown values and trailing bytes are ignored.
pub fn parse_states(raw: &[u8]) -> States {
    let mut s = States::default();
    for chunk in raw.chunks_exact(4) {
        let value = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        match value {
            0 => s.maximized = true,
            1 => s.minimized = true,
            2 => s.activated = true,
            3 => s.fullscreen = true,
            _ => {}
        }
    }
    s
}

/// Ordered list of toplevels as the rail shows them: in the order they first appeared.
#[derive(Debug, Default)]
pub struct Toplevels {
    items: Vec<Toplevel>,
    supported: bool,
}

impl Toplevels {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            supported: true,
        }
    }

    /// Applies an event. Returns `true` if what the rail shows changed.
    pub fn apply(&mut self, event: ToplevelEvent) -> bool {
        match event {
            ToplevelEvent::Updated(t) => match self.items.iter_mut().find(|i| i.id == t.id) {
                Some(existing) if *existing == t => false,
                Some(existing) => {
                    *existing = t;
                    true
                }
                None => {
                    self.items.push(t);
                    true
                }
            },
            ToplevelEvent::Closed(id) => {
                let before = self.items.len();
                self.items.retain(|i| i.id != id);
                self.items.len() != before
            }
            ToplevelEvent::Unsupported => {
                let changed = self.supported || !self.items.is_empty();
                self.supported = false;
                self.items.clear();
                changed
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Toplevel> {
        self.items.iter()
    }

    pub fn is_supported(&self) -> bool {
        self.supported
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_ne_bytes()).collect()
    }

    fn top(id: ToplevelId, title: &str) -> Toplevel {
        Toplevel {
            id,
            title: title.into(),
            ..Default::default()
        }
    }

    #[test]
    fn parses_known_states() {
        let s = parse_states(&raw(&[2, 0]));
        assert!(s.activated && s.maximized);
        assert!(!s.minimized && !s.fullscreen);
    }

    #[test]
    fn ignores_unknown_states_and_trailing_bytes() {
        let mut bytes = raw(&[99, 1]);
        bytes.push(7);
        let s = parse_states(&bytes);
        assert!(s.minimized);
        assert!(!s.activated);
    }

    #[test]
    fn keeps_first_seen_order_on_update() {
        let mut m = Toplevels::new();
        assert!(m.apply(ToplevelEvent::Updated(top(1, "a"))));
        assert!(m.apply(ToplevelEvent::Updated(top(2, "b"))));
        assert!(m.apply(ToplevelEvent::Updated(top(1, "a2"))));
        let titles: Vec<_> = m.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["a2", "b"]);
    }

    #[test]
    fn identical_update_reports_no_change() {
        let mut m = Toplevels::new();
        m.apply(ToplevelEvent::Updated(top(1, "a")));
        assert!(!m.apply(ToplevelEvent::Updated(top(1, "a"))));
    }

    #[test]
    fn closed_removes_and_unknown_close_is_noop() {
        let mut m = Toplevels::new();
        m.apply(ToplevelEvent::Updated(top(1, "a")));
        assert!(m.apply(ToplevelEvent::Closed(1)));
        assert!(!m.apply(ToplevelEvent::Closed(1)));
        assert_eq!(m.iter().count(), 0);
    }

    #[test]
    fn unsupported_clears_and_flags() {
        let mut m = Toplevels::new();
        m.apply(ToplevelEvent::Updated(top(1, "a")));
        assert!(m.apply(ToplevelEvent::Unsupported));
        assert!(!m.is_supported());
        assert_eq!(m.iter().count(), 0);
    }
}
