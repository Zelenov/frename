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
    Migration {
        version: 19,
        sql: schema::M19_PLAYBACK_POSITION,
    },
    Migration {
        version: 20,
        sql: schema::M20_AI_MOMENTS,
    },
    Migration {
        version: 21,
        sql: schema::M21_AI_TAG_SUGGESTIONS,
    },
    Migration {
        version: 22,
        sql: schema::M22_RECENT_FOLDERS,
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
            "playback_position",
            "recent_folders",
        ] {
            assert!(table_exists(&conn, table), "{table} must survive");
        }
    }

    #[test]
    fn running_migrations_twice_is_a_no_op() {
        let conn = database_at_version_1();
        run(&conn).expect("first run");
        run(&conn).expect("second run");
        assert_eq!(current_version(&conn).expect("version"), 22);
    }

    #[test]
    fn a_database_with_the_spend_ledger_loses_it_and_a_fresh_one_ends_up_without_it() {
        let conn = database_at_version_1();
        for m in MIGRATIONS.iter().filter(|m| (2..=17).contains(&m.version)) {
            conn.execute_batch(m.sql).expect("migration");
        }
        conn.execute("INSERT INTO ai_spend VALUES ('soniox', 0, 1.5)", [])
            .expect("a ledger row");
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

    #[test]
    fn a_version_19_database_gets_only_what_stands_out() {
        let conn = database_at_version_1();
        for m in MIGRATIONS.iter().filter(|m| (2..=19).contains(&m.version)) {
            conn.execute_batch(m.sql).expect("migration");
        }
        conn.execute("UPDATE schema_version SET version = 19", [])
            .expect("set version");
        conn.execute("INSERT INTO app_settings (id) VALUES (1)", [])
            .expect("settings row");
        run(&conn).expect("migrate");
        let moments: String = conn
            .query_row(
                "SELECT ai_moments FROM app_settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .expect("ai_moments column");
        assert_eq!(moments, "important");
    }

    /// A database at version 21 whose `folder_history` holds `count` folders, `/f/0` the oldest.
    fn database_with_folder_history(count: usize) -> Connection {
        let conn = database_at_version_1();
        for m in MIGRATIONS.iter().filter(|m| (2..=21).contains(&m.version)) {
            conn.execute_batch(m.sql).expect("migration");
        }
        conn.execute("UPDATE schema_version SET version = 21", [])
            .expect("set version");
        for n in 0..count {
            conn.execute(
                "INSERT INTO folder_history (opened_at, folder_path, last_file_path)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    format!("2026-09-{:02} 10:00:00", n + 1),
                    format!("/f/{n}"),
                    if n == 0 {
                        String::new()
                    } else {
                        format!("/f/{n}/a.mp4")
                    },
                ],
            )
            .expect("history row");
        }
        conn
    }

    #[test]
    fn the_recent_folders_start_from_the_ten_latest_of_the_folder_history() {
        let conn = database_with_folder_history(12);
        run(&conn).expect("migrate");
        let rows: Vec<(i64, String, i64, String)> = conn
            .prepare(
                "SELECT position, folder_path, opened_at_ms, last_file_path
                 FROM recent_folders ORDER BY position",
            )
            .expect("prepare")
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows");
        assert_eq!(rows.len(), 10);
        assert_eq!(
            (rows[0].0, rows[0].1.as_str(), rows[0].3.as_str()),
            (0, "/f/11", "/f/11/a.mp4"),
            "the newest first"
        );
        assert_eq!(rows[9].1, "/f/2");
        // 2026-09-12 10:00:00 UTC.
        assert_eq!(rows[0].2, 1_789_207_200_000);
    }

    #[test]
    fn a_database_without_history_gets_an_empty_recent_list() {
        let conn = Connection::open_in_memory().expect("open");
        run(&conn).expect("migrate");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM recent_folders", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 0);
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
