#!/usr/bin/env bash
# ADR-0001 spike: run Smithay's anvil example compositor on the test device (DRM/KMS), with
# bayleaf-shell and squeekboard on top, as a separate display-manager session "Anvil (test)".
#
# Build anvil elsewhere (never on the device) and copy it to ~/.local/bin/anvil first:
#   git clone https://github.com/Smithay/smithay && cd smithay/anvil && cargo build --release
# Then on the device: sudo ./setup.sh <user>
#
# There is no exit gesture: leave the session with Ctrl+Alt+Backspace on the dock or
# `pkill -x anvil` over SSH. Logs go to $XDG_RUNTIME_DIR/{anvil,bayleaf-shell,squeekboard}.log.
set -euo pipefail
user="${1:?usage: setup.sh <user>}"
home=$(getent passwd "$user" | cut -d: -f6)
launcher="$home/.local/bin/bayleaf-anvil-session"

install -d -o "$user" -g "$user" "$home/.local/bin"
cat > "$launcher" <<'EOF'
#!/bin/sh
run="${XDG_RUNTIME_DIR:-/tmp}"
export XDG_CURRENT_DESKTOP=Bayleaf XDG_SESSION_TYPE=wayland
"$HOME/.local/bin/anvil" --tty-udev >"$run/anvil.log" 2>&1 &
comp=$!
sock=""
for _ in $(seq 100); do
  sock=$(grep 'Listening on wayland socket' "$run/anvil.log" | grep -o 'wayland-[0-9]*' | head -n1)
  [ -n "$sock" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY="$sock" QT_QPA_PLATFORM=wayland GDK_BACKEND=wayland
GTK_THEME=Adwaita:dark squeekboard >"$run/squeekboard.log" 2>&1 &
"$HOME/.local/bin/bayleaf-shell" >"$run/bayleaf-shell.log" 2>&1 &
wait "$comp"
EOF
chown "$user:$user" "$launcher"
chmod 755 "$launcher"

cat > /usr/share/wayland-sessions/bayleaf-anvil.desktop <<EOF
[Desktop Entry]
Name=Anvil (test)
Comment=ADR-0001 spike: Smithay anvil + bayleaf-shell (exit: Ctrl+Alt+Backspace)
Exec=$launcher
Type=Application
DesktopNames=Bayleaf
EOF

echo "session installed: Anvil (test)"
