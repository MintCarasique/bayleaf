//! Bayleaf's tablet window model on top of niri, as pure functions (docs/design/niri-integration.md):
//! one screen = one niri workspace, holding one full-width column or two half-width columns.
//!
//! Takes a snapshot of niri's windows and workspaces and returns the commands to send; kept free
//! of IPC and GTK so it can be unit-tested.

use std::collections::HashMap;

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

/// Windows we made half-width, so they can grow back when they end up alone. `true` once a
/// snapshot has shown the window next to its partner: until then, events still describing the
/// state from before the move must not make it grow back.
pub type Halves = HashMap<u64, bool>;

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
    SetWidthPercent {
        window: u64,
        percent: f64,
    },
    /// Move the focused column to a 1-based index in its workspace.
    MoveFocusedColumnTo(usize),
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

/// A window just opened. If it landed next to another app, move it to a screen of its own.
pub fn place_new(s: &Snapshot, id: u64) -> Vec<Cmd> {
    let Some(win) = s.window(id) else {
        return Vec::new();
    };
    let Some(ws) = win.workspace.and_then(|w| s.workspace(w)) else {
        return Vec::new();
    };
    if win.floating || s.tiled_in(ws.id).iter().all(|w| w.id == id) {
        return Vec::new();
    }
    match s.empty_workspace_on(&ws.output) {
        Some(empty) => vec![
            Cmd::MoveToWorkspace {
                window: id,
                workspace: empty.id,
            },
            Cmd::Focus(id),
        ],
        None => Vec::new(),
    }
}

/// Opens `id` on one half of the shown screen, next to what is there.
pub fn open_on_side(s: &Snapshot, id: u64, side: Side, halves: &mut Halves) -> Vec<Cmd> {
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

    let mut cmds = Vec::new();

    // The partner it leaves behind on its old screen grows back to full width.
    if let Some(from) = win.workspace {
        let left_behind: Vec<&Win> = s
            .tiled_in(from)
            .into_iter()
            .filter(|w| w.id != id)
            .collect();
        if let [partner] = left_behind.as_slice() {
            if halves.remove(&partner.id).is_some() {
                cmds.push(Cmd::SetWidthPercent {
                    window: partner.id,
                    percent: 100.0,
                });
            }
        }
    }

    // A full pair: the app on the chosen side moves to a screen of its own.
    let stays = if others.len() >= 2 {
        let replaced = match side {
            Side::Left => others[0],
            Side::Right => others[others.len() - 1],
        };
        if let Some(empty) = s.empty_workspace_on(&shown.output) {
            cmds.push(Cmd::MoveToWorkspace {
                window: replaced.id,
                workspace: empty.id,
            });
        }
        cmds.push(Cmd::SetWidthPercent {
            window: replaced.id,
            percent: 100.0,
        });
        halves.remove(&replaced.id);
        others.iter().find(|w| w.id != replaced.id).map(|w| w.id)
    } else {
        Some(others[0].id)
    };

    cmds.push(Cmd::MoveToWorkspace {
        window: id,
        workspace: shown.id,
    });
    cmds.push(Cmd::Focus(id));
    for w in std::iter::once(id).chain(stays) {
        cmds.push(Cmd::SetWidthPercent {
            window: w,
            percent: 50.0,
        });
        halves.insert(w, false);
    }
    cmds.push(Cmd::MoveFocusedColumnTo(index));
    cmds
}

