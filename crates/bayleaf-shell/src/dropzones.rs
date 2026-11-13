//! Drop zones for dragging an app from the left rail onto the screen (docs/design/multitasking.md,
//! "Drop zones"): a transparent overlay over the work area with the left and right half outlined,
//! or a single "Full screen" zone, and a ghost of the dragged icon under the finger.
//!
//! The overlay never takes input: the drag stays with the rail surface that got the touch, and
//! the rail reports positions in screen coordinates.

use std::cell::Cell;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::rails::RAIL_WIDTH;

const GHOST_SIZE: i32 = 48;

/// Where a drop lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Left,
    Right,
    Full,
}

/// Target under `x`, in overlay coordinates (0 = right edge of the left rail). `None` outside the
/// work area, which cancels the drop.
pub fn target_at(x: f64, width: f64, halves: bool) -> Option<Target> {
    if x < 0.0 || x > width || width <= 0.0 {
        return None;
    }
    Some(match (halves, x < width / 2.0) {
        (false, _) => Target::Full,
        (true, true) => Target::Left,
        (true, false) => Target::Right,
    })
}

pub struct DropZones {
    window: gtk4::ApplicationWindow,
    zones: gtk4::Stack,
    halves_mode: Cell<bool>,
    halves: gtk4::Box,
    left: gtk4::Box,
    right: gtk4::Box,
    full: gtk4::Box,
    ghost: gtk4::Image,
    canvas: gtk4::Fixed,
}

fn zone(icon: &str, text: &str) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    b.add_css_class("bayleaf-zone");
    b.set_hexpand(true);
    b.set_vexpand(true);
    b.set_valign(gtk4::Align::Fill);
    let inner = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    inner.set_valign(gtk4::Align::Center);
    inner.set_vexpand(true);
    let image = gtk4::Image::from_icon_name(icon);
    image.set_pixel_size(32);
    inner.append(&image);
    inner.append(&gtk4::Label::new(Some(text)));
    b.append(&inner);
    b
}

impl DropZones {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::ApplicationWindow::new(app);
        window.add_css_class("bayleaf-dropzones");
        window.init_layer_shell();
        window.set_namespace(Some("bayleaf-dropzones"));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
            window.set_anchor(edge, true);
        }
        window.set_can_target(false);

        let left = zone("view-dual-symbolic", "Open on the left");
        let right = zone("view-dual-symbolic", "Open on the right");
        let halves = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        halves.set_homogeneous(true);
        halves.append(&left);
        halves.append(&right);
        let full = zone("view-fullscreen-symbolic", "Full screen");

        let zones = gtk4::Stack::new();
        zones.add_child(&halves);
        zones.add_child(&full);

        let ghost = gtk4::Image::new();
        ghost.set_pixel_size(GHOST_SIZE - 16);
        ghost.set_size_request(GHOST_SIZE, GHOST_SIZE);
        ghost.add_css_class("bayleaf-ghost");
        let canvas = gtk4::Fixed::new();
        canvas.put(&ghost, 0.0, 0.0);
        canvas.set_can_target(false);

        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&zones));
        overlay.add_overlay(&canvas);
        window.set_child(Some(&overlay));
        window.set_visible(false);

        Self {
            window,
            zones,
            halves_mode: Cell::new(true),
            halves,
            left,
            right,
            full,
            ghost,
            canvas,
        }
    }

    /// Shows the zones for a drag of `icon`: two halves, or one "Full screen" zone.
    pub fn begin(&self, icon: &str, halves: bool) {
        self.ghost.set_icon_name(Some(icon));
        self.halves_mode.set(halves);
        let shown: &gtk4::Box = if halves { &self.halves } else { &self.full };
        self.zones.set_visible_child(shown);
        for z in [&self.left, &self.right, &self.full] {
            z.remove_css_class("on");
        }
        self.window.present();
    }

    /// Moves the ghost to the finger (screen coordinates) and highlights the zone under it.
    pub fn update(&self, x: f64, y: f64) -> Option<Target> {
        let ox = x - f64::from(RAIL_WIDTH);
        let target = target_at(ox, f64::from(self.window.width()), self.halves_mode.get());
        let half = f64::from(GHOST_SIZE) / 2.0;
        self.canvas.move_(&self.ghost, ox - half, y - half);
        for (zone, t) in [
            (&self.left, Target::Left),
            (&self.right, Target::Right),
            (&self.full, Target::Full),
        ] {
            if target == Some(t) {
                zone.add_css_class("on");
            } else {
                zone.remove_css_class("on");
            }
        }
        target
    }

    pub fn end(&self) {
        self.window.set_visible(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_split_at_the_middle() {
        assert_eq!(target_at(10.0, 1000.0, true), Some(Target::Left));
        assert_eq!(target_at(499.0, 1000.0, true), Some(Target::Left));
        assert_eq!(target_at(501.0, 1000.0, true), Some(Target::Right));
    }

    #[test]
    fn single_zone_is_full_screen() {
        assert_eq!(target_at(900.0, 1000.0, false), Some(Target::Full));
    }

    #[test]
    fn outside_the_work_area_cancels() {
        assert_eq!(target_at(-5.0, 1000.0, true), None);
        assert_eq!(target_at(1001.0, 1000.0, true), None);
        assert_eq!(target_at(10.0, 0.0, true), None);
    }
}
