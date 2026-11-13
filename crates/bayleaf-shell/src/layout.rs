//! Bayleaf's tablet window model on top of niri, as pure functions (docs/design/niri-integration.md):
//! one screen = one niri workspace, holding one app or a pair.
//!
//! Widths and the view are niri's job: our fork's `screen-split` layout option makes one column
//! fill the screen and two columns split it in half, without scrolling, and opens new windows on
//! a screen of their own. The shell only decides which window goes to which screen and side.
//!
//! Takes a snapshot of niri's windows and workspaces and returns the commands to send; kept free
//! of IPC and GTK so it can be unit-tested.

use crate::toplevel_model::Toplevel;
use crate::windows::Side;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Win {
    pub id: u64,
    pub workspace: Option<u64>,
    pub floating: bool,
    pub focused: bool,
    /// Column index in the workspace's scrolling layout (tiled windows only).
    pub column: Option<usize>,
    pub app_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ws {
    pub id: u64,
    pub idx: u8,
    pub output: Option<String>,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Snapshot {
    pub windows: Vec<Win>,
    pub workspaces: Vec<Ws>,
}

/// One niri action, in Bayleaf terms.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Focus(u64),
    Close(u64),
    /// Move a window to a workspace without following it.
    MoveToWorkspace {
        window: u64,
        workspace: u64,
    },
    /// Move the focused column to a 1-based index in its workspace.
    MoveFocusedColumnTo(usize),
    /// Move a window to a workspace as a new column at a 1-based index and focus it, in one step
    /// (our fork's `MoveWindowToWorkspaceColumn`).
    PlaceInColumn {
        window: u64,
        workspace: u64,
        column: usize,
    },
}

impl Snapshot {
    fn window(&self, id: u64) -> Option<&Win> {
        self.windows.iter().find(|w| w.id == id)
    }

    fn workspace(&self, id: u64) -> Option<&Ws> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    fn focused_workspace(&self) -> Option<&Ws> {
        self.workspaces.iter().find(|w| w.focused)
    }

    /// Tiled windows of a workspace, left to right.
    fn tiled_in(&self, workspace: u64) -> Vec<&Win> {
        let mut v: Vec<&Win> = self
            .windows
            .iter()
            .filter(|w| w.workspace == Some(workspace) && !w.floating)
            .collect();
        v.sort_by_key(|w| (w.column, w.id));
        v
    }

    /// The empty workspace niri keeps at the end of an output.
    fn empty_workspace_on(&self, output: &Option<String>) -> Option<&Ws> {
        self.workspaces
            .iter()
            .filter(|ws| &ws.output == output)
            .filter(|ws| !self.windows.iter().any(|w| w.workspace == Some(ws.id)))
            .max_by_key(|ws| ws.idx)
    }

    /// Whether the window shares its screen with another tiled window.
    fn is_paired(&self, win: &Win) -> bool {
        win.workspace
            .is_some_and(|ws| self.tiled_in(ws).iter().any(|w| w.id != win.id))
    }

    /// The rail's view: windows in the order they were opened; the focused screen is visible.
    pub fn toplevels(&self) -> Vec<Toplevel> {
        let shown = self.focused_workspace().map(|ws| ws.id);
        let mut v: Vec<Toplevel> = self
            .windows
            .iter()
            .map(|w| Toplevel {
                id: w.id,
                app_id: w.app_id.clone(),
                title: w.title.clone(),
                activated: w.focused,
                visible: w.workspace.is_some() && w.workspace == shown,
                ..Default::default()
            })
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }
}

/// Moves the window to the empty workspace of its output and shows it there.
fn to_own_screen(s: &Snapshot, win: &Win) -> Vec<Cmd> {
    let output = win
        .workspace
        .and_then(|w| s.workspace(w))
        .and_then(|ws| ws.output.clone());
    match s.empty_workspace_on(&output) {
        Some(empty) => vec![
            Cmd::MoveToWorkspace {
                window: win.id,
                workspace: empty.id,
            },
            Cmd::Focus(win.id),
        ],
        None => vec![Cmd::Focus(win.id)],
    }
}

/// Shows the window alone: out of its pair onto a screen of its own, or just focused.
pub fn open_alone(s: &Snapshot, id: u64) -> Vec<Cmd> {
    match s.window(id) {
        Some(win) if !win.floating && s.is_paired(win) => to_own_screen(s, win),
        Some(_) => vec![Cmd::Focus(id)],
        None => Vec::new(),
    }
}

/// Opens `id` on one half of the shown screen, next to what is there. On a full pair, the app on
/// that side gets replaced and moves to a screen of its own.
pub fn open_on_side(s: &Snapshot, id: u64, side: Side) -> Vec<Cmd> {
    let (Some(win), Some(shown)) = (s.window(id), s.focused_workspace()) else {
        return Vec::new();
    };
    let index = match side {
        Side::Left => 1,
        Side::Right => 2,
    };
    let here = s.tiled_in(shown.id);

    // Already on this screen: swap sides if it is a pair, otherwise nothing to do.
    if win.workspace == Some(shown.id) {
        if here.len() == 2 && here.get(index - 1).map(|w| w.id) != Some(id) {
            return vec![Cmd::Focus(id), Cmd::MoveFocusedColumnTo(index)];
        }
        return Vec::new();
    }

    let others: Vec<&Win> = here.into_iter().filter(|w| w.id != id).collect();
    if others.is_empty() {
        return vec![Cmd::Focus(id)];
    }

    // On a full pair, niri (screen-split) moves the app on that side to a screen of its own in
    // the same step.
    vec![Cmd::PlaceInColumn {
        window: id,
        workspace: shown.id,
        column: index,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws(id: u64, idx: u8, focused: bool) -> Ws {
        Ws {
            id,
            idx,
            output: Some("DSI-1".into()),
            focused,
        }
    }

    fn win(id: u64, workspace: u64, column: usize) -> Win {
        Win {
            id,
            workspace: Some(workspace),
            column: Some(column),
            ..Default::default()
        }
    }

    /// Workspace 1 (focused) and 2 hold one app each; 3 is the empty one at the end.
    fn two_screens() -> Snapshot {
        Snapshot {
            windows: vec![win(10, 1, 1), win(20, 2, 1)],
            workspaces: vec![ws(1, 1, true), ws(2, 2, false), ws(3, 3, false)],
        }
    }

    fn move_to(window: u64, workspace: u64) -> Cmd {
        Cmd::MoveToWorkspace { window, workspace }
    }

    fn place(window: u64, workspace: u64, column: usize) -> Cmd {
        Cmd::PlaceInColumn {
            window,
            workspace,
            column,
        }
    }

    #[test]
    fn open_on_left_pairs_with_the_shown_app() {
        assert_eq!(
            open_on_side(&two_screens(), 20, Side::Left),
            vec![place(20, 1, 1)]
        );
    }

    #[test]
    fn open_on_right_of_a_pair_replaces_the_right_app() {
        let mut s = two_screens();
        s.windows.push(win(11, 1, 2));
        assert_eq!(open_on_side(&s, 20, Side::Right), vec![place(20, 1, 2)]);
    }

    #[test]
    fn swapping_sides_within_a_pair() {
        let mut s = two_screens();
        s.windows.push(win(11, 1, 2));
        assert_eq!(
            open_on_side(&s, 11, Side::Left),
            vec![Cmd::Focus(11), Cmd::MoveFocusedColumnTo(1)]
        );
        assert!(open_on_side(&s, 10, Side::Left).is_empty());
    }

    #[test]
    fn open_alone_takes_an_app_out_of_its_pair() {
        let mut s = two_screens();
        s.windows.push(win(11, 1, 2));
        assert_eq!(open_alone(&s, 11), vec![move_to(11, 3), Cmd::Focus(11)]);
        assert_eq!(open_alone(&s, 20), vec![Cmd::Focus(20)]);
    }

    #[test]
    fn toplevels_mark_the_shown_screen_visible() {
        let mut s = two_screens();
        s.windows.push(win(11, 1, 2));
        s.windows[0].focused = true;
        let t = s.toplevels();
        let visible: Vec<u64> = t.iter().filter(|t| t.visible).map(|t| t.id).collect();
        assert_eq!(visible, [10, 11]);
        assert!(t.iter().find(|t| t.id == 10).is_some_and(|t| t.activated));
    }
}
