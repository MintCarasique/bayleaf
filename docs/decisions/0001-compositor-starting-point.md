# ADR-0001: Compositor starting point

- **Status:** proposed
- **Date:** 2026-11-13

## Context
M1 runs `bayleaf-shell` on labwc. That covers the rails and the launcher, but not the parts that
make Bayleaf a tablet shell: a layout model with full-screen apps and 50/50 split view, window
transitions, the title strip, edge gestures and touch-correct rotation. Those need a compositor of
our own (`bayleaf-comp`), built on Smithay ([architecture.md](../architecture.md)).

## Requirements
In order of how much they shape the code:

1. **Split view as the window model** ([design/multitasking.md](../design/multitasking.md)): an
   ordered list of screens, each one app or a 50/50 pair; divider with a release threshold;
   crossfade and slide transitions on every change.
2. **Laptop mode**: floating, overlapping windows with minimise, snapping to halves; switching
   between the two models live when the dock is attached or detached
   ([design/windows.md](../design/windows.md)).
3. **Server-side title strip** (back, app menu, close) and `org_kde_kwin_appmenu`.
4. **Protocols the shell relies on**: layer-shell, foreign-toplevel, input-method-v2,
   virtual-keyboard, xdg-decoration, plus a Bayleaf IPC for "open on a half" and "show screen".
5. **Touch**: edge swipes in the compositor, long press as secondary click, drag and resize by
   touch, rotation from iio-sensor-proxy with touch mapped in every orientation.
6. **Budgets**: idle CPU ~0 %, whole session under 300 MB RSS on the reference tablet; DRM/KMS with
   libseat, XWayland, on Bay Trail graphics (i915, GLES 2/3).

## Options

### A. Fork niri
GPL-3, mature Smithay-based compositor: DRM backend, libinput, XWayland via xwayland-satellite,
layer-shell, foreign-toplevel, input methods, screen casting, animations with springs.

- **For:** the hard, boring parts (multi-GPU, session handling, damage tracking, frame scheduling)
  already work and are tested on real hardware; animation infrastructure exists.
- **Against:** the layout is a scrollable strip of columns per workspace, and most of niri's code
  is shaped around it. Requirements 1 and 2 mean replacing the layout wholesale and adding a
  floating model, which touches gestures, focus and rendering. A large fork drifts away from
  upstream quickly, so we would carry the whole backend alone anyway.

### B. Build up from Smithay's anvil example
Anvil is Smithay's reference compositor: DRM, winit and X11 backends, XWayland, basic floating
windows.

- **For:** the window model is ours from day one; screens, pairs and floating fit cleanly; the code
  stays small and fully understood; we pick only the protocols we need.
- **Against:** anvil is an example, not a product. Frame scheduling, damage tracking, input edge
  cases, layer-shell and input-method plumbing all need work that niri has already done. Slower
  path to the first usable session.

### C. Hybrid
Start from anvil for structure, and port specific, self-contained pieces from niri (with
attribution; both GPL-3-compatible) where we hit the same problem: frame clock, animation helpers,
input-method plumbing.

## Assessment so far
Requirement 1 is the deciding one: the window model is the centre of a compositor, and niri's model
is the one part we would have to throw away. That favours **B or C**. Before deciding:

- build anvil with the winit backend in WSL2 and on the tablet (DRM), check idle CPU and RSS;
- sketch the screen/pair layout on top of anvil's space and check how much of the drawing and
  input code can stay;
- list the niri modules worth porting and check their dependencies on its layout code.

## Decision
Pending.
