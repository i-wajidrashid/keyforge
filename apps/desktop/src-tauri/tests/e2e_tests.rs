//! End-to-end integration tests for the Tauri command layer.
//!
//! These tests exercise the **real** vault lifecycle — create, unlock,
//! token CRUD, OTP generation, import/export, lock — using a real
//! SQLCipher database on disk. No mocks, no stubs.

use keyforge_vault::db::Vault;
use keyforge_vault::token::NewToken;
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────

/// Reduced KDF params for fast tests.
fn test_kdf_params() -> keyforge_crypto::kdf::KdfParams {
    keyforge_crypto::kdf::KdfParams {
        memory_kib: 1024,
        time_cost: 1,
        parallelism: 1,
    }
}

/// Create a vault with password-derived keys (like the Tauri commands do).
fn create_vault_with_password(dir: &TempDir, password: &str) -> (Vault, [u8; 16], [u8; 16]) {
    let sqlcipher_salt = keyforge_crypto::random::generate_salt();
    let secret_salt = keyforge_crypto::random::generate_salt();

    let (sqlcipher_key, secret_key) = keyforge_crypto::kdf::derive_key_pair(
        password.as_bytes(),
        &sqlcipher_salt,
        &secret_salt,
        &test_kdf_params(),
    )
    .unwrap();

    let path = dir.path().join("e2e.vault");
    let vault = Vault::create(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
    (vault, sqlcipher_salt, secret_salt)
}

/// Reopen the vault with the same password and salts.
fn reopen_vault(
    dir: &TempDir,
    password: &str,
    sqlcipher_salt: &[u8; 16],
    secret_salt: &[u8; 16],
) -> Vault {
    let (sqlcipher_key, secret_key) = keyforge_crypto::kdf::derive_key_pair(
        password.as_bytes(),
        sqlcipher_salt,
        secret_salt,
        &test_kdf_params(),
    )
    .unwrap();

    let path = dir.path().join("e2e.vault");
    Vault::open(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap()
}

fn github_token() -> NewToken {
    NewToken {
        issuer: "GitHub".to_string(),
        account: "user@example.com".to_string(),
        secret: b"12345678901234567890".to_vec(),
        algorithm: "SHA1".to_string(),
        digits: 6,
        token_type: "totp".to_string(),
        period: 30,
        counter: 0,
        icon: None,
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn e2e_create_and_reopen_with_password() {
    let dir = TempDir::new().unwrap();
    let password = "correct-horse-battery-staple";

    let (sqlcipher_salt, secret_salt);
    {
        let (vault, s1, s2) = create_vault_with_password(&dir, password);
        sqlcipher_salt = s1;
        secret_salt = s2;

        // Add a token
        vault.add_token(github_token()).unwrap();
    }

    // Reopen with same password → tokens survive
    let vault = reopen_vault(&dir, password, &sqlcipher_salt, &secret_salt);
    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].issuer, "GitHub");
}

#[test]
fn e2e_wrong_password_fails() {
    let dir = TempDir::new().unwrap();
    let (_, sqlcipher_salt, secret_salt) = create_vault_with_password(&dir, "right-password");

    // Wrong password → derive different keys → SQLCipher rejects
    let (sqlcipher_key, secret_key) = keyforge_crypto::kdf::derive_key_pair(
        b"wrong-password",
        &sqlcipher_salt,
        &secret_salt,
        &test_kdf_params(),
    )
    .unwrap();

    let path = dir.path().join("e2e.vault");
    let result = Vault::open(path.to_str().unwrap(), &sqlcipher_key, secret_key);
    assert!(result.is_err(), "Wrong password should fail to open vault");
}

#[test]
fn e2e_full_token_lifecycle() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "test-password");

    // Add tokens
    let t1 = vault.add_token(github_token()).unwrap();
    let t2 = vault
        .add_token(NewToken {
            issuer: "AWS".to_string(),
            account: "root@aws.com".to_string(),
            secret: b"ABCDEFGHIJKLMNOPQRST".to_vec(),
            algorithm: "SHA256".to_string(),
            digits: 8,
            token_type: "totp".to_string(),
            period: 30,
            counter: 0,
            icon: None,
        })
        .unwrap();

    // List
    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].issuer, "GitHub");
    assert_eq!(tokens[1].issuer, "AWS");

    // Update
    vault
        .update_token(&t1.id, "GitHub Enterprise", "admin@corp.com")
        .unwrap();
    let updated = vault.get_token(&t1.id).unwrap().unwrap();
    assert_eq!(updated.issuer, "GitHub Enterprise");
    assert_eq!(updated.account, "admin@corp.com");

    // Reorder
    vault
        .reorder_tokens(&[t2.id.clone(), t1.id.clone()])
        .unwrap();
    let reordered = vault.list_tokens().unwrap();
    assert_eq!(reordered[0].issuer, "AWS");
    assert_eq!(reordered[1].issuer, "GitHub Enterprise");

    // Delete
    vault.delete_token(&t2.id).unwrap();
    let remaining = vault.list_tokens().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].issuer, "GitHub Enterprise");
}

