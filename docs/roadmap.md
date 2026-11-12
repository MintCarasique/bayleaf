# Roadmap

Every milestone ends with something usable on its own, so the project is worth it even if work
pauses for a while.

## M0 — Foundation
- [x] Repository, GPL-3.0-or-later, README, architecture draft
- [x] Cargo workspace skeleton, CI (fmt, clippy `-D warnings`, test)
- [x] Dev loop: build in Linux/WSL2, deploy to the tablet over SSH (`scripts/check.sh`, `scripts/deploy.sh`)
- [ ] Run the compositor nested (winit backend) — comes with ADR-0001
- [ ] ADR-0001: compositor starting point (niri fork vs. anvil)
- [ ] ADR-0002: shell toolkit (iced vs. slint)

## M1 — Shell on an existing compositor
Works on labwc / sway / KWin via layer-shell + foreign-toplevel.
Layout follows [the side-rails design](design/rails.md) (ADR-0003, ADR-0004).
- [ ] Left rail: start, running and pinned apps
- [ ] Right rail: status, vertical brightness and volume sliders, media controls, quick settings,
      keyboard toggle, notifications
- [ ] Launcher panel: app grid from `.desktop` files, search
- [ ] Quick settings panel: toggles (Wi-Fi, Bluetooth, rotation lock, tablet mode), volume and
      brightness sliders, media controls
- [ ] Notification server (org.freedesktop.Notifications) and list
- [ ] Portrait bottom bar and laptop-mode taskbar variants

## M2 — Compositor
- [ ] Boots on DRM/KMS with libseat, XWayland
- [ ] Tablet mode window model with the title strip (back, app menu, close), side by side;
      laptop mode floating windows, minimize
- [ ] Edge-swipe gestures, drag/resize by touch, long-press secondary click
- [ ] Rotation from iio-sensor-proxy with correct touch mapping
- [ ] Dock detection → automatic mode switch

## M3 — Integration
- [ ] On-screen keyboard via input-method-v2 (auto show/hide)
- [ ] App menu panel: `org_kde_kwin_appmenu` + DBusMenu, drill-down list with search
- [ ] Session lock with touch PIN entry
- [ ] Settings app for Bayleaf-specific options
- [ ] Packaging: Debian package, session file for display managers

## M4 — 0.1 release
- [ ] Meets the budgets in architecture.md on the reference tablet
- [ ] Docs for users and contributors
