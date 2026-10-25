use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

pub const VALID_TOOLS: &[&str] = &[
    "app_status",
    "list_chats",
    "open_chat",
    "read_messages",
    "conversation_summary",
    "send_message",
    "export_chat",
];

pub type ToolFuture = Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>;
pub type ToolDispatcherFn = Arc<dyn Fn(String, Value) -> ToolFuture + Send + Sync>;

static GLOBAL_DISPATCHER: RwLock<Option<ToolDispatcherFn>> = RwLock::new(None);

pub fn set_global_tool_dispatcher(dispatcher: ToolDispatcherFn) {
    if let Ok(mut lock) = GLOBAL_DISPATCHER.write() {
        *lock = Some(dispatcher);
    }
}

pub async fn dispatch_tool_call(name: &str, args: Value) -> Result<Value, String> {
    let dispatcher = {
        let lock = GLOBAL_DISPATCHER.read().map_err(|e| e.to_string())?;
        lock.clone()
    };

    if let Some(disp) = dispatcher {
        disp(name.to_string(), args).await
    } else {
        match name {
            "app_status" => Ok(json!({
                "ready": false,
                "service": "WA",
                "title": "WhatsApp Web (Headless / Connecting)"
            })),
            _ => Err("WhatsApp WebView is not connected or initialized yet".to_string()),
        }
    }
}

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
        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or_default();
            let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

            if !VALID_TOOLS.contains(&tool_name) {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Tool not found: {}", tool_name),
                    }),
                };
            }

            match dispatch_tool_call(tool_name, arguments).await {
                Ok(val) => {
                    let formatted = serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string());
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": formatted
                                }
                            ],
                            "isError": false
                        })),
                        error: None,
                    }
                }
                Err(err_msg) => {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": err_msg
                                }
                            ],
                            "isError": true
                        })),
                        error: None,
                    }
                }
            }
        }
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
