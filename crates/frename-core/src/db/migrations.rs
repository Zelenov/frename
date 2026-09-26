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
        sql: schema::M2_DROP_TAG_TABLES,
    },
    Migration {
        version: 3,
        sql: schema::M3_APP_SETTINGS,
    },
    Migration {
        version: 4,
        sql: schema::M4_COMMENT_STORAGE,
    },
    Migration {
        version: 5,
        sql: schema::M5_IN_OUT_STORAGE,
    },
    Migration {
        version: 6,
        sql: schema::M6_COMMENTED_TAG,
    },
    Migration {
        version: 7,
        sql: schema::M7_COMMENTED_TAG_ENABLED,
    },
    Migration {
        version: 8,
        sql: schema::M8_SPACE_AFTER_TAGS,
    },
    Migration {
        version: 9,
        sql: schema::M9_SUBTITLE_SETTINGS,
    },
];

/// Returns the current schema version, bootstrapping schema_version if needed.
fn current_version(conn: &Connection) -> Result<u32, rusqlite::Error> {
    conn.execute_batch(schema::BOOTSTRAP)?;
    conn.query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
        row.get(0)
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A database at the shipped M1 schema: tag tables present, version 1.
    fn database_at_version_1() -> Connection {
        let conn = Connection::open_in_memory().expect("open");
        conn.execute_batch(schema::BOOTSTRAP).expect("bootstrap");
        conn.execute_batch(schema::M1_FULL_SCHEMA).expect("m1");
        conn.execute("UPDATE schema_version SET version = 1", [])
            .expect("set version");
        conn
    }

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .expect("query sqlite_master")
            > 0
    }

    #[test]
    fn migrating_an_existing_database_drops_the_tag_tables() {
        let conn = database_at_version_1();
        assert!(table_exists(&conn, "stored_tags"), "precondition");
        run(&conn).expect("migrate");
        assert!(!table_exists(&conn, "stored_tags"));
        assert!(!table_exists(&conn, "tag_color_mapping"));
    }

    #[test]
    fn a_fresh_database_ends_up_without_the_tag_tables() {
        let conn = Connection::open_in_memory().expect("open");
        run(&conn).expect("migrate");
        assert!(!table_exists(&conn, "stored_tags"));
        assert!(!table_exists(&conn, "tag_color_mapping"));
    }

    #[test]
    fn migrating_keeps_the_tables_the_app_still_uses() {
        let conn = database_at_version_1();
        run(&conn).expect("migrate");
        for table in [
            "folder_history",
            "window_state",
            "video_settings",
            "app_settings",
        ] {
            assert!(table_exists(&conn, table), "{table} must survive");
        }
    }

    #[test]
    fn running_migrations_twice_is_a_no_op() {
        let conn = database_at_version_1();
        run(&conn).expect("first run");
        run(&conn).expect("second run");
        assert_eq!(current_version(&conn).expect("version"), 9);
    }
}
