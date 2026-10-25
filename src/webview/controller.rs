use std::path::Path;
use std::time::Duration;
use serde_json::Value;
use tokio::sync::mpsc;
use windows::core::{w, Interface, PCWSTR, PWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, MessageBoxW, IDYES, MB_ICONWARNING, MB_SYSTEMMODAL, MB_YESNO, SW_SHOWNORMAL,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use webview2_com::{
    CoreWebView2EnvironmentOptions,
    CreateCoreWebView2ControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler,
    NewWindowRequestedEventHandler,
    PermissionRequestedEventHandler,
    WebMessageReceivedEventHandler,
};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    CreateCoreWebView2EnvironmentWithOptions,
    GetAvailableCoreWebView2BrowserVersionString,
    ICoreWebView2, ICoreWebView2Controller, ICoreWebView2EnvironmentOptions,
    ICoreWebView2_19,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
    COREWEBVIEW2_PERMISSION_KIND_CAMERA,
    COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
    COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS,
    COREWEBVIEW2_PERMISSION_STATE_ALLOW,
};
use crate::error::{Result, WaDeskError};
use crate::webview::bridge::RpcCoordinator;

pub const WEBVIEW2_BROWSER_ARGS: &str = "--js-flags=--max-old-space-size=512 --renderer-process-limit=1 --process-per-site --disable-site-isolation-trials --disable-features=SitePerProcess,IsolateOrigins,OutOfProcessNetworkService,msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-background-networking --disable-component-update --no-first-run --disable-sync";

