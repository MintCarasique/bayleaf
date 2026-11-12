# Design: side rails

Accepted in [ADR-0003](../decisions/0003-ui-concept-rails.md). Sizes are logical pixels at scale 1
on the 1280×800 reference screen.

## Principles
1. **Thumb reach.** Everything frequent lives within ~60 px of the left or right edge, in the
   vertical middle of the screen where thumbs rest when holding a tablet.
2. **Left = apps, right = system.** The left hand moves between apps; the right hand controls the
   device. The split never changes, so muscle memory builds fast.
3. **Height belongs to apps.** No horizontal bars in tablet landscape.
4. **Touch targets ≥ 48 px**, spacing ≥ 8 px. Icons 24 px.

## Tablet, landscape (primary)

```
┌──────┬────────────────────────────────────────────┬──────┐
│ ⊞    │                                            │19:40 │
│ ──   │                                            │ ▮55% │
│ [●]  │                                            │ wifi │
│  ○   │              application                   │ vol  │
│  ○   │          (1168 × 800, maximized)            │ sun  │
│  ○   │                                            │      │
│      │                                            │  ⌨   │
│  ↶   │                                            │  🔔  │
└──────┴────────────────────────────────────────────┴──────┘
  56 px                                               56 px
```

### Left rail — apps
| Element | Tap | Long press / swipe |
|---|---|---|
| Start (top) | open launcher panel sliding from the left | — |
| Running apps (middle, scrollable) | switch to app | long press: close / app menu; drag out to the right: split view (later) |
| Back (bottom) | sends `Back` (Alt+Left / Escape by app class) | — |

The active app is marked with a filled background, not a dot — readable at a glance.

### Right rail — system
| Element | Tap | Swipe left from the rail |
|---|---|---|
| Clock, battery, network, volume, brightness (status, top) | open quick settings | open quick settings |
| Keyboard (bottom) | toggle on-screen keyboard | — |
| Notifications (bottom) | open notification list | — |

**Quick settings panel** slides out from the right rail (≈ 360 px wide): toggles (Wi-Fi, Bluetooth,
rotation lock, tablet mode, night light, do not disturb) as a 2-column grid of 48 px buttons,
brightness and volume sliders, media controls.

**Launcher panel** slides out from the left rail (≈ 420 px wide): search field on top, app grid
(4 columns, 72 px tiles), pinned apps first. The on-screen keyboard opens with the search field.

## Tablet, portrait (800 × 1280)
Rails merge into one bottom bar (64 px): start and running apps on the left half, status,
keyboard and notifications on the right half. Panels slide up from the bar instead of from the
sides.

## Laptop mode (keyboard dock attached)
Rails collapse into a single bottom taskbar (40 px): start, running apps with titles, then status
and clock — the familiar desktop layout for pointer use. Windows float (see architecture.md).

## Full-screen apps
Video players and games can hide the rails; a short swipe from either edge brings the
corresponding rail back for 3 s.

## Visual language
- Flat surfaces, no blur or shadows (GPU budget). One accent colour (leaf green).
- Dark and light themes from day one; follow the system preference.
- Motion: 150–200 ms ease-out slides for panels; nothing animates while idle.
