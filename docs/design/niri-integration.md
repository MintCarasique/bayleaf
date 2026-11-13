# Design: multitasking on niri

How [multitasking.md](multitasking.md) maps onto niri ([ADR-0001](../decisions/0001-compositor-starting-point.md)):
what the shell does over niri's IPC, and what our fork changes. Action and event names are from
`niri-ipc` v26.04 (`niri msg action …`, `niri msg event-stream`).

## The fork

[MintCarasique/niri](https://github.com/MintCarasique/niri), branch `bayleaf`, on top of niri
v26.04; built and deployed by `scripts/deploy-niri.sh`. Driving the layout from outside over
IPC meant chains of actions, each with its own animation, racing resize animations and stale
events; so the rules live in the compositor, behind one layout option:

- **`layout { screen-split }`**: a workspace is a screen. One column fills it, two get half each,
  the view never scrolls. New tiled windows open on the empty workspace instead of next to the
  focused app. Switching workspaces is a crossfade (the new screen's windows use the window-open
  animation) instead of a vertical slide; windows moved between workspaces don't fly across.
- **`MoveWindowToWorkspaceColumn { window_id, reference, column }`** (IPC action): put a window
  on a workspace as column N and focus it, in one step. On a full pair it replaces the window on
  that half, which moves to the empty workspace, and the newcomer fades in at half width.

The shell sends the new action as raw JSON, so it keeps using the published `niri-ipc` crate.

## The mapping: one screen = one workspace

niri gives every output a vertical list of workspaces, each a horizontal strip of columns, and
always keeps one empty workspace at the end. Bayleaf uses it like this:

| Bayleaf | niri |
|---|---|
| Screen with one app | Workspace with one column at 100 % width |
| Screen with a pair | Workspace with two columns at 50 % each |
| Most-recently-used list of screens | Workspaces; focus order from `WindowFocusTimestampChanged` |
| New screen | The empty workspace at the end |

Because a workspace never holds more than the screen's width, niri's free horizontal scrolling
never comes into play, and a pair stays a pair when you switch away and back.

## Actions

| Bayleaf action ([multitasking.md](multitasking.md)) | How | Still to do |
|---|---|---|
| Tap an app in the rail | `FocusWindow { id }`: niri switches (crossfade) to its screen | — |
| App opens while another is shown | Fork: opens on the empty workspace by itself | — |
| Open on the left / right half (menu or drag) | Fork: `MoveWindowToWorkspaceColumn` with column 1 or 2 | — |
| Replace a half | Same action; the fork moves the replaced app out in the same step | — |
| Swap sides | `FocusWindow` + `MoveColumnToIndex` | — |
| Take an app out of a pair (menu *Full screen* or drag onto *Full screen*) | `MoveWindowToWorkspace` to the empty workspace + `FocusWindow` | — |
| Partner fills the screen after a close or move | Fork: `screen-split` widths | — |
| ✕ Close | `CloseWindow { id }` (from the rail menu for now) | Title strip with ← ☰ ✕ drawn by the compositor |
| Divider drag, release past 20 % | — | Divider handle and its gesture |
| Overview | `ToggleOverview` (niri's zoomed-out workspaces; pairs show as one row) | Cards in our style, horizontal |
| Switch animation | Fork: new screen fades in with the window-open animation | The old screen fading out underneath |
| Drop zones while dragging an icon | Shell: a full-screen layer surface over the work area | — |
| Keyboard only after touching a field | — (keyboard mode: off until the rail button or the launcher search turns it on, [ADR-0005](../decisions/0005-on-screen-keyboard.md)) | Enable the input method only after a touch on the focused surface |
| Edge swipes, dock-driven mode switch | — | Yes |

## Window list: foreign-toplevel or niri IPC

The rail uses `wlr-foreign-toplevel-management` today, which works on any wlroots-style
compositor but knows nothing about workspaces, and its handle ids are not niri window ids. For the
actions above the shell needs niri ids and workspace membership, so on niri it switches to the IPC
event stream (`WindowsChanged`, `WindowOpenedOrChanged`, `WindowClosed`, `WindowFocusChanged`,
`WorkspacesChanged`). foreign-toplevel stays as the fallback for other compositors.

## Order of work
1. ~~Shell: niri IPC client (event stream + actions), window list from it.~~
2. ~~Screens, split view, drop zones; `screen-split` and `MoveWindowToWorkspaceColumn` in the fork.~~
3. Fork: title strip with ← ☰ ✕.
4. Fork: input method only after a touch on the field (replaces keyboard mode).
5. Fork: full crossfade, divider, overview cards.
