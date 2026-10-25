use std::path::{Path, PathBuf};
use std::sync::Mutex;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub const GITHUB_REPO: &str = "Fadhila36/WaDesk";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cache rilis yang sudah terunduh di latar belakang dan siap dipasang
static READY_UPDATE: Mutex<Option<(UpdateInfo, PathBuf)>> = Mutex::new(None);

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
    pub expected_checksum: Option<String>,
}

/// Mengembalikan versi rilis yang sudah siap diinstal (jika ada) untuk label tray menu
pub fn get_ready_update_version() -> Option<String> {
    READY_UPDATE.lock().unwrap().as_ref().map(|(info, _)| info.version.clone())
}

/// Menunda pemberitahuan pembaruan selama 24 jam
pub fn snooze_update() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    crate::gui::window::update_preferences(|p| p.snooze_24h(now));
}

/// Melewati notifikasi pembaruan untuk versi tertentu
pub fn skip_update(version: &str) {
    crate::gui::window::update_preferences(|p| p.skip_version(version));
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

/// Ekstrak hash SHA-256 64 karakter hex dari release body markdown jika tersedia
pub fn extract_expected_checksum(body: &str) -> Option<String> {
    for line in body.lines() {
        let lower = line.to_lowercase();
        if lower.contains("sha256") || lower.contains("checksum") {
            for word in line.split(|c: char| !c.is_alphanumeric()) {
                if word.len() == 64 && word.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(word.to_lowercase());
                }
            }
        }
    }
    None
}

/// Menghitung SHA-256 dari file di disk
pub fn compute_file_sha256(path: &Path) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut file, &mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Memverifikasi integritas file terhadap expected SHA-256 hash
pub fn verify_sha256(path: &Path, expected: &str) -> bool {
    match compute_file_sha256(path) {
        Ok(hash) => hash.eq_ignore_ascii_case(expected.trim()),
        Err(_) => false,
    }
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
        // Cari asset setup installer (.exe)
        let installer_asset = rel.assets.iter().find(|a| {
            let n = a.name.to_lowercase();
            n.ends_with(".exe") && (n.contains("setup") || n.contains("wadesk"))
        });

        let download_url = installer_asset
            .map(|a| a.browser_download_url.clone())
            .unwrap_or_else(|| rel.html_url.clone());

        let body_text = rel.body.unwrap_or_default();
        let expected_checksum = extract_expected_checksum(&body_text);

        Ok(Some(UpdateInfo {
            version: latest_ver,
            release_notes: body_text,
            release_url: rel.html_url,
            download_url: Some(download_url),
            expected_checksum,
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
pub fn show_update_modal(hwnd: HWND, info: &UpdateInfo, is_ready: bool) {
    let highlights = extract_highlights(&info.release_notes);
    let payload = serde_json::json!({
        "currentVersion": CURRENT_VERSION,
        "latestVersion": info.version,
        "highlights": highlights,
        "downloadUrl": info.download_url,
        "releaseUrl": info.release_url,
        "isReady": is_ready,
    });

    let script = format!(
        "window.wadeskShowUpdateModal && window.wadeskShowUpdateModal({});",
        payload
    );
    crate::gui::window::eval_script(hwnd, &script);
}

/// Jalankan installer pembaruan dengan silent flag otomatis (/VERYSILENT /SUPPRESSMSGBOXES /NORESTART)
pub fn execute_installer(installer_path: &Path) {
    let wide_path: Vec<u16> = installer_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // Parameter unattended installation untuk Inno Setup / NSIS
    let wide_args: Vec<u16> = "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART\0"
        .encode_utf16()
        .collect();

    unsafe {
        let _ = ShellExecuteW(
            HWND(std::ptr::null_mut()),
            w!("open"),
            PCWSTR(wide_path.as_ptr()),
            PCWSTR(wide_args.as_ptr()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }

    // Beri jeda 800ms lalu tutup WaDesk agar installer dapat mengganti binary tanpa konflik kunci file
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(800));
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
        }
    });
}

/// Menjalankan file installer yang sudah ter-preload di latar belakang
pub fn execute_preloaded_installer(_hwnd: HWND, version: &str) {
    let ready_opt = READY_UPDATE.lock().unwrap().take();
    let installer_path = if let Some((_, path)) = ready_opt {
        path
    } else {
        std::env::temp_dir().join(format!("WaDesk-Setup-v{}.exe", version))
    };

    if installer_path.exists() {
        execute_installer(&installer_path);
    } else {
        // Fallback jika file terhapus: buka halaman download
        let fallback_url = format!("https://github.com/{}/releases/latest", GITHUB_REPO);
        let wide_url: Vec<u16> = fallback_url.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let _ = ShellExecuteW(
                HWND(std::ptr::null_mut()),
                w!("open"),
                PCWSTR(wide_url.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}

/// Trigger manual saat user mengklik "Cek Pembaruan" atau "Pasang Pembaruan" dari Tray
pub fn trigger_manual_update_or_install(hwnd: HWND) {
    let ready_opt = READY_UPDATE.lock().unwrap().clone();
    if let Some((info, _)) = ready_opt {
        show_update_modal(hwnd, &info, true);
    } else {
        spawn_background_update_check(hwnd, false);
    }
}

/// Mengunduh installer pembaruan di latar belakang tanpa membuka browser eksternal,
/// memverifikasi SHA-256 (jika ada), melaporkan progres, dan menjalankan installer.
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

            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

            execute_installer(&temp_file);
        });
    });
}

