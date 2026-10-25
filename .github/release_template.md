# WaDesk {{VERSION}}

WaDesk is a desktop client for WhatsApp Web built with Rust and native platform webviews on Windows, Linux, and macOS.

## Changes in this Release

- Universal macOS binary with native execution on Apple Silicon (M1, M2, M3, M4) and Intel processors.
- Windows installer via Inno Setup alongside portable zip distribution.
- Linux packaging with Debian package (.deb) and portable tarball with automated installer script.
- Audio silence automation when the application window is minimized to the system tray.
- Privacy features:
  - Toggle privacy blur on chat text with Ctrl+Shift+P.
  - Hover-to-reveal blur on photos and videos.
  - Inactivity blur after 30 seconds without window focus.
- System tray context menu to configure privacy preferences and audio behavior.
- Hardware-backed session storage using platform-native security APIs.

## Installation Guide

### Windows
- Installer: Download and run `WaDesk-Setup-x86_64.exe` to install WaDesk with Start Menu and desktop shortcuts.
- Portable: Download `WaDesk-Windows-x86_64.zip`, extract to any folder, and launch `wadesk.exe`.

### macOS
- DMG Image: Open `WaDesk-macOS-Universal.dmg` and drag `WaDesk.app` to your `/Applications` directory.
- Application Archive: Download `WaDesk-macOS-Universal.zip`, extract, and move `WaDesk.app` to `/Applications`.
- Supports macOS 11 (Big Sur) and newer on both Apple Silicon and Intel systems.

### Linux
- Debian, Ubuntu, and Linux Mint:
  ```bash
  sudo dpkg -i wadesk_1.0.0_amd64.deb
  ```
- Other Linux distributions (Arch, Fedora, openSUSE):
  ```bash
  tar -xzf WaDesk-Linux-x86_64.tar.gz
  cd WaDesk-Linux-x86_64
  sudo ./install.sh
  ```

## Checksum Verification

Verify file integrity using SHA-256 before running:

```bash
sha256sum -c SHA256SUMS.txt
```
