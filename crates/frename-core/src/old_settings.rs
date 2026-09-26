//! Settings from an older frename: the zip versions kept `frename.db` next to `frename.exe`,
//! wherever the user unpacked them. An installed frename keeps its own database elsewhere, so on
//! its first start it looks for an old one and offers to import it, and Settings can schedule an
//! import from a folder the search missed.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rusqlite::{Connection, OpenFlags};

/// The database file name, in the data folder and next to an old `frename.exe`.
pub const DATABASE_FILE: &str = "frename.db";

/// How deep below each searched folder an unpacked zip is looked for: `Downloads\frename\` is
/// depth 1, `Downloads\frename-windows-x64-v0.66.0\frename\` depth 2.
pub const SEARCH_DEPTH: usize = 2;

/// Records a folder to import at the next start, in the data folder.
const PENDING_IMPORT_FILE: &str = "import-settings-from.txt";

/// The database moved aside by an import scheduled from Settings.
const REPLACED_DATABASE_FILE: &str = "frename.db.before-import";

/// Where a scheduled import is written before it replaces the database.
const IMPORTING_DATABASE_FILE: &str = "frename.db.importing";

/// Marks that the first-start offer was closed without an answer, so it comes back.
const ASK_AGAIN_FILE: &str = "import-settings-ask-again";

/// Whether `folder` holds an old frename: a `frename.db` with a `frename.exe` next to it.
pub fn is_old_frename_folder(folder: &Path) -> bool {
    folder.join(DATABASE_FILE).is_file() && folder.join("frename.exe").is_file()
}

/// Folders with an old frename in `roots` and their subfolders up to `depth` levels down, the most
/// recently used database first. Unreadable folders are skipped.
pub fn find_old_frename_folders(roots: &[PathBuf], depth: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in roots {
        collect_old_frename_folders(root, depth, &mut found);
    }
    found.sort();
    found.dedup();
    let modified = |folder: &PathBuf| {
        std::fs::metadata(folder.join(DATABASE_FILE))
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    };
    found.sort_by_key(|folder| std::cmp::Reverse(modified(folder)));
    found
}

fn collect_old_frename_folders(folder: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if is_old_frename_folder(folder) {
        found.push(folder.to_path_buf());
    }
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        // Not `Path::is_dir`: it follows junctions, which could lead the search anywhere.
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            collect_old_frename_folders(&entry.path(), depth - 1, found);
        }
    }
}

