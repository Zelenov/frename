//! Runs the app state DB schema. Two migrations only: initial migration table, then folder history (v1.0 prep). No more until told.

use rusqlite::Connection;

use super::schema;

/// One-time migration: version number and SQL to run.
struct Migration {
    version: u32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: schema::M1_SCHEMA_VERSION,
    },
    Migration {
        version: 2,
        sql: schema::M2_FOLDER_HISTORY,
    },
];

/// Returns the current schema version. Ensures schema_version table and initial row exist (CREATE IF NOT EXISTS / INSERT WHERE NOT EXISTS).
fn current_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
    conn.execute_batch(schema::M1_SCHEMA_VERSION)?;
    conn.query_row("SELECT version FROM schema_version LIMIT 1", [], |row| row.get(0))
}

/// Runs migrations with version greater than the stored version.
pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut version = current_version(conn)?;
    for m in MIGRATIONS {
        if m.version > version {
            conn.execute_batch(m.sql)?;
            conn.execute("UPDATE schema_version SET version = ?1", [m.version])?;
            version = m.version;
        }
    }
    Ok(())
}
