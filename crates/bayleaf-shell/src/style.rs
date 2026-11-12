//! Bayleaf CSS for the shell surfaces (see docs/design/rails.md, "Visual language").

use gtk4::gdk;

const CSS: &str = "
window.bayleaf-rail { background: #1e211e; color: #e6e6e6; }
window.bayleaf-rail.left  { border-right: 1px solid #2f3a2f; }
window.bayleaf-rail.right { border-left:  1px solid #2f3a2f; }
.bayleaf-rail button {
  min-width: 48px; min-height: 48px; padding: 0; border-radius: 8px;
  background: transparent; border: none; box-shadow: none; color: #e6e6e6;
}
.bayleaf-rail button:hover, .bayleaf-rail button:active { background: #2f3a2f; }
.bayleaf-rail button.active { background: #2f3a2f; }
.bayleaf-rail button.minimized { opacity: 0.55; }
.bayleaf-rail button.start { color: #8fc48f; }
.bayleaf-rail button image { -gtk-icon-size: 24px; }
.bayleaf-rail label.status { font-size: 12px; }
";

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
