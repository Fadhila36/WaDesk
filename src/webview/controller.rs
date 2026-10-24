use std::path::Path;
use std::time::Duration;
use serde_json::Value;
use tokio::sync::mpsc;
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use webview2_com::{
    CoreWebView2EnvironmentOptions,
    CreateCoreWebView2ControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler,
};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    CreateCoreWebView2EnvironmentWithOptions,
    ICoreWebView2, ICoreWebView2Controller, ICoreWebView2EnvironmentOptions,
    ICoreWebView2_19,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use crate::error::{Result, WaDeskError};
use crate::webview::bridge::RpcCoordinator;

pub const WEBVIEW2_BROWSER_ARGS: &str = "--js-flags=--max-old-space-size=512 --renderer-process-limit=1 --process-per-site --disable-site-isolation-trials --disable-gpu --disable-gpu-compositing --enable-low-end-device-mode --disable-features=SitePerProcess,IsolateOrigins,OutOfProcessNetworkService,msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-background-networking --disable-component-update --no-first-run --disable-sync";

/// Host controller yang membungkus ICoreWebView2 dan controller
pub struct WebViewHost {
    pub webview: ICoreWebView2,
    pub controller: ICoreWebView2Controller,
    pub coordinator: RpcCoordinator,
    pub ui_dispatch_tx: mpsc::UnboundedSender<Box<dyn FnOnce(&WebViewHost) + Send>>,
}

impl WebViewHost {
    pub fn create(hwnd: HWND, user_data_dir: &Path) -> Result<Self> {
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
            let mut bounds = RECT::default();
            let _ = GetClientRect(hwnd, &mut bounds);
            let _ = controller.SetBounds(bounds);
            let _ = controller.SetIsVisible(true);
        }

        let webview = unsafe {
            controller.CoreWebView2()
                .map_err(|e| WaDeskError::WebView2(format!("Failed to get CoreWebView2: {}", e)))?
        };

        let host = Self {
            webview,
            controller,
            coordinator,
            ui_dispatch_tx: ui_tx,
        };

        let init_scripts = crate::assets::get_init_scripts(&crate::assets::Prefs::default());
        for script in &init_scripts {
            host.add_init_script(script)?;
        }

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
            let mut bounds = RECT::default();
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
