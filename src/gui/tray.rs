use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIF_INFO, NIIF_INFO,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DestroyMenu, GetCursorPos, InsertMenuW, SetForegroundWindow,
    TrackPopupMenu, HICON, HMENU, MF_BYPOSITION, MF_CHECKED, MF_SEPARATOR,
    MF_STRING, MF_UNCHECKED, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
};

pub const WM_TRAYICON: u32 = 0x8000 + 1; // WM_APP + 1
pub const TRAY_ID: u32 = 1001;

pub const CMD_SHOW_HIDE: u32 = 2001;
pub const CMD_PRIVACY_TOGGLE: u32 = 2002;
pub const CMD_EXIT: u32 = 2003;

pub struct TrayIcon {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn new(hwnd: HWND, title: &str) -> Self {
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_ID,
            uFlags: NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            ..Default::default()
        };

        // Set tooltip
        let tip: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = tip.len().min(nid.szTip.len());
        nid.szTip[..copy_len].copy_from_slice(&tip[..copy_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        }

        Self { hwnd, nid }
    }

    pub fn set_icon(&mut self, icon: HICON) {
        self.nid.uFlags |= NIF_ICON;
        self.nid.hIcon = icon;
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &self.nid);
        }
    }

    pub fn show_balloon(&mut self, title: &str, message: &str) {
        self.nid.uFlags |= NIF_INFO;
        self.nid.dwInfoFlags = NIIF_INFO;

        let w_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_t = w_title.len().min(self.nid.szInfoTitle.len());
        self.nid.szInfoTitle = [0; 64];
        self.nid.szInfoTitle[..copy_t].copy_from_slice(&w_title[..copy_t]);

        let w_msg: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_m = w_msg.len().min(self.nid.szInfo.len());
        self.nid.szInfo = [0; 256];
        self.nid.szInfo[..copy_m].copy_from_slice(&w_msg[..copy_m]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &self.nid);
        }
    }

    pub fn show_context_menu(&self, is_window_visible: bool, is_privacy_on: bool) -> u32 {
        unsafe {
            let menu: HMENU = match CreatePopupMenu() {
                Ok(m) => m,
                Err(_) => return 0,
            };

            let toggle_label = if is_window_visible {
                "Hide WaDesk\0"
            } else {
                "Show WaDesk\0"
            };
            let w_toggle: Vec<u16> = toggle_label.encode_utf16().collect();
            let _ = InsertMenuW(menu, 0, MF_BYPOSITION | MF_STRING, CMD_SHOW_HIDE as usize, PCWSTR(w_toggle.as_ptr()));

            let privacy_state = if is_privacy_on { MF_CHECKED } else { MF_UNCHECKED };
            let privacy_flags = MF_BYPOSITION | MF_STRING | privacy_state;
            let w_privacy: Vec<u16> = "Privacy Blur Mode\0".encode_utf16().collect();
            let _ = InsertMenuW(menu, 1, privacy_flags, CMD_PRIVACY_TOGGLE as usize, PCWSTR(w_privacy.as_ptr()));

            let _ = InsertMenuW(menu, 2, MF_BYPOSITION | MF_SEPARATOR, 0, PCWSTR::null());

            let w_exit: Vec<u16> = "Exit\0".encode_utf16().collect();
            let _ = InsertMenuW(menu, 3, MF_BYPOSITION | MF_STRING, CMD_EXIT as usize, PCWSTR(w_exit.as_ptr()));

            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let _ = SetForegroundWindow(self.hwnd);

            let cmd = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );

            let _ = DestroyMenu(menu);
            cmd.0 as u32
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
    }
}
