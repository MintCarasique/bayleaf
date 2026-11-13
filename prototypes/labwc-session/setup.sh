#!/usr/bin/env bash
# Superseded by the niri-based session in session/ (ADR-0001); kept as a fallback.
#
# Install a bare labwc session ("Bayleaf on labwc (old)") on the test device to run
# bayleaf-shell and the toolkit prototypes without a desktop around them. Run on the device: sudo ./setup.sh <user>
#
# On-screen keyboard: squeekboard (ADR-0005). It shows itself when a text field gets focus
# (input-method-v2 + text-input-v3) and reserves its height, so maximized windows shrink above it.
#
# Keys: Win+0 bayleaf-shell, Win+1 GTK prototype, Win+2 iced prototype, Win+Q stop all,
#       Win+Enter Konsole, Win+B Firefox, Win+E Dolphin, Win+Tab next window, Win+Esc exit.
set -euo pipefail
user="${1:?usage: setup.sh <user>}"
home=$(getent passwd "$user" | cut -d: -f6)
cfg="$home/.config/bayleaf-proto"

DEBIAN_FRONTEND=noninteractive apt-get install -y -qq labwc swaybg squeekboard >/dev/null

install -d -o "$user" -g "$user" "$cfg"

stop='pkill -x bayleaf-shell; pkill -x rail-gtk; pkill -x rail-iced'
cat > "$cfg/rc.xml" <<EOF
<?xml version="1.0"?>
<labwc_config>
  <keyboard>
    <keybind key="W-0"><action name="Execute" command="sh -c '$stop; ~/.local/bin/bayleaf-shell >\$XDG_RUNTIME_DIR/bayleaf-shell.log 2>&amp;1'"/></keybind>
    <keybind key="W-1"><action name="Execute" command="sh -c '$stop; GTK_A11Y=none ~/.local/bin/rail-gtk'"/></keybind>
    <keybind key="W-2"><action name="Execute" command="sh -c '$stop; ~/.local/bin/rail-iced'"/></keybind>
    <keybind key="W-q"><action name="Execute" command="sh -c '$stop'"/></keybind>
    <keybind key="W-Return"><action name="Execute" command="konsole"/></keybind>
    <keybind key="W-b"><action name="Execute" command="firefox-esr"/></keybind>
    <keybind key="W-e"><action name="Execute" command="dolphin"/></keybind>
    <keybind key="W-Tab"><action name="NextWindow"/></keybind>
    <keybind key="W-Escape"><action name="Exit"/></keybind>
  </keyboard>
  <!-- Stand-in for Bayleaf tablet mode until bayleaf-comp exists (M2): every window maximized
       between the rails. -->
  <windowRules>
    <windowRule identifier="*"><action name="Maximize"/></windowRule>
  </windowRules>
</labwc_config>
EOF

cat > "$cfg/autostart" <<'EOF'
swaybg -c '#2b4a5a' >/dev/null 2>&1 &
# squeekboard stays hidden unless the GNOME a11y switch is on; layouts come from input-sources.
gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled true
gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'us'), ('xkb', 'ru')]"
GTK_THEME=Adwaita:dark squeekboard >"${XDG_RUNTIME_DIR:-/tmp}/squeekboard.log" 2>&1 &
~/.local/bin/bayleaf-shell >"${XDG_RUNTIME_DIR:-/tmp}/bayleaf-shell.log" 2>&1 &
EOF

chown "$user:$user" "$cfg/rc.xml" "$cfg/autostart"

rm -f /usr/share/wayland-sessions/bayleaf-proto.desktop /usr/share/wayland-sessions/bayleaf-dev.desktop
cat > /usr/share/wayland-sessions/bayleaf-labwc.desktop <<EOF
[Desktop Entry]
Name=Bayleaf on labwc (old)
Comment=labwc + bayleaf-shell for development (Win+0 shell, Win+1/2 prototypes, Win+Esc exit)
Exec=labwc -C $cfg
Type=Application
DesktopNames=Bayleaf
EOF

echo "session installed: Bayleaf on labwc (old)"
