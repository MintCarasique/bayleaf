# Guide for AI agents working on Bayleaf

Bayleaf is developed largely by AI agents directed by a human maintainer. This file is the
contract: read it before changing anything.

## What the project is
A touch-first Wayland shell (compositor + shell UI + on-screen keyboard) in Rust for low-end
tablets. Read `README.md`, `docs/architecture.md` and `docs/roadmap.md` first. Decisions live in
`docs/decisions/` (ADRs) — do not silently contradict an accepted ADR; propose a new one.

## Rules
- **Language**: code, comments, commits, docs in English. The maintainer talks in Russian in chat.
- **License**: GPL-3.0-or-later. Every new crate declares `license = "GPL-3.0-or-later"`.
  Only add dependencies with GPL-3-compatible licenses; note the license in the PR description.
- **Small steps**: one concern per change. Keep crates small with clear boundaries; prefer adding
  a module over growing a 2000-line file.
- **Budgets matter**: the reference device has 2 GB RAM and an Atom CPU. No polling loops, no
  busy timers, no heavy dependencies without a reason written down. Measure before/after for
  anything on a hot path.
- **Errors**: no `unwrap()`/`expect()` in compositor or shell runtime paths; use `anyhow` in
  binaries, `thiserror` in libraries. Never block the compositor event loop.
- **Logging**: `tracing`, structured fields, no `println!` in library code.
- **Tests**: unit tests for pure logic (gesture recognition, layout, config parsing) are required.
  Code that needs a display gets a nested/winit or headless test where feasible.
- **Before finishing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` must pass. Say so explicitly, or say what you could not run and why.

## Layout
```
crates/
  bayleaf-comp/    compositor binary (Smithay)
  bayleaf-shell/   taskbar, start menu, action center
  bayleaf-osk/     on-screen keyboard
  bayleaf-ipc/     shared protocol types (library)
docs/
  architecture.md  roadmap.md  decisions/  design/ (UI specs — rails.md is the source of truth for layout)
```

## Dev loop
- Build and run on Linux (or WSL2 with WSLg). The compositor runs nested in a window with the
  winit backend for most work; DRM/touch/rotation must be verified on real hardware.
- Reference tablet is reachable over SSH by the maintainer; deploy release binaries there, never
  build on it (2 GB RAM).
