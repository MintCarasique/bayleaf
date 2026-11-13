#!/usr/bin/env bash
# ADR-0001 spike: run niri on the test device (DRM/KMS) with bayleaf-shell and squeekboard on top,
# as a separate display-manager session "niri (test)". Compares a well-built Smithay compositor
# with labwc on the same hardware.
#
# niri is not packaged in Debian trixie. Build it elsewhere (never on the device):
#   git clone https://github.com/YaLTeR/niri && cd niri && git checkout v26.04
#   cargo build --release --no-default-features --features dbus,systemd
# copy target/release/niri to ~/.local/bin/niri, then on the device: sudo ./setup.sh <user>
#
# Exit: Ctrl+Alt+Backspace on the dock, or `pkill -x niri` over SSH.
# Logs: $XDG_RUNTIME_DIR/niri.log.
set -euo pipefail
user="${1:?usage: setup.sh <user>}"
home=$(getent passwd "$user" | cut -d: -f6)
cfg="$home/.config/bayleaf-niri"
launcher="$home/.local/bin/bayleaf-niri-session"

install -d -o "$user" -g "$user" "$cfg" "$home/.local/bin"

cat > "$cfg/config.kdl" <<EOF
// Close to the Bayleaf tablet model: one window fills the area between the rails.
input {
    keyboard {
        xkb {
            layout "us,ru"
            options "grp:alt_shift_toggle"
        }
    }
}

layout {
    gaps 0
    center-focused-column "never"
    default-column-width { proportion 1.0; }
    focus-ring { off; }
    border { off; }
}

prefer-no-csd
hotkey-overlay { skip-at-startup; }
screenshot-path null

environment {
    QT_QPA_PLATFORM "wayland"
    GTK_THEME "Adwaita:dark"
}

spawn-at-startup "squeekboard"
spawn-at-startup "sh" "-c" "$home/.local/bin/bayleaf-shell >\$XDG_RUNTIME_DIR/bayleaf-shell.log 2>&1"

binds {
    Ctrl+Alt+BackSpace { quit skip-confirmation=true; }
    Mod+Return { spawn "konsole"; }
    Mod+B { spawn "firefox-esr"; }
    Mod+E { spawn "dolphin"; }
    Mod+Q { close-window; }
    Mod+Tab { focus-column-right-or-first; }
}
EOF
chown "$user:$user" "$cfg/config.kdl"

cat > "$launcher" <<EOF
#!/bin/sh
exec "\$HOME/.local/bin/niri" -c "$cfg/config.kdl" >"\${XDG_RUNTIME_DIR:-/tmp}/niri.log" 2>&1
EOF
chown "$user:$user" "$launcher"
chmod 755 "$launcher"

cat > /usr/share/wayland-sessions/bayleaf-niri.desktop <<EOF
[Desktop Entry]
Name=niri (test)
Comment=ADR-0001 spike: niri + bayleaf-shell (exit: Ctrl+Alt+Backspace)
Exec=$launcher
Type=Application
DesktopNames=Bayleaf
EOF

echo "session installed: niri (test)"
