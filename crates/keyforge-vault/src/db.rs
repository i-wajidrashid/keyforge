//! SQLCipher vault

use rusqlite::Connection;
use zeroize::Zeroize;

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
        let conn = Connection::open(path).map_err(|e| format!("Failed to create vault: {}", e))?;

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
        let conn = Connection::open(path).map_err(|e| format!("Failed to open vault: {}", e))?;

        Self::set_key(&conn, sqlcipher_key)?;

        let vault = Vault { conn, secret_key };
        migrations::run_migrations(&vault.conn)?;

        Ok(vault)
    }

    fn set_key(conn: &Connection, key: &[u8; 32]) -> Result<(), String> {
        let hex_key: String = key.iter().map(|b| format!("{:02x}", b)).collect();
        conn.pragma_update(None, "key", format!("x'{}'", hex_key))
            .map_err(|e| format!("Failed to set encryption key: {}", e))?;

        conn.execute_batch("SELECT count(*) FROM sqlite_master;")
            .map_err(|_| "Wrong password or corrupted vault".to_string())?;

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
