#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run as root: sudo ./uninstall.sh"
  exit 1
fi

echo "Uninstalling WaDesk..."
rm -f /usr/local/bin/wadesk
rm -f /usr/share/applications/wadesk.desktop
rm -f /usr/share/icons/hicolor/scalable/apps/wadesk.svg

echo "✓ WaDesk uninstalled successfully."
