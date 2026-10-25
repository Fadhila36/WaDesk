use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::oneshot;

/// Pesan yang dikirim dari JavaScript via window.chrome.webview.postMessage ke host Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeMessage {
    #[serde(rename = "notification")]
    Notification {
        title: String,
        body: String,
        #[serde(default)]
        icon: String,
    },
    #[serde(rename = "privacy_toggle")]
    PrivacyToggle {
        enabled: bool,
    },
    #[serde(rename = "agent_reply")]
    AgentReply {
        id: String,
        result: Value,
    },
    #[serde(rename = "get_accounts")]
    GetAccounts,
    #[serde(rename = "add_account")]
    AddAccount {
        name: String,
    },
    #[serde(rename = "switch_account")]
    SwitchAccount {
        id: String,
    },
    #[serde(rename = "rename_account")]
    RenameAccount {
        id: String,
        name: String,
    },
    #[serde(rename = "remove_account")]
    RemoveAccount {
        id: String,
    },
    #[serde(rename = "download_update")]
    DownloadUpdate {
        url: String,
        version: String,
    },
    #[serde(rename = "snooze_update")]
    SnoozeUpdate,
    #[serde(rename = "skip_update")]
    SkipUpdate {
        version: String,
    },
    #[serde(rename = "install_update")]
    InstallUpdate {
        version: String,
    },
    #[serde(rename = "schedule_install_on_exit")]
    ScheduleInstallOnExit {
        version: String,
    },
}

/// Koordinator RPC asinkron antara async Tokio tasks dan WebView2 DOM Agent
#[derive(Clone)]
pub struct RpcCoordinator {
    seq: Arc<AtomicU64>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>,
}

impl Default for RpcCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcCoordinator {
    pub fn new() -> Self {
        Self {
            seq: Arc::new(AtomicU64::new(1)),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn next_id(&self) -> String {
        let val = self.seq.fetch_add(1, Ordering::SeqCst);
        format!("wadesk_req_{}", val)
    }

    pub fn register(&self, id: &str) -> oneshot::Receiver<Value> {
        let (tx, rx) = oneshot::channel();
        let mut map = self.pending.lock().unwrap();
        map.insert(id.to_string(), tx);
        rx
    }

    pub fn resolve(&self, id: &str, result: Value) -> bool {
        let mut map = self.pending.lock().unwrap();
        if let Some(tx) = map.remove(id) {
            let _ = tx.send(result);
            true
        } else {
            false
        }
    }
}
