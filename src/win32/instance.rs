use crate::error::{Result, WaDeskError};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};

pub struct SingleInstanceGuard {
    handle: HANDLE,
    is_primary: bool,
}

impl SingleInstanceGuard {
    pub fn acquire(profile_id: &str) -> Result<Self> {
        let mutex_name = format!("Local\\WaDesk_SingleInstance_{}\0", profile_id);
        let wide_name: Vec<u16> = mutex_name.encode_utf16().collect();

        unsafe {
            let handle = CreateMutexW(None, true, PCWSTR(wide_name.as_ptr()))
                .map_err(|e| WaDeskError::Win32(format!("CreateMutexW failed: {}", e)))?;

            let already_exists = GetLastError() == ERROR_ALREADY_EXISTS;

            Ok(Self {
                handle,
                is_primary: !already_exists,
            })
        }
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    /// Jika bukan instance utama, coba cari jendela yang sedang terbuka dan bawa ke foreground
    pub fn focus_existing_window(&self, window_title: &str) -> bool {
        let wide_title: Vec<u16> = window_title.encode_utf16().chain(std::iter::once(0)).collect();
        let class_name = windows::core::w!("WaDeskWindowClass");
        unsafe {
            // Coba cari berdasarkan judul jendela
            if let Ok(hwnd) = FindWindowW(class_name, PCWSTR(wide_title.as_ptr())) {
                if hwnd != HWND(std::ptr::null_mut()) {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    return true;
                }
            }
            // Fallback: cari jendela dengan window class WaDesk jika judul berubah di runtime
            if let Ok(hwnd) = FindWindowW(class_name, PCWSTR::null()) {
                if hwnd != HWND(std::ptr::null_mut()) {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    return true;
                }
            }
        }
        false
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

unsafe impl Send for SingleInstanceGuard {}
unsafe impl Sync for SingleInstanceGuard {}
