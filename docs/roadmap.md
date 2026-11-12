# Roadmap

Every milestone ends with something usable on its own, so the project is worth it even if work
pauses for a while.

## M0 — Foundation
- [x] Repository, GPL-3.0-or-later, README, architecture draft
- [ ] Cargo workspace skeleton, CI (fmt, clippy `-D warnings`, test)
- [ ] Dev loop: build in Linux/WSL2, run nested (winit backend), deploy to the tablet over SSH
- [ ] ADR-0001: compositor starting point (niri fork vs. anvil)
- [ ] ADR-0002: shell toolkit (iced vs. slint)

## M1 — Shell on an existing compositor
Works on labwc / sway / KWin via layer-shell + foreign-toplevel.
- [ ] Taskbar: start button, running apps, clock, battery, network, volume
- [ ] Start menu: app grid/list from `.desktop` files, search
- [ ] Action center: quick toggles (Wi-Fi, Bluetooth, rotation lock), volume and brightness sliders,
      notifications (org.freedesktop.Notifications server)

## M2 — Compositor
- [ ] Boots on DRM/KMS with libseat, XWayland
- [ ] Tablet mode window model, laptop mode floating windows, minimize
- [ ] Edge-swipe gestures, drag/resize by touch, long-press secondary click
- [ ] Rotation from iio-sensor-proxy with correct touch mapping
- [ ] Dock detection → automatic mode switch

## M3 — Integration
- [ ] On-screen keyboard via input-method-v2 (auto show/hide)
- [ ] Session lock with touch PIN entry
- [ ] Settings app for Bayleaf-specific options
- [ ] Packaging: Debian package, session file for display managers

## M4 — 0.1 release
- [ ] Meets the budgets in architecture.md on the reference tablet
- [ ] Docs for users and contributors
