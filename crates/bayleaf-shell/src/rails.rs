//! The two side rails as layer-shell surfaces (docs/design/rails.md).

use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use std::rc::Rc;

use crate::dropzones::{DropZones, Target};
use crate::toplevel_model::{Toplevel, ToplevelId, Toplevels};
use crate::windows::{Side, WindowControl};
use crate::{controls, device, osk, status, style};

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
    controller: Option<Rc<dyn WindowControl>>,
    drop_zones: Rc<DropZones>,
}

impl LeftRail {
    pub fn new(
        app: &gtk4::Application,
        controller: Option<Rc<dyn WindowControl>>,
        on_start: impl Fn() + 'static,
    ) -> Self {
        let window = rail_window(app, Edge::Left, "left");
        let drop_zones = Rc::new(DropZones::new(app));
        let col = rail_column();
        let apps = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        col.append(&apps);
        col.append(&spacer());

        let start = gtk4::Button::from_icon_name("view-app-grid-symbolic");
        start.add_css_class("start");
        start.set_halign(gtk4::Align::Center);
        style::name(&start, "Start");
        start.connect_clicked(move |_| on_start());
        col.append(&start);

        window.set_child(Some(&col));
        Self {
            window,
            apps,
            controller,
            drop_zones,
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
        let shown = toplevels.iter().filter(|t| t.visible).count();
        for t in toplevels.iter() {
            self.apps.append(&self.app_button(t, &theme, shown));
        }
    }

    fn app_button(&self, t: &Toplevel, theme: &gtk4::IconTheme, shown: usize) -> gtk4::Button {
        let icon = icon_for(&t.app_id, theme);
        let b = gtk4::Button::from_icon_name(&icon);
        b.set_halign(gtk4::Align::Center);
        style::name(
            &b,
            if t.title.is_empty() {
                &t.app_id
            } else {
                &t.title
            },
        );
        if t.visible {
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
        if let Some(c) = &self.controller {
            // Split only makes sense for an app that is not on screen, next to one that is.
            // Split only makes sense for an app that is not on screen, next to one that is;
            // "Full screen" for one half of the pair on screen.
            let split = c.supports_split() && !t.visible && shown > 0;
            let alone = c.supports_split() && t.visible && shown > 1;
            attach_app_menu(&b, c.clone(), id, split, alone);
            attach_drag(
                &b,
                &self.window,
                DragTarget {
                    zones: self.drop_zones.clone(),
                    controller: c.clone(),
                    id,
                    icon,
                    split,
                },
            );
        }
        b
    }
}

/// Horizontal distance a finger has to move off the rail before a drag starts; less is a tap or
/// a long press.
const DRAG_START: f64 = 24.0;

/// What a drag from one rail button can do.
struct DragTarget {
    zones: Rc<DropZones>,
    controller: Rc<dyn WindowControl>,
    id: ToplevelId,
    icon: String,
    split: bool,
}

/// Dragging an app off the rail shows drop zones (docs/design/multitasking.md): the left or right
/// half when it can go next to the shown app, otherwise one full-screen zone.
fn attach_drag(button: &gtk4::Button, rail: &gtk4::ApplicationWindow, target: DragTarget) {
    let drag = gtk4::GestureDrag::new();
    drag.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let start = Rc::new(std::cell::Cell::new((0.0, 0.0)));
    let active = Rc::new(std::cell::Cell::new(false));
    let over = Rc::new(std::cell::Cell::new(None::<Target>));
    let target = Rc::new(target);

    {
        let (start, button, rail) = (start.clone(), button.downgrade(), rail.downgrade());
        drag.connect_drag_begin(move |_, x, y| {
            // Rail coordinates are screen coordinates: the left rail sits at the origin.
            let p = gtk4::graphene::Point::new(x as f32, y as f32);
            if let (Some(b), Some(r)) = (button.upgrade(), rail.upgrade()) {
                if let Some(p) = b.compute_point(&r, &p) {
                    start.set((f64::from(p.x()), f64::from(p.y())));
                }
            }
        });
    }
    {
        let (start, active, over, target) =
            (start.clone(), active.clone(), over.clone(), target.clone());
        drag.connect_drag_update(move |gesture, dx, dy| {
            if !active.get() {
                if dx < DRAG_START {
                    return;
                }
                // Ours now: no click, no long-press menu.
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                active.set(true);
                target.zones.begin(&target.icon, target.split);
            }
            let (sx, sy) = start.get();
            over.set(target.zones.update(sx + dx, sy + dy));
        });
    }
    drag.connect_drag_end(move |_, _, _| {
        if !active.replace(false) {
            return;
        }
        target.zones.end();
        let t = &target;
        match over.take() {
            Some(Target::Left) => t.controller.open_on_side(t.id, Side::Left),
            Some(Target::Right) => t.controller.open_on_side(t.id, Side::Right),
            Some(Target::Full) => t.controller.open_alone(t.id),
            None => {}
        }
    });
    button.add_controller(drag);
}

type MenuAction = Box<dyn Fn(&dyn WindowControl)>;

/// Long press on an app opens its menu (docs/design/multitasking.md): open it on a half of the
/// shown screen or take it out of its pair, when the compositor can, and Close. Close stays here until the title strip exists
/// (docs/design/windows.md).
fn attach_app_menu(
    button: &gtk4::Button,
    controller: Rc<dyn WindowControl>,
    id: ToplevelId,
    split: bool,
    alone: bool,
) {
    let items = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let menu = gtk4::Popover::new();
    menu.set_position(gtk4::PositionType::Right);
    menu.set_has_arrow(false);
    menu.set_child(Some(&items));
    menu.set_parent(button);
    // The rail rebuilds its buttons on every change; a popover must be unparented by hand.
    let m = menu.clone();
    button.connect_destroy(move |_| m.unparent());

    let add = |label: &str, action: MenuAction| {
        let item = gtk4::Button::with_label(label);
        item.add_css_class("bayleaf-menu-item");
        let (m, c) = (menu.clone(), controller.clone());
        item.connect_clicked(move |_| {
            m.popdown();
            action(c.as_ref());
        });
        items.append(&item);
    };
    if split {
        add(
            "Open on the left",
            Box::new(move |c| c.open_on_side(id, Side::Left)),
        );
        add(
            "Open on the right",
            Box::new(move |c| c.open_on_side(id, Side::Right)),
        );
    }
    if alone {
        add("Full screen", Box::new(move |c| c.open_alone(id)));
    }
    add("Close", Box::new(move |c| c.close(id)));

    let press = gtk4::GestureLongPress::new();
    press.set_propagation_phase(gtk4::PropagationPhase::Capture);
    press.connect_pressed(move |gesture, _, _| {
        // Claim the touch so the button does not also switch to the app on release.
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        menu.popup();
    });
    button.add_controller(press);
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
pub fn right_rail(app: &gtk4::Application, keyboard: &osk::Keyboard) -> gtk4::ApplicationWindow {
    let window = rail_window(app, Edge::Right, "right");
    let col = rail_column();

    let clock = gtk4::Label::new(Some(&status::clock_text()));
    clock.add_css_class("status");
    let battery = gtk4::Label::new(status::battery_text().as_deref());
    battery.add_css_class("status");
    col.append(&clock);
    col.append(&battery);
    col.append(&controls::stack(device_controls()));
    col.append(&spacer());
    // Bottom of the right rail: next to the keyboard it controls, mirroring Start on the left.
    col.append(keyboard.button());

    glib::timeout_add_seconds_local(15, move || {
        clock.set_text(&status::clock_text());
        battery.set_text(status::battery_text().as_deref().unwrap_or(""));
        glib::ControlFlow::Continue
    });

    window.set_child(Some(&col));
    window
}

/// Brightness (if the device has a backlight) and volume, as collapsible rail controls.
fn device_controls() -> Vec<controls::Control> {
    let mut list = Vec::new();
    match device::Backlight::find() {
        Some(backlight) => {
            let backlight = Rc::new(backlight);
            let read_bl = backlight.clone();
            list.push(controls::Control {
                icon: "display-brightness-symbolic",
                label: "Brightness",
                read: Rc::new(move || {
                    let v = read_bl.percent();
                    Box::pin(async move { v })
                }),
                write: Rc::new(move |p| backlight.set_percent(p)),
            });
        }
        None => tracing::info!("no backlight device; brightness control hidden"),
    }
    let last_volume = Rc::new(std::cell::Cell::new(None::<f64>));
    list.push(controls::Control {
        icon: "audio-volume-high-symbolic",
        label: "Volume",
        read: Rc::new(|| Box::pin(device::volume_percent())),
        write: Rc::new(move |p| {
            device::set_volume_percent(p);
            if let Some(muted) = device::mute_change(last_volume.get(), p) {
                device::set_muted(muted);
            }
            last_volume.set(Some(p));
        }),
    });
    list
}
