# Contributing

Bayleaf is in its design phase. Issues with ideas, hardware reports (especially low-end
tablets and 2-in-1s) and design feedback are the most useful contributions right now.

## Building
Linux or WSL2 with WSLg, Rust stable (see `rust-version` in `Cargo.toml`):

```sh
cargo build --workspace
cargo test --workspace
```

Before sending a change run `scripts/check.sh` (fmt, clippy `-D warnings`, tests — same as CI).

### System libraries (Debian/Ubuntu)
```sh
sudo apt install build-essential clang pkg-config libwayland-dev libxkbcommon-dev libudev-dev \
  libinput-dev libgbm-dev libseat-dev libdrm-dev libegl-dev libgles-dev libpixman-1-dev \
  libdisplay-info-dev libfontconfig-dev libdbus-1-dev libpipewire-0.3-dev libgtk-4-dev \
  libgtk4-layer-shell-dev libpango1.0-dev libxcb1-dev libxcb-cursor-dev
```

### Windows: WSL2
Ubuntu 24.04 in WSL2 with WSLg works: Wayland clients and the nested compositor show up as
Windows windows. Keep the sources wherever you like, but put the build directory on the Linux
filesystem — building through `/mnt/c` or `/mnt/d` is slow:

```sh
export CARGO_TARGET_DIR=$HOME/.cache/bayleaf-target
```

### Testing on a device
Touch, rotation and DRM need real hardware. Build on your machine and copy the binaries over SSH
(never build on a 2 GB tablet):

```sh
BAYLEAF_DEVICE=user@tablet scripts/deploy.sh            # all binaries
BAYLEAF_DEVICE=user@tablet scripts/deploy.sh bayleaf-shell
```

The compositor (niri, ADR-0001) and xwayland-satellite are not packaged in Debian; build and copy
them at the pinned versions with:

```sh
BAYLEAF_DEVICE=user@tablet scripts/deploy-niri.sh
```

Then install the "Bayleaf" display-manager session on the device (it needs the files from
`session/` there):

```sh
sudo session/install.sh <user>
```

Inside the session: Ctrl+Alt+Backspace quits, Super+0 restarts the shell, logs are in
`$XDG_RUNTIME_DIR/{niri,bayleaf-shell,squeekboard}.log`.

Binaries go to `~/.local/bin` on the device. Binaries built on Ubuntu 24.04 (glibc 2.39) run on
Debian 13 (glibc 2.41); building on a newer distro than the device may not.

## Rules
See [CLAUDE.md](CLAUDE.md) — it applies to humans too: small focused changes, English everywhere,
GPL-3-compatible dependencies only, mind the memory budget.

By contributing you agree your contribution is licensed under GPL-3.0-or-later.
