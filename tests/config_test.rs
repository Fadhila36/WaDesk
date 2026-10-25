use std::path::Path;
use wadesk::config::{Account, AccountsManifest, AppPaths, DEFAULT_PROFILE_ID};

#[test]
fn test_default_profile_paths() {
    let paths = AppPaths::for_profile(DEFAULT_PROFILE_ID);
    assert!(paths.config_dir.ends_with("WaDesk"));
    assert!(paths.user_data_dir.ends_with(Path::new("WaDesk").join("UserData")));
    assert!(paths.discovery_file.ends_with(Path::new("WaDesk").join("mcp.json")));
    assert!(paths.token_file.ends_with(Path::new("WaDesk").join("mcp-token")));
}

#[test]
fn test_custom_profile_paths() {
    let profile_id = "sales_team";
    let paths = AppPaths::for_profile(profile_id);
    assert!(paths.user_data_dir.ends_with(Path::new("WaDesk").join("profiles").join("sales_team").join("UserData")));
    assert!(paths.discovery_file.ends_with(Path::new("WaDesk").join("mcp-sales_team.json")));
    assert!(paths.token_file.ends_with(Path::new("WaDesk").join("mcp-token-sales_team")));
}

#[test]
fn test_accounts_manifest_serialization() {
    let mut manifest = AccountsManifest::default();
    assert_eq!(manifest.accounts.len(), 1);
    assert_eq!(manifest.accounts[0].id, DEFAULT_PROFILE_ID);

    let new_account = Account {
        id: "work_01".to_string(),
        name: "Work Account".to_string(),
        service: "WA".to_string(),
        autostart: true,
    };
    manifest.add_or_update(new_account);

    assert_eq!(manifest.accounts.len(), 2);
    let found = manifest.get("work_01").expect("account should exist");
    assert_eq!(found.name, "Work Account");

    let json = serde_json::to_string(&manifest).expect("serialize should succeed");
    let deserialized: AccountsManifest = serde_json::from_str(&json).expect("deserialize should succeed");
    assert_eq!(deserialized.accounts.len(), 2);
}

#[test]
fn test_user_preferences_defaults_and_roundtrip() {
    use wadesk::config::UserPreferences;

    let default_prefs = UserPreferences::default();
    assert!(!default_prefs.privacy_blur);
    assert!(default_prefs.mute_on_minimize);
    assert!(default_prefs.auto_blur_inactive);
    assert_eq!(default_prefs.auto_blur_timeout_secs, 30);
    assert!(!default_prefs.media_blur);

    let json = serde_json::to_string_pretty(&default_prefs).expect("serialize should succeed");
    let mut deserialized: UserPreferences = serde_json::from_str(&json).expect("deserialize should succeed");
    assert_eq!(deserialized, default_prefs);

    deserialized.media_blur = true;
    deserialized.privacy_blur = true;
    assert!(deserialized.media_blur);
    assert!(deserialized.privacy_blur);
}

#[test]
fn test_update_snooze_and_skip_logic() {
    use wadesk::config::UserPreferences;

    let mut prefs = UserPreferences::default();
    let current_time = 1700000000;
    assert!(!prefs.is_update_suppressed("1.0.9", current_time));

    // Test snooze 24h
    prefs.snooze_24h(current_time);
    assert!(prefs.is_update_suppressed("1.0.9", current_time + 3600)); // 1 jam kemudian masih suppressed
    assert!(!prefs.is_update_suppressed("1.0.9", current_time + 86401)); // Lewat 24 jam sudah tidak suppressed

    // Test skip version
    prefs.snooze_until = None;
    prefs.skip_version("1.0.9");
    assert!(prefs.is_update_suppressed("1.0.9", current_time));
    assert!(!prefs.is_update_suppressed("1.1.0", current_time)); // Versi lebih baru tidak ter-suppress
}

