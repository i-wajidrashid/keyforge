//! Versioned schema migrations

use rusqlite::Connection;

use crate::constants::SCHEMA_VERSION;
use crate::error::VaultError;

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| VaultError::Migration(e.to_string()))?;

    let current_version = get_current_version(conn)?;

    if current_version < SCHEMA_VERSION {
        migrate_v1(conn)?;
    }

    Ok(())
}

fn get_current_version(conn: &Connection) -> Result<i32, String> {
    let version: Result<i32, _> = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM migrations",
        [],
        |row| row.get(0),
    );
    version.map_err(|e| VaultError::SchemaVersion(e.to_string()).to_string())
}

fn migrate_v1(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tokens (
            id TEXT PRIMARY KEY,
            issuer TEXT NOT NULL,
            account TEXT NOT NULL DEFAULT '',
            secret_encrypted BLOB NOT NULL,
            algorithm TEXT NOT NULL DEFAULT 'SHA1',
            digits INTEGER NOT NULL DEFAULT 6,
            type TEXT NOT NULL DEFAULT 'totp',
            period INTEGER NOT NULL DEFAULT 30,
            counter INTEGER NOT NULL DEFAULT 0,
            icon TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_modified TEXT,
            device_id TEXT,
            sync_version INTEGER
        );

        CREATE TABLE IF NOT EXISTS vault_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO migrations (version, applied_at) VALUES (1, datetime('now'));
        INSERT OR IGNORE INTO vault_meta (key, value) VALUES ('schema_version', '1');
        INSERT OR IGNORE INTO vault_meta (key, value) VALUES ('vault_created_at', datetime('now'));
        ",
    )
    .map_err(|e| VaultError::Migration(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_test_db() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_fresh_migration() {
        let conn = open_test_db();
        run_migrations(&conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_idempotent_migration() {
        let conn = open_test_db();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // Should be no-op

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_tables_created() {
        let conn = open_test_db();
        run_migrations(&conn).unwrap();

        // Verify tokens table exists
        let count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='tokens'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify vault_meta table exists
        let count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='vault_meta'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
