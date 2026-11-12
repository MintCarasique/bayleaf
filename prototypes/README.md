# Prototypes

Throwaway code for decisions, **not** part of the workspace or CI. Each prototype is its own crate.

## rail-gtk, rail-iced — ADR-0002 (shell toolkit)
The same minimal shell in two toolkits, to compare on the reference tablet:
- left rail (56 px, layer-shell, exclusive zone): start + 4 app buttons;
- right rail (56 px): clock, battery, vertical volume slider, keyboard and notification buttons;
- both rails in **one process**, like the real shell.

Each prints `startup_ms` (process start → first rail mapped/drawn) and `rss_kb` 3 s after start.

Build on Debian 13 (same as the tablet) and deploy:
```sh
cd prototypes/rail-gtk && cargo build --release
scp target/release/rail-gtk bayleaf-tablet:.local/bin/
```
Run inside a Wayland session that supports layer-shell (KDE Plasma, sway, labwc…).
