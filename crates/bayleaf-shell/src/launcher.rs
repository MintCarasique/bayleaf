//! Launcher panel: app grid with search, growing from the Start button in the bottom-left corner
//! (docs/design/rails.md). A transparent scrim behind it closes the panel on a tap outside.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::app_search;
use crate::osk::Keyboard;

const PANEL_WIDTH: i32 = 420;
const COLUMNS: u32 = 4;
const TILE_SIZE: i32 = 88;
const ICON_SIZE: i32 = 40;

struct AppEntry {
    info: gio::AppInfo,
    haystack: String,
}

pub struct Launcher {
    panel: gtk4::ApplicationWindow,
    scrim: gtk4::ApplicationWindow,
    search: gtk4::SearchEntry,
    scroll: gtk4::ScrolledWindow,
    grid: gtk4::FlowBox,
    apps: RefCell<Vec<AppEntry>>,
    query: RefCell<String>,
    keyboard: Rc<Keyboard>,
}

impl Launcher {
    pub fn new(app: &gtk4::Application, keyboard: Rc<Keyboard>) -> Rc<Self> {
        let scrim = layer_window(app, "bayleaf-launcher-scrim", Layer::Top);
        scrim.add_css_class("bayleaf-scrim");
        for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
            scrim.set_anchor(edge, true);
        }

        let panel = layer_window(app, "bayleaf-launcher", Layer::Overlay);
        panel.add_css_class("bayleaf-launcher");
        panel.set_anchor(Edge::Left, true);
        panel.set_anchor(Edge::Top, true);
        panel.set_anchor(Edge::Bottom, true);
        panel.set_width_request(PANEL_WIDTH);
        panel.set_keyboard_mode(KeyboardMode::OnDemand);

        let grid = gtk4::FlowBox::new();
        grid.set_min_children_per_line(COLUMNS);
        grid.set_max_children_per_line(COLUMNS);
        grid.set_homogeneous(true);
        grid.set_selection_mode(gtk4::SelectionMode::None);
        grid.set_valign(gtk4::Align::Start);
        grid.set_row_spacing(8);
        grid.set_column_spacing(8);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
        scroll.set_vexpand(true);
        scroll.set_child(Some(&grid));

        let search = gtk4::SearchEntry::new();
        search.set_placeholder_text(Some("Search apps"));
        // Typing on a hardware keyboard anywhere in the panel goes to the search field, so the
        // field itself does not need focus when the panel opens.
        search.set_key_capture_widget(Some(&panel));

        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        col.set_margin_top(12);
        col.set_margin_bottom(12);
        col.set_margin_start(12);
        col.set_margin_end(12);
        col.append(&scroll);
        col.append(&search);
        panel.set_child(Some(&col));

