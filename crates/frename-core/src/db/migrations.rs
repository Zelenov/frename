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
        sql: schema::M9_UPDATE_CHECK,
    },
    Migration {
        version: 10,
        sql: schema::M10_SUMMARY_LANGUAGE,
    },
    Migration {
        version: 11,
        sql: schema::M11_AI_MODEL,
    },
    Migration {
        version: 12,
        sql: schema::M12_SUBTITLE_SETTINGS,
    },
    Migration {
        version: 13,
        sql: schema::M13_MARKER_STORAGE,
    },
    Migration {
        version: 14,
        sql: schema::M14_UI_LANGUAGE,
    },
    Migration {
        version: 15,
        sql: schema::M15_IN_OUT_OUT_OF_NAMES,
    },
    Migration {
        version: 16,
        sql: schema::M16_BATCH_RUN,
    },
    Migration {
        version: 17,
        sql: schema::M17_AI_LEDGER,
    },
    Migration {
        version: 18,
        sql: schema::M18_DROP_AI_LEDGER,
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
        assert_eq!(current_version(&conn).expect("version"), 18);
    }

    #[test]
    fn a_database_with_the_old_spend_ledger_loses_it_and_a_fresh_one_never_has_it() {
        let conn = database_at_version_1();
        for m in MIGRATIONS.iter().filter(|m| (2..=16).contains(&m.version)) {
            conn.execute_batch(m.sql).expect("migration");
        }
        // The ledger as migration 17 created it before #172.
        conn.execute_batch(
            "CREATE TABLE ai_spend (service TEXT NOT NULL, at_ms INTEGER NOT NULL, usd REAL NOT NULL);
             CREATE INDEX ai_spend_service_at ON ai_spend (service, at_ms);
             CREATE TABLE ai_top_up (service TEXT PRIMARY KEY, usd REAL NOT NULL, at_ms INTEGER NOT NULL);
             INSERT INTO ai_spend VALUES ('soniox', 0, 1.5);",
        )
        .expect("old ledger");
        conn.execute("UPDATE schema_version SET version = 17", [])
            .expect("set version");
        run(&conn).expect("migrate");
        assert!(!table_exists(&conn, "ai_spend"));
        assert!(!table_exists(&conn, "ai_top_up"));

        let fresh = Connection::open_in_memory().expect("open");
        run(&fresh).expect("migrate");
        assert!(!table_exists(&fresh, "ai_spend"));
        assert!(!table_exists(&fresh, "ai_top_up"));
    }

    #[test]
    fn a_version_13_database_gains_an_empty_ui_language() {
        let conn = database_at_version_1();
        for m in MIGRATIONS.iter().filter(|m| (2..=13).contains(&m.version)) {
            conn.execute_batch(m.sql).expect("migration");
        }
        conn.execute("UPDATE schema_version SET version = 13", [])
            .expect("set version");
        conn.execute("INSERT INTO app_settings (id) VALUES (1)", [])
            .expect("settings row");
        run(&conn).expect("migrate");
        let language: String = conn
            .query_row(
                "SELECT ui_language FROM app_settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .expect("ui_language column");
        assert_eq!(language, "");
    }

    /// A database at `version` with a settings row whose in/out storage is `in_out`.
    fn database_with_in_out_storage(version: u32, in_out: &str) -> Connection {
        let conn = database_at_version_1();
        for m in MIGRATIONS
            .iter()
            .filter(|m| (2..=version).contains(&m.version))
        {
            conn.execute_batch(m.sql).expect("migration");
        }
        conn.execute("UPDATE schema_version SET version = ?1", [version])
            .expect("set version");
        conn.execute(
            "INSERT INTO app_settings (id, in_out_storage) VALUES (1, ?1)",
            [in_out],
        )
        .expect("settings row");
        conn
    }

    fn in_out_storage(conn: &Connection) -> String {
        conn.query_row(
            "SELECT in_out_storage FROM app_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("in_out_storage")
    }

    #[test]
    fn in_out_points_kept_in_file_names_move_to_the_video() {
        let conn = database_with_in_out_storage(14, "file_name");
        run(&conn).expect("migrate");
        assert_eq!(in_out_storage(&conn), "xmp");
    }

    #[test]
    fn in_out_points_kept_in_the_video_stay_there() {
        let conn = database_with_in_out_storage(14, "xmp");
        run(&conn).expect("migrate");
        assert_eq!(in_out_storage(&conn), "xmp");
    }

    #[test]
    fn a_row_inserted_after_migration_15_with_the_old_column_default_reads_as_in_video() {
        // Migration 15 cannot change the column's default ('file_name', from migration 5);
        // the app always writes the column, and reading maps the stale value to the video.
        let conn = Connection::open_in_memory().expect("open");
        run(&conn).expect("migrate");
        conn.execute("INSERT INTO app_settings (id) VALUES (1)", [])
            .expect("settings row");
        assert_eq!(
            crate::InOutStorage::from_name(&in_out_storage(&conn)),
            crate::InOutStorage::InVideo
        );
    }

    #[test]
    fn a_default_row_from_migration_5_moves_to_the_video() {
        let conn = database_at_version_1();
        for m in MIGRATIONS.iter().filter(|m| (2..=14).contains(&m.version)) {
            conn.execute_batch(m.sql).expect("migration");
        }
        conn.execute("UPDATE schema_version SET version = 14", [])
            .expect("set version");
        conn.execute("INSERT INTO app_settings (id) VALUES (1)", [])
            .expect("settings row");
        assert_eq!(in_out_storage(&conn), "file_name", "precondition");
        run(&conn).expect("migrate");
        assert_eq!(in_out_storage(&conn), "xmp");
    }
}
