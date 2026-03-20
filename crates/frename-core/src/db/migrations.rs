//! Runs the app state DB schema. SQL migrations only (schema + seed data in SQL).

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
    Migration {
        version: 3,
        sql: schema::M3_STORED_TAGS,
    },
    Migration {
        version: 4,
        sql: schema::M4_TAG_COLOR_INDEX,
    },
    Migration {
        version: 5,
        sql: schema::M5_TAG_COLOR_MAPPING,
    },
    Migration {
        version: 6,
        sql: schema::M6_STORED_TAGS_UUID,
    },
    Migration {
        version: 7,
        sql: schema::M7_STARRED_COLUMN,
    },
];

/// Returns the current schema version.
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
