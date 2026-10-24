#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run as root: sudo ./install.sh"
  exit 1
fi

echo "Installing WaDesk..."
install -Dm755 wadesk /usr/local/bin/wadesk
install -Dm644 wadesk.desktop /usr/share/applications/wadesk.desktop
if [ -f wadesk.svg ]; then
  install -Dm644 wadesk.svg /usr/share/icons/hicolor/scalable/apps/wadesk.svg
fi

echo "Updating desktop and icon cache..."
gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/share/applications 2>/dev/null || true

echo "✓ WaDesk installed successfully!"
echo "You can launch it from your application launcher or by running 'wadesk'."
