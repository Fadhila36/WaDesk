use std::path::Path;
use rand::RngCore;
use subtle::ConstantTimeEq;
use crate::error::Result;

pub struct TokenManager;

impl TokenManager {
    /// Menghasilkan 24 bytes acak dengan OsRng kernel entropy, di-encode sebagai 48 karakter hex.
    pub fn generate_token() -> String {
        let mut bytes = [0u8; 24];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        hex_encode(&bytes)
    }

    /// Menghasilkan 32 bytes acak untuk secret webhook
    pub fn generate_secret() -> String {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        hex_encode(&bytes)
    }

    /// Membaca token yang tersimpan di path, atau menghasilkan token baru dan menyimpannya.
    pub fn get_or_create_token(path: &Path) -> Result<String> {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let new_token = Self::generate_token();
        std::fs::write(path, &new_token)?;
        Ok(new_token)
    }

    /// Constant-time verification untuk mencegah timing attacks pada validasi Bearer token.
    pub fn verify_bearer(expected_token: &str, header_value: &str) -> bool {
        let trimmed = header_value.trim();
        if !trimmed.starts_with("Bearer ") {
            return false;
        }

        let provided = trimmed["Bearer ".len()..].trim();
        Self::verify_token(expected_token, provided)
    }

    /// Constant-time string comparison
    pub fn verify_token(expected: &str, provided: &str) -> bool {
        if expected.len() != provided.len() {
            return false;
        }

        expected.as_bytes().ct_eq(provided.as_bytes()).into()
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}
