# ADR-0002: Shell toolkit — GTK4 (gtk4-rs + gtk4-layer-shell)

Status: **accepted** (2026-11-12)

## Context
`bayleaf-shell` draws the rails and panels as layer-shell surfaces and has to feel good under a
finger on the reference tablet (Atom Z3736F, 2 GB RAM, Intel Gen7 GPU). Candidates:

- **iced** 0.14 + `iced_layershell` 0.19 (pure Rust, MIT, used by COSMIC);
- **GTK4** via `gtk4-rs` 0.11 + `gtk4-layer-shell` 0.8 (mature, many existing panels);
- **slint** — dropped before prototyping: no layer-shell support upstream, only the pre-stable
  third-party `layer-shika`.

The same prototype was built in both (`prototypes/rail-gtk`, `prototypes/rail-iced`): two 56 px
rails as layer-shell surfaces in one process, buttons, clock, battery, a vertical slider.
Measured on the tablet under KWin (Plasma 6.3) and tried by hand in a bare labwc session.

## Measurements (reference tablet)

| | GTK4 | iced |
|---|---|---|
| First frame, warm start | ~860 ms | ~520 ms |
| First frame, cold start | ~950–1500 ms | ~2300 ms (shader compilation) |
| RSS | ~123 MB | ~68 MB |
| Idle CPU | 0 | ≈0 |
| Binary | 0.4 MB + system GTK | 12 MB static |
| Touch press feedback | built in (`:active`) | missing — iced derives pressed/hover from the cursor, which touch does not move; needs custom state |
| Icons | system icon theme | none built in |
| Hands-on feel | **"more native"** (maintainer) | usable after workarounds |

## Decision
Use **GTK4 through gtk4-rs with gtk4-layer-shell** for `bayleaf-shell` (and later `bayleaf-osk`).
Plain GTK4 widgets with Bayleaf's own CSS; no libadwaita for now (its look and memory cost are
not needed for rails and panels).

## Consequences
- Touch behaviour (press states, scrolling, long press, gestures, accessibility) comes from a
  mature toolkit instead of being rebuilt.
- **Memory is the price**: ~55 MB more than iced for the same UI. Mitigations:
  - one shell process for all surfaces (rails, launcher, quick settings, notifications);
  - the on-screen keyboard lives in the same process rather than a second GTK process;
  - GTK's shared libraries are shared with GTK apps the user already runs (Firefox).
  The 300 MB session budget in architecture.md stays; re-measure at M1.
- Build needs `libgtk-4-dev` and `libgtk4-layer-shell-dev`. Ubuntu 24.04 has no
  `libgtk4-layer-shell-dev`, so CI moves to a Debian 13 container when the shell starts depending
  on GTK.
- GTK's a11y bus warning appears without `at-spi2-core`; harmless, set `GTK_A11Y=none` only in
  sessions that genuinely have no accessibility stack.
