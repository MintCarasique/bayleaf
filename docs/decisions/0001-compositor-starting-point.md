# ADR-0001: Compositor starting point

- **Status:** accepted
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

### D. Fork labwc
labwc is what M1 runs on today: C, wlroots, Openbox-like stacking window manager.

- **For:** already works on the reference tablet (about 50 MB RSS, idle CPU ~0 %), with every
  protocol the shell and squeekboard use; wlroots is the most battle-tested base; only the window
  model, gestures and animations would be new.
- **Against:** C instead of Rust, so no shared `bayleaf-ipc` with the shell and no memory safety in
  the process whose crash ends the session; upstream wants a classic stacking WM, so split view and
  animations mean a permanent fork; there is no animation code to build on.

## Spike: anvil on the reference tablet (2026-11-13)
Smithay main (`9ff2c5a`), anvil release build, DRM/KMS session from SDDM with `bayleaf-shell`
and squeekboard on top. Setup: [prototypes/anvil-session](../../prototypes/anvil-session/setup.sh).

**Works:** DRM/KMS and libseat on Bay Trail (GLES2), libinput touch, XWayland, layer-shell (both
rails and the launcher), input-method-v2 + text-input-v3 (squeekboard shows on text focus).

**Findings**
| | anvil | labwc (M1) |
|---|---|---|
| Compositor RSS, idle | 96 MB | ~50 MB |
| Compositor CPU, idle | ~8 % of a core (redraws all the time) | ~0 % |
| Xwayland | started eagerly, 84 MB | started on demand |
| Feels | slightly laggier | smooth |

- **GTK 4.18 (Debian trixie) crashes on linux-dmabuf v6**, which current Smithay advertises when a
  default feedback is set: `bayleaf-shell` segfaults inside `gtk_init` while handling the feedback.
  Capping the global at version 5 fixes it. Any Smithay compositor has to cap it until Debian ships
  a fixed GTK.
- No `wlr-foreign-toplevel-management` in Smithay: the left rail stays empty; we implement it
  ourselves (niri does, a few hundred lines).
- No `wlr-screencopy`; Smithay has `ext-image-copy-capture` instead.
- Example-level gaps, all ours to write in any Smithay-based option: the cursor is always drawn,
  also during touch; new windows ignore exclusive zones (Firefox opens under the rails); touch drag
  of a window jumps to the top-left corner; no frame scheduling or damage tracking worth the name.

None of this is a limit of Smithay itself, but it shows how much of a usable compositor anvil
leaves to us: idle behaviour, frame pacing, cursor handling and layout are exactly the parts that
make the difference on this hardware.

## Spike: niri on the reference tablet (2026-11-13)
niri v26.04 (Smithay `ff5fa7d`, linux-dmabuf capped at v5 there, so GTK 4.18 works), release build
without screencasting, DRM/KMS session from SDDM with `bayleaf-shell` and squeekboard. Config:
columns at 100 % width, no gaps, borders or focus ring. Setup:
[prototypes/niri-session](../../prototypes/niri-session/setup.sh).

| Idle, 30 s without touching | niri | anvil | labwc (M1) |
|---|---|---|---|
| Compositor CPU | 0.04 s (~0.1 % of a core) | ~2.3 s (~8 %) | ~0 % (not measured the same way) |
| Compositor RSS | 112 MB | 96 MB | ~50 MB |
| `bayleaf-shell` RSS | 68 MB | 149 MB | 146 MB |
| Xwayland | separate `xwayland-satellite`, on demand (not installed here) | eager, 84 MB | on demand |

- **Feels much smoother than labwc and anvil**; switching windows animates (horizontal slide).
- Left rail works: niri implements `wlr-foreign-toplevel-management`.
- squeekboard shows on text focus.
- **No way to close a window by touch**: niri draws no server-side title bars, and the config asks
  apps not to draw their own. The title strip (← ☰ ✕) is ours to add in any option.
- The shell is half the size under niri than under labwc; not investigated yet.

### What the niri spike changes
niri's layout is a horizontal strip of columns per workspace. With full-width columns that strip
*is* our full-screen model, and our 50/50 pair maps onto two half-width columns side by side,
which niri already supports (column width presets, consume/expel). The window model is closer than
assumed above; the differences are presentation (crossfade instead of a slide, no free scrolling
between columns, overview with cards) and the title strip.

niri also has an IPC (`niri msg`, a JSON socket): focus a window, close it, set a column width,
move columns. The shell can drive much of [multitasking](../design/multitasking.md) through it
without touching the compositor.

## Assessment so far
Smithay itself is fine on Bay Trail; what makes the difference is the compositor built on it, and
niri is already very good here. That reverses the earlier lean towards anvil:

- **A (niri) is the strongest option.** Start with upstream niri plus the IPC for M2 work; fork
  only for what IPC cannot do (server-side title strip, crossfade switching, pair-aware overview),
  and try to upstream generic pieces to keep the fork thin.
- **B/C** would mean rebuilding frame pacing, animations and input handling that niri already
  gets right on this hardware.
- **D (labwc)** stays the fallback if niri's memory (about 60 MB more than labwc) becomes a
  problem; idle CPU and smoothness favour niri.

Open before accepting:
- map every action in [multitasking.md](../design/multitasking.md) to niri IPC or to a needed
  compositor change;
- check how a title strip would fit into niri's rendering (it already draws borders and tab
  indicators per window);
- settle Xwayland (`xwayland-satellite` from source, or none at first).

## Decision
**Base `bayleaf-comp` on niri (option A).**

1. **Upstream niri first.** M2 starts on stock niri with a Bayleaf config; the shell drives windows
   through niri's IPC wherever it can (focus, close, column width, moving columns).
2. **Thin fork for the rest.** `bayleaf-comp` becomes a niri fork only for what IPC cannot do:
   the server-side title strip (← ☰ ✕), crossfade switching, the pair-aware overview, edge
   gestures and dock-driven mode switching. Generic pieces go upstream when niri wants them;
   the fork is rebased on niri releases.
3. **Attribution and licence.** niri is GPL-3.0-or-later, same as Bayleaf; the fork keeps its
   history and copyright notices.

The fork lives at [MintCarasique/niri](https://github.com/MintCarasique/niri), branch `bayleaf`
(see [design/niri-integration.md](../design/niri-integration.md) for what it changes).

Follow-ups (tracked in the roadmap): map [multitasking](../design/multitasking.md) onto niri IPC
and list the compositor changes; prototype the title strip in niri's renderer; decide on
Xwayland (`xwayland-satellite`).

## Consequences
- About 60 MB more compositor memory than labwc; still inside the 300 MB session budget with the
  shell (68 MB) and squeekboard (37 MB), to be watched.
- Our 50/50 pair is modelled as two half-width columns; the free horizontal scrolling of niri's
  strip has to be constrained in tablet mode.
- linux-dmabuf stays capped at v5 while Debian ships GTK 4.18 (niri already does).
- The anvil spike in `prototypes/anvil-session` is kept for reference only.
