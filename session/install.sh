#!/usr/bin/env bash
# Install the "Bayleaf" display-manager session on a device. Run on the device from a checkout
# (or a copy of this directory): sudo session/install.sh <user>
#
# Needs niri, xwayland-satellite and bayleaf-shell in ~<user>/.local/bin: build them elsewhere
# with scripts/deploy-niri.sh and scripts/deploy.sh. Never build on the device.
set -euo pipefail
user="${1:?usage: install.sh <user>}"
here="$(cd "$(dirname "$0")" && pwd)"
home=$(getent passwd "$user" | cut -d: -f6)

DEBIAN_FRONTEND=noninteractive apt-get install -y -qq squeekboard >/dev/null

install -d -o "$user" -g "$user" "$home/.config/bayleaf" "$home/.local/bin" "$home/Pictures/Screenshots"
install -m 644 -o "$user" -g "$user" "$here/niri.kdl" "$home/.config/bayleaf/niri.kdl"
install -m 755 -o "$user" -g "$user" "$here/bayleaf-session" "$here/bayleaf-brightness" "$home/.local/bin/"

cat > /usr/share/wayland-sessions/bayleaf.desktop <<DESKTOP
[Desktop Entry]
Name=Bayleaf
Comment=Touch-first Wayland shell on niri (exit: Ctrl+Alt+Backspace)
Exec=$home/.local/bin/bayleaf-session
Type=Application
DesktopNames=Bayleaf
DESKTOP

# Portals for XDG_CURRENT_DESKTOP=Bayleaf: without this file xdg-desktop-portal falls back to
# the GTK backend and apps wait for its timeouts. KDE first (Qt dialogs, the apps we ship with),
# GTK as fallback.
cat > /usr/share/xdg-desktop-portal/bayleaf-portals.conf <<PORTALS
[preferred]
default=kde;gtk
org.freedesktop.impl.portal.Secret=kwallet
PORTALS

# The niri spike session is superseded by this one.
rm -f /usr/share/wayland-sessions/bayleaf-niri.desktop

for bin in niri xwayland-satellite bayleaf-shell; do
  [[ -x "$home/.local/bin/$bin" ]] || echo "warning: $home/.local/bin/$bin is missing"
done
echo "session installed: Bayleaf"
