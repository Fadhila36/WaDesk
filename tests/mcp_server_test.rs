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
