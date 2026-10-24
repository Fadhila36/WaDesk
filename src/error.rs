use thiserror::Error;

#[derive(Error, Debug)]
pub enum WaDeskError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebView2 error: {0}")]
    WebView2(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Win32 error: {0}")]
    Win32(String),

    #[error("HTTP request error: {0}")]
    Http(String),

    #[error("Cryptography error: {0}")]
    Crypto(String),
}

pub type Result<T> = std::result::Result<T, WaDeskError>;
