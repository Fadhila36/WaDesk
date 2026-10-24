use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

pub async fn handle_json_rpc(req: JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "serverInfo": {
                    "name": "wadesk",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "app_status",
                        "description": "Account name, service badge, and the page's ready/title status",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "list_chats",
                        "description": "Visible chats with name, preview, unread count, and active status",
                        "inputSchema": { "type": "object", "properties": {} }
                    },
                    {
                        "name": "open_chat",
                        "description": "Opens the matching conversation by name",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "name": { "type": "string" } },
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "read_messages",
                        "description": "Recent messages of the active open conversation",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "limit": { "type": "integer", "default": 50 } }
                        }
                    },
                    {
                        "name": "conversation_summary",
                        "description": "Reply-decision data and stats for the open conversation",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "limit": { "type": "integer", "default": 50 } }
                        }
                    },
                    {
                        "name": "send_message",
                        "description": "Sends text message in the currently open conversation",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "text": { "type": "string" } },
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "export_chat",
                        "description": "Exports recent messages in the active conversation",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "limit": { "type": "integer", "default": 50 } }
                        }
                    }
                ]
            })),
            error: None,
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        },
    }
}
