# WaDesk {{VERSION}}

WaDesk is a desktop client for WhatsApp Web built with Rust and native platform webviews on Windows, Linux, and macOS.

## What is New in This Release

{{CHANGELOG}}

## Installation Guide

### Windows
- Installer: Download and run `WaDesk-Setup-x86_64.exe` to install WaDesk with Start Menu and desktop shortcuts.
- Portable: Download `WaDesk-Windows-x86_64.zip`, extract to any folder, and launch `wadesk.exe`.

### macOS
- DMG Image: Open `WaDesk-macOS-Universal.dmg` and drag `WaDesk.app` to your `/Applications` directory.
- Application Archive: Download `WaDesk-macOS-Universal.zip`, extract, and move `WaDesk.app` to `/Applications`.
- Supports macOS 11 (Big Sur) and newer on both Apple Silicon (M1-M4) and Intel systems.

### Linux
- Debian, Ubuntu, and Linux Mint:
  ```bash
  sudo dpkg -i {{DEB_FILENAME}}
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