        let launcher = Rc::new(Self {
            panel,
            scrim,
            search,
            scroll,
            grid,
            apps: RefCell::default(),
            query: RefCell::default(),
            keyboard,
        });
        launcher.connect_signals();
        launcher.reload();
        launcher
    }

    pub fn toggle(&self) {
        if self.panel.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    fn show(&self) {
        self.search.set_text("");
        self.reset_tiles();
        self.scroll.vadjustment().set_value(0.0);
        self.scrim.present();
        self.panel.present();
        // No focus in the search field on open: a focused field enables text-input and the
        // on-screen keyboard would pop up on every open. It shows when the field is tapped.
        GtkWindowExt::set_focus(&self.panel, None::<&gtk4::Widget>);
    }

    /// Clears hover/pressed state left on a tile by the touch that launched an app: the panel
    /// is hidden before the pointer "leaves", so GTK never clears it on its own.
    fn reset_tiles(&self) {
        let mut child = self.grid.first_child();
        while let Some(c) = child {
            c.unset_state_flags(gtk4::StateFlags::PRELIGHT | gtk4::StateFlags::ACTIVE);
            child = c.next_sibling();
        }
        self.grid.set_focus_child(None::<&gtk4::Widget>);
    }

    fn hide(&self) {
        self.keyboard.shell_field_gone();
        self.panel.set_visible(false);
        self.scrim.set_visible(false);
    }

    /// Rebuilds the grid from the installed applications.
    fn reload(&self) {
        while let Some(child) = self.grid.first_child() {
            self.grid.remove(&child);
        }
        let mut entries: Vec<AppEntry> = gio::AppInfo::all()
            .into_iter()
            .filter(|a| a.should_show())
            .map(|info| {
                let name = info.display_name().to_string();
                let exe = info.executable().to_string_lossy().into_owned();
                let desc = info
                    .description()
                    .map(|d| d.to_string())
                    .unwrap_or_default();
                AppEntry {
                    haystack: app_search::haystack(&[&name, &exe, &desc]),
                    info,
                }
            })
            .collect();
        entries.sort_by_key(|e| app_search::sort_key(&e.info.display_name()));
        let tiles: Vec<gtk4::Box> = entries.iter().map(|e| tile(&e.info)).collect();
        tracing::info!(count = entries.len(), "launcher apps loaded");
        // The filter runs as children are appended and reads `apps`, so the list must be in
        // place first; re-filter at the end in case the query changed meanwhile.
        *self.apps.borrow_mut() = entries;
        for t in &tiles {
            self.grid.append(t);
        }
        self.grid.invalidate_filter();
    }

    fn launch(&self, index: usize) {
        let info = match self.apps.borrow().get(index) {
            Some(e) => e.info.clone(),
            None => return,
        };
        let ctx = WidgetExt::display(&self.panel).app_launch_context();
        if let Err(e) = info.launch(&[], Some(&ctx)) {
            tracing::warn!(error = %e, app = %info.display_name(), "launch failed");
        }
        self.hide();
    }

    fn first_visible(&self) -> Option<usize> {
        let query = self.query.borrow();
        self.apps
            .borrow()
            .iter()
            .position(|e| app_search::matches(&e.haystack, &query))
    }

    fn connect_signals(self: &Rc<Self>) {
        // A tap on the search field brings the keyboard for as long as the launcher is open
        // (keyboard mode, see osk.rs).
        let tap = gtk4::GestureClick::new();
        tap.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        tap.connect_pressed(move |_, _, _, _| {
            if let Some(l) = weak.upgrade() {
                l.keyboard.show_for_shell_field();
            }
        });
        self.search.add_controller(tap);

        let weak = Rc::downgrade(self);
        self.grid.set_filter_func(move |child| {
            let Some(l) = weak.upgrade() else { return true };
            let Ok(index) = usize::try_from(child.index()) else {
                return false;
            };
            let query = l.query.borrow();
            l.apps
                .borrow()
                .get(index)
                .is_some_and(|e| app_search::matches(&e.haystack, &query))
        });

        let weak = Rc::downgrade(self);
        self.grid.connect_child_activated(move |_, child| {
            if let (Some(l), Ok(i)) = (weak.upgrade(), usize::try_from(child.index())) {
                l.launch(i);
            }
        });

        let weak = Rc::downgrade(self);
        self.search.connect_search_changed(move |s| {
            if let Some(l) = weak.upgrade() {
                *l.query.borrow_mut() = s.text().to_string();
                l.grid.invalidate_filter();
            }
        });

        let weak = Rc::downgrade(self);
        self.search.connect_activate(move |_| {
            if let Some(l) = weak.upgrade() {
                if let Some(i) = l.first_visible() {
                    l.launch(i);
                }
            }
        });

        let keys = gtk4::EventControllerKey::new();
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(l) = weak.upgrade() {
                    l.hide();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.panel.add_controller(keys);

        let tap = gtk4::GestureClick::new();
        let weak = Rc::downgrade(self);
        tap.connect_released(move |_, _, _, _| {
            if let Some(l) = weak.upgrade() {
                l.hide();
            }
        });
        self.scrim.add_controller(tap);

        let monitor = gio::AppInfoMonitor::get();
        let weak = Rc::downgrade(self);
        monitor.connect_changed(move |_| {
            if let Some(l) = weak.upgrade() {
                l.reload();
            }
        });
    }
}

fn layer_window(app: &gtk4::Application, namespace: &str, layer: Layer) -> gtk4::ApplicationWindow {
    let w = gtk4::ApplicationWindow::new(app);
    w.init_layer_shell();
    w.set_namespace(Some(namespace));
    w.set_layer(layer);
    w.set_visible(false);
    w
}

fn tile(info: &gio::AppInfo) -> gtk4::Box {
    let image = match info.icon() {
        Some(icon) => gtk4::Image::from_gicon(&icon),
        None => gtk4::Image::from_icon_name("application-x-executable"),
    };
    image.set_pixel_size(ICON_SIZE);

    let label = gtk4::Label::new(Some(&info.display_name()));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_max_width_chars(10);
    label.set_justify(gtk4::Justification::Center);

    let tile = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    tile.add_css_class("bayleaf-tile");
    tile.set_size_request(TILE_SIZE, TILE_SIZE);
    tile.set_valign(gtk4::Align::Center);
    tile.append(&image);
    tile.append(&label);
    tile
}
