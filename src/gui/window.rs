use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, PostQuitMessage, RegisterClassExW, SetForegroundWindow,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, GWLP_USERDATA, MSG,
    SW_HIDE, SW_RESTORE, SW_SHOW, WINDOW_EX_STYLE, WS_OVERLAPPEDWINDOW,
    WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, WM_CLOSE, WM_DESTROY, WM_SIZE,
    WM_LBUTTONUP, WM_LBUTTONDBLCLK, WM_USER,
};
use crate::error::{Result, WaDeskError};
use crate::gui::tray::{TrayIcon, WM_TRAYICON};
use crate::gui::state::WindowState;
use crate::webview::controller::WebViewHost;
use crate::win32::dwm::set_dark_titlebar;
use crate::win32::qos::set_eco_qos;

pub const WM_DISPATCH_UI: u32 = WM_USER + 100;

pub struct WindowConfig {
    pub title: String,
    pub width: i32,
    pub height: i32,
}

pub struct AppWindow {
    pub hwnd: HWND,
    pub tray: Option<TrayIcon>,
    pub webview: Option<WebViewHost>,
    pub is_visible: Arc<AtomicBool>,
}

impl AppWindow {
    pub fn create(config: WindowConfig, state: &WindowState) -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(None)
                .map_err(|e| WaDeskError::Win32(format!("GetModuleHandleW failed: {}", e)))?;

            let class_name = w!("WaDeskWindowClass");

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassExW(&wnd_class);

            let wide_title: Vec<u16> = config.title.encode_utf16().chain(std::iter::once(0)).collect();

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                PCWSTR(wide_title.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                state.x.unwrap_or(100),
                state.y.unwrap_or(100),
                state.width,
                state.height,
                HWND(std::ptr::null_mut()),
                None,
                instance,
                None,
            ).map_err(|e| WaDeskError::Win32(format!("CreateWindowExW failed: {}", e)))?;

            set_dark_titlebar(hwnd);
            state.apply_to_hwnd(hwnd);

            let tray = TrayIcon::new(hwnd, &config.title);
            let is_visible = Arc::new(AtomicBool::new(true));

            Ok(Self {
                hwnd,
                tray: Some(tray),
                webview: None,
                is_visible,
            })
        }
    }

    pub fn attach_webview(&mut self, webview: WebViewHost) {
        unsafe {
            let ctrl_ptr = &webview.controller as *const webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller;
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, ctrl_ptr as isize);
        }
        self.webview = Some(webview);
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            set_eco_qos(false);
            self.is_visible.store(true, Ordering::SeqCst);
        }
    }

    pub fn hide_to_tray(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
            set_eco_qos(true);
            self.is_visible.store(false, Ordering::SeqCst);
        }
    }

    pub fn run_message_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for AppWindow {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_SIZE => {
            let ctrl_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if ctrl_ptr != 0 {
                let ctrl = &*(ctrl_ptr as *const webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller);
                let mut bounds = RECT::default();
                let _ = GetClientRect(hwnd, &mut bounds);
                let _ = ctrl.SetBounds(bounds);
            }
            LRESULT(0)
        }
        WM_TRAYICON => {
            let evt = lparam.0 as u32;
            if evt == WM_LBUTTONUP || evt == WM_LBUTTONDBLCLK {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
                set_eco_qos(false);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            // Minimize to tray on close button click
            let _ = ShowWindow(hwnd, SW_HIDE);
            set_eco_qos(true);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
