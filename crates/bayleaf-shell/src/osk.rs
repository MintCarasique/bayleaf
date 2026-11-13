//! On-screen keyboard (squeekboard, ADR-0005) over its D-Bus interface `sm.puri.OSK0`: the
//! keyboard button for the right rail, and a stopgap against the keyboard popping up on its own.
//!
//! The keyboard shows itself whenever a text field gets focus, and many apps focus one when they
//! open or get switched to (Firefox's address bar, a terminal). Until the compositor only enables
//! the input method after a touch on the field (our niri fork), the shell hides the keyboard if it
//! appears within [`SUPPRESS_AFTER_FOCUS`] of a window change. The button always works.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::gio;
use gtk4::prelude::*;

const BUS_NAME: &str = "sm.puri.OSK0";
const OBJECT_PATH: &str = "/sm/puri/OSK0";
const INTERFACE: &str = "sm.puri.OSK0";

/// How long after a window opens or gets switched to the keyboard is kept hidden.
const SUPPRESS_AFTER_FOCUS: Duration = Duration::from_secs(3);

/// Visibility to request when the button is tapped, given what the keyboard reports.
pub fn next_visible(current: Option<bool>) -> bool {
    !current.unwrap_or(false)
}

/// Whether a keyboard that just became visible should be hidden again.
pub fn should_suppress(visible: bool, until: Option<Instant>, now: Instant) -> bool {
    visible && until.is_some_and(|u| now < u)
}

fn visible(proxy: &gio::DBusProxy) -> Option<bool> {
    proxy.cached_property("Visible")?.get::<bool>()
}

fn set_visible(proxy: &gio::DBusProxy, show: bool) {
    proxy.call(
        "SetVisible",
        Some(&(show,).to_variant()),
        gio::DBusCallFlags::NONE,
        -1,
        None::<&gio::Cancellable>,
        move |res| {
            if let Err(e) = res {
                tracing::warn!(error = %e, show, "SetVisible failed");
            }
        },
    );
}

pub struct Keyboard {
    button: gtk4::Button,
    suppress_until: Rc<Cell<Option<Instant>>>,
}

impl Keyboard {
    /// Connects to the keyboard in the background; the button stays hidden until it is there.
    pub fn new() -> Self {
        let button = gtk4::Button::from_icon_name("input-keyboard-symbolic");
        crate::style::name(&button, "On-screen keyboard");
        button.set_visible(false);
        let suppress_until: Rc<Cell<Option<Instant>>> = Rc::default();

        let b = button.clone();
        let until = suppress_until.clone();
        gtk4::glib::spawn_future_local(async move {
            let proxy = match gio::DBusProxy::for_bus_future(
                gio::BusType::Session,
                gio::DBusProxyFlags::DO_NOT_AUTO_START,
                None,
                BUS_NAME,
                OBJECT_PATH,
                INTERFACE,
            )
            .await
            {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = %e, "no session bus; keyboard button disabled");
                    return;
                }
            };
            sync(&b, &proxy);

            let weak = b.downgrade();
            let u = until.clone();
            proxy.connect_local("g-properties-changed", false, move |args| {
                let proxy = args.first()?.get::<gio::DBusProxy>().ok()?;
                let shown = visible(&proxy) == Some(true);
                if should_suppress(shown, u.get(), Instant::now()) {
                    tracing::debug!("keyboard popped up right after a window change; hiding");
                    set_visible(&proxy, false);
                }
                sync(&weak.upgrade()?, &proxy);
                None
            });
            let weak = b.downgrade();
            proxy.connect_notify_local(Some("g-name-owner"), move |proxy, _| {
                if let Some(b) = weak.upgrade() {
                    sync(&b, proxy);
                }
            });

            b.connect_clicked(move |_| {
                // An explicit tap always wins over the suppression window.
                until.set(None);
                set_visible(&proxy, next_visible(visible(&proxy)));
            });
        });
        Self {
            button,
            suppress_until,
        }
    }

    /// The rail button.
    pub fn button(&self) -> &gtk4::Button {
        &self.button
    }

    /// Call when a window opened or got focus: keeps the keyboard hidden for a moment.
    pub fn window_changed(&self) {
        self.suppress_until
            .set(Some(Instant::now() + SUPPRESS_AFTER_FOCUS));
    }
}

fn sync(button: &gtk4::Button, proxy: &gio::DBusProxy) {
    button.set_visible(proxy.name_owner().is_some());
    if visible(proxy) == Some(true) {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_reported_state() {
        assert!(!next_visible(Some(true)));
        assert!(next_visible(Some(false)));
    }

    #[test]
    fn unknown_state_shows() {
        assert!(next_visible(None));
    }

    #[test]
    fn suppresses_only_inside_window_and_when_visible() {
        let now = Instant::now();
        let later = now + Duration::from_secs(1);
        assert!(should_suppress(true, Some(later), now));
        assert!(!should_suppress(false, Some(later), now));
        assert!(!should_suppress(true, Some(now), later));
        assert!(!should_suppress(true, None, now));
    }
}
