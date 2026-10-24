# WaDesk Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Membangun **WaDesk**, client desktop Windows native untuk WhatsApp Web berbasis Rust dan WebView2, setara fitur dengan WaGramDeskLite (multi-akun terisolasi, system tray, dark frame, MCP server JSON-RPC, REST send API, webhooks unread watcher, audio session labeling, memory trimming), namun ditingkatkan dengan arsitektur Rust senior pattern dan security hardening.

**Architecture:** Arsitektur modular Rust: Main thread mengelola event loop Win32 native (Window subclassing, Shell Tray, DWM, WebView2 controller). State dikelola dalam `Arc<AppState>` yang thread-safe. Background Tokio runtime menangani HTTP loopback server (MCP JSON-RPC 2.0 & REST `/api/send`) dan Webhook watcher. Komunikasi antara server dan WebView2 menggunakan message passing channel yang dieksekusi di UI thread secara aman tanpa string injection.

**Tech Stack:** 
- Language: Rust 2024 / 1.85+
- Windows API: `windows` crate (Win32, COM, WASAPI, DWM, Shell, Registry)
- WebView2: `webview2-com` crate (official COM bindings untuk Microsoft Edge WebView2)
- Async HTTP / Server: `tokio`, `axum` (atau `hyper`), `tower-http` (loopback only)
- Serialization & Types: `serde`, `serde_json`
- Security & Crypto: `rand` (OsRng kernel entropy), `subtle` (constant-time token comparison), `zeroize`
- Resource compilation: `winres` (manifest DPI awareness, version info, icons)

**Spec:** Dokumen analisis reverse-engineering `scratch_repo` dan spesifikasi fitur asli di `docs/MCP.md` serta `README.md`.

## Global Constraints

- OS Target: Windows 10/11 x86_64 (`x86_64-pc-windows-msvc`).
- No console window on release: `#![windows_subsystem = "windows"]`.
- Executable size: Target biner release teroptimasi (< 3 MB).
- Security: Tidak ada fallback ke timestamp yang dapat ditebak; token auth timing-safe; HTTP server terikat eksklusif ke loopback `127.0.0.1`.
- Clean Code & Anti-Slop: Tidak ada mutable static globals; zero dead code; penanganan error eksplisit (`Result` idiomatic).

## Review Focus

1. **WebView2 Runtime Missing:** Deteksi registry WebView2 harus menampilkan dialog ramah pengguna (`MessageBoxW`) dan exit code 1 jika runtime tidak ditemukan.
2. **Single-Instance Mutex Collision:** Peluncuran kedua dengan profil yang sama harus mengaktifkan jendela yang sudah ada via `SetForegroundWindow` dan keluar tanpa error (exit code 0).
3. **Loopback Port Conflict:** Jika port default `5987` terpakai oleh akun lain atau aplikasi lain, fallback otomatis ke ephemeral port (`127.0.0.1:0`) dan simpan port aktual ke `mcp.json`.
4. **Token Authentication:** Request ke `/mcp` atau `/api/send` tanpa Bearer token yang valid harus segera ditolak dengan HTTP 401 secara constant-time.
5. **Thread Safety IPC:** Dispatch dari async task ke WebView2 harus melewati Windows message queue (`PostMessageW` dengan custom `WM_APP`) untuk menjamin COM WebView2 hanya dipanggil dari thread pembuatnya.

---

### Task 1: Project Scaffolding & Configuration Architecture

**Files:**
- Create: `Cargo.toml`
- Create: `build.rs`
- Create: `src/config.rs`
- Create: `src/error.rs`
- Test: `tests/config_test.rs`

**Interfaces:**
- Produces: `Config` struct, `AppPaths`, `Account`, `AccountsManifest`, dan fungsi resolver direktori `%APPDATA%\WaDesk`.

- [ ] **Step 1: Write the failing test for configuration and profile path resolution**
- [ ] **Step 2: Run test to verify it fails (`cargo test --test config_test`)**
- [ ] **Step 3: Implement `Cargo.toml`, `build.rs`, `src/error.rs`, dan `src/config.rs`**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit scaffolding & config**

---

### Task 2: Security & Authentication Module (Token & Crypto)

**Files:**
- Create: `src/security/mod.rs`
- Create: `src/security/token.rs`
- Test: `tests/security_test.rs`

**Interfaces:**
- Produces: `TokenManager` dengan `generate_token() -> String`, `get_or_create_token(profile_id) -> Result<String>`, `verify_token(&self, provided: &str) -> bool` (menggunakan `subtle::ConstantTimeEq`).

- [ ] **Step 1: Write the failing test for CSPRNG token generation and constant-time verification**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement `TokenManager` dengan `rand::rngs::OsRng` dan constant-time comparison**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit security module**

---

### Task 3: Win32 Platform Abstractions (DWM, Single Instance, QoS, Audio Session)

**Files:**
- Create: `src/win32/mod.rs`
- Create: `src/win32/instance.rs` (Single instance via Win32 Named Mutex)
- Create: `src/win32/dwm.rs` (Dark mode titlebar & window framing)
- Create: `src/win32/qos.rs` (Process power throttling / EcoQoS)
- Create: `src/win32/audio.rs` (WASAPI COM Core Audio session labeler)
- Test: `tests/instance_test.rs`

