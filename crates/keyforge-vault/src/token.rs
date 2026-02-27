//! Token CRUD operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use zeroize::Zeroize;

use crate::db::Vault;

/// Token representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub issuer: String,
    pub account: String,
    pub algorithm: String,
    pub digits: u32,
    pub token_type: String,
    pub period: u32,
    pub counter: u64,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
    pub last_modified: Option<String>,
    pub device_id: Option<String>,
    pub sync_version: Option<i64>,
}

/// Input for creating a new token
#[derive(Debug)]
pub struct NewToken {
    pub issuer: String,
    pub account: String,
    pub secret: Vec<u8>,
    pub algorithm: String,
    pub digits: u32,
    pub token_type: String,
    pub period: u32,
    pub counter: u64,
    pub icon: Option<String>,
}

impl Vault {
    /// Insert a new token into the vault
    pub fn add_token(&self, mut new_token: NewToken) -> Result<Token, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Encrypt the secret
        let encrypted_secret = keyforge_crypto::aead::encrypt(&new_token.secret, self.secret_key())
            .map_err(|e| format!("Failed to encrypt secret: {}", e))?;

        // Zeroize the plaintext secret
        new_token.secret.zeroize();

        // Get the next sort order
        let max_sort: i32 = self.conn()
            .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM tokens", [], |row| row.get(0))
            .unwrap_or(-1);

        self.conn().execute(
            "INSERT INTO tokens (id, issuer, account, secret_encrypted, algorithm, digits, type, period, counter, icon, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                id,
                new_token.issuer,
                new_token.account,
                encrypted_secret,
                new_token.algorithm,
                new_token.digits,
                new_token.token_type,
                new_token.period,
                new_token.counter,
                new_token.icon,
                max_sort + 1,
                now,
                now,
            ],
        ).map_err(|e| format!("Failed to insert token: {}", e))?;

        Ok(Token {
            id,
            issuer: new_token.issuer.clone(),
            account: new_token.account.clone(),
            algorithm: new_token.algorithm.clone(),
            digits: new_token.digits,
            token_type: new_token.token_type.clone(),
            period: new_token.period,
            counter: new_token.counter,
            icon: new_token.icon.clone(),
            sort_order: max_sort + 1,
            created_at: now.clone(),
            updated_at: now,
            last_modified: None,
            device_id: None,
            sync_version: None,
        })
    }

    /// Get all tokens (without decrypted secrets)
    pub fn list_tokens(&self) -> Result<Vec<Token>, String> {
        let mut stmt = self.conn().prepare(
            "SELECT id, issuer, account, algorithm, digits, type, period, counter, icon, sort_order, created_at, updated_at, last_modified, device_id, sync_version
             FROM tokens ORDER BY sort_order ASC"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let tokens = stmt.query_map([], |row| {
            Ok(Token {
                id: row.get(0)?,
                issuer: row.get(1)?,
                account: row.get(2)?,
                algorithm: row.get(3)?,
                digits: row.get(4)?,
                token_type: row.get(5)?,
                period: row.get(6)?,
                counter: row.get(7)?,
                icon: row.get(8)?,
                sort_order: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                last_modified: row.get(12)?,
                device_id: row.get(13)?,
                sync_version: row.get(14)?,
            })
        }).map_err(|e| format!("Failed to query tokens: {}", e))?;

        tokens.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect tokens: {}", e))
    }

    /// Get a single token by ID
    pub fn get_token(&self, id: &str) -> Result<Option<Token>, String> {
        let mut stmt = self.conn().prepare(
            "SELECT id, issuer, account, algorithm, digits, type, period, counter, icon, sort_order, created_at, updated_at, last_modified, device_id, sync_version
             FROM tokens WHERE id = ?1"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(Token {
                id: row.get(0)?,
                issuer: row.get(1)?,
                account: row.get(2)?,
                algorithm: row.get(3)?,
                digits: row.get(4)?,
                token_type: row.get(5)?,
                period: row.get(6)?,
                counter: row.get(7)?,
                icon: row.get(8)?,
                sort_order: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                last_modified: row.get(12)?,
                device_id: row.get(13)?,
                sync_version: row.get(14)?,
            })
        }).map_err(|e| format!("Failed to query token: {}", e))?;

        match rows.next() {
            Some(Ok(token)) => Ok(Some(token)),
            Some(Err(e)) => Err(format!("Failed to read token: {}", e)),
            None => Ok(None),
        }
    }

    /// Get the decrypted secret for a token
    pub fn get_token_secret(&self, id: &str) -> Result<Vec<u8>, String> {
        let encrypted: Vec<u8> = self.conn().query_row(
            "SELECT secret_encrypted FROM tokens WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        ).map_err(|e| format!("Token not found: {}", e))?;

        keyforge_crypto::aead::decrypt(&encrypted, self.secret_key())
            .map_err(|e| format!("Failed to decrypt secret: {}", e))
    }

    /// Update token metadata (issuer and account)
    pub fn update_token(&self, id: &str, issuer: &str, account: &str) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn().execute(
            "UPDATE tokens SET issuer = ?1, account = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![issuer, account, now, id],
        ).map_err(|e| format!("Failed to update token: {}", e))?;

        if rows == 0 {
            return Err("Token not found".to_string());
        }
        Ok(())
    }

    /// Delete a token
    pub fn delete_token(&self, id: &str) -> Result<(), String> {
        self.conn().execute(
            "DELETE FROM tokens WHERE id = ?1",
            rusqlite::params![id],
        ).map_err(|e| format!("Failed to delete token: {}", e))?;
        Ok(())
    }

    /// Update token sort orders
    pub fn reorder_tokens(&self, id_order: &[String]) -> Result<(), String> {
        let tx = self.conn().unchecked_transaction()
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        for (i, id) in id_order.iter().enumerate() {
            tx.execute(
                "UPDATE tokens SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![i as i32, Utc::now().to_rfc3339(), id],
            ).map_err(|e| format!("Failed to reorder token: {}", e))?;
        }

        tx.commit().map_err(|e| format!("Failed to commit reorder: {}", e))?;
        Ok(())
    }

    /// Increment HOTP counter and return the new value
    pub fn increment_counter(&self, id: &str) -> Result<u64, String> {
        let now = Utc::now().to_rfc3339();
        self.conn().execute(
            "UPDATE tokens SET counter = counter + 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        ).map_err(|e| format!("Failed to increment counter: {}", e))?;

        let counter: u64 = self.conn().query_row(
            "SELECT counter FROM tokens WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        ).map_err(|e| format!("Token not found: {}", e))?;

        Ok(counter)
    }
}
