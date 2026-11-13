//! niri IPC client (ADR-0001, docs/design/niri-integration.md).
//!
//! One background thread follows niri's event stream and turns it into [`Snapshot`]s for the GTK
//! thread; another sends actions, one connection per action as niri expects. The window model
//! itself (one screen per workspace, split view) lives in [`crate::layout`].
//!
//! Actions that only exist in our niri fork are sent as raw JSON, so the shell can keep using the
//! published `niri-ipc` crate.

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::sync::mpsc;

use anyhow::Context;
use async_channel::Sender;
use gtk4::glib;
use niri_ipc::socket::{SOCKET_PATH_ENV, Socket};
use niri_ipc::state::{EventStreamState, EventStreamStatePart};
use niri_ipc::{Action, Event, Reply, Request, Response, WorkspaceReferenceArg};

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

    let (snap_tx, snap_rx) = async_channel::unbounded::<Snapshot>();
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
                    match to_action(&cmd) {
                        Outgoing::Typed(action) => send_action(action),
                        Outgoing::Raw(json) => send_raw(&json),
                    }
                }
            }
        })
        .context("spawn niri action thread")?;

    let niri = Rc::new(Niri {
        snapshot: RefCell::default(),
        actions: act_tx,
    });
    let n = niri.clone();
    glib::spawn_future_local(async move {
        while let Ok(snapshot) = snap_rx.recv().await {
            *n.snapshot.borrow_mut() = snapshot;
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
        let cmds = layout::open_on_side(&self.snapshot.borrow(), id, side);
        self.send(cmds);
    }

    fn open_alone(&self, id: ToplevelId) {
        let cmds = layout::open_alone(&self.snapshot.borrow(), id);
        self.send(cmds);
    }
}

fn follow_events(socket: Socket, out: &async_channel::Sender<Snapshot>) {
    let mut read = socket.read_events();
    let mut state = EventStreamState::default();
    loop {
        let event = match read() {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "niri event stream closed");
                return;
            }
        };
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
        if out.send_blocking(snapshot_of(&state)).is_err() {
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

enum Outgoing {
    Typed(Action),
    /// A request for an action of our fork, already serialized.
    Raw(String),
}

fn to_action(cmd: &Cmd) -> Outgoing {
    Outgoing::Typed(match *cmd {
        Cmd::Focus(id) => Action::FocusWindow { id },
        Cmd::Close(id) => Action::CloseWindow { id: Some(id) },
        Cmd::MoveToWorkspace { window, workspace } => Action::MoveWindowToWorkspace {
            window_id: Some(window),
            reference: WorkspaceReferenceArg::Id(workspace),
            focus: false,
        },
        Cmd::MoveFocusedColumnTo(index) => Action::MoveColumnToIndex { index },
        Cmd::PlaceInColumn {
            window,
            workspace,
            column,
        } => {
            return Outgoing::Raw(place_in_column_request(window, workspace, column));
        }
    })
}

/// `MoveWindowToWorkspaceColumn` from our niri fork, in niri-ipc's JSON shape.
fn place_in_column_request(window: u64, workspace: u64, column: usize) -> String {
    format!(
        r#"{{"Action":{{"MoveWindowToWorkspaceColumn":{{"window_id":{window},"reference":{{"Id":{workspace}}},"column":{column}}}}}}}"#
    )
}

fn send_raw(json: &str) {
    let result = (|| -> std::io::Result<String> {
        let path = std::env::var_os(SOCKET_PATH_ENV)
            .ok_or_else(|| std::io::Error::other(format!("{SOCKET_PATH_ENV} is not set")))?;
        let mut stream = UnixStream::connect(path)?;
        stream.write_all(json.as_bytes())?;
        stream.write_all(b"\n")?;
        let mut reply = String::new();
        BufReader::new(stream).read_line(&mut reply)?;
        Ok(reply)
    })();
    match result {
        Ok(reply) if reply.starts_with(r#"{"Ok""#) => {}
        Ok(reply) => tracing::warn!(reply = reply.trim(), json, "niri rejected action"),
        Err(e) => tracing::warn!(error = %e, json, "niri action failed"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fork_request_has_niri_ipc_shape() {
        let json = place_in_column_request(7, 3, 2);
        assert_eq!(
            json,
            r#"{"Action":{"MoveWindowToWorkspaceColumn":{"window_id":7,"reference":{"Id":3},"column":2}}}"#
        );
    }
}
