# Design: multitasking in tablet mode

Part of the [side-rails](rails.md) design; window chrome is in [windows.md](windows.md). Sizes are
logical pixels at scale 1 on the 1280×800 reference screen. Laptop mode keeps floating windows and
is not covered here.

The model follows tablet split view: one app on screen by default, a second one only when asked
for, always split in half.

## Model

The compositor keeps an ordered list of **screens**, most recently used first. A screen is either

- **one app**, filling the area between the rails (1168 × 800), or
- **a pair**, two apps side by side, **exactly 50/50**. There is no ⅓ / ⅔ split and no free
  resizing: on a 1168 px work area anything narrower than half is unusable for desktop apps.

The first screen in the list is the one shown. An app belongs to at most one screen. Pairs are kept
as pairs: switching away and back returns both apps in the same order.

Closing is explicit: **an app only closes through ✕** (in its title strip or on its card in the
overview). Every other action that removes an app from view — replacing it, dragging the divider,
switching — sends it to the background, where it stays in the rail and the overview.

## Actions

| Action | Result |
|---|---|
| Tap an app in the left rail | Show the screen that app is on (its pair, if it is in one). Not running → launch it on a screen of its own. Already shown → small bounce of the icon, nothing else. |
| Drag an app from the rail onto the work area | Drop zones appear (below). Drop on a half → the app opens there and the shown app takes the other half. |
| Long press an app in the rail (450 ms) | Menu: *Full screen*, *Open on the left*, *Open on the right* (the last two only when another app is shown). Moving the finger more than 8 px after the menu opened turns it into a drag. |
| Drop on a half of an existing pair | The app replaces the one on that half; the replaced app moves to the background as a screen of its own. |
| Drop an app of the pair on its other half | The two apps swap sides. |
| Drag the divider | The windows follow the finger. Released beyond 20 % from an edge → the app on that side goes to the background and the other one fills the screen. Released anywhere else → springs back to 50/50. While past the threshold, the leaving app is dimmed to show what will happen. |
| ✕ in a title strip | Closes the app. In a pair the other app fills the screen; alone, the next screen in the list is shown. |
| Overview button in the left rail | Opens the overview (below). |

### Drop zones
While an icon is dragged over the work area:
- something else is shown → two zones, **left half** and **right half**, each with a dashed outline
  and a label; the zone under the finger is filled with the accent colour at 35 %;
- nothing shown, or the dragged app is the one shown → one zone, *Full screen*.

Zones are deliberately not a live preview: the windows do not move until the drop. Dropping outside
the work area cancels.

### Overview
A button in the left rail between the apps and Start. It is the only way in (no edge gesture), so it
never fights with app gestures.

- Cards in a horizontal row, most recent first; a pair is one card showing both apps.
- Tap a card → that screen. Tap ✕ under a card → close its apps. Tap the empty area or the
  button again → back to the shown screen.

## Motion

All window transitions use one curve, `cubic-bezier(0.2, 0.85, 0.25, 1)` (fast start, soft
landing), and one duration, **250 ms**. Nothing animates while idle.

| Transition | Incoming | Outgoing / others |
|---|---|---|
| Switch screens (tap in the rail, overview) — **crossfade** | scale 0.94 → 1, opacity 0 → 1, on top | scale 1 → 1.03, opacity → 0 |
| Open on a half | slides in from its edge of the screen (translate ∓105 % → 0) | the shown app resizes to the other half in the same motion |
| Replace a half | as above | the replaced app moves 40 % towards its edge and fades out |
| Swap sides | — | both windows move to their new halves |
| Divider released past the threshold | the remaining app grows to full width | the leaving app slides out through its edge |
| Divider released elsewhere | — | both windows spring back to 50/50 |
| ✕ Close | — | scale → 0.9, opacity → 0; the partner grows to full width |
| Open overview | scrim fades in; cards rise 14 px and scale 1.08 → 1, 35 ms apart | the shown windows scale to 0.92 under the scrim |
| Drop zones | fade in/out, 150 ms | — |
| Long-press menu | scale 0.92 → 1, opacity 0 → 1, 140 ms | — |

The interactive mockup used to tune this lives in the design discussion. If it turns out too slow
on the reference GPU, cut the duration first and keep the curve.

## Edge cases
- **Apps that do not fit in a half** (minimum width above 581 px): the drop is refused with a short
  shake of the zone and a note "Doesn't fit side by side"; the app opens full screen instead.
- **Dialogs** of an app in a pair are centred over that half, not over the whole screen.
- **Portrait** (800 × 1280): a pair stacks top and bottom, 50/50, with a horizontal divider.
  Rotating keeps pairs.
- **Docking** the keyboard turns a pair into two windows snapped to the left and right halves;
  undocking restores the pair (see [windows.md](windows.md)).
- **Full-screen apps** (video, games) cannot be in a pair; dropping one opens it full screen.

## Ownership
Window placement is the compositor's job: a layer-shell client cannot position other windows, and
labwc offers no external control over window geometry. So:

- `bayleaf-comp` owns the screen list, the layout, the divider and all window animations;
- `bayleaf-shell` owns the rail, the long-press menu, the drag ghost and the overview UI, and asks
  the compositor to open an app on a half or to show a screen (Bayleaf IPC, `bayleaf-ipc`).

On labwc (M1) only the shell side can be built; split view itself arrives with the compositor (M2).
This model is the main requirement for [ADR-0001](../decisions/0001-compositor-starting-point.md).
