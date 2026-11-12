# Design: window management

Part of the [side-rails](rails.md) design (ADR-0003). Sizes are logical pixels at scale 1.

## Tablet mode (no keyboard)

Every window fills the area between the rails (1168 × 800 in landscape).

### Title strip
A slim server-side strip on top of every window that accepts server-side decorations
(xdg-decoration). Everything that belongs to *this* window lives here
([ADR-0004](../decisions/0004-window-scoped-menus.md)):

```
┌──────────────────────────────────────────────────────────┐
│ [←]  [icon] App title                        [ ☰ ] [ ✕ ] │  40 px
├──────────────────────────────────────────────────────────┤
│                       application                        │
```

- Height **40 px**. Buttons have **56 × 40 px** hit areas, 20 px glyphs.
- **← Back**: sends the app's "back" (Alt+Left for browsers and file managers, Escape otherwise,
  per-app override in config). Hidden when the app has nothing to go back to, if it tells us.
- **☰ App menu**: opens the menu panel for this window (below).
- **✕ Close.**
- Minimise and maximise make no sense in tablet mode and are not shown.
- Dragging the strip downwards past 25 % of the screen height closes the window (same as the
  top-edge swipe); releasing earlier snaps it back.
- Clients that draw their own titlebar (GTK4/libadwaita CSD) keep it and their own menus; Bayleaf
  adds no second strip.

### App menu panel
Drops down from the ☰ button inside the window's own area (≈ 320 px wide, up to 70 % of the
window height), so in side-by-side mode each half opens its own menu.

- **Search field on top** across all menu items (HUD): typing "sav" offers "Save" and "Save as…".
  Opens with the on-screen keyboard only when the user taps the field.
- **Drill-down list** below: 48 px rows, submenus open in place with a breadcrumb
  ("Menu › File › Recent"); tap the breadcrumb to go up. No cascades.
- Disabled items are shown dimmed; check and radio items show their state.
- **Fallback** when the app exports no menu: window actions (side by side, full screen, close).

#### Where menus come from
| Clients | Source |
|---|---|
| Qt / KDE apps | DBusMenu + `org_kde_kwin_appmenu` Wayland protocol (implemented by `bayleaf-comp`) |
| LibreOffice | DBusMenu via its KDE/GTK integration |
| GTK3 apps | DBusMenu via `appmenu-gtk-module` (best effort) |
| Firefox | DBusMenu global menu support — to be verified on Wayland |
| GTK4 / libadwaita | none (they use their own hamburger menu in the CSD header) → fallback |

### Actions
| Action | How |
|---|---|
| Switch app | tap its icon in the left rail |
| Close | ✕ in the title strip; swipe down from the top edge; or "Close" in the rail menu |
| Side by side | drag an icon from the left rail onto the left or right half; drag the divider (6 px handle, 48 px grab zone) to resize. Max two windows. |
| App menu | ☰ in the title strip (see above) |
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
