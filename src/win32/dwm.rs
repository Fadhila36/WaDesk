use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};

// DWMWA_USE_IMMERSIVE_DARK_MODE is 20 in Windows 10 20H1+ and Windows 11, or 19 in older builds
const DWMWA_USE_IMMERSIVE_DARK_MODE: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(20);
const DWMWA_USE_IMMERSIVE_DARK_MODE_OLD: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(19);

/// Mengatur title bar dan frame jendela agar menggunakan dark mode native Windows
pub fn set_dark_titlebar(hwnd: HWND) {
    unsafe {
        let use_dark_mode = BOOL(1);
        let res = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &use_dark_mode as *const BOOL as *const _,
            std::mem::size_of::<BOOL>() as u32,
        );
        if res.is_err() {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE_OLD,
                &use_dark_mode as *const BOOL as *const _,
                std::mem::size_of::<BOOL>() as u32,
            );
        }
    }
}
