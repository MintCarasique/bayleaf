//! wlr-foreign-toplevel-management client.
//!
//! Runs its own Wayland connection on a background thread and reports changes through a
//! channel; GTK keeps its own connection. Requests (activate) are sent through [`Controller`],
//! which is safe to use from the GTK main thread.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use async_channel::Sender;
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, event_created_child};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::{
    self as handle, ZwlrForeignToplevelHandleV1,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::{
    self as manager, ZwlrForeignToplevelManagerV1,
};

use crate::toplevel_model::{Toplevel, ToplevelEvent, ToplevelId, parse_states};

type Handles = Arc<Mutex<HashMap<ToplevelId, ZwlrForeignToplevelHandleV1>>>;

/// Sends requests about toplevels to the compositor.
#[derive(Clone)]
pub struct Controller {
    conn: Connection,
    seat: Option<wl_seat::WlSeat>,
    handles: Handles,
}

impl Controller {
    fn handle(&self, id: ToplevelId) -> Option<ZwlrForeignToplevelHandleV1> {
        match self.handles.lock() {
            Ok(map) => map.get(&id).cloned(),
            Err(_) => None,
        }
    }

    fn flush(&self, what: &str) {
        if let Err(e) = self.conn.flush() {
            tracing::warn!(error = %e, what, "Wayland flush failed");
        }
    }

    /// Asks the app to close the toplevel (it may ask to save first).
    pub fn close(&self, id: ToplevelId) {
        if let Some(h) = self.handle(id) {
            h.close();
            self.flush("close");
        }
    }

    /// Asks the compositor to focus and raise the toplevel.
    pub fn activate(&self, id: ToplevelId) {
        let Some(seat) = &self.seat else {
            tracing::warn!("no wl_seat, cannot activate toplevel");
            return;
        };
        if let Some(h) = self.handle(id) {
            h.activate(seat);
            self.flush("activate");
        }
    }
}

/// Connects to the compositor and starts the event thread.
pub fn spawn(events: Sender<ToplevelEvent>) -> anyhow::Result<Controller> {
    let conn = Connection::connect_to_env().context("connect to Wayland display")?;
    let (globals, mut queue) =
        registry_queue_init::<State>(&conn).context("read Wayland globals")?;
    let qh = queue.handle();

    let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=7, ()).ok();
    let manager_bound = globals
        .bind::<ZwlrForeignToplevelManagerV1, _, _>(&qh, 1..=3, ())
        .is_ok();
    if !manager_bound {
        tracing::warn!("compositor lacks zwlr_foreign_toplevel_manager_v1; rail shows no windows");
        if events.send_blocking(ToplevelEvent::Unsupported).is_err() {
            tracing::debug!("toplevel event receiver gone");
        }
    }

    let handles: Handles = Arc::default();
    let mut state = State {
        events,
        handles: handles.clone(),
        pending: HashMap::new(),
    };

    std::thread::Builder::new()
        .name("bayleaf-toplevels".into())
        .spawn(move || {
            loop {
                if let Err(e) = queue.blocking_dispatch(&mut state) {
                    tracing::error!(error = %e, "toplevel event loop stopped");
                    break;
                }
            }
        })
        .context("spawn toplevel thread")?;

    Ok(Controller {
        conn,
        seat,
        handles,
    })
}

struct State {
    events: Sender<ToplevelEvent>,
    handles: Handles,
    /// Accumulates property events until `done`.
    pending: HashMap<ToplevelId, Toplevel>,
}

impl State {
    fn send(&self, event: ToplevelEvent) {
        if self.events.send_blocking(event).is_err() {
            tracing::debug!("toplevel event receiver gone");
        }
    }
}

fn id_of(h: &ZwlrForeignToplevelHandleV1) -> ToplevelId {
    h.id().protocol_id()
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            manager::Event::Toplevel { toplevel } => {
                let id = id_of(&toplevel);
                state.pending.insert(
                    id,
                    Toplevel {
                        id,
                        ..Default::default()
                    },
                );
                if let Ok(mut map) = state.handles.lock() {
                    map.insert(id, toplevel);
                }
            }
            manager::Event::Finished => state.send(ToplevelEvent::Unsupported),
            _ => {}
        }
    }

    event_created_child!(State, ZwlrForeignToplevelManagerV1, [
        manager::EVT_TOPLEVEL_OPCODE => (ZwlrForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZwlrForeignToplevelHandleV1,
        event: handle::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let id = id_of(proxy);
        match event {
            handle::Event::Title { title } => {
                if let Some(t) = state.pending.get_mut(&id) {
                    t.title = title;
                }
            }
            handle::Event::AppId { app_id } => {
                if let Some(t) = state.pending.get_mut(&id) {
                    t.app_id = app_id;
                }
            }
            handle::Event::State { state: raw } => {
                if let Some(t) = state.pending.get_mut(&id) {
                    let s = parse_states(&raw);
                    t.activated = s.activated;
                    t.minimized = s.minimized;
                    t.maximized = s.maximized;
                    t.fullscreen = s.fullscreen;
                }
            }
            handle::Event::Done => {
                if let Some(t) = state.pending.get(&id) {
                    state.send(ToplevelEvent::Updated(t.clone()));
                }
            }
            handle::Event::Closed => {
                state.pending.remove(&id);
                if let Ok(mut map) = state.handles.lock() {
                    map.remove(&id);
                }
                proxy.destroy();
                state.send(ToplevelEvent::Closed(id));
            }
            _ => {}
        }
    }
}
