# Design: window management

Part of the [side-rails](rails.md) design (ADR-0003). Sizes are logical pixels at scale 1.

## Tablet mode (no keyboard)

Every window fills the area between the rails (1168 × 800 in landscape).

### Title strip
A slim server-side strip on top of every window that accepts server-side decorations
(xdg-decoration):

```
┌──────────────────────────────────────────────────────────┐
│  [icon]  App title                                 [ ✕ ] │  40 px
├──────────────────────────────────────────────────────────┤
│                       application                        │
```

- Height **40 px**; close button **56 × 40 px** hit area at the right end, 20 px glyph.
- App icon and title on the left, nothing else — minimise and maximise make no sense in tablet mode.
- Dragging the strip downwards past 25 % of the screen height closes the window (same as the
  top-edge swipe); releasing earlier snaps it back.
- Clients that draw their own titlebar (GTK4/libadwaita CSD) keep it; Bayleaf adds no second strip.

### Actions
| Action | How |
|---|---|
| Switch app | tap its icon in the left rail |
| Close | ✕ in the title strip; swipe down from the top edge; or "Close" in the rail menu |
| Side by side | drag an icon from the left rail onto the left or right half; drag the divider (6 px handle, 48 px grab zone) to resize. Max two windows. |
| Window menu | long press an icon in the left rail: side by side, full screen, pin to rail, close |
| Minimise | not needed — the app stays in the rail |
| Dialogs | float centred over a dimmed backdrop, no title strip |

## Laptop mode (keyboard dock attached)

Floating, overlapping windows.

| Action | How |
|---|---|
| Minimise / maximise / close | buttons in the title bar (32 px high, server-side where supported) |
| Snap | drag to the left/right edge → half screen (edge highlight while dragging); drag to the top → maximise; Super+arrows |
| Resize | borders and corners; grab zones grow to 24 px when the drag starts from touch |
| Minimised windows | dimmed in the taskbar; tap or click restores |

## Switching modes
- **Dock detached → tablet**: all windows maximise; side-by-side pairs are kept; floating
  geometry is remembered per window.
- **Dock attached → laptop**: windows return to their remembered floating geometry.

## Decorations
- Bayleaf prefers server-side decorations and advertises them via xdg-decoration.
- Clients that insist on client-side decorations (GTK4/libadwaita) keep theirs in both modes.
- Qt/KDE apps, Firefox, terminals and most toolkits accept server-side decorations and get the
  Bayleaf title strip / title bar.
