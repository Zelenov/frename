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
use crate::search::{self, CommentFragment};
use crate::{File, FileId, FileKind, FileSnapshot, FileTagger, FolderAndFile, FolderInfo};

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
    /// When true, only files with a subtitle file are listed.
    subtitled_only: bool,
    /// When true, only files with a comment of the editor's are listed (an AI description alone
    /// does not count).
    commented_only: bool,
    /// When true, only files with clip markers are listed.
    marked_only: bool,
    /// Name filter as the user typed it (for display in the search bar).
    name_filter: String,
    /// The words of the filter, lowercased once, so matching a file never allocates.
    name_filter_words: Vec<String>,
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
            subtitled_only: false,
            commented_only: false,
            marked_only: false,
            name_filter: String::new(),
            name_filter_words: Vec::new(),
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
            .map_err(|e| {
                log::error!("Failed to scan directory: {e}");
                e
            })?;
        store.set_last_folder_and_file(&FolderAndFile::new(directory, None::<PathBuf>));
        let names: Vec<String> = files
            .iter()
            .filter_map(|f| f.file_path().file_name())
            .map(|name| name.to_string_lossy().to_string())
            .collect();
        store.tidy_playback_positions(directory, &names);
        log::info!("Directory scan complete: {} files found", files.len());
        Ok(Self::with_files(directory, files, store))
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// True when the directory holds no files at all (regardless of the untagged filter).
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Folder this directory was scanned from. Used to scope the folder's tag store.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the "untagged only" filter is active.
    pub fn untagged_only(&self) -> bool {
        self.untagged_only
    }

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

    /// Whether the "with subtitles only" filter is active.
    pub fn subtitled_only(&self) -> bool {
        self.subtitled_only
    }

    /// Turn the "with subtitles only" filter on or off. The selected file stays listed.
    pub fn set_subtitled_only(&mut self, subtitled_only: bool) {
        self.subtitled_only = subtitled_only;
    }

    /// Number of files with a subtitle file (ignores the filters).
    pub fn subtitled_count(&self) -> usize {
        self.files_by_id
            .values()
            .filter(|f| f.has_subtitles())
            .count()
    }

    /// Whether the "with comments only" filter is active.
    pub fn commented_only(&self) -> bool {
        self.commented_only
    }

    /// Turn the "with comments only" filter on or off. The selected file stays listed, even
    /// once its comment is cleared.
    pub fn set_commented_only(&mut self, commented_only: bool) {
        self.commented_only = commented_only;
    }

    /// Number of files with a comment (ignores the filters).
    pub fn commented_count(&self) -> usize {
        self.files_by_id
            .values()
            .filter(|f| crate::ai::has_editor_comment(f.comment()))
            .count()
    }

    /// Whether the "with markers only" filter is active.
    pub fn marked_only(&self) -> bool {
        self.marked_only
    }

    /// Turn the "with markers only" filter on or off. The selected file stays listed, even
    /// once its last marker is deleted.
    pub fn set_marked_only(&mut self, marked_only: bool) {
        self.marked_only = marked_only;
    }

    /// Number of files with clip markers (ignores the filters).
    pub fn marked_count(&self) -> usize {
        self.files_by_id
            .values()
            .filter(|f| f.snapshot().marker_count() > 0)
            .count()
    }

    /// Files whose comment is still loading (see [`FileSnapshot::comment_loading`]), in list
    /// order.
    pub fn files_loading_comments(&self) -> Vec<(FileId, PathBuf, FileSnapshot)> {
        self.order
            .iter()
            .filter_map(|id| self.files_by_id.get(id))
            .filter(|file| file.snapshot().comment_loading())
            .map(|file| {
                (
                    file.id(),
                    file.file_path().to_path_buf(),
                    file.snapshot().clone(),
                )
            })
            .collect()
    }

    /// Number of files whose comment is still loading.
    pub fn loading_comment_count(&self) -> usize {
        self.files_by_id
            .values()
            .filter(|f| f.snapshot().comment_loading())
            .count()
    }

    /// Take the snapshot loaded from `path` for a file whose comment was loading. Ignored when
    /// the file has since moved or has its comment already (it was opened, or saved).
    pub fn apply_loaded_comment(
        &mut self,
        id: FileId,
        path: &Path,
        snapshot: &FileSnapshot,
    ) -> bool {
        match self.files_by_id.get_mut(&id) {
            Some(file) if file.file_path() == path && file.snapshot().comment_loading() => {
                file.set_file_snapshot(snapshot);
                true
            }
            _ => false,
        }
    }

    /// Every file in the folder in list order, ignoring the filters.
    pub fn all_files(&self) -> impl Iterator<Item = &File> {
        self.order.iter().filter_map(|id| self.files_by_id.get(id))
    }

    /// Whether any filter that depends on a file's content (tags, subtitles, comment, markers)
    /// is on. Saving a file can move it in or out of the list while one is.
    pub fn has_content_filter(&self) -> bool {
        self.untagged_only || self.subtitled_only || self.commented_only || self.marked_only
    }

    /// Name filter as the user typed it. Empty means every file passes.
    pub fn name_filter(&self) -> &str {
        &self.name_filter
    }

    /// Set the name filter. It is words, and a file passes when each word is in its name (as it
    /// is on disk, so the tags in the name are searchable too) or in its comment, ignoring case.
    /// A comment still loading is not known yet: such a file passes on its name alone.
    pub fn set_name_filter(&mut self, query: String) {
        self.name_filter_words = search::words(&query);
        self.name_filter = query;
    }

    /// Whether the search has words.
    pub fn is_searching(&self) -> bool {
        !self.name_filter_words.is_empty()
    }

    /// Whether a file passes the current name filter.
    fn matches_name_filter(&self, file: &File) -> bool {
        let name = file.file_path().file_name().and_then(|n| n.to_str());
        let comment = (!file.snapshot().comment_loading()).then(|| file.comment());
        self.name_filter_words.iter().all(|word| {
            name.is_some_and(|n| search::contains_ignore_case(n, word))
                || comment.is_some_and(|c| search::contains_ignore_case(c, word))
        })
    }

    /// Why a file matches the search when its name alone does not: the line of its comment with
    /// the words in it, cut to show the first hit within `lead_chars` characters. `None` without
    /// a search, when the name has every word, or while the comment is loading.
    pub fn comment_fragment(&self, file: &File, lead_chars: usize) -> Option<CommentFragment> {
        if self.name_filter_words.is_empty() || file.snapshot().comment_loading() {
            return None;
        }
        let name = file.file_path().file_name().and_then(|n| n.to_str())?;
        let by_name = self
            .name_filter_words
            .iter()
            .all(|word| search::contains_ignore_case(name, word));
        if by_name {
            return None;
        }
        search::fragment(file.comment(), &self.name_filter_words, lead_chars)
    }

    /// Whether a file is part of the listed set under the current filters.
    /// The selected file is always listed: tagging it, or typing a query it no longer matches,
    /// must not pull the row out from under the cursor.
    fn is_listed(&self, file: &File) -> bool {
        if self.selected_id == Some(file.id()) {
            return true;
        }
        if self.untagged_only && !file.snapshot().tags().is_empty() {
            return false;
        }
        if self.subtitled_only && !file.has_subtitles() {
            return false;
        }
        if self.commented_only && !crate::ai::has_editor_comment(file.comment()) {
            return false;
        }
        if self.marked_only && file.snapshot().marker_count() == 0 {
            return false;
        }
        self.matches_name_filter(file)
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
        let id = self
            .files_by_id
            .values()
            .find(|f| f.file_path() == path)?
            .id();
        if self.selected_id == Some(id) {
            return None;
        }
        self.select_by_id(id)
    }

    /// Open the file this folder last remembered as open (#98): by `last_viewed`'s exact name,
    /// or, failing that, by the name and extension a file would have without its tags (a cheap
    /// fallback for a rename done outside frename that only changed the tag prefix — the
    /// extension must match too, or two files that happen to share a base name, such as
    /// `pick.clip.mkv` and `review.clip.mp4`, could resolve to the wrong one). Found regardless
    /// of the current list filter, like [`Self::open_path`] — the selected file always stays
    /// listed. `None` when `last_viewed` is empty or matches nothing.
    pub fn open_last_viewed(&mut self, last_viewed: &str) -> Option<File> {
        if last_viewed.is_empty() {
            return None;
        }
        let by_exact_name =
            |f: &&File| f.file_path().file_name().and_then(|n| n.to_str()) == Some(last_viewed);
        let id = if let Some(file) = self.files_by_id.values().find(by_exact_name) {
            file.id()
        } else {
            let remembered = FileSnapshot::parse(last_viewed);
            self.files_by_id
                .values()
                .find(|f| f.snapshot().same_clip_without_tags(&remembered))?
                .id()
        };
        self.select_by_id(id)
    }

    /// Select by stable ID. O(1) existence check. Returns the selected file if found.
    pub(crate) fn select_by_id(&mut self, id: FileId) -> Option<File> {
        if !self.files_by_id.contains_key(&id) {
            return None;
        }
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
        if current == 0 {
            return None;
        }
        self.select_index(current - 1)
    }

    pub fn select_next(&mut self) -> Option<File> {
        let current = self.selected_index().unwrap_or(0);
        if current + 1 >= self.listed_count() {
            return None;
        }
        self.select_index(current + 1)
    }

    // -----------------------------------------------------------------------
    // File mutation (rename / update)
    // -----------------------------------------------------------------------

    /// Update both the path and snapshot of the file identified by `id`. O(1). Where playback
    /// stopped in it goes along to the new path.
    /// Returns true if the file was found.
    pub fn rename_file(&mut self, id: FileId, new_path: &Path, snapshot: &FileSnapshot) -> bool {
        match self.files_by_id.get_mut(&id) {
            Some(file) => {
                let old_path = file.file_path().to_path_buf();
                file.set_file_path(new_path);
                file.set_file_snapshot(snapshot);
                if old_path != new_path {
                    self.store.move_playback_position(&old_path, new_path);
                    log::info!(
                        "Directory: renamed {} → {} ({} tag(s))",
                        old_path.display(),
                        new_path.display(),
                        snapshot.tags().len()
                    );
                } else {
                    log::info!(
                        "Directory: updated {} ({} tag(s))",
                        new_path.display(),
                        snapshot.tags().len()
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

    /// Record whether the file identified by `id` has a `.srt` next to it, for its marker, the
    /// "with subtitles" filter and its count. Returns true if the file was found.
    pub fn set_has_subtitles(&mut self, id: FileId, has_subtitles: bool) -> bool {
        match self.files_by_id.get_mut(&id) {
            Some(file) => {
                file.set_has_subtitles(has_subtitles);
                true
            }
            None => false,
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
    // Sizes and times come with the listing, so they cost nothing; with the tag file's file
    // list they let parsing take comments from the list instead of opening each video.
    let stats = entries
        .iter()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            let metadata = e.metadata().ok().filter(|m| m.is_file())?;
            let modified = crate::metadata::cache::modified_ms(metadata.modified().ok()?);
            Some((name, (metadata.len(), modified)))
        })
        .collect();
    let folder_info = FolderInfo::for_scan(
        entries
            .iter()
            .filter_map(|e| e.file_name().to_str().map(str::to_string))
            .collect(),
        stats,
        crate::FolderTagStore::read_file_cache(directory),
    );

    let mut files = Vec::with_capacity(entries.len());
    for entry in &entries {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        // file_type does not follow symlinks, so linked media needs the extra stat to be seen.
        let is_file = if file_type.is_symlink() {
            path.is_file()
        } else {
            file_type.is_file()
        };
        if !is_file {
            continue;
        }
        if FileTagger::is_sidecar_file(&path) {
            continue;
        }
        // Only videos are listed. Checked before the file is parsed, so the subtitles,
        // transcripts and images a shoot folder is full of cost nothing.
        if !is_listed_kind(&path) {
            continue;
        }
        // An unreadable entry sorts to the front rather than failing the whole scan.
        let modified_at = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        files.push(File::from_path_with_folder_info(
            path,
            modified_at,
            &folder_info,
        ));
    }
    files.sort_by_key(|a| a.modified_at());
    Ok(files)
}

/// Whether a file of this kind belongs in the folder list: videos only.
fn is_listed_kind(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    FileKind::from_extension(ext) == FileKind::Video
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    use super::is_listed_kind;
    use crate::db::fake_app_storage::FakeAppStorage;
    use crate::{Directory, File, FileSnapshot, FolderInfo};

    #[test]
    fn only_videos_are_listed() {
        assert!(is_listed_kind(Path::new(r"C:\shoot\clip.MP4")));
        assert!(is_listed_kind(Path::new(r"C:\shoot\clip.mov")));
        for name in [
            "clip.srt",
            "notes.txt",
            "photo.jpg",
            "photo.HEIC",
            "clip.soniox.json",
            "noext",
        ] {
            assert!(
                !is_listed_kind(&Path::new(r"C:\shoot").join(name)),
                "{name} must be hidden"
            );
        }
    }

    /// A migrated database of its own and a folder on disk, for one test.
    fn database_and_folder(name: &str) -> (crate::AppDatabase, PathBuf) {
        use crate::Initializable;
        let base = std::env::temp_dir().join(format!("frename-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let folder = base.join("folder");
        std::fs::create_dir_all(&folder).expect("folder");
        let db = crate::AppDatabase::with_path(base.join("frename.db"));
        db.initialize().expect("migrate");
        (db, folder)
    }

    /// Issue #161: a rename done in frename carries where playback stopped along.
    #[test]
    fn a_rename_carries_the_playback_position_along() {
        use crate::AppStateStore;
        let (db, folder) = database_and_folder("dir-playback-rename");
        let clip = folder.join("MVI_0410.mp4");
        db.set_playback_position(&clip, std::time::Duration::from_secs(40));
        let file = File::from_path(&clip, SystemTime::UNIX_EPOCH);
        let id = file.id();
        let mut dir = Directory::with_files(&folder, vec![file], db.clone());
        let mut snapshot = dir.file_by_id(id).expect("file").snapshot().clone();
        snapshot.set_tags(["pick"]);
        let renamed = folder.join(snapshot.file_name());
        dir.rename_file(id, &renamed, &snapshot);
        assert_eq!(db.get_playback_position(&clip), None);
        assert_eq!(
            db.get_playback_position(&renamed),
            Some(std::time::Duration::from_secs(40))
        );
    }

    /// Issue #161: opening a folder forgets positions of its gone files and follows a rename done
    /// outside frename.
    #[test]
    fn opening_a_folder_tidies_its_playback_positions() {
        use crate::AppStateStore;
        let (db, folder) = database_and_folder("dir-playback-tidy");
        std::fs::write(folder.join("skip.MVI_0410.mp4"), b"").expect("clip");
        let secs = std::time::Duration::from_secs;
        db.set_playback_position(&folder.join("MVI_0410.mp4"), secs(40));
        db.set_playback_position(&folder.join("gone.mp4"), secs(50));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime");
        runtime
            .block_on(Directory::open(&folder, db.clone()))
            .expect("open");
        assert_eq!(
            db.get_playback_position(&folder.join("skip.MVI_0410.mp4")),
            Some(secs(40))
        );
        assert_eq!(db.get_playback_position(&folder.join("gone.mp4")), None);
    }

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
    fn name_filter_keeps_only_matching_files() {
        let mut dir = directory_with(&["holiday.mp4", "work.mp4", "Holiday_2.mp4"]);
        dir.set_name_filter("holiday".to_string());
        assert_eq!(listed_names(&dir), vec!["holiday.mp4", "Holiday_2.mp4"]);
    }

    #[test]
    fn name_filter_matches_tags_in_the_name() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        tag_file_at(&mut dir, 0, &["Action"]);
        dir.set_name_filter("action".to_string());
        assert_eq!(listed_names(&dir), vec!["Action.a.mp4"]);
    }

    #[test]
    fn empty_name_filter_lists_every_file() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        dir.set_name_filter("   ".to_string());
        assert_eq!(listed_names(&dir).len(), 2);
    }

    /// Both filters narrow the list together.
    #[test]
    fn name_filter_and_untagged_filter_combine() {
        let mut dir = directory_with(&["trip_a.mp4", "trip_b.mp4", "other.mp4"]);
        tag_file_at(&mut dir, 0, &["Action"]);
        dir.set_untagged_only(true);
        dir.set_name_filter("trip".to_string());
        assert_eq!(listed_names(&dir), vec!["trip_b.mp4"]);
    }

    #[test]
    fn selected_file_stays_listed_while_typing_a_query_it_does_not_match() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        dir.select_index(1);
        dir.set_name_filter("c".to_string());
        assert_eq!(listed_names(&dir), vec!["b.mp4", "c.mp4"]);
        assert_eq!(
            dir.selected_index(),
            Some(0),
            "cursor still points at its file"
        );
    }

    /// Set a comment on a file the way a save does, by its listed position.
    fn comment_file_at(dir: &mut Directory<FakeAppStorage>, index: usize, comment: &str) {
        let file = dir.files_in_order().nth(index).expect("file at index");
        let (id, path) = (file.id(), file.file_path().to_path_buf());
        let mut snapshot = file.snapshot().clone();
        snapshot.set_comment(comment.to_string());
        dir.rename_file(id, &path, &snapshot);
    }

    #[test]
    fn the_search_finds_a_word_in_the_name_or_the_comment() {
        let mut dir = directory_with(&["goat_a.mp4", "b.mp4", "c.mp4"]);
        comment_file_at(&mut dir, 1, "A Goat on the hill");
        dir.set_name_filter("goat".to_string());
        assert_eq!(listed_names(&dir), vec!["goat_a.mp4", "b.mp4"]);
    }

    #[test]
    fn every_word_must_match_somewhere_in_any_field() {
        let mut dir = directory_with(&["trip_a.mp4", "trip_b.mp4", "other.mp4"]);
        comment_file_at(&mut dir, 0, "goat");
        comment_file_at(&mut dir, 1, "sheep");
        comment_file_at(&mut dir, 2, "goat trip");
        // "trip" is in the name of two, "goat" in the comment of two: one has both by name and
        // comment, one has both in its comment.
        dir.set_name_filter("  TRIP   goat ".to_string());
        assert_eq!(listed_names(&dir), vec!["trip_a.mp4", "other.mp4"]);
        dir.set_name_filter("goat sheep".to_string());
        assert!(listed_names(&dir).is_empty());
    }

    #[test]
    fn the_search_reads_the_ai_description_and_marker_lines_of_the_comment() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        comment_file_at(
            &mut dir,
            0,
            "Notes\nAI: A goat crosses a road.\n0:00–0:05 Road.\n— Claude Haiku 4.5, 2026-09-26 —",
        );
        comment_file_at(&mut dir, 1, "0:41–0:47 — Lion");
        dir.set_name_filter("crosses".to_string());
        assert_eq!(listed_names(&dir), vec!["a.mp4"]);
        dir.set_name_filter("lion".to_string());
        assert_eq!(listed_names(&dir), vec!["b.mp4"]);
    }

    #[test]
    fn a_comment_still_loading_matches_by_name_only() {
        let mut dir = directory_with(&["goat_a.mp4", "b.mp4"]);
        let file = dir.files_in_order().nth(1).expect("b");
        let (id, path) = (file.id(), file.file_path().to_path_buf());
        let mut snapshot = file.snapshot().clone();
        snapshot.set_comment_loading(true);
        dir.rename_file(id, &path, &snapshot);
        dir.set_name_filter("goat".to_string());
        assert_eq!(listed_names(&dir), vec!["goat_a.mp4"]);
        assert_eq!(dir.loading_comment_count(), 1);
        // Once it is loaded, the search sees its comment.
        snapshot.set_comment_loading(false);
        snapshot.set_comment("goat".to_string());
        dir.rename_file(id, &path, &snapshot);
        assert_eq!(listed_names(&dir), vec!["goat_a.mp4", "b.mp4"]);
    }

    #[test]
    fn the_search_folds_cyrillic_case() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        comment_file_at(&mut dir, 0, "Коза на лугу");
        dir.set_name_filter("КОЗА".to_string());
        assert_eq!(listed_names(&dir), vec!["a.mp4"]);
    }

    #[test]
    fn a_file_that_matches_by_comment_says_where() {
        let mut dir = directory_with(&["goat_a.mp4", "b.mp4"]);
        comment_file_at(&mut dir, 1, "First line\nThe goat runs");
        dir.set_name_filter("goat".to_string());
        let by_name = dir.files_in_order().next().expect("goat_a").clone();
        assert_eq!(dir.comment_fragment(&by_name, 20), None, "its name says it");
        let by_comment = dir.files_in_order().nth(1).expect("b").clone();
        let fragment = dir.comment_fragment(&by_comment, 20).expect("a fragment");
        assert_eq!(fragment.text, "The goat runs");
        assert_eq!(&fragment.text[fragment.highlights[0].clone()], "goat");
        dir.set_name_filter(String::new());
        assert_eq!(dir.comment_fragment(&by_comment, 20), None, "no search");
    }

    /// Typing stays instant with a big folder: a few thousand clips with real-sized comments.
    #[test]
    fn searching_thousands_of_clips_stays_fast() {
        let names: Vec<String> = (0..3000).map(|i| format!("clip_{i:05}.mp4")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut dir = directory_with(&refs);
        let filler = "The quick brown fox jumps over the lazy dog. ".repeat(20);
        let ids: Vec<(crate::FileId, PathBuf, FileSnapshot)> = dir
            .files_in_order()
            .enumerate()
            .map(|(i, f)| {
                let mut snapshot = f.snapshot().clone();
                snapshot.set_comment(format!("{filler} clip number {i} ends here"));
                (f.id(), f.file_path().to_path_buf(), snapshot)
            })
            .collect();
        for (id, path, snapshot) in &ids {
            dir.rename_file(*id, path, snapshot);
        }
        let started = std::time::Instant::now();
        for query in ["z", "zzz", "lazy dog ends", "number 2999 here"] {
            dir.set_name_filter(query.to_string());
            let _ = dir.listed_count();
        }
        let took = started.elapsed();
        assert!(
            took < std::time::Duration::from_secs(2),
            "four searches over 3000 clips took {took:?}"
        );
    }

    #[test]
    fn comment_filter_lists_only_commented_files() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        comment_file_at(&mut dir, 1, "goat");
        dir.set_commented_only(true);
        assert_eq!(listed_names(&dir), vec!["b.mp4"]);
        assert_eq!(dir.commented_count(), 1);
    }

    #[test]
    fn marker_filter_lists_only_files_with_markers() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        let file = dir.files_in_order().nth(2).expect("file at index");
        let (id, path) = (file.id(), file.file_path().to_path_buf());
        let mut snapshot = file.snapshot().clone();
        snapshot.set_marker_count(3);
        dir.rename_file(id, &path, &snapshot);
        dir.set_marked_only(true);
        assert_eq!(listed_names(&dir), vec!["c.mp4"]);
        assert_eq!(dir.marked_count(), 1);
    }

    #[test]
    fn markers_read_from_the_file_win_over_the_parsed_count() {
        let mut snapshot = FileSnapshot::default();
        snapshot.set_marker_count(3);
        assert_eq!(snapshot.marker_count(), 3);
        snapshot.set_markers(Some(Vec::new()));
        assert_eq!(snapshot.marker_count(), 0);
    }

    #[test]
    fn a_comment_that_is_only_an_ai_description_is_not_commented() {
        let block = "AI: A walk.\n— Claude Haiku 4.5, 2026-09-26 —";
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        comment_file_at(&mut dir, 0, block);
        comment_file_at(&mut dir, 1, &format!("Mine\n\n{block}"));
        dir.set_commented_only(true);
        assert_eq!(listed_names(&dir), vec!["b.mp4"]);
        assert_eq!(dir.commented_count(), 1);
    }

    #[test]
    fn subtitle_filter_lists_only_files_with_subtitles() {
        let root = PathBuf::from("C:/test");
        let info = FolderInfo::new(vec!["a.mp4".into(), "b.mp4".into(), "b.srt".into()]);
        let files = ["a.mp4", "b.mp4"]
            .iter()
            .map(|name| {
                File::from_path_with_folder_info(root.join(name), SystemTime::UNIX_EPOCH, &info)
            })
            .collect();
        let mut dir = Directory::with_files(root, files, FakeAppStorage::new());
        dir.set_subtitled_only(true);
        assert_eq!(listed_names(&dir), vec!["b.mp4"]);
        assert_eq!(dir.subtitled_count(), 1);
    }

    #[test]
    fn generated_subtitles_show_in_the_filter_and_its_count() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        let id = dir.files_in_order().nth(1).expect("b.mp4").id();
        dir.set_subtitled_only(true);
        assert_eq!(dir.subtitled_count(), 0);

        assert!(dir.set_has_subtitles(id, true));
        assert_eq!(listed_names(&dir), vec!["b.mp4"]);
        assert_eq!(dir.subtitled_count(), 1);
        assert!(dir.file_by_id(id).is_some_and(|f| f.has_subtitles()));

        dir.set_has_subtitles(id, false);
        assert_eq!(dir.subtitled_count(), 0);
    }

    #[test]
    fn content_filters_combine() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        comment_file_at(&mut dir, 0, "one");
        comment_file_at(&mut dir, 1, "two");
        tag_file_at(&mut dir, 0, &["Action"]);
        dir.set_commented_only(true);
        dir.set_untagged_only(true);
        assert_eq!(listed_names(&dir), vec!["b.mp4"]);
    }

    #[test]
    fn selected_file_stays_listed_when_its_comment_is_cleared() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        comment_file_at(&mut dir, 0, "goat");
        dir.set_commented_only(true);
        dir.select_index(0);
        comment_file_at(&mut dir, 0, "");
        assert_eq!(listed_names(&dir), vec!["a.mp4"]);
    }

    #[test]
    fn untagged_count_ignores_the_filter() {
        let mut dir = directory_with(&["a.mp4", "b.mp4", "c.mp4"]);
        tag_file_at(&mut dir, 0, &["Action"]);
        dir.set_untagged_only(true);
        assert_eq!(dir.untagged_count(), 2);
    }

    #[test]
    fn open_last_viewed_selects_the_file_with_that_exact_name() {
        let mut dir = directory_with(&["a.mp4", "b.mp4"]);
        let opened = dir.open_last_viewed("b.mp4").expect("found");
        assert_eq!(opened.file_path().file_name().unwrap(), "b.mp4");
    }

    #[test]
    fn open_last_viewed_falls_back_to_the_name_without_tags_after_a_rename() {
        // Remembered as "b.mp4"; a tag added since (outside or inside frename) renamed it.
        let mut dir = directory_with(&["a.mp4", "pick.b.mp4"]);
        let opened = dir.open_last_viewed("b.mp4").expect("found by base name");
        assert_eq!(opened.file_path().file_name().unwrap(), "pick.b.mp4");
    }

    #[test]
    fn open_last_viewed_s_fallback_does_not_cross_extensions() {
        // "clip.mp4" is gone; an unrelated "clip.mkv" must not be mistaken for it just because
        // they share a base name once their own tags are stripped.
        let mut dir = directory_with(&["drop.clip.mkv", "keep.clip.mp4"]);
        let opened = dir
            .open_last_viewed("clip.mp4")
            .expect("found by base name and extension");
        assert_eq!(opened.file_path().file_name().unwrap(), "keep.clip.mp4");
    }

    #[test]
    fn open_last_viewed_s_fallback_ignores_extension_case() {
        // A camera-style uppercase extension, renamed outside frename with a lowercased one:
        // still the same clip.
        let mut dir = directory_with(&["pick.IMG_0424.mov"]);
        let opened = dir
            .open_last_viewed("IMG_0424.MOV")
            .expect("found despite the extension's case");
        assert_eq!(opened.file_path().file_name().unwrap(), "pick.IMG_0424.mov");
    }

    #[test]
    fn open_last_viewed_is_none_when_empty_or_nothing_matches() {
        let mut dir = directory_with(&["a.mp4"]);
        assert!(dir.open_last_viewed("").is_none());
        assert!(dir.open_last_viewed("gone.mp4").is_none());
    }

    #[test]
    fn open_last_viewed_finds_a_file_hidden_by_the_current_filter() {
        let mut dir = directory_with(&["a.mp4", "pick.b.mp4"]);
        dir.set_untagged_only(true);
        assert_eq!(
            listed_names(&dir),
            vec!["a.mp4"],
            "b is hidden by the filter"
        );
        let opened = dir
            .open_last_viewed("pick.b.mp4")
            .expect("found despite the filter");
        assert_eq!(opened.file_path().file_name().unwrap(), "pick.b.mp4");
    }
}