/// A half-width window left alone on its screen (its partner closed or moved) grows back.
/// Also confirms halves that are now seen in a pair, and forgets windows that no longer exist.
pub fn fill_lonely(s: &Snapshot, halves: &mut Halves) -> Vec<Cmd> {
    halves.retain(|id, _| s.window(*id).is_some());
    let mut cmds = Vec::new();
    for ws in &s.workspaces {
        let tiled = s.tiled_in(ws.id);
        if tiled.len() >= 2 {
            for w in &tiled {
                if let Some(confirmed) = halves.get_mut(&w.id) {
                    *confirmed = true;
                }
            }
        }
        if let [only] = tiled.as_slice() {
            if halves.get(&only.id) == Some(&true) {
                halves.remove(&only.id);
                cmds.push(Cmd::SetWidthPercent {
                    window: only.id,
                    percent: 100.0,
                });
            }
        }
    }
    cmds
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

    #[test]
    fn new_window_next_to_another_moves_to_the_empty_screen() {
        let mut s = two_screens();
        s.windows.push(win(30, 1, 2));
        assert_eq!(
            place_new(&s, 30),
            vec![
                Cmd::MoveToWorkspace {
                    window: 30,
                    workspace: 3
                },
                Cmd::Focus(30)
            ]
        );
    }

    #[test]
    fn new_window_alone_or_floating_stays() {
        let mut s = two_screens();
        s.windows.push(win(30, 3, 1));
        assert!(place_new(&s, 30).is_empty());

        let mut s = two_screens();
        let mut dialog = win(31, 1, 0);
        dialog.floating = true;
        dialog.column = None;
        s.windows.push(dialog);
        assert!(place_new(&s, 31).is_empty());
    }

    #[test]
    fn open_on_left_pairs_with_the_shown_app() {
        let s = two_screens();
        let mut halves = Halves::new();
        assert_eq!(
            open_on_side(&s, 20, Side::Left, &mut halves),
            vec![
                Cmd::MoveToWorkspace {
                    window: 20,
                    workspace: 1
                },
                Cmd::Focus(20),
                Cmd::SetWidthPercent {
                    window: 20,
                    percent: 50.0
                },
                Cmd::SetWidthPercent {
                    window: 10,
                    percent: 50.0
                },
                Cmd::MoveFocusedColumnTo(1),
            ]
        );
        assert_eq!(halves, Halves::from([(10, false), (20, false)]));
    }

    #[test]
    fn open_on_right_of_a_pair_replaces_the_right_app() {
        let mut s = two_screens();
        s.windows.push(win(11, 1, 2));
        s.windows.push(win(30, 4, 1));
        s.workspaces.push(ws(4, 4, false));
        s.workspaces.push(ws(5, 5, false));
        let mut halves = Halves::from([(10, true), (11, true)]);
        let cmds = open_on_side(&s, 30, Side::Right, &mut halves);
        assert_eq!(
            cmds[..2],
            [
                Cmd::MoveToWorkspace {
                    window: 11,
                    workspace: 5
                },
                Cmd::SetWidthPercent {
                    window: 11,
                    percent: 100.0
                },
            ]
        );
        assert_eq!(cmds.last(), Some(&Cmd::MoveFocusedColumnTo(2)));
        assert_eq!(halves, Halves::from([(10, false), (30, false)]));
    }

    #[test]
    fn moving_out_of_a_pair_grows_the_partner_back() {
        let mut s = two_screens();
        s.windows.push(win(21, 2, 2));
        let mut halves = Halves::from([(20, true), (21, true)]);
        let cmds = open_on_side(&s, 21, Side::Right, &mut halves);
        assert_eq!(
            cmds[0],
            Cmd::SetWidthPercent {
                window: 20,
                percent: 100.0
            }
        );
        assert!(!halves.contains_key(&20));
    }

    #[test]
    fn swapping_sides_within_a_pair() {
        let mut s = two_screens();
        s.windows.push(win(11, 1, 2));
        let mut halves = Halves::from([(10, true), (11, true)]);
        assert_eq!(
            open_on_side(&s, 11, Side::Left, &mut halves),
            vec![Cmd::Focus(11), Cmd::MoveFocusedColumnTo(1)]
        );
        assert!(open_on_side(&s, 10, Side::Left, &mut halves).is_empty());
    }

    #[test]
    fn lonely_half_grows_back_and_dead_ids_are_forgotten() {
        let s = two_screens();
        let mut halves = Halves::from([(10, true), (99, true)]);
        assert_eq!(
            fill_lonely(&s, &mut halves),
            vec![Cmd::SetWidthPercent {
                window: 10,
                percent: 100.0
            }]
        );
        assert!(halves.is_empty());
    }

    #[test]
    fn unconfirmed_half_waits_for_the_pair() {
        // Right after "open on the left": events from before the move still show 10 alone.
        let s = two_screens();
        let mut halves = Halves::from([(10, false), (20, false)]);
        assert!(fill_lonely(&s, &mut halves).is_empty());

        // The pair shows up: both confirmed, nothing to do.
        let mut paired = two_screens();
        paired.windows[1].workspace = Some(1);
        paired.windows[1].column = Some(2);
        assert!(fill_lonely(&paired, &mut halves).is_empty());
        assert_eq!(halves, Halves::from([(10, true), (20, true)]));

        // 20 closes: 10 is alone and confirmed, so it grows back.
        paired.windows.pop();
        assert_eq!(
            fill_lonely(&paired, &mut halves),
            vec![Cmd::SetWidthPercent {
                window: 10,
                percent: 100.0
            }]
        );
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
