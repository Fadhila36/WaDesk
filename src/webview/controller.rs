use std::time::Duration;
use serde_json::Value;
use tokio::sync::mpsc;
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, ICoreWebView2Controller, ICoreWebView2_19,
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
