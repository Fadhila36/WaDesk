pub mod assets;
pub mod config;
pub mod error;
pub mod security;
pub mod server;
pub mod webhooks;

#[cfg(windows)]
pub mod gui;
#[cfg(windows)]
pub mod webview;
#[cfg(windows)]
pub mod win32;