**Interfaces:**
- Consumes: `Config`
- Produces: `SingleInstanceGuard`, `set_dark_titlebar(hwnd)`, `set_eco_qos(enabled)`, `rename_audio_session(label, volume)`.

- [ ] **Step 1: Write unit tests for single instance mutex naming and state tracking**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement Win32 platform layer using `windows` crate**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit win32 modules**

---

### Task 4: Injected JavaScript Assets & Polyfills

**Files:**
- Create: `src/assets/mod.rs`
- Create: `src/assets/scripts/agent.js` (DOM interactions: search chat, read messages, composer, media)
- Create: `src/assets/scripts/overlay.js` (Shadow DOM Account switcher & settings)
- Create: `src/assets/scripts/privacy.js` (CSS blur filters hover-to-reveal)
- Create: `src/assets/scripts/notification.js` (Notification API polyfill to host)
- Test: `tests/assets_test.rs`

**Interfaces:**
- Produces: `get_init_scripts(prefs: &Prefs) -> Vec<String>`

- [ ] **Step 1: Write failing test verifying JS scripts are non-empty and syntactically valid**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Port and harden JS scripts from WaGramDeskLite into clean Rust static assets**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit injected scripts**

---

### Task 5: Native Window & System Tray Event Loop

**Files:**
- Create: `src/gui/mod.rs`
- Create: `src/gui/window.rs` (Win32 Window Class, WndProc, Subclassing)
- Create: `src/gui/tray.rs` (Shell_NotifyIconW, Balloon notifications with avatars, dark context menu)
- Create: `src/gui/state.rs` (Window placement persistence)

**Interfaces:**
- Consumes: `src/win32`, `src/config`
- Produces: `AppWindow`, `TrayManager`, message pump runner with thread dispatch channel.

- [ ] **Step 1: Write unit tests for window state JSON serialization and tray menu command dispatch**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement Win32 Window procedure and System Tray integration**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit GUI and Tray modules**

---

### Task 6: WebView2 Controller & Type-Safe Host Bridge

**Files:**
- Create: `src/webview/mod.rs`
- Create: `src/webview/controller.rs` (ICoreWebView2 creation, memory level target, settings)
- Create: `src/webview/bridge.rs` (Bi-directional RPC dispatcher via WebMessage)

**Interfaces:**
- Consumes: `src/gui/window.rs`, `src/assets`
- Produces: `WebViewHost` dengan `call_agent(fn_name, args) -> Result<Value>`, `navigate(url)`, `trim_memory()`.

- [ ] **Step 1: Write mock tests for bridge protocol message serialization**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement WebView2 controller using `webview2-com` and native event handlers**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit WebView2 module**

---

### Task 7: Loopback Server (MCP Server JSON-RPC 2.0 & REST `/api/send`)

**Files:**
- Create: `src/server/mod.rs`
- Create: `src/server/mcp.rs` (JSON-RPC 2.0 router: app_status, list_chats, open_chat, read_messages, conversation_summary, send_message, export_chat)
- Create: `src/server/send_api.rs` (POST /api/send multipart handler dengan humanizer jitter & pacing)
- Create: `src/server/discovery.rs` (Tulis `mcp.json` dan `mcp-<profile>.json`)
- Test: `tests/mcp_server_test.rs`

**Interfaces:**
- Consumes: `src/security`, `src/webview`
- Produces: `start_loopback_server(state: Arc<AppState>) -> Result<ServerHandle>`

- [ ] **Step 1: Write integration tests for MCP JSON-RPC protocol and Bearer token auth**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement loopback HTTP server with Axum/Tokio and constant-time auth**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit server module**

---

### Task 8: Webhook Unread Watcher & Background Scheduler

**Files:**
- Create: `src/webhooks/mod.rs`
- Create: `src/webhooks/watcher.rs` (Polling 5 detik unread count, push ke URL dengan header `X-Wadesk-Secret`)
- Test: `tests/webhook_test.rs`

**Interfaces:**
- Consumes: `src/webview`, `src/config`
- Produces: `start_webhook_watcher(state: Arc<AppState>)`

- [ ] **Step 1: Write unit tests for unread difference detection and webhook HMAC signature headers**
- [ ] **Step 2: Run test to verify it fails**
- [ ] **Step 3: Implement async webhook watcher**
- [ ] **Step 4: Run test to verify it passes**
- [ ] **Step 5: Commit webhook module**

---

### Task 9: Main Assembly, Binary Optimization & End-to-End Verification

**Files:**
- Modify: `src/main.rs`
- Modify: `Cargo.toml` (LTO, opt-level=3, strip=true, panic=abort)
- Create: `assets/icon.ico`

- [ ] **Step 1: Integrate all components in `src/main.rs`**
- [ ] **Step 2: Test single-instance launch, profile flag, autostart logic**
- [ ] **Step 3: Build release binary (`cargo build --release`) and verify binary footprint**
- [ ] **Step 4: Full verification of MCP endpoints and Win32 tray functionality**
- [ ] **Step 5: Commit completed application**
