use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::Result;

pub const DEFAULT_PROFILE_ID: &str = "default";
pub const APP_NAME: &str = "WaDesk";
pub const APP_URL: &str = "https://web.whatsapp.com";
pub const DEFAULT_MCP_PORT: u16 = 5987;
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

/// Struktur paths untuk file dan direktori runtime WaDesk
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub user_data_dir: PathBuf,
    pub discovery_file: PathBuf,
    pub token_file: PathBuf,
    pub accounts_file: PathBuf,
    pub webhooks_file: PathBuf,
    pub window_state_file: PathBuf,
    pub prefs_file: PathBuf,
    pub profile_id: String,
}

impl AppPaths {
    pub fn for_profile(profile_id: &str) -> Self {
        let base_dir = std::env::var("APPDATA")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|h| PathBuf::from(h).join(".config"))
            })
            .unwrap_or_else(|_| PathBuf::from("."));
        let config_dir = base_dir.join(APP_NAME);

        let user_data_dir = if profile_id == DEFAULT_PROFILE_ID {
            config_dir.join("UserData")
        } else {
            config_dir.join("profiles").join(profile_id).join("UserData")
        };

        let (discovery_file, token_file) = if profile_id == DEFAULT_PROFILE_ID {
            (config_dir.join("mcp.json"), config_dir.join("mcp-token"))
        } else {
            (
                config_dir.join(format!("mcp-{}.json", profile_id)),
                config_dir.join(format!("mcp-token-{}", profile_id)),
            )
        };

        let accounts_file = config_dir.join("accounts.json");
        let webhooks_file = config_dir.join("webhooks.json");
        let window_state_file = config_dir.join(if profile_id == DEFAULT_PROFILE_ID {
            "window_state.json".to_string()
        } else {
            format!("window_state_{}.json", profile_id)
        });
        let prefs_file = config_dir.join("prefs.json");

        Self {
            config_dir,
            user_data_dir,
            discovery_file,
            token_file,
            accounts_file,
            webhooks_file,
            window_state_file,
            prefs_file,
            profile_id: profile_id.to_string(),
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.user_data_dir)?;
        Ok(())
    }
}

/// Metadata akun pengguna
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(default = "default_service")]
    pub service: String,
    #[serde(default = "default_true")]
    pub autostart: bool,
}

fn default_service() -> String {
    "WA".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: DEFAULT_PROFILE_ID.to_string(),
            name: "Account 1".to_string(),
            service: "WA".to_string(),
            autostart: true,
        }
    }
}

/// Manifest penyimpanan daftar akun
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsManifest {
    pub accounts: Vec<Account>,
}

impl Default for AccountsManifest {
    fn default() -> Self {
        Self {
            accounts: vec![Account::default()],
        }
    }
}

impl AccountsManifest {
    pub fn load_or_default(path: &Path) -> Self {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(manifest) = serde_json::from_slice::<AccountsManifest>(&bytes) {
                if !manifest.accounts.is_empty() {
                    return manifest;
                }
            }
        }
        let default_manifest = Self::default();
        let _ = default_manifest.save(path);
        default_manifest
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn add_or_update(&mut self, account: Account) {
        if let Some(existing) = self.accounts.iter_mut().find(|a| a.id == account.id) {
            *existing = account;
        } else {
            self.accounts.push(account);
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.accounts.len();
        self.accounts.retain(|a| a.id != id);
        self.accounts.len() < initial_len
    }

    pub fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}
