#!/usr/bin/env bash
# Build release binaries and copy them to a test device over SSH.
#
#   BAYLEAF_DEVICE=user@host scripts/deploy.sh [crate...]
#
# BAYLEAF_DEVICE may also be an alias from ~/.ssh/config. Binaries land in ~/.local/bin on the
# device. Never build on the device itself: low-end tablets don't have the RAM for it.
set -euo pipefail
cd "$(dirname "$0")/.."

device="${BAYLEAF_DEVICE:?set BAYLEAF_DEVICE=user@host (or an ssh config alias)}"
crates=("$@")
[[ ${#crates[@]} -gt 0 ]] || crates=(bayleaf-comp bayleaf-shell bayleaf-osk)

args=()
for c in "${crates[@]}"; do args+=(-p "$c"); done
cargo build --release "${args[@]}"

target="${CARGO_TARGET_DIR:-target}/release"
ssh "$device" 'mkdir -p ~/.local/bin'
for c in "${crates[@]}"; do
  scp -q "$target/$c" "$device:.local/bin/$c.new"
  ssh "$device" "mv -f ~/.local/bin/$c.new ~/.local/bin/$c"
  echo "deployed $c -> $device:~/.local/bin/$c"
done
