# WaDesk

WaDesk is a desktop client for WhatsApp Web built with Rust and Microsoft Edge WebView2 on Windows, with support for macOS and Linux.

It provides native desktop windowing, system tray integration, multi-account isolation, and a local Model Context Protocol (MCP) server for AI assistants, packaged in an executable of approximately 1.1 MB.

---

## Features

### Performance and Resource Usage
- **Compact footprint**: Compiles to a single binary under 2 MB, using the operating system's existing WebView runtime instead of bundling an independent Chromium installation.
- **EcoQoS and memory trimming**: Applies Windows EcoQoS efficiency mode and trims WebView memory allocations when minimized to the system tray.
- **Background audio controls**: Integrates with the Windows WASAPI audio mixer under the name "WhatsApp" and can automatically mute audio when the window is hidden.

### Privacy and Desktop Usability
- **Privacy blur modes**: Blur chat previews, contact names, or media until hovered. Includes an optional timer that blurs the window after 30 seconds of inactivity.
- **System tray integration**: Closes to the system tray, displays native balloon notifications for new incoming messages, and restores on single or double click.
- **External link guard**: Opens chat links in your default web browser instead of navigating the WhatsApp session.
- **Startup on boot**: Toggle Windows autostart directly from the system tray menu.

### Multi-Account Profiles
- **Isolated sessions**: Run multiple accounts simultaneously. Each profile stores cookies, local cache, and indexed databases in its own directory under `%APPDATA%\WaDesk\profiles\<id>`.
- **Fast switching**: Open the account manager using `Ctrl+Shift+A` to launch or switch between profiles.

### Non-Disruptive Updates
- **Background downloads**: Downloads installer packages in the background using temporary `.part` files without interrupting active calls or chats.
- **Install on Exit**: Schedule pending updates to install silently when you close the application, preventing sudden restarts while drafting messages.
- **Integrity verification**: Validates downloaded installers using SHA-256 checksums before execution.

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+Shift+P` | Toggle privacy blur mode |
| `Ctrl+Shift+A` | Open account switcher |
| `Ctrl+M` | Minimize directly to the system tray |
| `F5` or `Ctrl+R` | Refresh current WhatsApp Web session |
| `Ctrl+0` | Reset zoom level to 100% |
| `Ctrl+=` / `Ctrl+-` | Zoom in and zoom out |

---

## Model Context Protocol (MCP) Integration

WaDesk includes an embedded Model Context Protocol (MCP) loopback server listening on `http://127.0.0.1:5987/mcp`. AI coding assistants and desktop agents can interact with WhatsApp chats through structured tools.

### Authentication

On first launch, WaDesk generates a cryptographically secure random bearer token, encrypts it with Windows DPAPI, and writes the endpoint configuration to:

```text
%APPDATA%\WaDesk\mcp.json
```

File content structure:
```json
{
  "url": "http://127.0.0.1:5987/mcp",
  "token": "wadesk_sec_..."
}
```

### Client Configuration

#### Claude Desktop (`claude_desktop_config.json`)

Add the following entry to `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "wadesk": {
      "command": "curl",
      "args": [
        "-s",
        "-X", "POST",
        "http://127.0.0.1:5987/mcp",
        "-H", "Content-Type: application/json",
        "-H", "Authorization: Bearer <TOKEN_FROM_MCP_JSON>"
      ]
    }
  }
}
```

#### Antigravity, Cursor, and Other MCP Clients

Configure an HTTP POST connection:
- **Endpoint URL**: `http://127.0.0.1:5987/mcp`
- **Headers**:
  - `Content-Type: application/json`
  - `Authorization: Bearer <TOKEN_FROM_MCP_JSON>`

#### Terminal Verification (PowerShell)

```powershell
$config = Get-Content "$env:APPDATA\WaDesk\mcp.json" | ConvertFrom-Json
curl.exe -X POST $config.url `
  -H "Content-Type: application/json" `
  -H "Authorization: Bearer $($config.token)" `
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

### Available Tools

| Tool | Purpose | Parameters |
| :--- | :--- | :--- |
| `app_status` | Returns runtime state, active profile name, and connection status | None |
| `list_chats` | Returns recent conversations, unread badges, and last messages | None |
| `open_chat` | Navigates to a chat by contact name or phone number | `{"name": "Contact Name"}` |
| `read_messages` | Fetches recent messages from the currently active conversation | `{"limit": 50}` |
| `conversation_summary` | Retrieves conversation text formatted for model analysis | `{"limit": 50}` |
| `send_message` | Sends a plain text message to the currently opened chat | `{"text": "Message content"}` |
| `export_chat` | Exports message history from the active chat | `{"limit": 100}` |

---

## Data Storage

Session storage, authentication credentials, and local settings are stored locally on your device:

```text
%APPDATA%\WaDesk\UserData                  # Primary profile session and cache
%APPDATA%\WaDesk\profiles\<id>\UserData    # Additional profile data
%APPDATA%\WaDesk\accounts.json             # Profile list and display names
%APPDATA%\WaDesk\window.json               # Window dimensions and position
%APPDATA%\WaDesk\mcp_token                 # DPAPI-encrypted API token
```

WaDesk does not transmit messages, chat logs, or account tokens to external telemetry servers. Network requests connect directly to `web.whatsapp.com` and GitHub for release checks.

---

## Installation

### Windows
- **Installer**: Download `WaDesk-Setup-<version>-x86_64.exe` from the latest release to install Start Menu and desktop shortcuts.
- **Portable**: Download `WaDesk-<version>-Windows-x86_64.zip`, extract to any folder, and run `wadesk.exe`.

### macOS
Download `WaDesk-<version>-macOS-Universal.dmg` (supports both Apple Silicon and Intel systems):
1. Open the `.dmg` file and drag `WaDesk.app` into your `/Applications` directory.
2. Because WaDesk is distributed directly via GitHub without a paid Apple Developer certificate, macOS Gatekeeper blocks execution on first launch with a warning (*"Apple could not verify WaDesk is free of malware"*).
3. Choose one of the following methods to grant permission (only required once):
   - **Method 1 (System Settings)**: Click **Done** on the alert dialog. Open **System Settings > Privacy & Security**, scroll to the **Security** section, and click **Open Anyway**. Enter your Mac password or Touch ID when prompted.
   - **Method 2 (Control-Click in Finder)**: Open Finder, go to `/Applications`, hold the **Control** key, and click `WaDesk.app` (or right-click). Select **Open** from the context menu, then click **Open** in the dialog.
   - **Method 3 (Terminal)**: Open Terminal and remove the quarantine attribute:
     ```bash
     xattr -cr /Applications/WaDesk.app
     ```

### Linux
- **Debian / Ubuntu**:
  ```bash
  sudo dpkg -i wadesk_<version>_amd64.deb
  ```
- **Archive**: Extract `WaDesk-<version>-Linux-x86_64.tar.gz` and execute `./install.sh`.

---

## Building from Source

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (version 1.80 or newer)
- On Windows: Windows 10/11 with the Microsoft Edge WebView2 runtime installed
- On Linux: `libwebkit2gtk-4.1-dev` and standard build utilities

### Compile

```bash
# Run unit and integration tests
cargo test

# Compile an optimized release binary
cargo build --release
```

The resulting binary will be located at `target/release/wadesk.exe` (or `target/release/wadesk` on Unix).

---

## License

This project is licensed under either the MIT License or the Apache License 2.0.

WaDesk is an independent open-source project and is not affiliated with, sponsored by, or endorsed by WhatsApp LLC, Meta Platforms, Inc., or Microsoft Corporation.
