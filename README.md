<p align="center">
  <img src="assets/whatsapp.png" alt="WaDesk Logo" width="128" height="128">
</p>

<h2 align="center">WaDesk - Lightweight Native WhatsApp Client for Windows</h2>

<p align="center">
  Built in 100% Native Rust with WebView2. Zero Electron. Zero bloat. Ultra-secure.
</p>

---

## 🚀 Overview

**WaDesk** is a senior-engineered, lightweight native Windows desktop wrapper for WhatsApp Web built with **Rust** and Microsoft Edge **WebView2**.

Unlike typical Electron-based desktop wrappers that consume 200MB+ disk space and bundle full Chromium copies, **WaDesk** compiles down to a single **~1.1 MB** executable with native Win32 windowing, DWM dark mode, system tray integration, and hardware-backed DPAPI security.

---

## ✨ Features & Highlights

| Feature | Details |
| :--- | :--- |
| **Ultra-Lightweight** | Single standalone binary **~1.1 MB** with zero GC overhead |
| **Official Visuals** | Native official WhatsApp green branding on window frame, taskbar, and tray |
| **Multiple Accounts** | Independent sessions (`%APPDATA%\WaDesk\profiles\<id>`), each with its own storage, session, and tray icon |
| **System Tray & Context Menu** | Minimize to tray, left-click/double-click restore, and right-click popup context menu |
| **Notification Balloons** | Intercepts web notifications and renders native Windows System Tray balloons |
| **Desktop Hotkeys** | `Ctrl+Shift+P` (Privacy Blur), `Ctrl+Shift+A` (Account Switcher), `Ctrl+M` (Minimize) |
| **Navigation Guard** | External links in chats are automatically routed to your default Windows web browser |
| **DWM Dark Mode** | Native Windows immersive dark title bar and Explorer frame theme |
| **EcoQoS & Memory Trimming** | Automatically throttles CPU power and trims browser memory level when minimized to tray |
| **Audio Mixer Integration** | Renames the Windows Core Audio WASAPI session to "WhatsApp" in Windows Volume Mixer |
| **Model Context Protocol (MCP)** | Built-in loopback MCP server (`2025-06-18`) & REST `/api/send` for AI agent interaction |
| **Security Hardening** | CSPRNG token generation (`OsRng`), constant-time bearer verification (`subtle`), and Windows **DPAPI encryption** |
| **Auto-Detect WebView2** | Detects runtime availability with an automatic prompt to install Microsoft Evergreen Bootstrapper if missing |

---

## 🛠️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd> | Toggle **Privacy Blur Mode** (blur chats & contact names in public) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>A</kbd> | Open **Account Switcher** modal |
| <kbd>Ctrl</kbd> + <kbd>M</kbd> | Minimize directly to System Tray |

---

## 🤖 MCP / AI Agent Access

WaDesk dilengkapi dengan server bawaan **Model Context Protocol (MCP)** (protokol standar komunikasi AI Agent ke tools desktop). Setiap profil WaDesk yang berjalan otomatis membuka endpoint loopback lokal yang aman di `http://127.0.0.1:5987/mcp` (atau fallback port jika 5987 terpakai).

### 🔑 Autentikasi & Lokasi Token
URL endpoint dan token otentikasi disimpan secara otomatis di:
```text
%APPDATA%\WaDesk\mcp.json     # Berisi endpoint URL dan bearer token
%APPDATA%\WaDesk\mcp_token    # Token aman terenkripsi Windows DPAPI
```

Isi file `%APPDATA%\WaDesk\mcp.json`:
```json
{
  "url": "http://127.0.0.1:5987/mcp",
  "token": "wadesk_sec_xxxxxxxxxxxxxxxx"
}
```

---

### 📖 Panduan Menghubungkan ke AI Tools & IDE

