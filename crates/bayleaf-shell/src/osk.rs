//! Keyboard button for the right rail: shows and hides the on-screen keyboard (squeekboard,
//! ADR-0005) over its D-Bus interface `sm.puri.OSK0`.
//!
//! The keyboard also shows itself when a text field gets focus; the button is the manual way to
//! hide it (or to call it up for apps without text-input support). The button is hidden while no
//! keyboard is running.

use gtk4::gio;
use gtk4::prelude::*;

const BUS_NAME: &str = "sm.puri.OSK0";
const OBJECT_PATH: &str = "/sm/puri/OSK0";
const INTERFACE: &str = "sm.puri.OSK0";

/// Visibility to request when the button is tapped, given what the keyboard reports.
pub fn next_visible(current: Option<bool>) -> bool {
    !current.unwrap_or(false)
}

fn visible(proxy: &gio::DBusProxy) -> Option<bool> {
    proxy.cached_property("Visible")?.get::<bool>()
}

fn sync(button: &gtk4::Button, proxy: &gio::DBusProxy) {
    button.set_visible(proxy.name_owner().is_some());
    if visible(proxy) == Some(true) {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}

/// The rail button. Connects to the keyboard in the background; stays hidden until it is there.
pub fn button() -> gtk4::Button {
    let button = gtk4::Button::from_icon_name("input-keyboard-symbolic");
    crate::style::name(&button, "On-screen keyboard");
    button.set_visible(false);

    let b = button.clone();
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
        proxy.connect_local("g-properties-changed", false, move |args| {
            let proxy = args.first()?.get::<gio::DBusProxy>().ok()?;
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
            let show = next_visible(visible(&proxy));
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
        });
    });
    button
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
}
