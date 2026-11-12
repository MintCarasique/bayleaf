//! The two side rails as layer-shell surfaces (docs/design/rails.md).

use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::status;
use crate::toplevel::Controller;
use crate::toplevel_model::{Toplevel, Toplevels};

pub const RAIL_WIDTH: i32 = 56;
const FALLBACK_ICON: &str = "application-x-executable-symbolic";

/// Creates a rail window anchored to `edge` that reserves its width (exclusive zone).
fn rail_window(app: &gtk4::Application, edge: Edge, side: &str) -> gtk4::ApplicationWindow {
    let w = gtk4::ApplicationWindow::new(app);
    w.add_css_class("bayleaf-rail");
    w.add_css_class(side);
    w.init_layer_shell();
    w.set_namespace(Some("bayleaf-rail"));
    w.set_layer(Layer::Top);
    w.set_anchor(edge, true);
    w.set_anchor(Edge::Top, true);
    w.set_anchor(Edge::Bottom, true);
    w.set_width_request(RAIL_WIDTH);
    w.auto_exclusive_zone_enable();
    w
}

fn rail_column() -> gtk4::Box {
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    col.set_margin_top(12);
    col.set_margin_bottom(12);
    col
}

fn spacer() -> gtk4::Box {
    let s = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    s.set_vexpand(true);
    s
}

/// Left rail: running apps on top, Start at the bottom.
pub struct LeftRail {
    window: gtk4::ApplicationWindow,
    apps: gtk4::Box,
    controller: Option<Controller>,
}

impl LeftRail {
    pub fn new(app: &gtk4::Application, controller: Option<Controller>) -> Self {
        let window = rail_window(app, Edge::Left, "left");
        let col = rail_column();
        let apps = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        col.append(&apps);
        col.append(&spacer());

        let start = gtk4::Button::from_icon_name("view-app-grid-symbolic");
        start.add_css_class("start");
        start.set_halign(gtk4::Align::Center);
        start.set_tooltip_text(Some("Start"));
        start.connect_clicked(|_| tracing::info!("start pressed (launcher comes in a later step)"));
        col.append(&start);

        window.set_child(Some(&col));
        Self {
            window,
            apps,
            controller,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Rebuilds the app buttons from the model.
    pub fn show(&self, toplevels: &Toplevels) {
        while let Some(child) = self.apps.first_child() {
            self.apps.remove(&child);
        }
        if !toplevels.is_supported() {
            let hint = gtk4::Image::from_icon_name("dialog-warning-symbolic");
            hint.set_tooltip_text(Some(
                "This compositor does not share its window list \
                 (wlr-foreign-toplevel-management)",
            ));
            self.apps.append(&hint);
            return;
        }
        let theme = gtk4::IconTheme::for_display(&WidgetExt::display(&self.window));
        for t in toplevels.iter() {
            self.apps.append(&self.app_button(t, &theme));
        }
    }

    fn app_button(&self, t: &Toplevel, theme: &gtk4::IconTheme) -> gtk4::Button {
        let b = gtk4::Button::from_icon_name(&icon_for(&t.app_id, theme));
        b.set_halign(gtk4::Align::Center);
        b.set_tooltip_text(Some(if t.title.is_empty() {
            &t.app_id
        } else {
            &t.title
        }));
        if t.activated {
            b.add_css_class("active");
        }
        if t.minimized {
            b.add_css_class("minimized");
        }
        let id = t.id;
        let controller = self.controller.clone();
        b.connect_clicked(move |_| {
            if let Some(c) = &controller {
                c.activate(id);
            }
        });
        b
    }
}

/// Picks an icon name for an app id: exact, lower-case, or a generic fallback.
fn icon_for(app_id: &str, theme: &gtk4::IconTheme) -> String {
    let lower = app_id.to_lowercase();
    [app_id, lower.as_str()]
        .into_iter()
        .find(|name| !name.is_empty() && theme.has_icon(name))
        .unwrap_or(FALLBACK_ICON)
        .to_string()
}

/// Right rail: device status. Sliders, quick settings and notifications come in later steps.
pub fn right_rail(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = rail_window(app, Edge::Right, "right");
    let col = rail_column();

    let clock = gtk4::Label::new(Some(&status::clock_text()));
    clock.add_css_class("status");
    let battery = gtk4::Label::new(status::battery_text().as_deref());
    battery.add_css_class("status");
    col.append(&clock);
    col.append(&battery);
    col.append(&spacer());

    glib::timeout_add_seconds_local(15, move || {
        clock.set_text(&status::clock_text());
        battery.set_text(status::battery_text().as_deref().unwrap_or(""));
        glib::ControlFlow::Continue
    });

    window.set_child(Some(&col));
    window
}
