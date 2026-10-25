use serde::Deserialize;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

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

/// Mengekstrak poin-poin fitur & perbaikan dari release body Markdown agar tampil bersih dan profesional
pub fn extract_highlights(raw_notes: &str) -> Vec<String> {
    let mut highlights = Vec::new();
    let mut in_changelog = false;

    for line in raw_notes.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## What is New") || trimmed.starts_with("### ") {
            in_changelog = true;
            continue;
        }
        if trimmed.starts_with("## Installation") || trimmed.starts_with("## Checksum") {
            break;
        }

        if in_changelog && trimmed.starts_with("- ") {
            // Hilangkan hash commit "(xxxxxxx)" di akhir jika ada
            let item = trimmed.trim_start_matches("- ").trim();
            let clean_item = if let Some(idx) = item.rfind('(') {
                if item.ends_with(')') {
                    item[..idx].trim()
                } else {
                    item
                }
            } else {
                item
            };

            if !clean_item.is_empty() {
                highlights.push(format!("• {}", clean_item));
            }
        }

        if highlights.len() >= 5 {
            break;
        }
    }

    if highlights.is_empty() {
        // Fallback jika format markdown berbeda
        for line in raw_notes.lines() {
            let t = line.trim();
            if t.starts_with("- ") || t.starts_with("* ") {
                let item = t.trim_start_matches(|c| c == '-' || c == '*' || c == ' ').trim();
                if !item.is_empty() {
                    highlights.push(format!("• {}", item));
                }
            }
            if highlights.len() >= 4 {
                break;
            }
        }
    }

    highlights
}

/// Menampilkan dialog pembaruan profesional di dalam WebView (in-app modal)
pub fn show_update_modal(hwnd: HWND, info: &UpdateInfo) {
    let highlights = extract_highlights(&info.release_notes);
    let payload = serde_json::json!({
        "currentVersion": CURRENT_VERSION,
        "latestVersion": info.version,
        "highlights": highlights,
        "downloadUrl": info.download_url,
        "releaseUrl": info.release_url,
    });

    let script = format!(
        "window.wadeskShowUpdateModal && window.wadeskShowUpdateModal({});",
        payload
    );
    crate::gui::window::eval_script(hwnd, &script);
}

/// Fallback kompatibilitas: menampilkan modal update di webview
pub fn prompt_user_for_update(hwnd: HWND, info: &UpdateInfo) {
    show_update_modal(hwnd, info);
}

