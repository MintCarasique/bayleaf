# Design: multitasking on niri

How [multitasking.md](multitasking.md) maps onto niri ([ADR-0001](../decisions/0001-compositor-starting-point.md)):
what the shell does over niri's IPC today, and what needs the fork. Action and event names are
from `niri-ipc` v26.04 (`niri msg action …`, `niri msg event-stream`).

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

| Bayleaf action ([multitasking.md](multitasking.md)) | niri today | Needs the fork |
|---|---|---|
| Tap an app in the rail | `FocusWindow { id }`: niri switches to its workspace | — |
| App opens while another is shown | Shell sees `WindowOpenedOrChanged`, moves it with `MoveWindowToWorkspace` to the empty workspace and focuses it | A window rule "open on a new workspace" would avoid the visible jump |
| Open on the left / right half | `MoveWindowToWorkspace` into the shown workspace, `SetWindowWidth { id, 50% }` on both, `MoveColumnLeft/Right` or `MoveColumnToIndex` for the side | — |
| Replace a half | Move the replaced window to the empty workspace, then as above | — |
| Swap sides | `SwapWindowLeft/Right` or `MoveColumnLeft/Right` | — |
| ✕ Close | `CloseWindow { id }` (from the rail menu for now) | Title strip with ← ☰ ✕ drawn by the compositor |
| Partner fills the screen after a close | `SetWindowWidth { id, 100% }` on the remaining window | Automatic, without a resize flash |
| Divider drag, release past 20 % | — | Divider handle and its gesture |
| Overview | `ToggleOverview` (niri's zoomed-out workspaces; pairs show as one row) | Cards in our style, horizontal |
| Switch animation | niri's vertical workspace slide | Crossfade (scale 0.94 → 1, opacity) |
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
1. Shell: niri IPC client (event stream + actions), window list from it.
2. Shell: new apps open on their own screen; tap switches; close from the rail menu.
3. Shell: long-press menu *Open on the left / right*, drag with drop zones.
4. Fork: title strip, then crossfade, then the divider.
