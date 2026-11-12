# ADR-0003: UI concept — side rails

Status: **accepted** (2026-11-12)

## Context
The reference device is a 10" 1280×800 tablet held in landscape with two hands. Vertical space is
the scarce resource (800 px), horizontal space is relatively plentiful. Three concepts were
compared:

- **A. Familiar** — Windows 10 tablet mode: bottom taskbar, full-screen start grid.
  Zero learning curve, but costs height and the start button sits in a corner.
- **B. Dock and gestures** — iPadOS-like: thin status line, floating dock, control center from
  the top-right corner. Maximises app area, but depends heavily on gestures.
- **C. Side rails** — a narrow rail on each side: apps on the left, system on the right, both
  under the thumbs.

## Decision
Use **C. Side rails** as the primary layout, borrowing:
- from **B**: quick settings slide out from the right rail with a swipe;
- from **A**: in laptop mode (keyboard dock attached) the rails collapse into a conventional bottom
  taskbar, because pointer users expect it.

## Consequences
- Width, not height, pays for the shell UI: ~2 × 56 px of 1280 px in landscape.
- Portrait needs its own layout (rails become a bottom bar), so the shell must be
  orientation-aware from day one.
- Rails are plain layer-shell surfaces, so M1 works on any compositor; edge-swipe behaviour comes
  with `bayleaf-comp` in M2.
- Gives the project a recognisable identity instead of cloning Windows or iPadOS.

Design details: [../design/rails.md](../design/rails.md).
