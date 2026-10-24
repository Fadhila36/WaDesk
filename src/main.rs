#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use wadesk::config::{
    Account, AccountsManifest, AppPaths, DEFAULT_PROFILE_ID, APP_NAME,
};
use wadesk::gui::state::WindowState;
use wadesk::gui::window::{AppWindow, WindowConfig};
use wadesk::security::token::TokenManager;
use wadesk::server::start_loopback_server;
use wadesk::win32::audio::rename_audio_session;
use wadesk::win32::instance::SingleInstanceGuard;

fn parse_profile_arg() -> (String, bool) {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--profile" || args[i] == "-p") && i + 1 < args.len() {
            return (args[i + 1].clone(), true);
        }
        i += 1;
    }
    (DEFAULT_PROFILE_ID.to_string(), false)
}

fn main() -> anyhow::Result<()> {
    let (profile_id, is_explicit) = parse_profile_arg();
    let paths = AppPaths::for_profile(&profile_id);
    paths.ensure_dirs()?;

    // Load or initialize accounts manifest
    let mut manifest = AccountsManifest::load_or_default(&paths.accounts_file);
    let account = manifest.get(&profile_id).cloned().unwrap_or_else(|| {
        let new_acct = Account {
            id: profile_id.clone(),
            name: if profile_id == DEFAULT_PROFILE_ID {
                "Account 1".to_string()
            } else {
                format!("Account {}", profile_id)
            },
            service: "WA".to_string(),
            autostart: true,
        };
        manifest.add_or_update(new_acct.clone());
        let _ = manifest.save(&paths.accounts_file);
        new_acct
    });

    let window_title = format!("{} - {}", APP_NAME, account.name);

    // 1. Single Instance check per profile
    let guard = SingleInstanceGuard::acquire(&profile_id)?;
    if !guard.is_primary() {
        guard.focus_existing_window(&window_title);
        return Ok(());
    }

    // 2. Start sibling accounts if cold launch
    if !is_explicit {
        let exe = env::current_exe()?;
        for acct in &manifest.accounts {
            if acct.id != profile_id && acct.autostart {
                let _ = std::process::Command::new(&exe)
                    .arg("--profile")
                    .arg(&acct.id)
                    .spawn();
            }
        }
    }

    // 3. Audio session renaming di Windows Volume Mixer
    rename_audio_session(&window_title);

    // 4. Token & Loopback server initialization
    let token = TokenManager::get_or_create_token(&paths.token_file)?;
    let paths_clone = paths.clone();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.spawn(async move {
        let _ = start_loopback_server(token, &paths_clone).await;
    });

    // 5. GUI Window Creation & State Restoration
    let window_state = WindowState::load_or_default(&paths.window_state_file);
    let window = AppWindow::create(
        WindowConfig {
            title: window_title,
            width: window_state.width,
            height: window_state.height,
        },
        &window_state,
    )?;

    window.show();
    window.run_message_loop();

    // 6. Save window state on exit
    let current_state = WindowState::from_hwnd(window.hwnd);
    let _ = current_state.save(&paths.window_state_file);

    Ok(())
}
