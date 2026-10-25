use serde::{Deserialize, Serialize};
use crate::config::USER_AGENT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    pub privacy_blur: bool,
    pub notifications: bool,
    pub volume: f32,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            privacy_blur: false,
            notifications: true,
            volume: 1.0,
        }
    }
}

pub fn get_init_scripts(_prefs: &Prefs) -> Vec<String> {
    let ua_override = format!(
        r#"
        (function() {{
            Object.defineProperty(navigator, 'userAgent', {{ get: () => {} }});
            Object.defineProperty(navigator, 'appVersion', {{ get: () => {} }});
        }})();
        "#,
        serde_json::to_string(USER_AGENT).unwrap_or_default(),
        serde_json::to_string(USER_AGENT).unwrap_or_default(),
    );

    let notification_js = include_str!("scripts/notification.js");
    let full_notification_script = format!("{}\n{}", ua_override, notification_js);

    let overlay_js = include_str!("scripts/overlay.js").to_string();
    let privacy_js = include_str!("scripts/privacy.js").to_string();
    let agent_js = include_str!("scripts/agent.js").to_string();
    let updater_js = include_str!("scripts/updater.js").to_string();

    vec![
        full_notification_script,
        overlay_js,
        privacy_js,
        agent_js,
        updater_js,
    ]
}
