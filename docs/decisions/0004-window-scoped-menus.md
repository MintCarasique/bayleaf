# ADR-0004: Window-scoped app menus; right rail controls the device

Status: **accepted** (2026-11-12). Refines [ADR-0003](0003-ui-concept-rails.md).

## Context
With the first rails draft the right rail held only a clock, a few status icons, the keyboard
toggle and notifications — 56 px of mostly empty width along the whole screen height.

Putting a global app menu on the rail (as Ubuntu Unity did in its top bar) was considered, but it
breaks in side-by-side mode: with two windows visible, it is unclear whose menu the rail button
opens, and "the focused one" is easy to get wrong with touch.

Other options considered: vertical cascading menubar, menu icons on the rail, radial menu,
search-only HUD, frequent actions on the rail, a single rail, an edge handle instead of a rail.

## Decision
- **Everything window-specific lives on the window.** The tablet-mode title strip carries
  ← Back, ☰ App menu and ✕ Close. The menu opens as a drill-down panel with search inside the
  window's own area.
- **Rails are global.** Left rail: apps. Right rail: the device — status, vertical brightness and
  volume sliders under the thumb, media controls while something plays, quick settings, keyboard,
  notifications.
- Back moves from the left rail to the title strip.

## Consequences
- No ambiguity with two windows side by side: each has its own strip and menu.
- The right rail earns its width with controls used constantly on a tablet (brightness, volume).
- `bayleaf-comp` must implement the `org_kde_kwin_appmenu` protocol and the shell needs a
  DBusMenu client. GTK4/libadwaita apps export no menu and get window actions as a fallback.
- Rail sliders are collapsed to icons and open one at a time on tap (accordion), so they cannot
  be moved by an accidental touch and do not eat rail height when unused.