#[test]
fn e2e_totp_code_generation_with_rfc_test_vectors() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "test-password");

    // RFC 6238 test vector secret: "12345678901234567890"
    let token = vault.add_token(github_token()).unwrap();

    // Retrieve the secret and verify it roundtrips
    let secret = vault.get_token_secret(&token.id).unwrap();
    assert_eq!(secret, b"12345678901234567890");

    // RFC 6238 test vector: time=59, SHA1, 6 digits, period=30 → "287082"
    let code =
        keyforge_crypto::totp::generate(&secret, 59, 30, 6, keyforge_crypto::hotp::Algorithm::SHA1);
    assert_eq!(code, "287082");

    // RFC 6238 test vector: time=1111111109, SHA1, 8 digits → "07081804"
    let code = keyforge_crypto::totp::generate(
        &secret,
        1111111109,
        30,
        8,
        keyforge_crypto::hotp::Algorithm::SHA1,
    );
    assert_eq!(code, "07081804");
}

#[test]
fn e2e_hotp_counter_and_code_generation() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "test-password");

    let token = vault
        .add_token(NewToken {
            issuer: "HOTP Test".to_string(),
            account: "test@test.com".to_string(),
            secret: b"12345678901234567890".to_vec(),
            algorithm: "SHA1".to_string(),
            digits: 6,
            token_type: "hotp".to_string(),
            period: 30,
            counter: 0,
            icon: None,
        })
        .unwrap();

    // RFC 4226 test vectors (counter 0..9 with SHA1)
    let expected_codes = [
        "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583", "399871",
        "520489",
    ];

    let secret = vault.get_token_secret(&token.id).unwrap();

    for (counter, expected) in expected_codes.iter().enumerate() {
        let code = keyforge_crypto::hotp::generate(
            &secret,
            counter as u64,
            6,
            keyforge_crypto::hotp::Algorithm::SHA1,
        );
        assert_eq!(
            &code, expected,
            "HOTP code at counter {counter} should be {expected}, got {code}"
        );
    }

    // Increment counter via vault
    let c1 = vault.increment_counter(&token.id).unwrap();
    assert_eq!(c1, 1);
    let c2 = vault.increment_counter(&token.id).unwrap();
    assert_eq!(c2, 2);
}

#[test]
fn e2e_export_import_uris() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "test-password");

    // Add two tokens
    vault.add_token(github_token()).unwrap();
    vault
        .add_token(NewToken {
            issuer: "AWS".to_string(),
            account: "root".to_string(),
            secret: b"ABCDEFGHIJKLMNOPQRST".to_vec(),
            algorithm: "SHA256".to_string(),
            digits: 8,
            token_type: "totp".to_string(),
            period: 60,
            counter: 0,
            icon: None,
        })
        .unwrap();

    // Export
    let uris = vault.export_uris().unwrap();
    assert_eq!(uris.len(), 2);
    assert!(uris[0].starts_with("otpauth://totp/"));
    assert!(uris[0].contains("GitHub"));

    // Import into a fresh vault
    let dir2 = TempDir::new().unwrap();
    let (vault2, _, _) = create_vault_with_password(&dir2, "other-password");
    let count = vault2.import_uris(&uris).unwrap();
    assert_eq!(count, 2);

    let tokens = vault2.list_tokens().unwrap();
    assert_eq!(tokens.len(), 2);
}

#[test]
fn e2e_export_import_encrypted() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "vault-password");

    vault.add_token(github_token()).unwrap();

    // Export encrypted with export password
    let export_data = vault.export_encrypted(b"export-secret").unwrap();
    assert!(!export_data.is_empty());

    // Import into fresh vault
    let dir2 = TempDir::new().unwrap();
    let (vault2, _, _) = create_vault_with_password(&dir2, "other-vault");
    let count = vault2
        .import_encrypted(&export_data, b"export-secret")
        .unwrap();
    assert_eq!(count, 1);

    let tokens = vault2.list_tokens().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].issuer, "GitHub");

    // Wrong export password should fail
    let dir3 = TempDir::new().unwrap();
    let (vault3, _, _) = create_vault_with_password(&dir3, "third-vault");
    let result = vault3.import_encrypted(&export_data, b"wrong-password");
    assert!(result.is_err(), "Import with wrong password should fail");
}

