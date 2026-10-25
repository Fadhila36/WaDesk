#![cfg(windows)]

use serde_json::json;
use wadesk::webview::bridge::{BridgeMessage, RpcCoordinator};

#[test]
fn test_bridge_message_deserialization() {
    let json_notif = r#"{"type":"notification","title":"Alice","body":"Halo!","icon":"data:image/png;base64,abc"}"#;
    let msg: BridgeMessage = serde_json::from_str(json_notif).expect("parse notification should succeed");
    match msg {
        BridgeMessage::Notification { title, body, icon } => {
            assert_eq!(title, "Alice");
            assert_eq!(body, "Halo!");
            assert_eq!(icon, "data:image/png;base64,abc");
        }
        _ => panic!("expected BridgeMessage::Notification"),
    }

    let json_agent = r#"{"type":"agent_reply","id":"req_42","result":{"ready":true}}"#;
    let msg2: BridgeMessage = serde_json::from_str(json_agent).expect("parse agent reply should succeed");
    match msg2 {
        BridgeMessage::AgentReply { id, result } => {
            assert_eq!(id, "req_42");
            assert_eq!(result, json!({"ready": true}));
        }
        _ => panic!("expected BridgeMessage::AgentReply"),
    }

    let json_update = r#"{"type":"download_update","url":"https://github.com/test.exe","version":"1.0.8"}"#;
    let msg3: BridgeMessage = serde_json::from_str(json_update).expect("parse download update should succeed");
    match msg3 {
        BridgeMessage::DownloadUpdate { url, version } => {
            assert_eq!(url, "https://github.com/test.exe");
            assert_eq!(version, "1.0.8");
        }
        _ => panic!("expected BridgeMessage::DownloadUpdate"),
    }

    let json_snooze = r#"{"type":"snooze_update"}"#;
    let msg_snooze: BridgeMessage = serde_json::from_str(json_snooze).expect("parse snooze should succeed");
    assert!(matches!(msg_snooze, BridgeMessage::SnoozeUpdate));

    let json_skip = r#"{"type":"skip_update","version":"1.0.8"}"#;
    let msg_skip: BridgeMessage = serde_json::from_str(json_skip).expect("parse skip should succeed");
    match msg_skip {
        BridgeMessage::SkipUpdate { version } => assert_eq!(version, "1.0.8"),
        _ => panic!("expected BridgeMessage::SkipUpdate"),
    }

    let json_install = r#"{"type":"install_update","version":"1.0.8"}"#;
    let msg_install: BridgeMessage = serde_json::from_str(json_install).expect("parse install should succeed");
    match msg_install {
        BridgeMessage::InstallUpdate { version } => assert_eq!(version, "1.0.8"),
        _ => panic!("expected BridgeMessage::InstallUpdate"),
    }
}

#[tokio::test]
async fn test_rpc_coordinator_reply() {
    let coordinator = RpcCoordinator::new();
    let req_id = coordinator.next_id();

    let rx = coordinator.register(&req_id);

    // Simulasi respons datang dari JavaScript
    let sample_result = json!({"chats": [{"name": "Group A"}]});
    let resolved = coordinator.resolve(&req_id, sample_result.clone());
    assert!(resolved, "resolve should find the registered channel");

    let received = tokio::time::timeout(std::time::Duration::from_millis(500), rx)
        .await
        .expect("should not timeout")
        .expect("channel receive should succeed");

    assert_eq!(received, sample_result);
}