/// Deteksi runtime WebView2; jika tidak ditemukan, tampilkan prompt dialog untuk mengunduh installer resmi.
pub fn ensure_webview2_installed() -> Result<()> {
    let mut version = PWSTR(std::ptr::null_mut());
    let res = unsafe {
        GetAvailableCoreWebView2BrowserVersionString(PCWSTR::null(), &mut version)
    };

    if res.is_ok() && !version.is_null() {
        let _ = webview2_com::CoTaskMemPWSTR::from(version);
        return Ok(());
    }

    unsafe {
        let title = w!("WaDesk - WebView2 Runtime Diperlukan");
        let text = w!("Microsoft Edge WebView2 Runtime tidak ditemukan pada komputer ini.\n\nWaDesk memerlukan komponen ini untuk menjalankan WhatsApp Web.\nApakah Anda ingin mengunduhnya sekarang dari situs resmi Microsoft?");
        let btn = MessageBoxW(
            HWND(std::ptr::null_mut()),
            text,
            title,
            MB_YESNO | MB_ICONWARNING | MB_SYSTEMMODAL,
        );

        if btn == IDYES {
            let url = w!("https://go.microsoft.com/fwlink/p/?LinkId=2124703");
            let _ = ShellExecuteW(
                HWND(std::ptr::null_mut()),
                w!("open"),
                url,
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
    }

    Err(WaDeskError::WebView2("Microsoft Edge WebView2 Runtime is not installed".to_string()))
}

/// Host controller yang membungkus ICoreWebView2 dan controller
pub struct WebViewHost {
    pub webview: ICoreWebView2,
    pub controller: ICoreWebView2Controller,
    pub coordinator: RpcCoordinator,
    pub ui_dispatch_tx: mpsc::UnboundedSender<Box<dyn FnOnce(&WebViewHost) + Send>>,
}

impl WebViewHost {
    pub fn create(hwnd: HWND, user_data_dir: &Path) -> Result<Self> {
        // 1. Verifikasi WebView2 terpasang
        ensure_webview2_installed()?;

        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let coordinator = RpcCoordinator::new();

        let options = CoreWebView2EnvironmentOptions::default();
        unsafe {
            options.set_additional_browser_arguments(WEBVIEW2_BROWSER_ARGS.to_string());
        }
        let env_options: ICoreWebView2EnvironmentOptions = options.into();

        let data_dir_str = user_data_dir.to_string_lossy().to_string();
        let wide_data_dir: Vec<u16> = data_dir_str.encode_utf16().chain(std::iter::once(0)).collect();

        let environment = {
            let (tx, rx) = std::sync::mpsc::channel();
            CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe {
                    CreateCoreWebView2EnvironmentWithOptions(
                        PCWSTR::null(),
                        PCWSTR(wide_data_dir.as_ptr()),
                        &env_options,
                        &handler,
                    )
                    .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, env| {
                    error_code?;
                    tx.send(env.ok_or_else(|| windows::core::Error::from(windows::Win32::Foundation::E_POINTER)))
                        .map_err(|_| windows::core::Error::from(windows::Win32::Foundation::E_FAIL))?;
                    Ok(())
                }),
            )
            .map_err(|e| WaDeskError::WebView2(format!("Environment creation failed: {}", e)))?;

            rx.recv()
                .map_err(|_| WaDeskError::WebView2("Channel receive failed".into()))?
                .map_err(|e| WaDeskError::WebView2(format!("Environment result failed: {}", e)))?
        };

        let controller = {
            let (tx, rx) = std::sync::mpsc::channel();
            CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe {
                    environment
                        .CreateCoreWebView2Controller(hwnd, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, ctrl| {
                    error_code?;
                    tx.send(ctrl.ok_or_else(|| windows::core::Error::from(windows::Win32::Foundation::E_POINTER)))
                        .map_err(|_| windows::core::Error::from(windows::Win32::Foundation::E_FAIL))?;
                    Ok(())
                }),
            )
            .map_err(|e| WaDeskError::WebView2(format!("Controller creation failed: {}", e)))?;

            rx.recv()
                .map_err(|_| WaDeskError::WebView2("Channel receive failed".into()))?
                .map_err(|e| WaDeskError::WebView2(format!("Controller result failed: {}", e)))?
        };

        unsafe {
            let mut bounds = windows::Win32::Foundation::RECT::default();
            let _ = GetClientRect(hwnd, &mut bounds);
            let _ = controller.SetBounds(bounds);
            let _ = controller.SetIsVisible(true);
        }

        let webview = unsafe {
            controller.CoreWebView2()
                .map_err(|e| WaDeskError::WebView2(format!("Failed to get CoreWebView2: {}", e)))?
        };

        // 2. Navigation Guard: Buka link eksternal di browser default Windows
        let new_window_handler = NewWindowRequestedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                let mut uri = PWSTR(std::ptr::null_mut());
                if unsafe { args.Uri(&mut uri) }.is_ok() {
                    let uri_pwstr = webview2_com::CoTaskMemPWSTR::from(uri);
                    let url_str = uri_pwstr.to_string();
                    let _ = unsafe { args.SetHandled(true) };
                    let wide_url: Vec<u16> = url_str.encode_utf16().chain(std::iter::once(0)).collect();
                    unsafe {
                        let _ = ShellExecuteW(
                            HWND(std::ptr::null_mut()),
                            w!("open"),
                            PCWSTR(wide_url.as_ptr()),
                            PCWSTR::null(),
                            PCWSTR::null(),
                            SW_SHOWNORMAL,
                        );
                    }
                }
            }
            Ok(())
        }));
        let mut new_window_token = windows::Win32::System::WinRT::EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_NewWindowRequested(&new_window_handler, &mut new_window_token);
        }

        // 2b. Permission Request Handler: Izinkan otomatis Mikrofon & Kamera untuk WebRTC Call WhatsApp Web
        let permission_handler = PermissionRequestedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                let mut kind = webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_PERMISSION_KIND(0);
                if unsafe { args.PermissionKind(&mut kind) }.is_ok() {
                    if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
                        || kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA
                        || kind == COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS
                    {
                        let _ = unsafe { args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW) };
                    }
                }
            }
            Ok(())
        }));
        let mut perm_token = windows::Win32::System::WinRT::EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_PermissionRequested(&permission_handler, &mut perm_token);
        }

        // 2c. Pengaturan WebView2 untuk Rilis Publik (Matikan DevTools pada release build, nonaktifkan status bar)
        if let Ok(settings) = unsafe { webview.Settings() } {
            let _ = unsafe { settings.SetAreDevToolsEnabled(cfg!(debug_assertions)) };
            let _ = unsafe { settings.SetIsStatusBarEnabled(false) };
        }

        // 3. WebMessage Received: Hubungkan event notifikasi ke tray balloon dan response RPC
        let coord_clone = coordinator.clone();
        let ctrl_clone = controller.clone();
        let msg_handler = WebMessageReceivedEventHandler::create(Box::new(move |_sender, args| {
            if let Some(args) = args {
                let mut raw = PWSTR(std::ptr::null_mut());
                if unsafe { args.WebMessageAsJson(&mut raw) }.is_ok() {
                    let mem = webview2_com::CoTaskMemPWSTR::from(raw);
                    let json_str = mem.to_string();
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        let msg_type = val.get("type").and_then(|t| t.as_str());
                        match msg_type {
                            Some("notification") => {
                                let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("WhatsApp");
                                let body = val.get("body").and_then(|b| b.as_str()).unwrap_or("");
                                crate::gui::tray::TrayIcon::show_balloon_static(hwnd, title, body);
                            }
                            Some("download_update") => {
                                if let Some(url) = val.get("url").and_then(|u| u.as_str()) {
                                    let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("latest").to_string();
                                    crate::gui::updater::spawn_background_download(hwnd, url.to_string(), version);
                                }
                            }
                            Some("snooze_update") => {
                                crate::gui::updater::snooze_update();
                            }
                            Some("skip_update") => {
                                if let Some(version) = val.get("version").and_then(|v| v.as_str()) {
                                    crate::gui::updater::skip_update(version);
                                }
                            }
                            Some("install_update") => {
                                let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("latest");
                                crate::gui::updater::execute_preloaded_installer(hwnd, version);
                            }
                            Some("get_accounts") => {
                                let paths = crate::config::AppPaths::for_profile(crate::config::DEFAULT_PROFILE_ID);
                                let manifest = crate::config::AccountsManifest::load_or_default(&paths.accounts_file);
                                if let Ok(json) = serde_json::to_string(&manifest.accounts) {
                                    let script = format!("window.wadeskRenderAccounts && window.wadeskRenderAccounts({}, 'default');", json);
                                    crate::gui::window::eval_script(hwnd, &script);
                                }
                            }
                            Some("add_account") => {
                                if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                                    let paths = crate::config::AppPaths::for_profile(crate::config::DEFAULT_PROFILE_ID);
                                    let mut manifest = crate::config::AccountsManifest::load_or_default(&paths.accounts_file);
                                    let new_id = format!("acct_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
                                    let new_acct = crate::config::Account {
                                        id: new_id.clone(),
                                        name: name.to_string(),
                                        service: "WA".to_string(),
                                        autostart: true,
                                    };
                                    manifest.add_or_update(new_acct);
                                    let _ = manifest.save(&paths.accounts_file);
                                    let _ = crate::config::AppPaths::for_profile(&new_id).ensure_dirs();
                                    if let Ok(json) = serde_json::to_string(&manifest.accounts) {
                                        let script = format!("window.wadeskRenderAccounts && window.wadeskRenderAccounts({}, 'default');", json);
                                        crate::gui::window::eval_script(hwnd, &script);
                                    }
                                }
                            }
                            Some("rename_account") => {
                                if let (Some(id), Some(new_name)) = (val.get("id").and_then(|i| i.as_str()), val.get("name").and_then(|n| n.as_str())) {
                                    let paths = crate::config::AppPaths::for_profile(crate::config::DEFAULT_PROFILE_ID);
                                    let mut manifest = crate::config::AccountsManifest::load_or_default(&paths.accounts_file);
                                    if let Some(acct) = manifest.accounts.iter_mut().find(|a| a.id == id) {
                                        acct.name = new_name.to_string();
                                        let _ = manifest.save(&paths.accounts_file);
                                    }
                                    if let Ok(json) = serde_json::to_string(&manifest.accounts) {
                                        let script = format!("window.wadeskRenderAccounts && window.wadeskRenderAccounts({}, 'default');", json);
                                        crate::gui::window::eval_script(hwnd, &script);
                                    }
                                }
                            }
                            Some("remove_account") => {
                                if let Some(id) = val.get("id").and_then(|i| i.as_str()) {
                                    if id != crate::config::DEFAULT_PROFILE_ID {
                                        let paths = crate::config::AppPaths::for_profile(crate::config::DEFAULT_PROFILE_ID);
                                        let mut manifest = crate::config::AccountsManifest::load_or_default(&paths.accounts_file);
                                        manifest.remove(id);
                                        let _ = manifest.save(&paths.accounts_file);
                                        if let Ok(json) = serde_json::to_string(&manifest.accounts) {
                                            let script = format!("window.wadeskRenderAccounts && window.wadeskRenderAccounts({}, 'default');", json);
                                            crate::gui::window::eval_script(hwnd, &script);
                                        }
                                    }
                                }
                            }
                            Some("switch_account") => {
                                if let Some(id) = val.get("id").and_then(|i| i.as_str()) {
                                    if let Ok(exe) = std::env::current_exe() {
                                        let _ = std::process::Command::new(exe).arg("--profile").arg(id).spawn();
                                    }
                                }
                            }
                            Some("switch_account_index") => {
                                if let Some(idx) = val.get("index").and_then(|i| i.as_u64()) {
                                    let paths = crate::config::AppPaths::for_profile(crate::config::DEFAULT_PROFILE_ID);
                                    let manifest = crate::config::AccountsManifest::load_or_default(&paths.accounts_file);
                                    if let Some(acct) = manifest.accounts.get(idx as usize) {
                                        if let Ok(exe) = std::env::current_exe() {
                                            let _ = std::process::Command::new(exe).arg("--profile").arg(&acct.id).spawn();
                                        }
                                    }
                                }
                            }
                            Some("toggle_privacy") => {
                                crate::gui::window::toggle_privacy_blur(hwnd);
                            }
                            Some("zoom_in") => {
                                let mut zoom = 1.0;
                                if unsafe { ctrl_clone.ZoomFactor(&mut zoom) }.is_ok() {
                                    let _ = unsafe { ctrl_clone.SetZoomFactor((zoom + 0.1).min(2.5)) };
                                }
                            }
                            Some("zoom_out") => {
                                let mut zoom = 1.0;
                                if unsafe { ctrl_clone.ZoomFactor(&mut zoom) }.is_ok() {
                                    let _ = unsafe { ctrl_clone.SetZoomFactor((zoom - 0.1).max(0.5)) };
                                }
                            }
                            Some("zoom_reset") => {
                                let _ = unsafe { ctrl_clone.SetZoomFactor(1.0) };
                            }
                            _ => {
                                if let Some(rpc_id) = val.get("rpc_id").and_then(|r| r.as_str()).map(|s| s.to_string()) {
                                    coord_clone.resolve(&rpc_id, val);
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut msg_token = windows::Win32::System::WinRT::EventRegistrationToken::default();
        unsafe {
            let _ = webview.add_WebMessageReceived(&msg_handler, &mut msg_token);
        }

        let host = Self {
            webview,
            controller,
            coordinator,
            ui_dispatch_tx: ui_tx,
        };

        // 4. Injeksi polyfill scripts
        let init_scripts = crate::assets::get_init_scripts(&crate::assets::Prefs::default());
        for script in &init_scripts {
            host.add_init_script(script)?;
        }

        // 5. Muat WhatsApp Web
        host.navigate("https://web.whatsapp.com")?;

        Ok(host)
    }

    pub fn navigate(&self, url: &str) -> Result<()> {
        let wide: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            self.webview.Navigate(PCWSTR(wide.as_ptr()))
                .map_err(|e| WaDeskError::WebView2(format!("Navigate failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn eval_script(&self, script: &str) -> Result<()> {
        let wide: Vec<u16> = script.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            self.webview.ExecuteScript(PCWSTR(wide.as_ptr()), None)
                .map_err(|e| WaDeskError::WebView2(format!("ExecuteScript failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn add_init_script(&self, script: &str) -> Result<()> {
        let wide: Vec<u16> = script.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            self.webview.AddScriptToExecuteOnDocumentCreated(PCWSTR(wide.as_ptr()), None)
                .map_err(|e| WaDeskError::WebView2(format!("AddScript failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn trim_memory(&self, low: bool) {
        unsafe {
            if let Ok(v19) = self.webview.cast::<ICoreWebView2_19>() {
                let target = if low {
                    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
                } else {
                    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
                };
                let _ = v19.SetMemoryUsageTargetLevel(target);
            }
        }
    }

    pub fn resize(&self, hwnd: HWND) {
        unsafe {
            let mut bounds = windows::Win32::Foundation::RECT::default();
            let _ = GetClientRect(hwnd, &mut bounds);
            let _ = self.controller.SetBounds(bounds);
        }
    }

    /// Panggil fungsi agent di DOM secara asinkron dari thread mana saja (mis. MCP/REST request)
    pub async fn call_agent(&self, fn_name: &str, args: Value, timeout: Duration) -> Result<Value> {
        let req_id = self.coordinator.next_id();
        let rx = self.coordinator.register(&req_id);

        let script = format!(
            "window.wadeskAgent && window.wadeskAgent.call({}, {}, {});",
            serde_json::to_string(&req_id)?,
            serde_json::to_string(fn_name)?,
            serde_json::to_string(&args.to_string())?
        );

        self.eval_script(&script)?;

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(_)) => Err(WaDeskError::WebView2("RPC channel closed".to_string())),
            Err(_) => Err(WaDeskError::WebView2("Agent RPC request timed out".to_string())),
        }
    }
}