/// Mengunduh installer pembaruan di latar belakang tanpa membuka browser eksternal,
/// melaporkan progres secara langsung ke UI, dan menjalankan installer setelah selesai.
pub fn spawn_background_download(hwnd: HWND, url: String, version: String) {
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
            let client = match reqwest::Client::builder()
                .user_agent(format!("WaDesk/{}", CURRENT_VERSION))
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    crate::gui::window::eval_script(
                        hwnd,
                        &format!("window.wadeskUpdateError && window.wadeskUpdateError('Koneksi gagal: {}');", e),
                    );
                    return;
                }
            };

            crate::gui::window::eval_script(
                hwnd,
                "window.wadeskUpdateProgress && window.wadeskUpdateProgress(5, 'Menghubungkan ke server rilis...');",
            );

            let mut resp = match client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    crate::gui::window::eval_script(
                        hwnd,
                        &format!("window.wadeskUpdateError && window.wadeskUpdateError('Gagal mengunduh: {}');", e),
                    );
                    return;
                }
            };

            if !resp.status().is_success() {
                crate::gui::window::eval_script(
                    hwnd,
                    &format!("window.wadeskUpdateError && window.wadeskUpdateError('Status server HTTP {}');", resp.status()),
                );
                return;
            }

            let total_size = resp.content_length().unwrap_or(0);
            let temp_file = std::env::temp_dir().join(format!("WaDesk-Setup-v{}.exe", version));

            let mut file = match std::fs::File::create(&temp_file) {
                Ok(f) => f,
                Err(e) => {
                    crate::gui::window::eval_script(
                        hwnd,
                        &format!("window.wadeskUpdateError && window.wadeskUpdateError('Gagal membuat file: {}');", e),
                    );
                    return;
                }
            };

            use std::io::Write;
            let mut downloaded: u64 = 0;
            let mut last_percent: u32 = 0;

            while let Ok(Some(chunk)) = resp.chunk().await {
                if let Err(e) = file.write_all(&chunk) {
                    crate::gui::window::eval_script(
                        hwnd,
                        &format!("window.wadeskUpdateError && window.wadeskUpdateError('Gagal menyimpan unduhan: {}');", e),
                    );
                    return;
                }
                downloaded += chunk.len() as u64;

                let percent = if total_size > 0 {
                    (((downloaded as f64) / (total_size as f64)) * 100.0) as u32
                } else {
                    50
                };

                if percent >= last_percent + 2 || percent >= 100 {
                    last_percent = percent;
                    let downloaded_mb = (downloaded as f64) / (1024.0 * 1024.0);
                    let total_mb = (total_size as f64) / (1024.0 * 1024.0);
                    let status = if total_size > 0 {
                        format!("Mengunduh... ({:.1} MB / {:.1} MB)", downloaded_mb, total_mb)
                    } else {
                        format!("Mengunduh... ({:.1} MB)", downloaded_mb)
                    };
                    crate::gui::window::eval_script(
                        hwnd,
                        &format!("window.wadeskUpdateProgress && window.wadeskUpdateProgress({}, '{}');", percent.min(99), status),
                    );
                }
            }

            let _ = file.flush();

            crate::gui::window::eval_script(
                hwnd,
                "window.wadeskUpdateProgress && window.wadeskUpdateProgress(100, 'Unduhan selesai! Memulai penginstalan...');",
            );

            // Jeda singkat agar user melihat progress 100%
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

            let wide_path: Vec<u16> = temp_file.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                let _ = ShellExecuteW(
                    HWND(std::ptr::null_mut()),
                    w!("open"),
                    PCWSTR(wide_path.as_ptr()),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOWNORMAL,
                );
            }

            // Berikan jeda 1.5 detik lalu keluar aplikasi agar installer dapat mengupdate tanpa konflik file
            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
            }
        });
    });
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
            // Tunggu 4 detik jika background auto-check saat startup agar WebView selesai memuat
            if silent_if_latest {
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
            }

            match fetch_latest_release().await {
                Ok(Some(info)) => {
                    show_update_modal(hwnd, &info);
                }
                Ok(None) => {
                    if !silent_if_latest {
                        let script = format!(
                            "window.wadeskShowToast && window.wadeskShowToast('WaDesk v{} sudah versi terbaru.');",
                            CURRENT_VERSION
                        );
                        crate::gui::window::eval_script(hwnd, &script);
                    }
                }
                Err(e) => {
                    if !silent_if_latest {
                        let script = format!(
                            "window.wadeskShowToast && window.wadeskShowToast('Gagal memeriksa pembaruan: {}');",
                            e
                        );
                        crate::gui::window::eval_script(hwnd, &script);
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

    #[test]
    fn test_extract_highlights() {
        let raw_notes = r#"# WaDesk v1.0.8

WaDesk is a desktop client for WhatsApp Web built with Rust.

## What is New in This Release
- Refactor overlay FAB into centered modal (abc1234)
- Add Ctrl+Shift+A global shortcut (def5678)
- Update CI/CD runner cleanup workflow

## Installation
Run the installer or portable exe.
"#;
        let highlights = extract_highlights(raw_notes);
        assert_eq!(highlights.len(), 3);
        assert_eq!(highlights[0], "• Refactor overlay FAB into centered modal");
        assert_eq!(highlights[1], "• Add Ctrl+Shift+A global shortcut");
        assert_eq!(highlights[2], "• Update CI/CD runner cleanup workflow");
    }
}
