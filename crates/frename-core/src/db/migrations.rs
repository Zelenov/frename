//! Runs the app state DB schema migrations.

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
        sql: schema::M1_FULL_SCHEMA,
    },
    Migration {
        version: 2,
        sql: schema::M2_PANEL_WIDTHS,
    },
];

/// Returns the current schema version, bootstrapping schema_version if needed.
fn current_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
    conn.execute_batch(schema::BOOTSTRAP)?;
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
