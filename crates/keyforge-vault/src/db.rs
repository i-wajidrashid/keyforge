//! SQLCipher vault

use rusqlite::Connection;
use zeroize::Zeroize;

use crate::error::VaultError;
use crate::migrations;

pub struct Vault {
    conn: Connection,
    secret_key: [u8; 32],
}

impl Vault {
    /// Create a new encrypted vault at `path`.
    pub fn create(
        path: &str,
        sqlcipher_key: &[u8; 32],
        secret_key: [u8; 32],
    ) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| VaultError::DatabaseOpen(e.to_string()))?;

        Self::set_key(&conn, sqlcipher_key)?;

        let vault = Vault { conn, secret_key };
        migrations::run_migrations(&vault.conn)?;

        Ok(vault)
    }

    /// Open an existing encrypted vault.
    pub fn open(
        path: &str,
        sqlcipher_key: &[u8; 32],
        secret_key: [u8; 32],
    ) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| VaultError::DatabaseOpen(e.to_string()))?;

        Self::set_key(&conn, sqlcipher_key)?;

        let vault = Vault { conn, secret_key };
        migrations::run_migrations(&vault.conn)?;

        Ok(vault)
    }

    fn set_key(conn: &Connection, key: &[u8; 32]) -> Result<(), String> {
        let mut hex_key: String = key.iter().map(|b| format!("{:02x}", b)).collect();
        let mut pragma_value = format!("x'{}'", hex_key);
        let result = conn
            .pragma_update(None, "key", &pragma_value)
            .map_err(|e| VaultError::SetEncryptionKey(e.to_string()));

        // Zeroize key material from heap strings
        hex_key.zeroize();
        pragma_value.zeroize();
        result?;

        conn.execute_batch("SELECT count(*) FROM sqlite_master;")
            .map_err(|_| VaultError::WrongPasswordOrCorrupted)?;

        Ok(())
    }

    /// Get a reference to the database connection.
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get the secret encryption key.
    pub(crate) fn secret_key(&self) -> &[u8; 32] {
        &self.secret_key
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        self.secret_key.zeroize();
    }
}
