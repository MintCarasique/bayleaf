#!/usr/bin/env bash
# Build the compositor pieces Bayleaf runs on (ADR-0001) and copy them to a test device:
# niri and xwayland-satellite, at pinned versions. Neither is packaged in Debian trixie.
#
#   BAYLEAF_DEVICE=user@host scripts/deploy-niri.sh
#
# Sources and build directories live in BAYLEAF_DEPS_DIR (default ~/.cache/bayleaf-deps), on the
# Linux filesystem. Binaries are stripped and land in ~/.local/bin on the device.
# Extra build dependencies on top of CONTRIBUTING.md: libpango1.0-dev libxcb1-dev
# libxcb-cursor-dev.
set -euo pipefail

NIRI_VERSION=v26.04
XWS_VERSION=v0.8.3

device="${BAYLEAF_DEVICE:?set BAYLEAF_DEVICE=user@host (or an ssh config alias)}"
deps="${BAYLEAF_DEPS_DIR:-$HOME/.cache/bayleaf-deps}"
mkdir -p "$deps"

# fetch <url> <dir> <tag>: clone once, then check out the pinned tag.
fetch() {
  if [[ ! -d "$deps/$2" ]]; then
    git clone -q "$1" "$deps/$2"
  fi
  git -C "$deps/$2" fetch -q --tags
  git -C "$deps/$2" checkout -q "$3"
}

fetch https://github.com/YaLTeR/niri.git niri "$NIRI_VERSION"
fetch https://github.com/Supreeeme/xwayland-satellite.git xwayland-satellite "$XWS_VERSION"

# No screencasting (needs PipeWire and xdg-desktop-portal-gnome); D-Bus and systemd stay on.
CARGO_TARGET_DIR="$deps/target-niri" cargo build --release --locked \
  --manifest-path "$deps/niri/Cargo.toml" --no-default-features --features dbus,systemd
CARGO_TARGET_DIR="$deps/target-xws" cargo build --release --locked \
  --manifest-path "$deps/xwayland-satellite/Cargo.toml"

out="$deps/out"
mkdir -p "$out"
cp "$deps/target-niri/release/niri" "$deps/target-xws/release/xwayland-satellite" "$out/"
strip "$out/niri" "$out/xwayland-satellite"

ssh "$device" 'mkdir -p ~/.local/bin'
for bin in niri xwayland-satellite; do
  scp -q "$out/$bin" "$device:.local/bin/$bin.new"
  ssh "$device" "mv -f ~/.local/bin/$bin.new ~/.local/bin/$bin"
  echo "deployed $bin -> $device:~/.local/bin/$bin"
done
