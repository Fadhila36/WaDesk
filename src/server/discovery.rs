use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpEndpointDiscovery {
    pub url: String,
    pub token: String,
}

pub fn write_mcp_discovery(path: &Path, url: &str, token: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let discovery = McpEndpointDiscovery {
        url: url.to_string(),
        token: token.to_string(),
    };
    let json = serde_json::to_string_pretty(&discovery)?;
    std::fs::write(path, json)?;
    Ok(())
}
