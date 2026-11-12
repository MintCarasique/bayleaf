//! ADR-0002 prototype: both Bayleaf rails as layer-shell surfaces in GTK4.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

const RAIL_WIDTH: i32 = 56;

const CSS: &str = "
window.rail { background: #1e211e; }
window.rail.left  { border-right: 1px solid #2f3a2f; }
window.rail.right { border-left:  1px solid #2f3a2f; }
.rail button { min-width: 48px; min-height: 48px; padding: 0; border-radius: 8px;
               background: transparent; border: none; color: #e6e6e6; }
.rail button:hover, .rail button:active { background: #2f3a2f; }
.rail button.start { color: #8fc48f; }
.rail button image { -gtk-icon-size: 24px; }
.rail button.active { background: #2f3a2f; }
.rail label { color: #e6e6e6; font-size: 12px; }
.rail scale trough { min-width: 8px; border-radius: 4px; background: #3a3f3a; }
.rail scale highlight { background: #4b7f52; border-radius: 4px; }
.rail scale slider { min-width: 22px; min-height: 22px; border-radius: 11px; background: #e6e6e6; }
";

fn main() -> glib::ExitCode {
    let started = Instant::now();
    let app = gtk4::Application::builder()
        .application_id("org.bayleaf.prototype.RailGtk")
        .build();
    app.connect_activate(move |app| activate(app, started));
    app.run()
}

fn activate(app: &gtk4::Application, started: Instant) {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let reported = Rc::new(Cell::new(false));
    let left = rail_window(app, Edge::Left, "left");
    left.set_child(Some(&left_rail()));
    let right = rail_window(app, Edge::Right, "right");
    right.set_child(Some(&right_rail()));

    for w in [&left, &right] {
        let reported = reported.clone();
        w.connect_map(move |_| {
            if !reported.replace(true) {
                println!("startup_ms={}", started.elapsed().as_millis());
            }
        });
        w.present();
    }

    glib::timeout_add_local_once(Duration::from_secs(3), || {
        println!("rss_kb={}", rss_kb().unwrap_or(0));
    });
}

fn rail_window(app: &gtk4::Application, edge: Edge, side: &str) -> gtk4::ApplicationWindow {
    let w = gtk4::ApplicationWindow::new(app);
    w.add_css_class("rail");
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

fn icon_button(icon: &str, class: Option<&str>) -> gtk4::Button {
    let b = gtk4::Button::from_icon_name(icon);
    if let Some(c) = class {
        b.add_css_class(c);
    }
    b.set_halign(gtk4::Align::Center);
    b
}

fn left_rail() -> gtk4::Box {
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    col.set_margin_top(12);
    col.set_margin_bottom(12);
    col.append(&icon_button("internet-web-browser-symbolic", Some("active")));
    col.append(&icon_button("system-file-manager-symbolic", None));
    col.append(&icon_button("utilities-terminal-symbolic", None));
    col.append(&icon_button("media-playback-start-symbolic", None));
    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    col.append(&spacer);
    col.append(&icon_button("view-app-grid-symbolic", Some("start")));
    col
}

fn right_rail() -> gtk4::Box {
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    col.set_margin_top(12);
    col.set_margin_bottom(12);

    let clock = gtk4::Label::new(Some(&now_hhmm()));
    let battery = gtk4::Label::new(Some(&battery_text()));
    col.append(&clock);
    col.append(&battery);
    glib::timeout_add_seconds_local(30, move || {
        clock.set_text(&now_hhmm());
        battery.set_text(&battery_text());
        glib::ControlFlow::Continue
    });

    col.append(&gtk4::Image::from_icon_name("audio-volume-medium-symbolic"));
    let volume = gtk4::Scale::with_range(gtk4::Orientation::Vertical, 0.0, 100.0, 1.0);
    volume.set_inverted(true);
    volume.set_value(50.0);
    volume.set_draw_value(false);
    volume.set_size_request(-1, 160);
    volume.set_halign(gtk4::Align::Center);
    col.append(&volume);

    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    col.append(&spacer);
    col.append(&icon_button("input-keyboard-symbolic", None));
    col.append(&icon_button("preferences-system-notifications-symbolic", None));
    col
}

fn now_hhmm() -> String {
    glib::DateTime::now_local()
        .and_then(|t| t.format("%H:%M"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn battery_text() -> String {
    std::fs::read_dir("/sys/class/power_supply")
        .ok()
        .and_then(|dir| {
            dir.flatten()
                .find(|e| e.file_name().to_string_lossy().starts_with("BAT"))
        })
        .and_then(|e| std::fs::read_to_string(e.path().join("capacity")).ok())
        .map(|c| format!("{}%", c.trim()))
        .unwrap_or_else(|| "—".into())
}

fn rss_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|l| l.starts_with("VmRSS:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}
