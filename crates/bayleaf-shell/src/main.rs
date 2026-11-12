//! Bayleaf shell: side rails, and later the launcher, quick settings and notifications.
//!
//! One process hosts every shell surface (ADR-0002).

mod accordion;
mod app_search;
mod controls;
mod device;
mod launcher;
mod rails;
mod status;
mod style;
mod toplevel;
mod toplevel_model;

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

fn activate(app: &gtk4::Application) {
    style::install();

    let (tx, rx) = async_channel::unbounded();
    let controller = match toplevel::spawn(tx) {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::error!(error = %e, "window list unavailable");
            None
        }
    };

    let launcher = launcher::Launcher::new(app);
    let left = Rc::new(rails::LeftRail::new(app, controller, move || {
        launcher.toggle()
    }));
    let right = rails::right_rail(app);
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
