//! niri IPC client (ADR-0001, docs/design/niri-integration.md).
//!
//! One background thread follows niri's event stream and turns it into [`Snapshot`]s for the GTK
//! thread; another sends actions, one connection per action as niri expects. The window model
//! itself (one screen per workspace, split view) lives in [`crate::layout`].

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::mpsc;

use anyhow::Context;
use async_channel::Sender;
use gtk4::glib;
use niri_ipc::socket::{SOCKET_PATH_ENV, Socket};
use niri_ipc::state::{EventStreamState, EventStreamStatePart};
use niri_ipc::{Action, Event, Reply, Request, Response, SizeChange, WorkspaceReferenceArg};

use crate::layout::{self, Cmd, Snapshot, Win, Ws};
use crate::toplevel_model::{ToplevelEvent, ToplevelId};
use crate::windows::{Side, WindowControl};

/// Whether the shell runs inside niri.
pub fn available() -> bool {
    std::env::var_os(SOCKET_PATH_ENV).is_some()
}

/// State on the GTK thread.
pub struct Niri {
    snapshot: RefCell<Snapshot>,
    /// Windows we made half-width; they grow back when left alone.
    halves: RefCell<layout::Halves>,
    /// Window ids already seen; `None` until niri sent the initial window list.
    known: RefCell<Option<HashSet<u64>>>,
    actions: mpsc::Sender<Vec<Cmd>>,
}

/// Connects to niri and starts both threads. Window lists go to `events` as snapshots.
pub fn start(events: Sender<ToplevelEvent>) -> anyhow::Result<Rc<Niri>> {
    let mut socket = Socket::connect().context("connect to the niri socket")?;
    match socket
        .send(Request::EventStream)
        .context("request the event stream")?
    {
        Ok(Response::Handled) => {}
        Ok(other) => anyhow::bail!("unexpected reply to EventStream: {other:?}"),
        Err(e) => anyhow::bail!("niri refused the event stream: {e}"),
    }

    let (snap_tx, snap_rx) = async_channel::unbounded::<(Snapshot, bool)>();
    std::thread::Builder::new()
        .name("niri-events".into())
        .spawn(move || follow_events(socket, &snap_tx))
        .context("spawn niri event thread")?;

    let (act_tx, act_rx) = mpsc::channel::<Vec<Cmd>>();
    std::thread::Builder::new()
        .name("niri-actions".into())
        .spawn(move || {
            for batch in act_rx {
                for cmd in batch {
                    send_action(to_action(&cmd));
                }
            }
        })
        .context("spawn niri action thread")?;

    let niri = Rc::new(Niri {
        snapshot: RefCell::default(),
        halves: RefCell::default(),
        known: RefCell::default(),
        actions: act_tx,
    });
    let n = niri.clone();
    glib::spawn_future_local(async move {
        while let Ok((snapshot, windows_loaded)) = snap_rx.recv().await {
            n.on_snapshot(snapshot, windows_loaded);
            let list = n.snapshot.borrow().toplevels();
            if events.send(ToplevelEvent::Snapshot(list)).await.is_err() {
                break;
            }
        }
        tracing::warn!("niri event stream ended");
    });
    Ok(niri)
}

impl Niri {
    fn on_snapshot(&self, snapshot: Snapshot, windows_loaded: bool) {
        let ids: HashSet<u64> = snapshot.windows.iter().map(|w| w.id).collect();
        let mut cmds = Vec::new();
        if windows_loaded {
            let mut known = self.known.borrow_mut();
            if let Some(seen) = known.as_ref() {
                for id in ids.difference(seen) {
                    cmds.extend(layout::place_new(&snapshot, *id));
                }
            }
            *known = Some(ids);
        }
        cmds.extend(layout::fill_lonely(
            &snapshot,
            &mut self.halves.borrow_mut(),
        ));
        *self.snapshot.borrow_mut() = snapshot;
        self.send(cmds);
    }

    fn send(&self, cmds: Vec<Cmd>) {
        if cmds.is_empty() {
            return;
        }
        tracing::debug!(?cmds, "niri actions");
        if self.actions.send(cmds).is_err() {
            tracing::warn!("niri action thread gone");
        }
    }
}

impl WindowControl for Niri {
    fn activate(&self, id: ToplevelId) {
        self.send(vec![Cmd::Focus(id)]);
    }

    fn close(&self, id: ToplevelId) {
        self.send(vec![Cmd::Close(id)]);
    }

    fn supports_split(&self) -> bool {
        true
    }

    fn open_on_side(&self, id: ToplevelId, side: Side) {
        let cmds = layout::open_on_side(
            &self.snapshot.borrow(),
            id,
            side,
            &mut self.halves.borrow_mut(),
        );
        self.send(cmds);
    }
}

fn follow_events(socket: Socket, out: &async_channel::Sender<(Snapshot, bool)>) {
    let mut read = socket.read_events();
    let mut state = EventStreamState::default();
    let mut windows_loaded = false;
    loop {
        let event = match read() {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "niri event stream closed");
                return;
            }
        };
        if matches!(event, Event::WindowsChanged { .. }) {
            windows_loaded = true;
        }
        // Parts ignore events that are not theirs; only window and workspace changes matter here.
        let relevant = matches!(
            event,
            Event::WindowsChanged { .. }
                | Event::WindowOpenedOrChanged { .. }
                | Event::WindowClosed { .. }
                | Event::WindowFocusChanged { .. }
                | Event::WindowLayoutsChanged { .. }
                | Event::WorkspacesChanged { .. }
                | Event::WorkspaceActivated { .. }
                | Event::WorkspaceActiveWindowChanged { .. }
        );
        if state.apply(event).is_some() || !relevant {
            continue;
        }
        if out
            .send_blocking((snapshot_of(&state), windows_loaded))
            .is_err()
        {
            return;
        }
    }
}

fn snapshot_of(state: &EventStreamState) -> Snapshot {
    Snapshot {
        windows: state
            .windows
            .windows
            .values()
            .map(|w| Win {
                id: w.id,
                workspace: w.workspace_id,
                floating: w.is_floating,
                focused: w.is_focused,
                column: w.layout.pos_in_scrolling_layout.map(|(col, _)| col),
                app_id: w.app_id.clone().unwrap_or_default(),
                title: w.title.clone().unwrap_or_default(),
            })
            .collect(),
        workspaces: state
            .workspaces
            .workspaces
            .values()
            .map(|ws| Ws {
                id: ws.id,
                idx: ws.idx,
                output: ws.output.clone(),
                focused: ws.is_focused,
            })
            .collect(),
    }
}

fn to_action(cmd: &Cmd) -> Action {
    match *cmd {
        Cmd::Focus(id) => Action::FocusWindow { id },
        Cmd::Close(id) => Action::CloseWindow { id: Some(id) },
        Cmd::MoveToWorkspace { window, workspace } => Action::MoveWindowToWorkspace {
            window_id: Some(window),
            reference: WorkspaceReferenceArg::Id(workspace),
            focus: false,
        },
        Cmd::SetWidthPercent { window, percent } => Action::SetWindowWidth {
            id: Some(window),
            change: SizeChange::SetProportion(percent),
        },
        Cmd::MoveFocusedColumnTo(index) => Action::MoveColumnToIndex { index },
    }
}

fn send_action(action: Action) {
    let reply: std::io::Result<Reply> =
        Socket::connect().and_then(|mut s| s.send(Request::Action(action.clone())));
    match reply {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => tracing::warn!(error = %e, ?action, "niri rejected action"),
        Err(e) => tracing::warn!(error = %e, ?action, "niri action failed"),
    }
}
