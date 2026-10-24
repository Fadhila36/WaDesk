use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    PostQuitMessage, RegisterClassExW,
    ShowWindow, TranslateMessage, MSG, SW_HIDE, SW_SHOW,
    WINDOW_EX_STYLE, WS_OVERLAPPEDWINDOW, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW,
    WM_CLOSE, WM_DESTROY, WM_USER,
};
use crate::error::{Result, WaDeskError};
use crate::gui::tray::TrayIcon;
use crate::gui::state::WindowState;
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
                is_visible,
            })
        }
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

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
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
