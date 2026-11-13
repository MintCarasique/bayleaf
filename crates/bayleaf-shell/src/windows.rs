//! What the rail can ask the compositor to do with a window, independent of the protocol behind
//! it: wlr-foreign-toplevel (any wlroots-style compositor) or niri IPC (ADR-0001).

use crate::toplevel_model::ToplevelId;

/// Half of the screen in split view (docs/design/multitasking.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub trait WindowControl {
    /// Show the window (its screen, on niri) and focus it.
    fn activate(&self, id: ToplevelId);

    /// Ask the app to close the window; it may ask to save first.
    fn close(&self, id: ToplevelId);

    /// Whether [`WindowControl::open_on_side`] does anything with this compositor.
    fn supports_split(&self) -> bool {
        false
    }

    /// Open the window on one half of the shown screen, next to what is there now.
    fn open_on_side(&self, id: ToplevelId, side: Side) {
        let _ = (id, side);
    }

    /// Show the window on a screen of its own (out of its pair, if it is in one).
    fn open_alone(&self, id: ToplevelId) {
        self.activate(id);
    }
}
