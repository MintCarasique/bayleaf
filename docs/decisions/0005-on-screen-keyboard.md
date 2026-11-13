# ADR-0005: On-screen keyboard: squeekboard, not our own (for now)

- **Status:** accepted
- **Date:** 2026-11-13

## Context
Without a keyboard the tablet is half usable: no launcher search, no address bar, no terminal.
The architecture planned our own keyboard (`bayleaf-osk`, input-method-v2 + virtual-keyboard).
Writing one is a project of its own: layouts, shift and symbol views, key repeat, popovers for
accented letters, prediction later. Several maintained Wayland keyboards already exist.

## Options
| | squeekboard | wvkbd | Maliit |
|---|---|---|---|
| Origin | Phosh (Librem 5) | sxmo / postmarketOS | KDE, Plasma Mobile |
| Stack | Rust + GTK3 | C | Qt + QML |
| Shows itself on text focus | yes (input-method-v2 + text-input-v3) | no, signal to show/hide | only under KWin |
| Russian layout | yes | yes | yes |
| Debian trixie | yes (1.43) | yes (0.15) | yes |

## Decision
Use **squeekboard**, started by the session. It runs on labwc 0.8.3 on the reference tablet:
it appears when a text field is tapped in GTK4 (the launcher search), Firefox and Konsole, reserves
its height (an exclusive zone) so maximized windows shrink above it, and switches en ↔ ru.

- Dark theme via `GTK_THEME=Adwaita:dark`; layouts from `org.gnome.desktop.input-sources`;
  it only shows when `org.gnome.desktop.a11y.applications screen-keyboard-enabled` is true.
- The right rail has a keyboard button at the bottom that toggles it over D-Bus
  (`sm.puri.OSK0.SetVisible`), mainly to hide it. See the update below for how it works now.
- Shell surfaces never take focus into a text field on their own; the keyboard should appear only
  when the user taps a field.
- `bayleaf-osk` stays a placeholder. We write our own only if squeekboard hits a limit that
  matters (split layout for thumbs, gestures, memory); that would be a new ADR.

## Update 2026-11-13: keyboard mode
On niri, squeekboard showed itself every time an app with a text field got focus: on launch
(Firefox focuses its address bar), on every switch to a terminal, and in split view. Hiding it from
the shell right after it appeared caused a visible flash. squeekboard has no "only on touch"
setting, and with the a11y switch on it shows whenever the input method is activated, which
Smithay does as soon as the app enables text input.

Until the compositor enables the input method only after a touch on the field (our niri fork),
the keyboard runs in **keyboard mode**:
- off by default: the session starts squeekboard with `screen-keyboard-enabled` false, so it never
  shows by itself (starting it with the switch on also made it flash at login);
- the right-rail button turns the mode on (switch on + `SetVisible(true)`, retried for 2 s because
  dconf is slow here) and off (`SetVisible(false)`, switch off);
- the launcher search turns it on while the launcher is open, unless it was already on by hand.

## Consequences
- About **43 MB RSS** for the keyboard process on the reference tablet. It pulls GTK3 and
  feedbackd next to the shell's GTK4; acceptable within the 300 MB session budget, to be watched.
- Its look follows Adwaita, not the Bayleaf palette; restyling goes through squeekboard's own
  theming (`squeekboard-restyled` / user CSS) if needed.
- Our compositor (M2) must implement input-method-v2, text-input-v3 and virtual-keyboard, as
  labwc does today, so the keyboard keeps working after the switch.