#[test]
fn e2e_import_otpauth_uris() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "test-password");

    let uris = vec![
        "otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP&algorithm=SHA1&digits=6&period=30".to_string(),
        "otpauth://hotp/AWS:admin?secret=JBSWY3DPEHPK3PXP&counter=42".to_string(),
    ];

    let count = vault.import_uris(&uris).unwrap();
    assert_eq!(count, 2);

    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens.len(), 2);

    // Verify parsed correctly
    let github = &tokens[0];
    assert_eq!(github.issuer, "GitHub");
    assert_eq!(github.account, "user@example.com");
    assert_eq!(github.token_type, "totp");

    let aws = &tokens[1];
    assert_eq!(aws.issuer, "AWS");
    assert_eq!(aws.account, "admin");
    assert_eq!(aws.token_type, "hotp");
    assert_eq!(aws.counter, 42);
}

#[test]
fn e2e_secret_encryption_roundtrip() {
    let dir = TempDir::new().unwrap();
    let password = "strong-password-123!@#";
    let (vault, sqlcipher_salt, secret_salt) = create_vault_with_password(&dir, password);

    let original_secret = b"SUPER_SECRET_KEY_12345";
    let token = vault
        .add_token(NewToken {
            issuer: "Test".to_string(),
            account: "user".to_string(),
            secret: original_secret.to_vec(),
            algorithm: "SHA512".to_string(),
            digits: 8,
            token_type: "totp".to_string(),
            period: 60,
            counter: 0,
            icon: None,
        })
        .unwrap();

    // Secret roundtrips within same session
    let decrypted = vault.get_token_secret(&token.id).unwrap();
    assert_eq!(decrypted, original_secret);

    // Close and reopen — secret still roundtrips
    drop(vault);
    let vault2 = reopen_vault(&dir, password, &sqlcipher_salt, &secret_salt);
    let decrypted2 = vault2.get_token_secret(&token.id).unwrap();
    assert_eq!(decrypted2, original_secret);

    // Generate a TOTP code from the decrypted secret
    let code = keyforge_crypto::totp::generate(
        &decrypted2,
        59,
        60,
        8,
        keyforge_crypto::hotp::Algorithm::SHA512,
    );
    // Just verify it's a valid 8-digit code
    assert_eq!(code.len(), 8);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn e2e_multi_algorithm_totp() {
    let dir = TempDir::new().unwrap();
    let (vault, _, _) = create_vault_with_password(&dir, "test-password");

    // SHA1, SHA256, SHA512 with RFC 6238 secrets
    let secret_sha1 = b"12345678901234567890".to_vec();
    let secret_sha256 = b"12345678901234567890123456789012".to_vec();
    let secret_sha512 =
        b"1234567890123456789012345678901234567890123456789012345678901234".to_vec();

    let t1 = vault
        .add_token(NewToken {
            issuer: "SHA1 Test".to_string(),
            account: "test".to_string(),
            secret: secret_sha1,
            algorithm: "SHA1".to_string(),
            digits: 8,
            token_type: "totp".to_string(),
            period: 30,
            counter: 0,
            icon: None,
        })
        .unwrap();

    let t2 = vault
        .add_token(NewToken {
            issuer: "SHA256 Test".to_string(),
            account: "test".to_string(),
            secret: secret_sha256,
            algorithm: "SHA256".to_string(),
            digits: 8,
            token_type: "totp".to_string(),
            period: 30,
            counter: 0,
            icon: None,
        })
        .unwrap();

    let t3 = vault
        .add_token(NewToken {
            issuer: "SHA512 Test".to_string(),
            account: "test".to_string(),
            secret: secret_sha512,
            algorithm: "SHA512".to_string(),
            digits: 8,
            token_type: "totp".to_string(),
            period: 30,
            counter: 0,
            icon: None,
        })
        .unwrap();

    // RFC 6238 test vector: time=59 → counter=1
    let s1 = vault.get_token_secret(&t1.id).unwrap();
    let s2 = vault.get_token_secret(&t2.id).unwrap();
    let s3 = vault.get_token_secret(&t3.id).unwrap();

    let code_sha1 =
        keyforge_crypto::totp::generate(&s1, 59, 30, 8, keyforge_crypto::hotp::Algorithm::SHA1);
    let code_sha256 =
        keyforge_crypto::totp::generate(&s2, 59, 30, 8, keyforge_crypto::hotp::Algorithm::SHA256);
    let code_sha512 =
        keyforge_crypto::totp::generate(&s3, 59, 30, 8, keyforge_crypto::hotp::Algorithm::SHA512);

    // RFC 6238 §Appendix B
    assert_eq!(code_sha1, "94287082");
    assert_eq!(code_sha256, "46119246");
    assert_eq!(code_sha512, "90693936");
}
