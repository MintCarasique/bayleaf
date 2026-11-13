//! Bayleaf CSS for the shell surfaces (see docs/design/rails.md, "Visual language").

use gtk4::gdk;
use gtk4::prelude::*;

const CSS: &str = "
window.bayleaf-rail { background: #1e211e; color: #e6e6e6; }
window.bayleaf-rail.left  { border-right: 1px solid #2f3a2f; }
window.bayleaf-rail.right { border-left:  1px solid #2f3a2f; }
.bayleaf-rail button {
  min-width: 48px; min-height: 48px; padding: 0; border-radius: 8px;
  background: transparent; border: none; box-shadow: none; color: #e6e6e6;
}
.bayleaf-rail button:active { background: #2f3a2f; }
.bayleaf-rail button.active { background: #2f3a2f; }
.bayleaf-rail button.minimized { opacity: 0.55; }
.bayleaf-rail button.start { color: #8fc48f; }
.bayleaf-rail button image { -gtk-icon-size: 24px; }
.bayleaf-rail label.status { font-size: 12px; }
.bayleaf-rail popover button.bayleaf-menu-item { min-height: 48px; min-width: 120px; padding: 0 16px; }
window.bayleaf-scrim { background: rgba(0, 0, 0, 0.25); }
window.bayleaf-launcher { background: #1e211e; color: #e6e6e6; border-right: 1px solid #2f3a2f; }
.bayleaf-launcher flowboxchild { padding: 0; border-radius: 8px; }
.bayleaf-launcher flowboxchild:active { background: #2f3a2f; }
.bayleaf-launcher flowboxchild:focus-visible { outline: 2px solid #4b7f52; outline-offset: -2px; }
.bayleaf-launcher .bayleaf-tile label { font-size: 11px; }
.bayleaf-launcher entry { min-height: 44px; border-radius: 8px; }
";

/// Names a control for screen readers. Used instead of tooltips: on a touchscreen a tooltip pops
/// up on tap and lands under the finger or under the on-screen keyboard.
pub fn name(widget: &impl IsA<gtk4::Accessible>, text: &str) {
    widget.update_property(&[gtk4::accessible::Property::Label(text)]);
}

/// Installs the Bayleaf stylesheet for the default display.
pub fn install() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(CSS);
    match gdk::Display::default() {
        Some(display) => gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ),
        None => tracing::warn!("no default display; shell style not applied"),
    }
}