/// Membersihkan file installer lama di direktori sementara Windows (%TEMP%)
pub fn cleanup_temp_installers() {
    let temp_dir = std::env::temp_dir();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.starts_with("WaDesk-Setup-v") && file_name.ends_with(".exe") {
                    let is_current_cached = READY_UPDATE
                        .lock()
                        .unwrap()
                        .as_ref()
                        .map(|(_, p)| p == &path)
                        .unwrap_or(false);

                    if !is_current_cached {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

/// Background task yang otomatis memeriksa pembaruan saat startup atau manual trigger,
/// melakukan silent background pre-download saat startup agar pembaruan instan,
/// dan mematuhi aturan snooze / skip version.
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
            // Bersihkan installer usang di %TEMP% dan tunggu WebView selesai memuat
            if silent_if_latest {
                cleanup_temp_installers();
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
            }

            let prefs = crate::gui::window::get_preferences();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            match fetch_latest_release().await {
                Ok(Some(info)) => {
                    // Cek apakah update disnooze atau dilewati oleh pengguna
                    if silent_if_latest && prefs.is_update_suppressed(&info.version, now) {
                        return;
                    }

                    // Silent Background Pre-download jika ada download URL
                    if let Some(ref dl_url) = info.download_url {
                        let temp_file = std::env::temp_dir().join(format!("WaDesk-Setup-v{}.exe", info.version));
                        let should_download = !temp_file.exists();

                        if should_download {
                            let client = reqwest::Client::builder()
                                .user_agent(format!("WaDesk/{}", CURRENT_VERSION))
                                .build();

                            if let Ok(c) = client {
                                if let Ok(mut resp) = c.get(dl_url).send().await {
                                    if resp.status().is_success() {
                                        if let Ok(mut f) = std::fs::File::create(&temp_file) {
                                            use std::io::Write;
                                            while let Ok(Some(chunk)) = resp.chunk().await {
                                                let _ = f.write_all(&chunk);
                                            }
                                            let _ = f.flush();
                                        }
                                    }
                                }
                            }
                        }

                        // Verifikasi SHA-256 jika ada hash
                        let is_valid = if let Some(ref expected) = info.expected_checksum {
                            verify_sha256(&temp_file, expected)
                        } else {
                            temp_file.exists() && std::fs::metadata(&temp_file).map(|m| m.len() > 0).unwrap_or(false)
                        };

                        if is_valid {
                            *READY_UPDATE.lock().unwrap() = Some((info.clone(), temp_file));
                            show_update_modal(hwnd, &info, true);
                            return;
                        }
                    }

                    // Jika pre-download tidak selesai atau download_url berupa release page
                    show_update_modal(hwnd, &info, false);
                }
                Ok(None) => {
                    if !silent_if_latest {
                        let msg = format!("WaDesk v{} sudah menggunakan versi terbaru.", CURRENT_VERSION);
                        let script = format!(
                            "window.wadeskShowToast && window.wadeskShowToast('{}');",
                            msg
                        );
                        crate::gui::window::eval_script(hwnd, &script);
                        crate::gui::tray::TrayIcon::show_balloon_static(hwnd, "WaDesk Update", &msg);
                    }
                }
                Err(e) => {
                    if !silent_if_latest {
                        let script = format!(
                            "window.wadeskShowToast && window.wadeskShowToast('Gagal memeriksa pembaruan: {}');",
                            e
                        );
                        crate::gui::window::eval_script(hwnd, &script);
                        crate::gui::tray::TrayIcon::show_balloon_static(hwnd, "WaDesk Update", "Gagal terhubung ke server pembaruan.");
                    }
                }
            }
        });
    });
}

/// Pemeriksaan pembaruan manual yang dipicu pengguna dari menu tray
pub fn check_for_updates_manual(hwnd: HWND) {
    spawn_background_update_check(hwnd, false);
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

    #[test]
    fn test_extract_expected_checksum() {
        let body = "Release v1.0.8\nSHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\nEnjoy!";
        let hash = extract_expected_checksum(body);
        assert_eq!(hash, Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()));

        let no_hash_body = "Regular notes without checksum";
        assert_eq!(extract_expected_checksum(no_hash_body), None);
    }

    #[test]
    fn test_compute_file_sha256() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("wadesk_test_hash.txt");
        std::fs::write(&test_file, b"WaDesk Secure Updater").expect("write test file");

        let hash = compute_file_sha256(&test_file).expect("compute hash");
        assert_eq!(hash.len(), 64);

        assert!(verify_sha256(&test_file, &hash));
        assert!(!verify_sha256(&test_file, "0000000000000000000000000000000000000000000000000000000000000000"));

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_cleanup_temp_installers() {
        let temp_dir = std::env::temp_dir();
        let dummy_old_installer = temp_dir.join("WaDesk-Setup-v0.9.0-test.exe");
        std::fs::write(&dummy_old_installer, b"old installer payload").expect("create dummy old installer");
        assert!(dummy_old_installer.exists());

        cleanup_temp_installers();
        assert!(!dummy_old_installer.exists(), "Old temp installer should be cleaned up");
    }
}
