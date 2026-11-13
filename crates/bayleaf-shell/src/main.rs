//! Bayleaf shell: side rails, and later the launcher, quick settings and notifications.
//!
//! One process hosts every shell surface (ADR-0002).

mod accordion;
mod app_search;
mod controls;
mod device;
mod dropzones;
mod launcher;
mod layout;
mod niri;
mod osk;
mod rails;
mod status;
mod style;
mod toplevel;
mod toplevel_model;
mod windows;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use crate::toplevel_model::Toplevels;

const APP_ID: &str = "org.bayleaf.Shell";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let app = gtk4::Application::builder().application_id(APP_ID).build();
    app.connect_activate(activate);
    app.run()
}

/// niri IPC when running inside niri, wlr-foreign-toplevel otherwise.
fn window_backend(
    tx: async_channel::Sender<toplevel_model::ToplevelEvent>,
) -> Option<Rc<dyn windows::WindowControl>> {
    if niri::available() {
        match niri::start(tx.clone()) {
            Ok(n) => {
                tracing::info!("window list from niri IPC");
                return Some(n);
            }
            Err(e) => tracing::warn!(error = %e, "niri IPC unavailable; trying foreign-toplevel"),
        }
    }
    match toplevel::spawn(tx) {
        Ok(c) => Some(Rc::new(c)),
        Err(e) => {
            tracing::error!(error = %e, "window list unavailable");
            None
        }
    }
}

fn activate(app: &gtk4::Application) {
    style::install();

    let (tx, rx) = async_channel::unbounded();
    let controller = window_backend(tx);

    let keyboard = osk::Keyboard::new();
    let launcher = launcher::Launcher::new(app, keyboard.clone());
    let left = Rc::new(rails::LeftRail::new(app, controller, move || {
        launcher.toggle()
    }));
    let right = rails::right_rail(app, &keyboard);
    left.present();
    right.present();

    let model = Rc::new(RefCell::new(Toplevels::new()));
    glib::spawn_future_local(async move {
        while let Ok(event) = rx.recv().await {
            let changed = model.borrow_mut().apply(event);
            if changed {
                left.show(&model.borrow());
            }
        }
    });
}
