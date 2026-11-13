# Roadmap

Every milestone ends with something usable on its own, so the project is worth it even if work
pauses for a while.

## M0 — Foundation
- [x] Repository, GPL-3.0-or-later, README, architecture draft
- [x] Cargo workspace skeleton, CI (fmt, clippy `-D warnings`, test)
- [x] Dev loop: build in Linux/WSL2, deploy to the tablet over SSH (`scripts/check.sh`, `scripts/deploy.sh`)
- [ ] Run the compositor nested (winit backend) — comes with ADR-0001
- [ ] ADR-0001: compositor starting point (niri, anvil or labwc); anvil spike on the tablet done
- [x] ADR-0002: shell toolkit — GTK4 (prototypes in `prototypes/`)

## M1 — Shell on an existing compositor
Works on labwc / sway / KWin via layer-shell + foreign-toplevel.
Layout follows [the side-rails design](design/rails.md) (ADR-0003, ADR-0004).
- [x] Left rail: running apps via wlr-foreign-toplevel, tap to switch, Start at the bottom
- [ ] Left rail: pinned apps, long-press menu, drag onto a screen half, overview button
- [x] Right rail: clock, battery, brightness and volume as collapsible sliders (accordion)
- [ ] Right rail: UPower and native PipeWire/logind clients instead of wpctl/busctl, media
      controls (MPRIS), quick settings, notifications
- [x] On-screen keyboard: squeekboard in the dev session, toggle at the bottom of the right rail
      (ADR-0005); the launcher opens it only when the search field is tapped
- [x] Launcher panel: app grid from `.desktop` files, search at the bottom, closes on outside tap
- [ ] Launcher: pinned apps
- [ ] Quick settings panel: toggles (Wi-Fi, Bluetooth, rotation lock, tablet mode), volume and
      brightness sliders, media controls
- [ ] Notification server (org.freedesktop.Notifications) and list
- [ ] Portrait bottom bar and laptop-mode taskbar variants

## M2 — Compositor
- [ ] Boots on DRM/KMS with libseat, XWayland
- [ ] Tablet mode window model ([multitasking](design/multitasking.md)): screens, 50/50 split
      view, divider, crossfade and slide transitions, overview; title strip (back, app menu, close)
- [ ] Laptop mode floating windows, minimize, snapping
- [ ] Edge-swipe gestures, drag/resize by touch, long-press secondary click
- [ ] Rotation from iio-sensor-proxy with correct touch mapping
- [ ] Dock detection → automatic mode switch

## M3 — Integration
- [ ] input-method-v2, text-input-v3 and virtual-keyboard in `bayleaf-comp`, so squeekboard keeps
      working on our compositor
- [ ] App menu panel: `org_kde_kwin_appmenu` + DBusMenu, drill-down list with search
- [ ] Session lock with touch PIN entry
- [ ] Settings app for Bayleaf-specific options
- [ ] Packaging: Debian package, session file for display managers

## M4 — 0.1 release
- [ ] Meets the budgets in architecture.md on the reference tablet
- [ ] Docs for users and contributors
