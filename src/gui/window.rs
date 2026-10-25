use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_CONTROL, MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetClientRect, GetCursorPos, GetMessageW, GetWindowLongPtrW, InsertMenuW,
    LoadIconW, PostQuitMessage, RegisterClassExW, SendMessageW, SetForegroundWindow,
    SetWindowLongPtrW, ShowWindow, TrackPopupMenu, TranslateMessage, GWLP_USERDATA,
    HICON, IDI_APPLICATION, MF_BYPOSITION, MF_CHECKED, MF_SEPARATOR, MF_STRING,
    MF_UNCHECKED, MSG, SIZE_MINIMIZED, SW_HIDE, SW_RESTORE, SW_SHOW, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, TPM_RETURNCMD, WINDOW_EX_STYLE, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
    CS_HREDRAW, CS_VREDRAW, ICON_BIG, ICON_SMALL, WA_INACTIVE, WM_ACTIVATE, WM_CLOSE,
    WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WM_SETICON,
    WM_SIZE, WM_USER,
};
use crate::config::UserPreferences;
use crate::error::{Result, WaDeskError};
use crate::gui::tray::{TrayIcon, WM_TRAYICON};
use crate::gui::state::WindowState;
use crate::webview::controller::WebViewHost;
use crate::win32::dwm::set_dark_titlebar;
use crate::win32::qos::set_eco_qos;

static PREFS: std::sync::Mutex<Option<UserPreferences>> = std::sync::Mutex::new(None);
static PREFS_PATH: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

pub fn init_preferences(path: &std::path::Path) {
    let prefs = UserPreferences::load_or_default(path);
    *PREFS.lock().unwrap() = Some(prefs);
    *PREFS_PATH.lock().unwrap() = Some(path.to_path_buf());
}

pub fn get_preferences() -> UserPreferences {
    PREFS.lock().unwrap().clone().unwrap_or_default()
}

pub fn save_current_preferences() {
    let prefs = PREFS.lock().unwrap().clone();
    let path = PREFS_PATH.lock().unwrap().clone();
    if let (Some(prefs), Some(path)) = (prefs, path) {
        let _ = prefs.save(&path);
    }
}

pub fn apply_preferences_to_hwnd(hwnd: HWND) {
    let prefs = get_preferences();
    let script = format!(
        "window.wadeskPrivacy && window.wadeskPrivacy.set({}); \
         window.wadeskSetMediaBlur && window.wadeskSetMediaBlur({}); \
         window.wadeskSetAutoBlur && window.wadeskSetAutoBlur({}, {});",
        prefs.privacy_blur,
        prefs.media_blur,
        prefs.auto_blur_inactive,
        prefs.auto_blur_timeout_secs
    );
    unsafe {
        eval_script_on_hwnd(hwnd, &script);
    }
}

pub const WM_DISPATCH_UI: u32 = WM_USER + 100;

pub const HOTKEY_PRIVACY: i32 = 101;
pub const HOTKEY_OVERLAY: i32 = 102;
pub const HOTKEY_MINIMIZE: i32 = 103;

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

            // Load WhatsApp icon embedded by build.rs (ID 1)
            let hicon = LoadIconW(instance, PCWSTR(1 as *const u16)).unwrap_or_else(|_| {
                LoadIconW(None, IDI_APPLICATION).unwrap_or(HICON(std::ptr::null_mut()))
            });

            let class_name = w!("WaDeskWindowClass");

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                hIcon: hicon,
                hIconSm: hicon,
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

            // Set big & small icon on window frame and taskbar
            let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_BIG as usize), LPARAM(hicon.0 as isize));
            let _ = SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_SMALL as usize), LPARAM(hicon.0 as isize));

            set_dark_titlebar(hwnd);
            state.apply_to_hwnd(hwnd);

            // Register desktop hotkeys
            let _ = RegisterHotKey(hwnd, HOTKEY_PRIVACY, MOD_CONTROL | MOD_SHIFT, 0x50); // Ctrl+Shift+P
            let _ = RegisterHotKey(hwnd, HOTKEY_OVERLAY, MOD_CONTROL | MOD_SHIFT, 0x41); // Ctrl+Shift+A
            let _ = RegisterHotKey(hwnd, HOTKEY_MINIMIZE, MOD_CONTROL, 0x4D);            // Ctrl+M

            let tray = TrayIcon::new(hwnd, &config.title, hicon);
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
        apply_preferences_to_hwnd(self.hwnd);
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
            let _ = UnregisterHotKey(self.hwnd, HOTKEY_PRIVACY);
            let _ = UnregisterHotKey(self.hwnd, HOTKEY_OVERLAY);
            let _ = UnregisterHotKey(self.hwnd, HOTKEY_MINIMIZE);
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
        }
    }
}

unsafe fn eval_script_on_hwnd(hwnd: HWND, script: &str) {
    let ctrl_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if ctrl_ptr != 0 {
        let ctrl = &*(ctrl_ptr as *const webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller);
        if let Ok(webview) = ctrl.CoreWebView2() {
            let wide: Vec<u16> = script.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = webview.ExecuteScript(PCWSTR(wide.as_ptr()), None);
        }
    }
}

