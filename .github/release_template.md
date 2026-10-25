# WaDesk {{VERSION}}

WaDesk is a desktop client for WhatsApp Web built with Rust and native platform webviews on Windows, Linux, and macOS.

## What is New in This Release

{{CHANGELOG}}

## Installation Guide

### Windows
- Installer: Download and run `WaDesk-Setup-{{CLEAN_VERSION}}-x86_64.exe` to install WaDesk with Start Menu and desktop shortcuts.
- Portable: Download `WaDesk-{{CLEAN_VERSION}}-Windows-x86_64.zip`, extract to any folder, and launch `wadesk.exe`.

### macOS
- DMG Image: Open `WaDesk-{{CLEAN_VERSION}}-macOS-Universal.dmg` and drag `WaDesk.app` to your `/Applications` directory.
- Application Archive: Download `WaDesk-{{CLEAN_VERSION}}-macOS-Universal.zip`, extract, and move `WaDesk.app` to `/Applications`.
- Supports macOS 11 (Big Sur) and newer on both Apple Silicon (M1-M4) and Intel systems.

> 💡 **First Time Opening on macOS (Apple Gatekeeper):**
> If macOS shows *"Apple could not verify WaDesk is free of malware"*:
> - **Option 1 (System Settings)**: Open **System Settings > Privacy & Security**, scroll down to the Security section, and click **Open Anyway**.
> - **Option 2 (Terminal - 1 Second)**: Open Terminal and run:
>   ```bash
>   xattr -cr /Applications/WaDesk.app
>   ```

### Linux
- Debian, Ubuntu, and Linux Mint:
  ```bash
  sudo dpkg -i {{DEB_FILENAME}}
  ```
- Other Linux distributions (Arch, Fedora, openSUSE):
  ```bash
  tar -xzf WaDesk-{{CLEAN_VERSION}}-Linux-x86_64.tar.gz
  cd WaDesk-{{CLEAN_VERSION}}-Linux-x86_64
  sudo ./install.sh
  ```

## Checksum Verification

Verify file integrity using SHA-256 before running:

```bash
sha256sum -c SHA256SUMS.txt
```
