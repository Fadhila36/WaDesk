use wadesk::security::token::TokenManager;

#[test]
fn test_token_generation_entropy() {
    let token1 = TokenManager::generate_token();
    let token2 = TokenManager::generate_token();

    assert_eq!(token1.len(), 48, "Token must be 48 hex characters (24 bytes)");
    assert_eq!(token2.len(), 48);
    assert_ne!(token1, token2, "Subsequent tokens must be cryptographically unique");
}

#[test]
fn test_constant_time_verification() {
    let token = "4568ca030bfaf29c780531d238a03b346fb021733fd07743";
    let valid_bearer = "Bearer 4568ca030bfaf29c780531d238a03b346fb021733fd07743";
    let invalid_bearer = "Bearer 4568ca030bfaf29c780531d238a03b346fb021733fd07744";
    let invalid_format = "Token 4568ca030bfaf29c780531d238a03b346fb021733fd07743";

    assert!(TokenManager::verify_bearer(token, valid_bearer));
    assert!(!TokenManager::verify_bearer(token, invalid_bearer));
    assert!(!TokenManager::verify_bearer(token, invalid_format));
    assert!(!TokenManager::verify_bearer(token, ""));
}

#[test]
fn test_token_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("wadesk_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let token_path = temp_dir.join("mcp-token");

    let t1 = TokenManager::get_or_create_token(&token_path).expect("first read/create must succeed");
    assert_eq!(t1.len(), 48);

    let t2 = TokenManager::get_or_create_token(&token_path).expect("second read must succeed");
    assert_eq!(t1, t2, "Token must be persistent across reads");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[cfg(windows)]
#[test]
fn test_dpapi_encryption() {
    let plain = b"Secret data for testing DPAPI encryption";
    let encrypted = TokenManager::dpapi_encrypt(plain).expect("DPAPI encrypt should succeed");
    assert_ne!(encrypted, plain, "Encrypted data must differ from plaintext");

    let decrypted = TokenManager::dpapi_decrypt(&encrypted).expect("DPAPI decrypt should succeed");
    assert_eq!(decrypted, plain, "Decrypted data must match original plaintext");
}
