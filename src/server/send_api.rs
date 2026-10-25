use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub text: String,
    #[serde(default)]
    pub chat: Option<String>,
    #[serde(default)]
    pub pace_ms: Option<u64>,
}

#[derive(Default)]
pub struct SendPacer {
    last_send: Mutex<Option<Instant>>,
}

impl SendPacer {
    pub async fn pace(&self, pace_ms: u64) {
        if pace_ms == 0 {
            return;
        }
        let mut lock = self.last_send.lock().await;
        if let Some(prev) = *lock {
            let elapsed = prev.elapsed();
            let target = Duration::from_millis(pace_ms);
            if elapsed < target {
                tokio::time::sleep(target - elapsed).await;
            }
        }
        *lock = Some(Instant::now());
    }
}

pub fn normalize_phone(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}
