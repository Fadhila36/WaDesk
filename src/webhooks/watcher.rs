use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatItem {
    pub name: String,
    pub unread: u32,
    #[serde(default)]
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookPayload {
    pub event: String,
    pub account: String,
    pub profile: String,
    pub service: String,
    pub chat: String,
    #[serde(default)]
    pub phone: String,
    pub text: String,
    pub unread: u32,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhooksConfig {
    pub urls: Vec<String>,
}

impl WebhooksConfig {
    pub fn load_or_default(path: &Path) -> Self {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(cfg) = serde_json::from_slice::<WebhooksConfig>(&bytes) {
                return cfg;
            }
        }
        Self::default()
    }
}

/// Mendeteksi perubahan unread message yang bertambah dibandingkan baseline
pub fn detect_unread_changes(
    baseline: &[ChatItem],
    current: &[ChatItem],
    account_name: &str,
    profile_id: &str,
) -> Vec<WebhookPayload> {
    let mut baseline_map: HashMap<&str, u32> = HashMap::new();
    for c in baseline {
        baseline_map.insert(&c.name, c.unread);
    }

    let mut events = Vec::new();

    for curr in current {
        let prev_unread = baseline_map.get(curr.name.as_str()).copied().unwrap_or(0);

        if curr.unread > prev_unread {
            events.push(WebhookPayload {
                event: "message.incoming".to_string(),
                account: account_name.to_string(),
                profile: profile_id.to_string(),
                service: "WA".to_string(),
                chat: curr.name.clone(),
                phone: String::new(),
                text: curr.preview.clone(),
                unread: curr.unread,
                time: "now".to_string(),
            });
        }
    }

    events
}

/// Mengirimkan event webhook ke endpoint HTTP dengan header X-Wadesk-Secret dan X-Wadesk-Profile
pub async fn send_webhook_event(
    client: &reqwest::Client,
    url: &str,
    secret: &str,
    profile_id: &str,
    payload: &WebhookPayload,
) {
    let _ = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Wadesk-Secret", secret)
        .header("X-Wadesk-Profile", profile_id)
        .json(payload)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
}

/// Task latar belakang untuk polling unread chat dan mengirimkan webhook event ke URL terdaftar
pub async fn run_webhook_watcher_task(
    webhooks_file: std::path::PathBuf,
    secret: String,
    profile_id: String,
    account_name: String,
) {
    let client = reqwest::Client::builder()
        .user_agent("WaDesk-Webhook-Watcher")
        .build()
        .unwrap_or_default();

    let mut baseline: Option<Vec<ChatItem>> = None;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(4));

    loop {
        interval.tick().await;

        let cfg = WebhooksConfig::load_or_default(&webhooks_file);
        if cfg.urls.is_empty() {
            continue;
        }

        if let Ok(val) = crate::server::mcp::dispatch_tool_call("list_chats", serde_json::json!({})).await {
            if let Ok(current_chats) = serde_json::from_value::<Vec<ChatItem>>(val) {
                if let Some(ref base) = baseline {
                    let events = detect_unread_changes(base, &current_chats, &account_name, &profile_id);
                    for ev in &events {
                        for url in &cfg.urls {
                            send_webhook_event(&client, url, &secret, &profile_id, ev).await;
                        }
                    }
                }
                baseline = Some(current_chats);
            }
        }
    }
}