#### 1. Claude for Desktop (`claude_desktop_config.json`)
Tambahkan konfigurasi berikut ke file konfigurasi Claude Desktop Anda (`%APPDATA%\Claude\claude_desktop_config.json`):
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
        "-H", "Authorization: Bearer <TOKEN_DARI_MCP_JSON>"
      ]
    }
  }
}
```

#### 2. Antigravity IDE / Cursor / Roo Code / Cline
Gunakan konfigurasi server MCP berbasis HTTP/SSE atau loopback client dengan bearer token:
- **Server URL:** `http://127.0.0.1:5987/mcp`
- **Method:** `POST`
- **Headers:**
  - `Content-Type: application/json`
  - `Authorization: Bearer <TOKEN_DARI_MCP_JSON>`

#### 3. Tes Langsung via Terminal (PowerShell / cURL)
Anda dapat mencoba memeriksa daftar tools yang tersedia secara langsung:
```powershell
# Ambil token dari mcp.json
$mcp = Get-Content "$env:APPDATA\WaDesk\mcp.json" | ConvertFrom-Json

# Request tools list via JSON-RPC 2.0
curl -X POST $mcp.url `
  -H "Content-Type: application/json" `
  -H "Authorization: Bearer $($mcp.token)" `
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

---

### 🛠️ Daftar MCP Tools yang Tersedia untuk AI Agent

| Tool | Deskripsi | Parameter |
| :--- | :--- | :--- |
| `app_status` | Memeriksa status aplikasi, nama akun profil aktif, dan koneksi WhatsApp | `{}` |
| `list_chats` | Menampilkan daftar chat terbaru, nama kontak, pesan terakhir, dan unread badge | `{}` |
| `open_chat` | Membuka percakapan berdasarkan nama kontak / nomor obrolan | `{"name": "Nama Kontak"}` |
| `read_messages` | Membaca pesan-pesan terbaru dari chat yang sedang dibuka | `{"limit": 50}` (default: 50) |
| `conversation_summary` | Mengambil data rangkuman obrolan aktif untuk keperluan analisis agent | `{"limit": 50}` |
| `send_message` | Mengirim pesan teks ke percakapan yang sedang aktif | `{"text": "Pesan yang ingin dikirim"}` |
| `export_chat` | Mengekspor riwayat percakapan yang sedang dibuka | `{"limit": 100}` |

---

## 📂 Data Storage

All profile cookies, session tokens, and local cache are stored strictly locally:
```text
%APPDATA%\WaDesk\UserData                  # Primary account session
%APPDATA%\WaDesk\profiles\<id>\UserData    # Secondary account sessions
%APPDATA%\WaDesk\accounts.json             # Account manifest
%APPDATA%\WaDesk\window.json               # Window dimensions & positions memory
```

---

## 📦 Building from Source

### Prerequisites:
- [Rust](https://rustup.rs/) (1.80+ recommended)
- Windows 10/11 with WebView2 Runtime

### Build Release:
```powershell
# Run unit & integration tests
cargo test

# Build optimized release binary
cargo build --release
```
The output binary will be created at `target\release\wadesk.exe`.
 
---

## 🍎 macOS Installation & Gatekeeper Note

Untuk aplikasi open-source independen yang diunduh langsung dari GitHub (tanpa sertifikat berbayar Apple Developer Program), macOS Gatekeeper akan menampilkan peringatan keamanan saat pertama kali dibuka:
> *"Apple could not verify WaDesk is free of malware..."*

**Cara Mengizinkan WaDesk Berjalan (Hanya 1x saat pertama install):**

- **Cara 1 (Melalui System Settings):**
  1. Klik **Done** pada kotak dialog peringatan.
  2. Buka **System Settings** ➔ **Privacy & Security**.
  3. Gulir ke bawah ke bagian **Security**, Anda akan melihat teks *"WaDesk was blocked from use..."*.
  4. Klik tombol **Open Anyway**, lalu masukkan password/Touch ID Mac Anda.

- **Cara 2 (Super Cepat via Terminal):**
  Buka aplikasi **Terminal** di Mac Anda, lalu salin dan jalankan perintah berikut:
  ```bash
  xattr -cr /Applications/WaDesk.app
  ```
  *(Atau jika file masih di folder Downloads: `xattr -cr ~/Downloads/WaDesk.app`)*

---

## 📄 License

MIT or Apache-2.0. Not affiliated with or endorsed by WhatsApp LLC or Meta Platforms, Inc.
