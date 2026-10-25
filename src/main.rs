#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows_app {
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

    fn parse_cli_args() -> (String, bool) {
        let args: Vec<String> = env::args().collect();
        for arg in &args[1..] {
            if arg == "--help" || arg == "-h" {
                println!("WaDesk v{} - Lightweight, secure native Windows desktop wrapper for WhatsApp Web", env!("CARGO_PKG_VERSION"));
                println!("\nUsage: wadesk.exe [OPTIONS]");
                println!("\nOptions:");
                println!("  -p, --profile <ID>    Start with a specific account profile (default: 'default')");
                println!("  -h, --help            Print help information");
                println!("  -V, --version         Print version information");
                std::process::exit(0);
            } else if arg == "--version" || arg == "-V" {
                println!("WaDesk v{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
        }

        let mut i = 1;
        while i < args.len() {
            if (args[i] == "--profile" || args[i] == "-p") && i + 1 < args.len() {
                return (args[i + 1].clone(), true);
            }
            i += 1;
        }
        (DEFAULT_PROFILE_ID.to_string(), false)
    }

    pub fn run() -> anyhow::Result<()> {
        let (profile_id, is_explicit) = parse_cli_args();
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

        // 4. Token, Loopback server & Webhook watcher initialization
        let token = TokenManager::get_or_create_token(&paths.token_file)?;
        let paths_clone = paths.clone();
        let token_clone = token.clone();
        let profile_id_clone = profile_id.clone();
        let account_name_clone = account.name.clone();
        let webhooks_file = paths.webhooks_file.clone();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        rt.spawn(async move {
            let _ = start_loopback_server(token, &paths_clone).await;
        });

        rt.spawn(async move {
            wadesk::webhooks::watcher::run_webhook_watcher_task(
                webhooks_file,
                token_clone,
                profile_id_clone,
                account_name_clone,
            ).await;
        });

        // 5. GUI Window Creation & State Restoration
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
            );
        }

        wadesk::gui::window::init_preferences(&paths.prefs_file);
        let window_state = WindowState::load_or_default(&paths.window_state_file);
        let mut window = AppWindow::create(
            WindowConfig {
                title: window_title,
                width: window_state.width,
                height: window_state.height,
            },
            &window_state,
        )?;

        // 6. Attach active WebView2 instance to window & register MCP tool dispatcher
        match wadesk::webview::controller::WebViewHost::create(window.hwnd, &paths.user_data_dir) {
            Ok(webview) => {
                let coordinator = webview.coordinator.clone();
                let hwnd_val = window.hwnd.0 as isize;
                wadesk::server::mcp::set_global_tool_dispatcher(std::sync::Arc::new(move |name, args| {
                    let coord = coordinator.clone();
                    Box::pin(async move {
                        let req_id = coord.next_id();
                        let rx = coord.register(&req_id);
                        let script = format!(
                            "window.wadeskAgent && window.wadeskAgent.call({}, {}, {});",
                            serde_json::to_string(&req_id).map_err(|e| e.to_string())?,
                            serde_json::to_string(&name).map_err(|e| e.to_string())?,
                            serde_json::to_string(&args.to_string()).map_err(|e| e.to_string())?
                        );
                        wadesk::gui::window::eval_script(windows::Win32::Foundation::HWND(hwnd_val as *mut _), &script);
                        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                            Ok(Ok(val)) => Ok(val),
                            Ok(Err(_)) => Err("RPC channel closed".to_string()),
                            Err(_) => Err("Agent RPC request timed out".to_string()),
                        }
                    })
                }));

                window.attach_webview(webview);
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize WebView2: {}", e);
            }
        }

        window.show();
        window.run_message_loop();

        // 7. Save window state on exit
        let current_state = WindowState::from_hwnd(window.hwnd);
        let _ = current_state.save(&paths.window_state_file);

        Ok(())
    }
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    windows_app::run()
}

#[cfg(not(windows))]
fn main() -> anyhow::Result<()> {
    println!("WaDesk v{} - Core Engine & MCP Server", env!("CARGO_PKG_VERSION"));
    println!("Native desktop GUI is powered by WebView2 on Windows.");
    println!("Running in cross-platform headless server mode...");
    let paths = wadesk::config::AppPaths::for_profile(wadesk::config::DEFAULT_PROFILE_ID);
    paths.ensure_dirs()?;
    let token = wadesk::security::token::TokenManager::get_or_create_token(&paths.token_file)?;
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async move {
        let _ = wadesk::server::start_loopback_server(token, &paths).await;
    });
    Ok(())
}
