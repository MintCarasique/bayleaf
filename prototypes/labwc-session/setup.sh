#!/usr/bin/env bash
# Install a bare labwc session ("Bayleaf prototypes") on the test device to try the rail
# prototypes without a desktop around them. Run on the device: sudo ./setup.sh <user>
set -euo pipefail
user="${1:?usage: setup.sh <user>}"
home=$(getent passwd "$user" | cut -d: -f6)
cfg="$home/.config/bayleaf-proto"

DEBIAN_FRONTEND=noninteractive apt-get install -y -qq labwc swaybg >/dev/null

install -d -o "$user" -g "$user" "$cfg"

cat > "$cfg/rc.xml" <<'EOF'
<?xml version="1.0"?>
<labwc_config>
  <keyboard>
    <keybind key="W-1"><action name="Execute" command="sh -c 'pkill -x rail-iced; pkill -x rail-gtk; GTK_A11Y=none ~/.local/bin/rail-gtk'"/></keybind>
    <keybind key="W-2"><action name="Execute" command="sh -c 'pkill -x rail-gtk; pkill -x rail-iced; ~/.local/bin/rail-iced'"/></keybind>
    <keybind key="W-q"><action name="Execute" command="sh -c 'pkill -x rail-gtk; pkill -x rail-iced'"/></keybind>
    <keybind key="W-Escape"><action name="Exit"/></keybind>
  </keyboard>
</labwc_config>
EOF

cat > "$cfg/autostart" <<'EOF'
swaybg -c '#2b4a5a' >/dev/null 2>&1 &
GTK_A11Y=none ~/.local/bin/rail-gtk >/dev/null 2>&1 &
EOF

chown "$user:$user" "$cfg/rc.xml" "$cfg/autostart"

cat > /usr/share/wayland-sessions/bayleaf-proto.desktop <<EOF
[Desktop Entry]
Name=Bayleaf prototypes
Comment=labwc with the Bayleaf rail prototypes (Win+1 GTK, Win+2 iced, Win+Q clear, Win+Esc exit)
Exec=labwc -C $cfg
Type=Application
DesktopNames=Bayleaf
EOF

echo "session installed: Bayleaf prototypes"
