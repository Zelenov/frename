//! Directory scanning and file list management.
//!
//! Files are stored in two structures:
//!   1. `files_by_id: HashMap<FileId, File>` — O(1) lookup and mutation by stable ID.
//!   2. `order: Vec<FileId>`                 — modification-date order for indexed access.
//!
//! Selection is stored as `selected_id: Option<FileId>`, stable across renames.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::db::AppStateStore;
use crate::{File, FileId, FileSnapshot, FileTagger, FolderAndFile, FolderInfo};

/// A scanned directory. Generic over the store type S; store is used only for session persistence.
#[derive(Clone, Debug)]
pub struct Directory<S> {
    path: Box<Path>,
    files_by_id: HashMap<FileId, File>,
    /// Creation-date order; stable across renames.
    order: Vec<FileId>,
    /// Identity of the currently selected file (stable across renames).
    selected_id: Option<FileId>,
    /// When true, only files without tags are listed (the selected file always stays listed).
    untagged_only: bool,
    store: S,
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl<S: AppStateStore + Clone> Directory<S> {
    /// Create a directory from an already-sorted file list. For testing and programmatic use.
    pub fn with_files(path: impl AsRef<Path>, files: Vec<File>, store: S) -> Self {
        let order: Vec<FileId> = files.iter().map(|f| f.id()).collect();
        let files_by_id: HashMap<FileId, File> = files.into_iter().map(|f| (f.id(), f)).collect();
        Self {
            path: path.as_ref().to_path_buf().into_boxed_path(),
            files_by_id,
            order,
            selected_id: None,
            untagged_only: false,
            store,
        }
    }

    /// Open a directory asynchronously: scan all files and sort by modification date.
    pub async fn open(directory: &Path, store: S) -> Result<Self, std::io::Error> {
        log::info!("Scanning directory: {}", directory.display());
        let scan_path = directory.to_path_buf();
        let files = tokio::task::spawn_blocking(move || scan_files(&scan_path))
            .await
            .map_err(|e| {
                log::error!("Directory scan task failed: {e}");
                std::io::Error::other(e)
            })?
            .map_err(|e| { log::error!("Failed to scan directory: {e}"); e })?;
        store.set_last_folder_and_file(&FolderAndFile::new(directory, None::<PathBuf>));
        log::info!("Directory scan complete: {} files found", files.len());
        Ok(Self::with_files(directory, files, store))
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// True when the directory holds no files at all (regardless of the untagged filter).
    pub fn is_empty(&self) -> bool { self.order.is_empty() }

    /// Whether the "untagged only" filter is active.
    pub fn untagged_only(&self) -> bool { self.untagged_only }

    /// Turn the "untagged only" filter on or off. Selection is kept: the selected file stays
    /// listed even once it has tags, so the row under the cursor never disappears under the user.
    pub fn set_untagged_only(&mut self, untagged_only: bool) {
        self.untagged_only = untagged_only;
    }

    /// Number of files without tags (ignores the filter and the selection exemption).
    pub fn untagged_count(&self) -> usize {
        self.files_by_id
            .values()
            .filter(|f| f.snapshot().tags().is_empty())
            .count()
    }

    /// Whether a file is part of the listed set under the current filter.
    /// The selected file is always listed: tagging it must not pull the row out from under the cursor.
    fn is_listed(&self, file: &File) -> bool {
        !self.untagged_only
            || self.selected_id == Some(file.id())
            || file.snapshot().tags().is_empty()
    }

    /// Files in modification-date order, filtered (for rendering the list).
    /// All index-based operations (`selected_index`, `select_index`, prev/next) use this same order.
    pub fn files_in_order(&self) -> impl Iterator<Item = &File> {
        self.order
            .iter()
            .filter_map(|id| self.files_by_id.get(id))
            .filter(|file| self.is_listed(file))
    }

    /// IDs of the listed files, in list order.
    fn listed_ids(&self) -> impl Iterator<Item = FileId> + '_ {
        self.files_in_order().map(|f| f.id())
    }

    /// Number of listed files under the current filter.
    pub fn listed_count(&self) -> usize {
        self.files_in_order().count()
    }

    /// Look up a file by its stable ID. O(1). Ignores the filter.
    pub fn file_by_id(&self, id: FileId) -> Option<&File> {
        self.files_by_id.get(&id)
    }

    /// Index of the selected file within the listed files.
    pub fn selected_index(&self) -> Option<usize> {
        let selected = self.selected_id?;
        self.listed_ids().position(|id| id == selected)
    }

    pub fn selected_file(&self) -> Option<&File> {
        self.selected_id.and_then(|id| self.files_by_id.get(&id))
    }

    pub fn has_previous_next(&self) -> (bool, bool) {
        let idx = self.selected_index().unwrap_or(0);
        (idx > 0, idx + 1 < self.listed_count())
    }

    // -----------------------------------------------------------------------
    // Selection
    // -----------------------------------------------------------------------

    /// Open a file by path. Returns the file if it is in this directory and was opened.
    pub fn open_path(&mut self, path: &Path) -> Option<File> {
        let id = self.files_by_id.values().find(|f| f.file_path() == path)?.id();
        if self.selected_id == Some(id) { return None; }
        self.select_by_id(id)
    }

    /// Select by stable ID. O(1) existence check. Returns the selected file if found.
    pub(crate) fn select_by_id(&mut self, id: FileId) -> Option<File> {
        if !self.files_by_id.contains_key(&id) { return None; }
        self.selected_id = Some(id);
        self.persist_session();
        self.files_by_id.get(&id).cloned()
    }

    /// Select by index within the listed files (same order the list is rendered in).
    pub fn select_index(&mut self, index: usize) -> Option<File> {
        let id = self.listed_ids().nth(index)?;
        self.select_by_id(id)
    }

    pub fn select_previous(&mut self) -> Option<File> {
        let current = self.selected_index().unwrap_or(0);
        if current == 0 { return None; }
        self.select_index(current - 1)
    }

    pub fn select_next(&mut self) -> Option<File> {
        let current = self.selected_index().unwrap_or(0);
        if current + 1 >= self.listed_count() { return None; }
        self.select_index(current + 1)
    }

    // -----------------------------------------------------------------------
    // File mutation (rename / update)
    // -----------------------------------------------------------------------

    /// Update both the path and snapshot of the file identified by `id`. O(1).
    /// Returns true if the file was found.
    pub fn rename_file(&mut self, id: FileId, new_path: &Path, snapshot: &FileSnapshot) -> bool {
        match self.files_by_id.get_mut(&id) {
            Some(file) => {
                let old_path = file.file_path().to_path_buf();
                file.set_file_path(new_path);
                file.set_file_snapshot(snapshot);
                if old_path != new_path {
                    log::info!(
                        "Directory: renamed {} → {} ({} tag(s))",
                        old_path.display(), new_path.display(), snapshot.tags().len()
                    );
                } else {
                    log::info!(
                        "Directory: updated {} ({} tag(s))",
                        new_path.display(), snapshot.tags().len()
                    );
                }
                true
            }
            None => {
                log::warn!("Directory::rename_file: id not found");
                false
            }
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn persist_session(&self) {
        self.store.set_last_folder_and_file(&FolderAndFile::new(
            self.path.as_ref(),
            self.selected_file().map(|f| f.file_path().to_path_buf()),
        ));
    }
}

/// Reads the folder in one pass and returns its files in modification-date order.
///
/// Blocking on purpose — it runs on the blocking pool. `std`'s `DirEntry` carries the metadata
/// Windows already returned from `FindNextFile`, so `file_type` and `metadata` cost nothing here.
/// The previous version enumerated the folder twice (once for [FolderInfo], once for the files)
/// and awaited a separate `metadata()` syscall per entry.
fn scan_files(directory: &Path) -> Result<Vec<File>, std::io::Error> {
    let entries: Vec<std::fs::DirEntry> =
        std::fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    let folder_info = FolderInfo::new(
        entries
            .iter()
            .filter_map(|e| e.file_name().to_str().map(str::to_string))
            .collect(),
    );

    let mut files = Vec::with_capacity(entries.len());
    for entry in &entries {
        let Ok(file_type) = entry.file_type() else { continue };
        let path = entry.path();
        // file_type does not follow symlinks, so linked media needs the extra stat to be seen.
        let is_file = if file_type.is_symlink() { path.is_file() } else { file_type.is_file() };
        if !is_file { continue; }
        if FileTagger::is_sidecar_file(&path) { continue; }
        // An unreadable entry sorts to the front rather than failing the whole scan.
        let modified_at = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        files.push(File::from_path_with_folder_info(path, modified_at, &folder_info));
    }
    files.sort_by(|a, b| a.modified_at().cmp(&b.modified_at()));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    use crate::db::fake_app_storage::FakeAppStorage;
    use crate::{Directory, File};

    /// Directory of `file_0.mp4` … `file_n.mp4`, all without tags.
    fn directory_with(names: &[&str]) -> Directory<FakeAppStorage> {
        let root = PathBuf::from("C:/test");
        let files: Vec<File> = names
            .iter()
            .map(|name| File::from_path(root.join(name), SystemTime::UNIX_EPOCH))
            .collect();
        Directory::with_files(root, files, FakeAppStorage::new())
    }

    fn listed_names(dir: &Directory<FakeAppStorage>) -> Vec<String> {
        dir.files_in_order()
            .map(|f| {
                f.file_path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string()
            })
            .collect()
    }

    /// Apply tags to a file the way a deferred rename does, by its listed position.
    fn tag_file_at(dir: &mut Directory<FakeAppStorage>, index: usize, tags: &[&str]) {
        let file = dir.files_in_order().nth(index).expect("file at index");
        let id = file.id();
        let mut snapshot = file.snapshot().clone();
        snapshot.set_tags(tags.iter().copied());
        let new_path: PathBuf = Path::new("C:/test").join(snapshot.file_name());
        dir.rename_file(id, &new_path, &snapshot);
    }

    #[test]
    fn filter_off_lists_every_file() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        tag_file_at(&mut dir, 0, &["Action"]);
        assert_eq!(listed_names(&dir).len(), 2);
    }

    #[test]
    fn filter_hides_tagged_files() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        tag_file_at(&mut dir, 1, &["Action"]);
        dir.set_untagged_only(true);
        assert_eq!(listed_names(&dir), vec!["a.mp4", "c.mp4"]);
    }

    #[test]
    fn selected_file_stays_listed_after_it_is_tagged() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        dir.set_untagged_only(true);
        dir.select_index(1);
        tag_file_at(&mut dir, 1, &["Action"]);
        assert_eq!(listed_names(&dir), vec!["a.mp4", "Action.b.mp4", "c.mp4"]);
        assert_eq!(dir.selected_index(), Some(1));
    }

