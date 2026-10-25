pub mod discovery;
pub mod mcp;
pub mod send_api;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use tower_http::cors::CorsLayer;
use crate::config::{AppPaths, DEFAULT_MCP_PORT};
use crate::error::Result;
use crate::security::token::TokenManager;
use crate::server::discovery::write_mcp_discovery;
use crate::server::mcp::{handle_json_rpc, JsonRpcRequest};
use crate::server::send_api::SendPacer;

#[derive(Clone)]
pub struct ServerState {
    pub token: String,
    pub pacer: Arc<SendPacer>,
}

pub struct ServerHandle {
    pub addr: SocketAddr,
    pub port: u16,
}

pub async fn start_loopback_server(token: String, paths: &AppPaths) -> Result<ServerHandle> {
    let state = ServerState {
        token: token.clone(),
        pacer: Arc::new(SendPacer::default()),
    };

    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .route("/send", post(send_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Coba bind port default 5987 terlebih dahulu
    let listener = match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_MCP_PORT)).await {
        Ok(l) => l,
        Err(_) => {
            // Fallback ke ephemeral port
            tokio::net::TcpListener::bind("127.0.0.1:0").await?
        }
    };

    let addr = listener.local_addr()?;
    let url = format!("http://{}/mcp", addr);

    // Tulis endpoint discovery
    let _ = write_mcp_discovery(&paths.discovery_file, &url, &token);

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok(ServerHandle {
        addr,
        port: addr.port(),
    })
}

async fn mcp_handler(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> Response {
    // Constant-time token verification
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if !TokenManager::verify_bearer(&state.token, auth_header) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let resp = handle_json_rpc(req).await;
    (StatusCode::OK, Json(resp)).into_response()
}

async fn send_handler(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(payload): Json<send_api::SendMessageRequest>,
) -> Response {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if !TokenManager::verify_bearer(&state.token, auth_header) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    if let Some(pace) = payload.pace_ms {
        state.pacer.pace(pace).await;
    }

    if let Some(ref chat_name) = payload.chat {
        let _ = mcp::dispatch_tool_call("open_chat", serde_json::json!({ "name": chat_name })).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    match mcp::dispatch_tool_call("send_message", serde_json::json!({ "text": payload.text })).await {
        Ok(res) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "result": res }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "ok": false, "error": e }))).into_response(),
    }
}