/// Copy the database in `old_folder` to `new_database`, which must not exist yet.
///
/// The old frename kept its database in WAL mode, so recent changes may still be in
/// `frename.db-wal`; `VACUUM INTO` reads them, where copying the file alone would miss them.
/// The old database is opened read-only and left as it was.
pub fn import_database(old_folder: &Path, new_database: &Path) -> Result<(), rusqlite::Error> {
    let old = Connection::open_with_flags(
        old_folder.join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    old.execute("VACUUM INTO ?1", [new_database.to_string_lossy().as_ref()])?;
    Ok(())
}

/// Schedule importing the database in `old_folder` at the next start; the data folder's database
/// is open while frename runs, and `VACUUM INTO` needs a target that does not exist yet.
pub fn schedule_import(data_dir: &Path, old_folder: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(
        data_dir.join(PENDING_IMPORT_FILE),
        old_folder.to_string_lossy().as_bytes(),
    )
}

/// The folder an import was scheduled from, if one is waiting.
pub fn scheduled_import(data_dir: &Path) -> Option<PathBuf> {
    let folder = std::fs::read_to_string(data_dir.join(PENDING_IMPORT_FILE)).ok()?;
    let folder = folder.trim();
    (!folder.is_empty()).then(|| PathBuf::from(folder))
}

/// Run the import scheduled from Settings, if any, before the database is opened. The schedule is
/// removed first, so a failing import is not retried on every start.
pub fn run_scheduled_import(data_dir: &Path) -> Result<Option<PathBuf>, String> {
    let Some(old_folder) = scheduled_import(data_dir) else {
        return Ok(None);
    };
    std::fs::remove_file(data_dir.join(PENDING_IMPORT_FILE)).map_err(|e| e.to_string())?;
    import_into_data_dir(data_dir, &old_folder)?;
    Ok(Some(old_folder))
}

/// Make the database in `old_folder` the data folder's database. A database already there is
/// kept as `frename.db.before-import`, and replaced only once the import succeeded.
pub fn import_into_data_dir(data_dir: &Path, old_folder: &Path) -> Result<(), String> {
    std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let imported = data_dir.join(IMPORTING_DATABASE_FILE);
    let _ = std::fs::remove_file(&imported);
    import_database(old_folder, &imported).map_err(|e| e.to_string())?;

    let database = data_dir.join(DATABASE_FILE);
    if database.exists() {
        let replaced = data_dir.join(REPLACED_DATABASE_FILE);
        let _ = std::fs::remove_file(&replaced);
        std::fs::rename(&database, &replaced).map_err(|e| e.to_string())?;
        // A clean close leaves no WAL behind; a crash may. It belongs to the database moved aside.
        for suffix in ["-wal", "-shm"] {
            let _ = std::fs::remove_file(data_dir.join(format!("{DATABASE_FILE}{suffix}")));
        }
    }
    std::fs::rename(&imported, &database).map_err(|e| e.to_string())
}

/// Whether the first-start offer to import old settings should be made: the data folder has no
/// database yet, or the user closed the offer last time without answering.
pub fn import_offer_due(data_dir: &Path) -> bool {
    !data_dir.join(DATABASE_FILE).exists() || data_dir.join(ASK_AGAIN_FILE).exists()
}

/// Record whether the first-start offer comes back at the next start (it was closed without an
/// answer) or not (it was answered).
pub fn set_import_offer_again(data_dir: &Path, again: bool) -> std::io::Result<()> {
    let marker = data_dir.join(ASK_AGAIN_FILE);
    if again {
        std::fs::create_dir_all(data_dir)?;
        std::fs::write(marker, b"")
    } else {
        match std::fs::remove_file(marker) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty folder of its own in the temp dir.
    fn temp_folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "frename-old-settings-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    /// An old frename in `folder`: its exe and a WAL database holding `value`, with the change
    /// still in the WAL. The connection is returned so the WAL is not checkpointed away.
    fn old_frename(folder: &Path, value: &str) -> Connection {
        std::fs::create_dir_all(folder).expect("folder");
        std::fs::write(folder.join("frename.exe"), b"MZ").expect("exe");
        let conn = Connection::open(folder.join(DATABASE_FILE)).expect("open");
        let _: String = conn
            .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
            .expect("wal");
        conn.pragma_update(None, "wal_autocheckpoint", 0)
            .expect("no checkpoint");
        conn.execute_batch("CREATE TABLE t (v TEXT)")
            .expect("table");
        conn.execute("INSERT INTO t (v) VALUES (?1)", [value])
            .expect("insert");
        conn
    }

    fn value_in(database: &Path) -> String {
        Connection::open(database)
            .expect("open")
            .query_row("SELECT v FROM t", [], |row| row.get(0))
            .expect("value")
    }

    #[test]
    fn the_import_keeps_changes_still_in_the_wal_and_leaves_the_old_files_alone() {
        let root = temp_folder("wal");
        let old = root.join("old");
        let _open = old_frename(&old, "recent change");
        assert!(old.join("frename.db-wal").exists(), "precondition");
        let before = std::fs::read(old.join(DATABASE_FILE)).expect("read");

        let new = root.join("new.db");
        import_database(&old, &new).expect("import");

        assert_eq!(value_in(&new), "recent change");
        assert_eq!(
            std::fs::read(old.join(DATABASE_FILE)).expect("read"),
            before
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_search_finds_old_frename_folders_two_levels_down_newest_first() {
        let root = temp_folder("search");
        let downloads = root.join("Downloads");
        let _a = old_frename(&downloads.join("frename"), "a");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _b = old_frename(&downloads.join("frename-v0.66").join("frename"), "b");
        let _too_deep = old_frename(&downloads.join("x").join("y").join("frename"), "c");
        // A database without an exe next to it is not an old frename.
        let lonely = downloads.join("backup");
        std::fs::create_dir_all(&lonely).expect("folder");
        std::fs::write(lonely.join(DATABASE_FILE), b"").expect("db");

        let found = find_old_frename_folders(std::slice::from_ref(&downloads), SEARCH_DEPTH);

        assert_eq!(
            found,
            vec![
                downloads.join("frename-v0.66").join("frename"),
                downloads.join("frename"),
            ]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_search_folders_find_nothing() {
        let root = temp_folder("missing");
        assert!(find_old_frename_folders(&[root.join("nowhere")], SEARCH_DEPTH).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_scheduled_import_replaces_the_database_once() {
        let root = temp_folder("scheduled");
        let old = root.join("old");
        let _open = old_frename(&old, "old settings");
        let data = root.join("data");
        std::fs::create_dir_all(&data).expect("data");
        let current = Connection::open(data.join(DATABASE_FILE)).expect("open");
        current
            .execute_batch("CREATE TABLE t (v TEXT); INSERT INTO t (v) VALUES ('current');")
            .expect("current");
        drop(current);

        assert_eq!(run_scheduled_import(&data), Ok(None), "nothing scheduled");
        schedule_import(&data, &old).expect("schedule");
        assert_eq!(scheduled_import(&data), Some(old.clone()));

        assert_eq!(run_scheduled_import(&data), Ok(Some(old.clone())));
        assert_eq!(value_in(&data.join(DATABASE_FILE)), "old settings");
        assert_eq!(value_in(&data.join(REPLACED_DATABASE_FILE)), "current");
        assert_eq!(scheduled_import(&data), None);
        assert_eq!(run_scheduled_import(&data), Ok(None), "done once");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_offer_is_due_without_a_database_or_after_it_was_closed_unanswered() {
        let data = temp_folder("offer");
        assert!(import_offer_due(&data));
        std::fs::write(data.join(DATABASE_FILE), b"").expect("db");
        assert!(!import_offer_due(&data));
        set_import_offer_again(&data, true).expect("again");
        assert!(import_offer_due(&data));
        set_import_offer_again(&data, false).expect("answered");
        assert!(!import_offer_due(&data));
        set_import_offer_again(&data, false).expect("answering twice is fine");
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn a_failed_scheduled_import_keeps_the_database_and_is_not_retried() {
        let root = temp_folder("failed");
        let data = root.join("data");
        std::fs::create_dir_all(&data).expect("data");
        let current = Connection::open(data.join(DATABASE_FILE)).expect("open");
        current
            .execute_batch("CREATE TABLE t (v TEXT); INSERT INTO t (v) VALUES ('current');")
            .expect("current");
        drop(current);
        schedule_import(&data, &root.join("gone")).expect("schedule");

        assert!(run_scheduled_import(&data).is_err());
        assert_eq!(value_in(&data.join(DATABASE_FILE)), "current");
        assert_eq!(scheduled_import(&data), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
