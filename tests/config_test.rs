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
