# Architecture

Status: draft. Decisions that are still open are marked **OPEN** and tracked in [decisions/](decisions/).

## Overview

```
┌──────────────────────── bayleaf-comp (one process) ─────────────────────────┐
│  Smithay: DRM/KMS + libinput + libseat, XWayland                             │
│  ├─ window model: tablet mode (one maximized window, tabs) / laptop mode     │
│  ├─ touch: edge-swipe gestures, drag/resize windows by finger                │
│  ├─ sensors: accelerometer → output transform + touch mapping                │
│  ├─ dock detection: input device hotplug → mode switch                       │
│  └─ protocols: xdg-shell, layer-shell, foreign-toplevel, input-method-v2,    │
│                virtual-keyboard, text-input-v3, idle, session-lock           │
└───────────────▲───────────────────────────────▲─────────────────────────────┘
                │ Wayland (layer-shell,          │ bayleaf-ipc
                │ foreign-toplevel)              │ (events: mode, rotation, gestures)
┌───────────────┴──────────────┐  ┌──────────────┴──────────────┐
│ bayleaf-shell                │  │ bayleaf-osk                  │
│ taskbar · start · action ctr │  │ on-screen keyboard           │
└──────────────────────────────┘  └─────────────────────────────┘
```

## Components

### bayleaf-comp — compositor
- Built on **Smithay**. Starting point **OPEN** (ADR-0001): fork of niri (GPL-3, mature backend and
  input handling) vs. building up from Smithay's `anvil` example.
- **Window model**
  - *Tablet mode*: every toplevel maximized (or two side by side), with a 40 px server-side title
    strip and a close button; switching via the left rail. Dialogs float centered.
    Details: [design/windows.md](design/windows.md).
  - *Laptop mode*: floating, overlapping windows with snap-to-half; drag/resize with pointer or finger.
  - Minimize is a first-class state (shown in the taskbar), unlike tiling compositors.
- **Touch**
  - Edge swipes recognized in the compositor (not by grabbing `/dev/input`): right → action
    center, left → window list, top → close, bottom → keyboard.
  - Gesture coordinates are in *output* space, so rotation never breaks them.
  - Long-press = secondary click for clients that don't handle touch.
- **Sensors**: accelerometer via iio-sensor-proxy (D-Bus); rotation lock; transform applied to
  the output and the touch mapping in one step.
- **Dock**: keyboard dock hotplug → laptop mode; detach → tablet mode. Configurable device match.

### bayleaf-shell — shell UI
- One process, several layer-shell surfaces laid out per [the side-rails design](design/rails.md)
  (ADR-0003): left rail (apps), right rail (system), launcher and quick-settings panels; a bottom
  bar in portrait and a taskbar in laptop mode.
- Toolkit **OPEN** (ADR-0002): iced (MIT, used by COSMIC) vs. slint.
- Talks to the compositor via standard protocols where they exist (foreign-toplevel for the
  taskbar) and `bayleaf-ipc` for Bayleaf-specific state (mode, rotation lock, gestures).
- Quick toggles via system D-Bus services: NetworkManager, BlueZ, UPower, PipeWire/WirePlumber.

### bayleaf-osk — on-screen keyboard
- input-method-v2 so it appears automatically on text focus in GTK/Qt/Firefox; falls back to
  virtual-keyboard for manual use.
- Layouts from data files (en-US, ru-RU first).

### bayleaf-ipc
- Small versioned protocol (serde types) over a Unix socket in `$XDG_RUNTIME_DIR`.
- Also used by a CLI (`bayleafctl`) for scripting and tests.

## Budgets (reference hardware)

| | Target |
|---|---|
| Idle RSS, whole session | < 300 MB |
| Compositor idle CPU | ~0% (no polling loops) |
| Shell start → taskbar visible | < 1 s |
| Edge swipe → action center visible | < 100 ms |

## Rendering
- GLES2 renderer (Bay Trail Gen7 GPU), damage tracking everywhere, no blur/shadows by default.
