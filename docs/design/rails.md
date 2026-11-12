# Design: side rails

Accepted in [ADR-0003](../decisions/0003-ui-concept-rails.md), refined by
[ADR-0004](../decisions/0004-window-scoped-menus.md). Sizes are logical pixels at scale 1 on the
1280×800 reference screen.

## Principles
1. **Thumb reach.** Everything frequent lives within ~60 px of the left or right edge, in the
   vertical middle of the screen where thumbs rest when holding a tablet.
2. **One job per screen side.**
   - **Left rail — apps**: switch, launch, pin.
   - **Right rail — device**: status and controls for the tablet itself.
   - **Window title strip — the app**: back, app menu, close. Anything that belongs to a window
     lives on that window, so two windows side by side are never ambiguous.
3. **Height belongs to apps.** No horizontal bars in tablet landscape except each window's own
   title strip.
4. **Touch targets ≥ 48 px**, spacing ≥ 8 px. Icons 24 px.

## Tablet, landscape (primary)

```
┌──────┬────────────────────────────────────────────┬──────┐
│ [●]  │ ← ▣ App title                      ☰   ✕   │19:40 │
│  ○   ├────────────────────────────────────────────┤ ▮55% │
│  ○   │                                            │  ☀   │
│  ○   │                                            │  ┃   │
│      │              application                   │  ♪   │
│      │                                            │  ┃   │
│      │                                            │ ▶ ⏸  │
│      │                                            │  ⚙   │
│ ──   │                                            │  ⌨   │
│ ⊞    │                                            │  🔔  │
└──────┴────────────────────────────────────────────┴──────┘
  56 px                                               56 px
```

### Left rail — apps
| Element | Tap | Long press / drag |
|---|---|---|
| Running and pinned apps (top, scrollable) | switch to app | long press: side by side, full screen, pin, close; drag onto a screen half: side by side |
| Start (bottom) | open the launcher panel from the bottom-left corner | — |

The active app is marked with a filled background, not a dot — readable at a glance. In side-by-side
mode both visible apps are marked.

### Right rail — device ("remote control")
Top to bottom:

| Element | Interaction |
|---|---|
| Clock, battery, network | tap: quick settings panel |
| Brightness, volume (collapsed icons) | tap an icon: a vertical slider (~140 px) slides open right below it and pushes the rest of the rail down. **Only one slider is open at a time** — opening another closes the first. Tap the icon again to close; it also closes by itself 5 s after the last touch. |
| Media controls | shown only while something plays (MPRIS): play/pause, next; tap the cover for the full player in quick settings |
| Quick settings | tap: panel sliding from the right; also a swipe left from anywhere on the rail |
| Keyboard | toggle on-screen keyboard |
| Notifications | tap: notification list; badge with unread count |

**Quick settings panel** slides out from the right rail (≈ 360 px wide): toggles (Wi-Fi, Bluetooth,
rotation lock, tablet mode, night light, do not disturb) as a 2-column grid of 48 px buttons,
network and Bluetooth device lists, full media player, link to settings.

**Launcher panel** grows from the Start button in the bottom-left corner (≈ 420 px wide, up to the
full height): app grid (4 columns, 72 px tiles), pinned apps first, search field at the bottom next
to the thumb. The on-screen keyboard opens with the search field.

## Tablet, portrait (800 × 1280)
Rails merge into one bottom bar (64 px): start and running apps on the left half; status,
quick settings, keyboard and notifications on the right half. Sliders move into the quick settings
panel. Panels slide up from the bar instead of from the sides.

## Laptop mode (keyboard dock attached)
Rails collapse into a single bottom taskbar (40 px): start, running apps with titles, then status,
volume, quick settings and clock — the familiar desktop layout for pointer use. Windows float with
title bars (see [windows.md](windows.md)).

## Windows and app menus
The title strip (back, app menu, close) and window behaviour in both modes are specified in
[windows.md](windows.md).

## Full-screen apps
Video players and games can hide the rails and the title strip; a short swipe from either edge
brings the corresponding rail back for 3 s, a swipe from the top brings the strip back.

## Visual language
- Flat surfaces, no blur or shadows (GPU budget). One accent colour (leaf green).
- Dark and light themes from day one; follow the system preference.
- Motion: 150–200 ms ease-out slides for panels; nothing animates while idle.