unsafe fn show_tray_popup_menu(hwnd: HWND) {
    let menu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(_) => return,
    };

    let prefs = get_preferences();

    let _ = InsertMenuW(menu, 0, MF_BYPOSITION | MF_STRING, 2001, w!("Buka WaDesk"));
    let _ = InsertMenuW(menu, 1, MF_BYPOSITION | MF_SEPARATOR, 0, PCWSTR::null());

    let p_flag = if prefs.privacy_blur { MF_CHECKED } else { MF_UNCHECKED };
    let m_flag = if prefs.media_blur { MF_CHECKED } else { MF_UNCHECKED };
    let a_flag = if prefs.auto_blur_inactive { MF_CHECKED } else { MF_UNCHECKED };
    let mute_flag = if prefs.mute_on_minimize { MF_CHECKED } else { MF_UNCHECKED };

    let _ = InsertMenuW(menu, 2, MF_BYPOSITION | MF_STRING | p_flag, 2002, w!("Privacy Blur (Ctrl+Shift+P)"));
    let _ = InsertMenuW(menu, 3, MF_BYPOSITION | MF_STRING | m_flag, 2005, w!("Media Blur (Hover to Reveal)"));
    let _ = InsertMenuW(menu, 4, MF_BYPOSITION | MF_STRING | a_flag, 2006, w!("Auto-Blur saat Inaktif (30s)"));
    let _ = InsertMenuW(menu, 5, MF_BYPOSITION | MF_STRING | mute_flag, 2007, w!("Mute Audio saat Minimized"));

    let _ = InsertMenuW(menu, 6, MF_BYPOSITION | MF_SEPARATOR, 0, PCWSTR::null());
    let _ = InsertMenuW(menu, 7, MF_BYPOSITION | MF_STRING, 2003, w!("Ganti Akun (Ctrl+Shift+A)"));
    let _ = InsertMenuW(menu, 8, MF_BYPOSITION | MF_SEPARATOR, 0, PCWSTR::null());
    let _ = InsertMenuW(menu, 9, MF_BYPOSITION | MF_STRING, 2004, w!("Keluar"));

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);

    let cmd = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);

    match cmd.0 as u32 {
        2001 => {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
            set_eco_qos(false);
            if prefs.mute_on_minimize {
                crate::win32::audio::set_audio_muted(false);
            }
        }
        2002 => {
            let mut p = PREFS.lock().unwrap();
            if let Some(ref mut prefs) = *p {
                prefs.privacy_blur = !prefs.privacy_blur;
            }
            drop(p);
            save_current_preferences();
            apply_preferences_to_hwnd(hwnd);
        }
        2003 => {
            eval_script_on_hwnd(hwnd, "window.wadeskOverlay && window.wadeskOverlay.open();");
        }
        2004 => {
            PostQuitMessage(0);
        }
        2005 => {
            let mut p = PREFS.lock().unwrap();
            if let Some(ref mut prefs) = *p {
                prefs.media_blur = !prefs.media_blur;
            }
            drop(p);
            save_current_preferences();
            apply_preferences_to_hwnd(hwnd);
        }
        2006 => {
            let mut p = PREFS.lock().unwrap();
            if let Some(ref mut prefs) = *p {
                prefs.auto_blur_inactive = !prefs.auto_blur_inactive;
            }
            drop(p);
            save_current_preferences();
            apply_preferences_to_hwnd(hwnd);
        }
        2007 => {
            let mut p = PREFS.lock().unwrap();
            if let Some(ref mut prefs) = *p {
                prefs.mute_on_minimize = !prefs.mute_on_minimize;
            }
            drop(p);
            save_current_preferences();
        }
        _ => {}
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ACTIVATE => {
            let active_state = (wparam.0 & 0xFFFF) as u32;
            if active_state == WA_INACTIVE {
                // Saat jendela kehilangan fokus (misal Alt+Tab ke aplikasi lain)
                eval_script_on_hwnd(hwnd, "window.wadeskOnWindowBlur && window.wadeskOnWindowBlur();");
            } else {
                // Saat jendela mendapatkan kembali fokus
                eval_script_on_hwnd(hwnd, "window.wadeskOnWindowFocus && window.wadeskOnWindowFocus();");
            }
            LRESULT(0)
        }
        WM_SIZE => {
            if wparam.0 == SIZE_MINIMIZED as usize {
                if get_preferences().mute_on_minimize {
                    crate::win32::audio::set_audio_muted(true);
                }
                set_eco_qos(true);
            } else {
                if get_preferences().mute_on_minimize {
                    crate::win32::audio::set_audio_muted(false);
                }
                set_eco_qos(false);
            }

            let ctrl_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if ctrl_ptr != 0 {
                let ctrl = &*(ctrl_ptr as *const webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller);
                let mut bounds = RECT::default();
                let _ = GetClientRect(hwnd, &mut bounds);
                let _ = ctrl.SetBounds(bounds);
            }
            LRESULT(0)
        }
        WM_HOTKEY => {
            match wparam.0 as i32 {
                HOTKEY_PRIVACY => {
                    let mut p = PREFS.lock().unwrap();
                    if let Some(ref mut prefs) = *p {
                        prefs.privacy_blur = !prefs.privacy_blur;
                    }
                    drop(p);
                    save_current_preferences();
                    apply_preferences_to_hwnd(hwnd);
                }
                HOTKEY_OVERLAY => {
                    eval_script_on_hwnd(hwnd, "window.wadeskOverlay && window.wadeskOverlay.open();");
                }
                HOTKEY_MINIMIZE => {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    set_eco_qos(true);
                    if get_preferences().mute_on_minimize {
                        crate::win32::audio::set_audio_muted(true);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_TRAYICON => {
            let evt = (lparam.0 & 0xFFFF) as u32;
            match evt {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    set_eco_qos(false);
                    if get_preferences().mute_on_minimize {
                        crate::win32::audio::set_audio_muted(false);
                    }
                }
                WM_RBUTTONUP => {
                    show_tray_popup_menu(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            // Minimize to tray on close button click
            let _ = ShowWindow(hwnd, SW_HIDE);
            set_eco_qos(true);
            if get_preferences().mute_on_minimize {
                crate::win32::audio::set_audio_muted(true);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