    /// Leaving a file that was just tagged drops it from the list, and the cursor keeps the
    /// screen row it had: the new selection takes over the index the old file occupied.
    #[test]
    fn tagged_file_drops_out_once_the_cursor_leaves_it() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4", "d.mp4"]);
        dir.set_untagged_only(true);
        dir.select_index(1);
        tag_file_at(&mut dir, 1, &["Action"]);
        dir.select_next();
        assert_eq!(listed_names(&dir), vec!["a.mp4", "c.mp4", "d.mp4"]);
        assert_eq!(dir.selected_index(), Some(1), "cursor keeps its screen row");
        assert_eq!(
            dir.selected_file().map(|f| f.file_path().to_path_buf()),
            Some(PathBuf::from("C:/test/c.mp4"))
        );
    }

    #[test]
    fn next_skips_tagged_files() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        tag_file_at(&mut dir, 1, &["Action"]);
        dir.set_untagged_only(true);
        dir.select_index(0);
        let next = dir.select_next().expect("c.mp4 is the next untagged file");
        assert_eq!(next.file_path(), Path::new("C:/test/c.mp4"));
    }

    #[test]
    fn has_previous_next_follows_the_filtered_list() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        tag_file_at(&mut dir, 2, &["Action"]);
        dir.set_untagged_only(true);
        dir.select_index(1);
        assert_eq!(dir.has_previous_next(), (true, false));
    }

    #[test]
    fn untagged_count_ignores_the_filter() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        tag_file_at(&mut dir, 0, &["Action"]);
        dir.set_untagged_only(true);
        assert_eq!(dir.untagged_count(), 2);
    }
}
