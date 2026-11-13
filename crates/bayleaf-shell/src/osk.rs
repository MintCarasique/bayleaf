//! On-screen keyboard (squeekboard, ADR-0005): the keyboard button for the right rail.
//!
//! squeekboard shows itself whenever the focused app enables text input, and many apps do that
//! when they open or get focus (Firefox's address bar, a terminal), so it kept popping up on window
//! switches. The fix belongs in the compositor: enable the input method only after a touch on the
//! field (our niri fork). Until then the keyboard runs in *keyboard mode*: off by default, nothing
//! shows by itself; the rail button turns the mode on (keyboard shown, and it follows text fields
//! as usual) and off. Off means the GNOME a11y switch is off, which is the only way to stop
//! squeekboard from showing itself.
//!
//! The launcher search turns the mode on temporarily while the launcher is open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::gio;
use gtk4::prelude::*;

const BUS_NAME: &str = "sm.puri.OSK0";
const OBJECT_PATH: &str = "/sm/puri/OSK0";
const INTERFACE: &str = "sm.puri.OSK0";
const A11Y_SCHEMA: &str = "org.gnome.desktop.a11y.applications";
const A11Y_KEY: &str = "screen-keyboard-enabled";
/// squeekboard sees the a11y switch through dconf, which can take a second on this hardware, and
/// ignores SetVisible(true) until then: ask again a few times while the mode stays on.
const SHOW_RETRIES: [Duration; 3] = [
    Duration::from_millis(300),
    Duration::from_millis(1000),
    Duration::from_millis(2000),
];

/// Keyboard mode: whether it is on, and whether only for a moment (the launcher search).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    pub on: bool,
    pub temporary: bool,
}

impl Mode {
    /// The rail button: always ends a temporary mode, otherwise flips.
    pub fn toggled(self) -> Self {
        if self.on {
            Self::default()
        } else {
            Self {
                on: true,
                temporary: false,
            }
        }
    }

    /// A text field of the shell wants the keyboard; it does not override a mode set by hand.
    pub fn with_temporary(self) -> Self {
        if self.on {
            self
        } else {
            Self {
                on: true,
                temporary: true,
            }
        }
    }

    /// The shell's text field is gone; only a temporary mode ends.
    pub fn without_temporary(self) -> Self {
        if self.temporary {
            Self::default()
        } else {
            self
        }
    }
}

pub struct Keyboard {
    button: gtk4::Button,
    mode: Rc<Cell<Mode>>,
    proxy: RefCell<Option<gio::DBusProxy>>,
    a11y: Option<gio::Settings>,
}

impl Keyboard {
    /// Connects to the keyboard in the background; the button stays hidden until it is there.
    pub fn new() -> Rc<Self> {
        let button = gtk4::Button::from_icon_name("input-keyboard-symbolic");
        crate::style::name(&button, "On-screen keyboard");
        button.set_visible(false);

        let a11y = gio::SettingsSchemaSource::default()
            .and_then(|src| src.lookup(A11Y_SCHEMA, true))
            .map(|_| gio::Settings::new(A11Y_SCHEMA));
        if a11y.is_none() {
            tracing::warn!("no {A11Y_SCHEMA} schema; keyboard mode cannot stop auto-show");
        }

        let kb = Rc::new(Self {
            button,
            mode: Rc::default(),
            proxy: RefCell::default(),
            a11y,
        });
        // Start in a known state: mode off.
        kb.apply(Mode::default());

        let weak = Rc::downgrade(&kb);
        kb.button.connect_clicked(move |_| {
            if let Some(kb) = weak.upgrade() {
                kb.apply(kb.mode.get().toggled());
            }
        });

        let weak = Rc::downgrade(&kb);
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
            let Some(kb) = weak.upgrade() else { return };
            kb.button.set_visible(proxy.name_owner().is_some());
            let w = weak.clone();
            proxy.connect_notify_local(Some("g-name-owner"), move |proxy, _| {
                if let Some(kb) = w.upgrade() {
                    kb.button.set_visible(proxy.name_owner().is_some());
                }
            });
            *kb.proxy.borrow_mut() = Some(proxy);
        });
        kb
    }

    /// The rail button.
    pub fn button(&self) -> &gtk4::Button {
        &self.button
    }

    /// A shell text field was tapped (the launcher search).
    pub fn show_for_shell_field(&self) {
        self.apply(self.mode.get().with_temporary());
    }

    /// The shell text field went away (the launcher closed).
    pub fn shell_field_gone(&self) {
        self.apply(self.mode.get().without_temporary());
    }

    fn apply(&self, mode: Mode) {
        let changed = mode.on != self.mode.get().on;
        self.mode.set(mode);
        if mode.on {
            self.button.add_css_class("active");
        } else {
            self.button.remove_css_class("active");
        }
        if !changed && mode.on {
            return;
        }
        // Order matters: the switch first so squeekboard accepts SetVisible(true); when turning
        // off, hide first, then stop auto-show.
        if mode.on {
            self.set_a11y(true);
            self.set_visible(true);
            for delay in SHOW_RETRIES {
                let proxy = self.proxy.borrow().clone();
                let mode = self.mode.clone();
                gtk4::glib::timeout_add_local_once(delay, move || {
                    if let (Some(p), true) = (proxy, mode.get().on) {
                        call_set_visible(&p, true);
                    }
                });
            }
        } else {
            self.set_visible(false);
            self.set_a11y(false);
        }
    }

    fn set_a11y(&self, on: bool) {
        if let Some(s) = &self.a11y {
            if let Err(e) = s.set_boolean(A11Y_KEY, on) {
                tracing::warn!(error = %e, on, "cannot set {A11Y_KEY}");
            }
        }
    }

    fn set_visible(&self, show: bool) {
        if let Some(proxy) = self.proxy.borrow().as_ref() {
            call_set_visible(proxy, show);
        }
    }
}

fn call_set_visible(proxy: &gio::DBusProxy, show: bool) {
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

#[cfg(test)]
mod tests {
    use super::*;

    const OFF: Mode = Mode {
        on: false,
        temporary: false,
    };
    const ON: Mode = Mode {
        on: true,
        temporary: false,
    };
    const TEMP: Mode = Mode {
        on: true,
        temporary: true,
    };

    #[test]
    fn button_flips_and_ends_temporary() {
        assert_eq!(OFF.toggled(), ON);
        assert_eq!(ON.toggled(), OFF);
        assert_eq!(TEMP.toggled(), OFF);
    }

    #[test]
    fn shell_field_turns_on_only_temporarily() {
        assert_eq!(OFF.with_temporary(), TEMP);
        assert_eq!(TEMP.without_temporary(), OFF);
    }

    #[test]
    fn shell_field_keeps_a_mode_set_by_hand() {
        assert_eq!(ON.with_temporary(), ON);
        assert_eq!(ON.without_temporary(), ON);
    }
}
