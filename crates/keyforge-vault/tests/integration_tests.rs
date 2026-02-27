use keyforge_vault::db::Vault;
use keyforge_vault::token::NewToken;
use tempfile::TempDir;

fn create_test_vault() -> (Vault, TempDir) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.vault");
    let sqlcipher_key = [0x42u8; 32];
    let secret_key = [0x43u8; 32];
    let vault = Vault::create(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
    (vault, dir)
}

fn test_token(issuer: &str) -> NewToken {
    NewToken {
        issuer: issuer.to_string(),
        account: "test@example.com".to_string(),
        secret: b"12345678901234567890".to_vec(),
        algorithm: "SHA1".to_string(),
        digits: 6,
        token_type: "totp".to_string(),
        period: 30,
        counter: 0,
        icon: None,
    }
}

#[test]
fn test_create_and_open_vault() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.vault");
    let sqlcipher_key = [0x42u8; 32];
    let secret_key = [0x43u8; 32];

    // Create vault
    {
        let _vault = Vault::create(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
    }

    // Re-open vault
    {
        let _vault = Vault::open(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
    }
}

#[test]
fn test_wrong_key_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.vault");
    let sqlcipher_key = [0x42u8; 32];
    let wrong_key = [0x99u8; 32];
    let secret_key = [0x43u8; 32];

    // Create vault
    {
        let _vault = Vault::create(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
    }

    // Try opening with wrong key
    let result = Vault::open(path.to_str().unwrap(), &wrong_key, secret_key);
    assert!(result.is_err());
}

#[test]
fn test_add_and_list_tokens() {
    let (vault, _dir) = create_test_vault();

    vault.add_token(test_token("GitHub")).unwrap();
    vault.add_token(test_token("AWS")).unwrap();

    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].issuer, "GitHub");
    assert_eq!(tokens[1].issuer, "AWS");
}

#[test]
fn test_get_token() {
    let (vault, _dir) = create_test_vault();
    let token = vault.add_token(test_token("GitHub")).unwrap();

    let retrieved = vault.get_token(&token.id).unwrap().unwrap();
    assert_eq!(retrieved.issuer, "GitHub");
}

#[test]
fn test_get_nonexistent_token() {
    let (vault, _dir) = create_test_vault();
    let result = vault.get_token("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_token_secret_roundtrip() {
    let (vault, _dir) = create_test_vault();
    let secret = b"12345678901234567890";
    let token = vault
        .add_token(NewToken {
            issuer: "Test".to_string(),
            account: "user".to_string(),
            secret: secret.to_vec(),
            algorithm: "SHA1".to_string(),
            digits: 6,
            token_type: "totp".to_string(),
            period: 30,
            counter: 0,
            icon: None,
        })
        .unwrap();

    let decrypted = vault.get_token_secret(&token.id).unwrap();
    assert_eq!(decrypted, secret);
}

#[test]
fn test_update_token() {
    let (vault, _dir) = create_test_vault();
    let token = vault.add_token(test_token("GitHub")).unwrap();

    vault
        .update_token(&token.id, "GitLab", "new@example.com")
        .unwrap();

    let updated = vault.get_token(&token.id).unwrap().unwrap();
    assert_eq!(updated.issuer, "GitLab");
    assert_eq!(updated.account, "new@example.com");
}

#[test]
fn test_delete_token() {
    let (vault, _dir) = create_test_vault();
    let token = vault.add_token(test_token("GitHub")).unwrap();

    vault.delete_token(&token.id).unwrap();

    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_delete_nonexistent_is_noop() {
    let (vault, _dir) = create_test_vault();
    vault.delete_token("nonexistent").unwrap(); // Should not error
}

#[test]
fn test_reorder_tokens() {
    let (vault, _dir) = create_test_vault();
    let t1 = vault.add_token(test_token("First")).unwrap();
    let t2 = vault.add_token(test_token("Second")).unwrap();
    let t3 = vault.add_token(test_token("Third")).unwrap();

    // Reverse the order
    vault
        .reorder_tokens(&[t3.id.clone(), t2.id.clone(), t1.id.clone()])
        .unwrap();

    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens[0].issuer, "Third");
    assert_eq!(tokens[1].issuer, "Second");
    assert_eq!(tokens[2].issuer, "First");
}

#[test]
fn test_increment_counter() {
    let (vault, _dir) = create_test_vault();
    let token = vault
        .add_token(NewToken {
            token_type: "hotp".to_string(),
            counter: 0,
            issuer: "HOTP Test".to_string(),
            account: "test@example.com".to_string(),
            secret: b"12345678901234567890".to_vec(),
            algorithm: "SHA1".to_string(),
            digits: 6,
            period: 30,
            icon: None,
        })
        .unwrap();

    let counter = vault.increment_counter(&token.id).unwrap();
    assert_eq!(counter, 1);

    let counter = vault.increment_counter(&token.id).unwrap();
    assert_eq!(counter, 2);
}

#[test]
fn test_sort_order_auto_increment() {
    let (vault, _dir) = create_test_vault();
    vault.add_token(test_token("A")).unwrap();
    vault.add_token(test_token("B")).unwrap();
    vault.add_token(test_token("C")).unwrap();

    let tokens = vault.list_tokens().unwrap();
    assert_eq!(tokens[0].sort_order, 0);
    assert_eq!(tokens[1].sort_order, 1);
    assert_eq!(tokens[2].sort_order, 2);
}

#[test]
fn test_full_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.vault");
    let sqlcipher_key = [0x42u8; 32];
    let secret_key = [0x43u8; 32];
    let secret = b"12345678901234567890";

    // Create vault and add tokens
    let token_id;
    {
        let vault = Vault::create(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
        let token = vault
            .add_token(NewToken {
                issuer: "GitHub".to_string(),
                account: "user@test.com".to_string(),
                secret: secret.to_vec(),
                algorithm: "SHA1".to_string(),
                digits: 6,
                token_type: "totp".to_string(),
                period: 30,
                counter: 0,
                icon: None,
            })
            .unwrap();
        token_id = token.id;
    }

    // Close and reopen (simulating lock/unlock)
    {
        let vault = Vault::open(path.to_str().unwrap(), &sqlcipher_key, secret_key).unwrap();
        let tokens = vault.list_tokens().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].issuer, "GitHub");

        let decrypted = vault.get_token_secret(&token_id).unwrap();
        assert_eq!(decrypted, secret);

        // Generate a code to verify
        let code = keyforge_crypto::totp::generate(
            &decrypted,
            59,
            30,
            6,
            keyforge_crypto::hotp::Algorithm::SHA1,
        );
        assert_eq!(code, "287082");
    }
}
