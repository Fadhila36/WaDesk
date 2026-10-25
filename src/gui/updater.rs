use serde::Deserialize;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO, IDYES, SW_SHOWNORMAL,
};

pub const GITHUB_REPO: &str = "Fadhila36/WaDesk";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub release_notes: String,
    pub release_url: String,
    pub download_url: Option<String>,
}

/// Membandingkan semver: mengembalikan true jika `latest` > `current`.
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let clean_v = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split(['.', '-'])
            .filter_map(|part| part.parse::<u64>().ok())
            .collect()
    };

    let cur_parts = clean_v(current);
    let lat_parts = clean_v(latest);

    for (c, l) in cur_parts.iter().zip(lat_parts.iter()) {
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }
    lat_parts.len() > cur_parts.len()
}

/// Mengecek rilis terbaru secara asinkron dari GitHub Releases API
pub async fn fetch_latest_release() -> anyhow::Result<Option<UpdateInfo>> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let client = reqwest::Client::builder()
        .user_agent(format!("WaDesk-AutoUpdater/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let rel: GitHubRelease = resp.json().await?;
    let latest_ver = rel.tag_name.trim_start_matches('v').to_string();

    if is_newer_version(CURRENT_VERSION, &latest_ver) {
        // Cari asset setup installer jika ada
        let installer_asset = rel.assets.into_iter().find(|a| {
            let n = a.name.to_lowercase();
            n.ends_with(".exe") && (n.contains("setup") || n.contains("wadesk"))
        });

        let download_url = installer_asset
            .map(|a| a.browser_download_url)
            .unwrap_or_else(|| rel.html_url.clone());

        Ok(Some(UpdateInfo {
            version: latest_ver,
            release_notes: rel.body.unwrap_or_default(),
            release_url: rel.html_url,
            download_url: Some(download_url),
        }))
    } else {
        Ok(None)
    }
}

/// Menampilkan dialog konfirmasi update ke user
pub fn prompt_user_for_update(hwnd: HWND, info: &UpdateInfo) {
    let clean_notes = if info.release_notes.is_empty() {
        "Peningkatan stabilitas dan performa.".to_string()
    } else {
        info.release_notes.lines().take(6).collect::<Vec<_>>().join("\n")
    };

    let msg = format!(
        "Versi terbaru WaDesk v{} sudah tersedia!\n\nVersi saat ini: v{}\n\nCatatan Rilis:\n{}\n\nApakah Anda ingin mengunduh & memperbarui sekarang?",
        info.version, CURRENT_VERSION, clean_notes
    );

    let wide_msg: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "Pembaruan Tersedia - WaDesk\0".encode_utf16().collect();

    unsafe {
        let result = MessageBoxW(
            hwnd,
            PCWSTR(wide_msg.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_YESNO | MB_ICONQUESTION,
        );

        if result == IDYES {
            let target_url = info.download_url.as_deref().unwrap_or(&info.release_url);
            let wide_url: Vec<u16> = target_url.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = ShellExecuteW(
                hwnd,
                w!("open"),
                PCWSTR(wide_url.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}

/// Background task yang otomatis memeriksa pembaruan saat startup atau manual trigger
pub fn spawn_background_update_check(hwnd: HWND, silent_if_latest: bool) {
    let hwnd_raw = hwnd.0 as usize;
    std::thread::spawn(move || {
        let hwnd = HWND(hwnd_raw as *mut core::ffi::c_void);
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(r) => r,
            Err(_) => return,
        };

        rt.block_on(async move {
            // Tunggu 3 detik jika background auto-check saat startup
            if silent_if_latest {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }

            match fetch_latest_release().await {
                Ok(Some(info)) => {
                    prompt_user_for_update(hwnd, &info);
                }
                Ok(None) => {
                    if !silent_if_latest {
                        let msg = format!("Anda sudah menggunakan versi terbaru (v{}).", CURRENT_VERSION);
                        let wide_msg: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                        let title: Vec<u16> = "WaDesk Sudah Terbaru\0".encode_utf16().collect();
                        unsafe {
                            let _ = MessageBoxW(
                                hwnd,
                                PCWSTR(wide_msg.as_ptr()),
                                PCWSTR(title.as_ptr()),
                                MB_OK | MB_ICONINFORMATION,
                            );
                        }
                    }
                }
                Err(e) => {
                    if !silent_if_latest {
                        let msg = format!("Gagal memeriksa pembaruan:\n{}", e);
                        let wide_msg: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                        let title: Vec<u16> = "Cek Pembaruan\0".encode_utf16().collect();
                        unsafe {
                            let _ = MessageBoxW(
                                hwnd,
                                PCWSTR(wide_msg.as_ptr()),
                                PCWSTR(title.as_ptr()),
                                MB_OK | MB_ICONINFORMATION,
                            );
                        }
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.0.4", "1.0.5"));
        assert!(is_newer_version("1.0.4", "1.1.0"));
        assert!(is_newer_version("1.0.4", "2.0.0"));
        assert!(!is_newer_version("1.0.5", "1.0.5"));
        assert!(!is_newer_version("1.0.5", "1.0.4"));
        assert!(!is_newer_version("2.0.0", "1.9.9"));
    }
}
