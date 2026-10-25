use serde_json::json;
use wadesk::server::discovery::write_mcp_discovery;
use wadesk::server::mcp::{handle_json_rpc, JsonRpcRequest};

#[test]
fn test_discovery_file_format() {
    let temp_dir = std::env::temp_dir().join(format!("wadesk_disc_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let discovery_path = temp_dir.join("mcp.json");

    let url = "http://127.0.0.1:5987/mcp";
    let token = "4568ca030bfaf29c780531d238a03b346fb021733fd07743";

    write_mcp_discovery(&discovery_path, url, token).expect("write discovery should succeed");

    let content = std::fs::read_to_string(&discovery_path).expect("read discovery should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse discovery json should succeed");

    assert_eq!(parsed["url"], url);
    assert_eq!(parsed["token"], token);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_json_rpc_tools_list() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let resp = handle_json_rpc(req).await;
    assert_eq!(resp.jsonrpc, "2.0");
    assert_eq!(resp.id, Some(json!(1)));

    let result = resp.result.expect("result should be present for tools/list");
    let tools = result["tools"].as_array().expect("tools array should be returned");
    assert!(tools.len() >= 6, "Must expose app_status, list_chats, open_chat, read_messages, conversation_summary, send_message");

    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"list_chats"));
    assert!(tool_names.contains(&"send_message"));
    assert!(tool_names.contains(&"open_chat"));
}

#[tokio::test]
async fn test_json_rpc_tools_call_unknown_tool() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "non_existent_tool",
            "arguments": {}
        })),
    };

    let resp = handle_json_rpc(req).await;
    assert_eq!(resp.id, Some(json!(2)));
    assert!(resp.result.is_none());
    let err = resp.error.expect("error should be returned for unknown tool");
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("Tool not found"));
}

#[tokio::test]
async fn test_json_rpc_tools_call_with_dispatcher() {
    use std::sync::Arc;
    use wadesk::server::mcp::set_global_tool_dispatcher;

    // Register test dispatcher
    set_global_tool_dispatcher(Arc::new(|name, args| {
        Box::pin(async move {
            if name == "send_message" {
                let text = args.get("text").and_then(|t| t.as_str()).unwrap_or("");
                Ok(json!({ "ok": true, "echo": text }))
            } else {
                Err("test error".to_string())
            }
        })
    }));

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "send_message",
            "arguments": { "text": "Halo dari Antigravity" }
        })),
    };

    let resp = handle_json_rpc(req).await;
    assert_eq!(resp.id, Some(json!(3)));
    assert!(resp.error.is_none());

    let result = resp.result.expect("result should be present");
    assert_eq!(result["isError"], false);
    let content = result["content"].as_array().expect("content array");
    assert!(!content.is_empty());
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("Halo dari Antigravity"));
}

#[tokio::test]
async fn test_send_api_request_and_normalization() {
    use wadesk::server::send_api::{normalize_phone, SendMessageRequest, SendPacer};

    let cleaned = normalize_phone("+62 812-3456-7890");
    assert_eq!(cleaned, "6281234567890");

    let pacer = SendPacer::default();
    pacer.pace(10).await;

    let req_json = serde_json::json!({
        "text": "Pesan tes langsung",
        "chat": "Keluarga",
        "pace_ms": 50
    });

    let parsed: SendMessageRequest = serde_json::from_value(req_json).expect("valid parse");
    assert_eq!(parsed.text, "Pesan tes langsung");
    assert_eq!(parsed.chat, Some("Keluarga".to_string()));
    assert_eq!(parsed.pace_ms, Some(50));
}

#[tokio::test]
async fn test_start_loopback_server_live() {
    use wadesk::config::AppPaths;
    use wadesk::server::start_loopback_server;

    let temp_dir = std::env::temp_dir().join(format!("wadesk_srv_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let paths = AppPaths {
        config_dir: temp_dir.clone(),
        user_data_dir: temp_dir.clone(),
        discovery_file: temp_dir.join("mcp.json"),
        token_file: temp_dir.join("mcp-token"),
        accounts_file: temp_dir.join("accounts.json"),
        webhooks_file: temp_dir.join("webhooks.json"),
        window_state_file: temp_dir.join("window_state.json"),
        prefs_file: temp_dir.join("prefs.json"),
        profile_id: "test".to_string(),
    };

    let token = "test_token_1234567890".to_string();
    let handle = start_loopback_server(token.clone(), &paths).await.expect("server must start");
    assert!(handle.port > 0);

    // Make an actual HTTP POST to /mcp
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}/mcp", handle.addr))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.expect("json parse");
    assert_eq!(body["id"], 1);
    assert!(body["result"]["tools"].as_array().unwrap().len() >= 6);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
