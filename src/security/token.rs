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

    #[cfg(windows)]
    pub fn dpapi_encrypt(data: &[u8]) -> Result<Vec<u8>> {
        use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
        use windows::core::PCWSTR;

        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut out_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            CryptProtectData(
                &mut in_blob,
                PCWSTR::null(),
                None,
                None,
                None,
                0,
                &mut out_blob,
            ).map_err(|e| crate::error::WaDeskError::Crypto(format!("CryptProtectData failed: {}", e)))?;

            let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
            let result = slice.to_vec();
            windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(out_blob.pbData as *mut _));
            Ok(result)
        }
    }

    #[cfg(windows)]
    pub fn dpapi_decrypt(cipher: &[u8]) -> Result<Vec<u8>> {
        use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: cipher.len() as u32,
            pbData: cipher.as_ptr() as *mut u8,
        };
        let mut out_blob = CRYPT_INTEGER_BLOB::default();

        unsafe {
            CryptUnprotectData(
                &mut in_blob,
                None,
                None,
                None,
                None,
                0,
                &mut out_blob,
            ).map_err(|e| crate::error::WaDeskError::Crypto(format!("CryptUnprotectData failed: {}", e)))?;

            let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
            let result = slice.to_vec();
            windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(out_blob.pbData as *mut _));
            Ok(result)
        }
    }

    /// Membaca token yang tersimpan di path, atau menghasilkan token baru dan menyimpannya.
    /// Menggunakan enkripsi Windows DPAPI untuk keamanan penyimpanan token.
    pub fn get_or_create_token(path: &Path) -> Result<String> {
        if let Ok(bytes) = std::fs::read(path) {
            #[cfg(windows)]
            {
                if let Ok(decrypted) = Self::dpapi_decrypt(&bytes) {
                    if let Ok(token_str) = String::from_utf8(decrypted) {
                        let trimmed = token_str.trim().to_string();
                        if !trimmed.is_empty() {
                            return Ok(trimmed);
                        }
                    }
                }
            }

            // Fallback jika file sebelumnya disimpan dalam plaintext
            if let Ok(content) = String::from_utf8(bytes) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let new_token = Self::generate_token();
        #[cfg(windows)]
        {
            if let Ok(encrypted) = Self::dpapi_encrypt(new_token.as_bytes()) {
                std::fs::write(path, encrypted)?;
                return Ok(new_token);
            }
        }

        std::fs::write(path, &new_token)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(path, perms);
            }
        }

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
