//! Right-rail controls collapsed to icons; tapping one slides its vertical slider open below it
//! (docs/design/rails.md). Only one is open at a time; it closes again after a few idle seconds.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;

use crate::accordion::Accordion;

const AUTO_CLOSE: Duration = Duration::from_secs(5);
/// Minimum gap between device writes while dragging; each write spawns a process on a slow CPU.
const WRITE_INTERVAL: Duration = Duration::from_millis(100);
const SLIDER_HEIGHT: i32 = 140;

pub type ReadFn = Rc<dyn Fn() -> Pin<Box<dyn Future<Output = Option<f64>>>>>;
pub type WriteFn = Rc<dyn Fn(f64)>;

/// One collapsible control: an icon and the 0–100 value behind it.
pub struct Control {
    pub icon: &'static str,
    pub label: &'static str,
    pub read: ReadFn,
    pub write: WriteFn,
}

struct Shared {
    accordion: RefCell<Accordion>,
    revealers: RefCell<Vec<gtk4::Revealer>>,
    timer: RefCell<Option<glib::SourceId>>,
}

impl Shared {
    fn apply(&self) {
        let open = self.accordion.borrow().open();
        for (i, r) in self.revealers.borrow().iter().enumerate() {
            r.set_reveal_child(open == Some(i));
        }
    }

    /// (Re)starts the idle timer that closes the open control.
    fn touch(self: &Rc<Self>) {
        if let Some(id) = self.timer.borrow_mut().take() {
            id.remove();
        }
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(AUTO_CLOSE, move || {
            if let Some(s) = weak.upgrade() {
                s.timer.borrow_mut().take();
                s.accordion.borrow_mut().close();
                s.apply();
            }
        });
        *self.timer.borrow_mut() = Some(id);
    }
}

/// Sets the slider from the device value once `read` resolves, without writing it back.
fn sync_from_device(
    scale: &gtk4::Scale,
    syncing: &Rc<Cell<bool>>,
    read: Pin<Box<dyn Future<Output = Option<f64>>>>,
) {
    let scale = scale.clone();
    let syncing = syncing.clone();
    glib::spawn_future_local(async move {
        if let Some(v) = read.await {
            syncing.set(true);
            scale.set_value(v);
            syncing.set(false);
        }
    });
}

/// Builds the stack of collapsible controls.
pub fn stack(controls: Vec<Control>) -> gtk4::Box {
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    let shared = Rc::new(Shared {
        accordion: RefCell::default(),
        revealers: RefCell::default(),
        timer: RefCell::default(),
    });

    for (i, control) in controls.into_iter().enumerate() {
        let button = gtk4::Button::from_icon_name(control.icon);
        button.set_halign(gtk4::Align::Center);
        crate::style::name(&button, control.label);

        let scale = gtk4::Scale::with_range(gtk4::Orientation::Vertical, 0.0, 100.0, 1.0);
        scale.set_inverted(true);
        scale.set_draw_value(false);
        scale.set_size_request(-1, SLIDER_HEIGHT);
        scale.set_halign(gtk4::Align::Center);

        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(150);
        revealer.set_child(Some(&scale));

        col.append(&button);
        col.append(&revealer);
        shared.revealers.borrow_mut().push(revealer);

        // Programmatic set_value must not echo back to the device.
        let syncing = Rc::new(Cell::new(false));

        // Prefetch so the slider opens instantly with a real value; opening refreshes it again.
        sync_from_device(&scale, &syncing, (control.read)());

        {
            let shared = shared.clone();
            let scale = scale.clone();
            let syncing = syncing.clone();
            let read = control.read.clone();
            button.connect_clicked(move |_| {
                let opened = shared.accordion.borrow_mut().toggle(i) == Some(i);
                shared.apply();
                if !opened {
                    return;
                }
                shared.touch();
                sync_from_device(&scale, &syncing, read());
            });
        }

        // Throttle writes while dragging: at most one per WRITE_INTERVAL, last value wins.
        let pending = Rc::new(Cell::new(None::<f64>));
        let scheduled = Rc::new(Cell::new(false));
        {
            let shared = shared.clone();
            let write = control.write.clone();
            scale.connect_value_changed(move |s| {
                if syncing.get() {
                    return;
                }
                shared.touch();
                pending.set(Some(s.value()));
                if scheduled.replace(true) {
                    return;
                }
                let pending = pending.clone();
                let scheduled = scheduled.clone();
                let write = write.clone();
                glib::timeout_add_local_once(WRITE_INTERVAL, move || {
                    scheduled.set(false);
                    if let Some(v) = pending.take() {
                        write(v);
                    }
                });
            });
        }
    }
    col
}
