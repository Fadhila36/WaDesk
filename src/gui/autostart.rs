use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};
use crate::error::{Result, WaDeskError};

const RUN_SUBKEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const VALUE_NAME: PCWSTR = w!("WaDesk");

/// Memeriksa apakah WaDesk terdaftar untuk mulai otomatis saat booting Windows
pub fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, 0, KEY_READ, &mut hkey).is_err() {
            return false;
        }

        let mut data_len = 0;
        let status = RegQueryValueExW(
            hkey,
            VALUE_NAME,
            None,
            None,
            None,
            Some(&mut data_len),
        );
        let _ = RegCloseKey(hkey);

        status.is_ok() && data_len > 0
    }
}

/// Menetapkan atau mencabut registrasi autostart WaDesk di registry Windows
pub fn set_autostart(enabled: bool) -> Result<()> {
    unsafe {
        let mut hkey = HKEY::default();
        let open_status = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, 0, KEY_WRITE, &mut hkey);
        if open_status.is_err() {
            return Err(WaDeskError::Config(format!("Gagal membuka registry Run: {:?}", open_status)));
        }

        let res = if enabled {
            let exe_path = std::env::current_exe()
                .map_err(|e| WaDeskError::Config(format!("Gagal mendapatkan path exe: {}", e)))?;
            let formatted_cmd = format!("\"{}\" --minimized", exe_path.to_string_lossy());
            let wide_cmd: Vec<u16> = formatted_cmd.encode_utf16().chain(std::iter::once(0)).collect();
            let slice = std::slice::from_raw_parts(
                wide_cmd.as_ptr() as *const u8,
                wide_cmd.len() * std::mem::size_of::<u16>(),
            );

            RegSetValueExW(
                hkey,
                VALUE_NAME,
                0,
                REG_SZ,
                Some(slice),
            )
        } else {
            let _ = RegDeleteValueW(hkey, VALUE_NAME);
            windows::Win32::Foundation::WIN32_ERROR(0)
        };

        let _ = RegCloseKey(hkey);
        if res.is_err() {
            return Err(WaDeskError::Config(format!("Gagal memperbarui autostart di registry: {:?}", res)));
        }
        Ok(())
    }
}
