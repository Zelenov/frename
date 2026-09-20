//! Tag store backed by a `.frename` file in the folder being worked on.
//!
//! Tags belong to the folder, not to the application: a Kenya shoot wants `nairobi`, a China
//! trip wants `guangzhou`. Each folder therefore carries its own tag file, which travels with
//! the footage and can be read and edited by hand or by an AI:
//!
//! ```json
//! {
//!   "version": 1,
//!   "tags": [
//!     {"id":"00000000-0000-0000-0000-000000000000","name":"pick","color":9,"starred":true},
//!     {"id":"00000000-0000-0000-0000-000000000003","name":"wide","color":3}
//!   ]
//! }
//! ```
//!
//! Array order is the tag order — there are no sort keys in the file, so reordering tags means
//! moving lines. One tag per line keeps that edit, and its diff, to a single line.
//!
//! A folder with no tag file gets one written from [`DEFAULT_TAGS`] the first time it is opened.
//! A store with no folder at all (before anything is open) holds no tags and drops writes.
//!
//! The file is read on every call rather than cached: the store is cloned into `TagList` and
//! `FileWorkspace`, and a cache in one clone would go stale in the others. The file is a few
//! kilobytes, so a read costs far less than the SQLite open it replaces.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::StoredTagStore;
use crate::{StoredTag, TagColorMapping};

use super::default_tags::DEFAULT_TAGS;

/// Name of the per-folder tag file.
pub const TAG_FILE_NAME: &str = ".frename";

/// Scratch file the tag file is written through, so a crash mid-write cannot truncate it.
const TAG_FILE_TEMP_NAME: &str = ".frename.tmp";

/// Format version written into new files. Bump only on a breaking layout change.
const FORMAT_VERSION: u32 = 1;

/// Gap between the order keys derived from adjacent lines, leaving room to insert a tag
/// between two neighbours without respreading every key.
const ORDER_GAP: i64 = 1_000_000;

/// The tag file as it is stored on disk.
#[derive(Debug, Serialize, Deserialize)]
struct TagFile {
    version: u32,
    tags: Vec<TagEntry>,
}

/// One tag line in the file. Position in `tags` is the tag's order.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagEntry {
    id: Uuid,
    name: String,
    /// Index into the 16-color palette. Absent in a hand-written file; filled in on the next save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<u8>,
    /// Pinned to the starred section. Omitted when false to keep the common line short.
    #[serde(default, skip_serializing_if = "is_not_starred")]
    starred: bool,
}

fn is_not_starred(starred: &bool) -> bool {
    !*starred
}

/// Order key for the tag on line `index`.
fn order_for_index(index: usize) -> i64 {
    (index as i64 + 1) * ORDER_GAP
}

/// Stored tags for one folder. Cheap to clone; `TagList` and `FileWorkspace` each hold one.
#[derive(Clone, Debug)]
pub struct FolderTagStore {
    /// Folder these tags belong to. `None` before any folder is open.
    folder: Option<PathBuf>,
}

impl FolderTagStore {
    /// Store for the given folder. The tag file is not touched until it is first read.
    pub fn for_folder(folder: impl AsRef<Path>) -> Self {
        Self {
            folder: Some(folder.as_ref().to_path_buf()),
        }
    }

    /// Store used before a folder is open: reads no tags and drops writes.
    pub fn empty() -> Self {
        Self { folder: None }
    }

    /// Folder these tags belong to, or `None` before a folder is open.
    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_deref()
    }

    /// Whether this path is a folder's tag file (or the scratch file it is written through).
    /// Such files back the folder list rather than appearing in it.
    pub fn is_tag_file(path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };
        name.eq_ignore_ascii_case(TAG_FILE_NAME) || name.eq_ignore_ascii_case(TAG_FILE_TEMP_NAME)
    }

    /// Path of the tag file, or `None` before a folder is open.
    fn tag_file_path(&self) -> Option<PathBuf> {
        Some(self.folder.as_ref()?.join(TAG_FILE_NAME))
    }

    /// The built-in tag set as file entries.
    fn default_entries() -> Vec<TagEntry> {
        DEFAULT_TAGS
            .iter()
            .map(|t| TagEntry {
                id: t.id,
                name: t.name.to_string(),
                color: Some(t.color_index),
                starred: t.starred,
            })
            .collect()
    }

    /// Reads the folder's tags, in file order.
    ///
    /// A folder without a tag file gets one written from the built-in set, so that opening a new
    /// folder leaves behind a file the user can edit rather than an empty panel. No folder open
    /// means no tags. A file that cannot be parsed is reported and left untouched — a typo in a
    /// hand-edited file must not cost the user their tag list.
    fn read_entries(&self) -> Result<Vec<TagEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let Some(path) = self.tag_file_path() else {
            return Ok(Vec::new());
        };
        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                log::info!("No tag file at {}; writing the built-in tags", path.display());
                let entries = Self::default_entries();
                self.write_entries(&entries)?;
                return Ok(entries);
            }
            Err(e) => return Err(with_path("read tag file", &path, e)),
        };
        let file: TagFile = serde_json::from_str(&contents)
            .map_err(|e| with_path("parse tag file", &path, e))?;
        if file.version > FORMAT_VERSION {
            log::warn!(
                "Tag file {} is version {}, newer than the supported {FORMAT_VERSION}; reading it anyway",
                path.display(),
                file.version
            );
        }
        Ok(file.tags)
    }

    /// Writes the tags, replacing the file. Does nothing when no folder is open.
    ///
    /// The file is written to a scratch file and then renamed over the real one, which is atomic
    /// on Windows and POSIX alike: an interrupted write leaves the previous tag list intact.
    fn write_entries(
        &self,
        entries: &[TagEntry],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(folder) = self.folder.as_ref() else {
            return Ok(());
        };
        let path = folder.join(TAG_FILE_NAME);
        let temp_path = folder.join(TAG_FILE_TEMP_NAME);
        std::fs::write(&temp_path, render(entries)?)
            .map_err(|e| with_path("write tag file", &temp_path, e))?;
        std::fs::rename(&temp_path, &path).map_err(|e| {
            // The scratch file would otherwise be left behind next to the user's footage.
            let _ = std::fs::remove_file(&temp_path);
            with_path("replace tag file", &path, e)
        })?;
        Ok(())
    }

    /// Reads the tags paired with the order key each one currently has.
    fn read_ordered(
        &self,
    ) -> Result<Vec<(TagEntry, i64)>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .read_entries()?
            .into_iter()
            .enumerate()
            .map(|(index, entry)| (entry, order_for_index(index)))
            .collect())
    }

    /// Sorts by order key and writes the result, turning order keys back into line positions.
    fn write_ordered(
        &self,
        mut ordered: Vec<(TagEntry, i64)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        ordered.sort_by_key(|(_, order)| *order);
        let entries: Vec<TagEntry> = ordered.into_iter().map(|(entry, _)| entry).collect();
        self.write_entries(&entries)
    }
}

/// Renders the file with one tag per line, so reordering a tag is a one-line edit and shows up
/// as a one-line diff. `serde_json`'s pretty printer would spread every tag over six lines.
fn render(entries: &[TagEntry]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut lines = Vec::with_capacity(entries.len());
    for entry in entries {
        lines.push(format!("    {}", serde_json::to_string(entry)?));
    }
    Ok(format!(
        "{{\n  \"version\": {FORMAT_VERSION},\n  \"tags\": [\n{}\n  ]\n}}\n",
        lines.join(",\n")
    ))
}

/// Wraps an error with the file it happened on, so a failure names the path in the log.
fn with_path(
    action: &str,
    path: &Path,
    source: impl std::fmt::Display,
) -> Box<dyn std::error::Error + Send + Sync> {
    let message = format!("failed to {action} {}: {source}", path.display());
    log::error!("{message}");
    Box::new(io::Error::other(message))
}

impl Default for FolderTagStore {
    fn default() -> Self {
        Self::empty()
    }
}

impl StoredTagStore for FolderTagStore {
    fn get_stored_tags(&self) -> Result<Vec<StoredTag>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .read_entries()?
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                StoredTag::with_all(entry.id, entry.name, order_for_index(index), entry.starred)
            })
            .collect())
    }

    fn get_tag_color_mapping(
        &self,
    ) -> Result<TagColorMapping, Box<dyn std::error::Error + Send + Sync>> {
        let entries = self.read_entries()?;
        let colors: Vec<(String, u8)> = entries
            .into_iter()
            .filter_map(|entry| entry.color.map(|color| (entry.name, color)))
            .collect();
        Ok(TagColorMapping::from_entries(colors))
    }

    fn save_tag(
        &mut self,
        tag: StoredTag,
        color_index: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.folder.is_none() {
            return Ok(());
        }
        let mut ordered = self.read_ordered()?;
        let saved = TagEntry {
            id: tag.id(),
            name: tag.value().to_string(),
            color: Some(color_index),
            starred: tag.starred(),
        };
        match ordered.iter_mut().find(|(entry, _)| entry.id == tag.id()) {
            Some(slot) => *slot = (saved, tag.sort_order()),
            None => ordered.push((saved, tag.sort_order())),
        }
        self.write_ordered(ordered)
    }

    fn remove_stored_tag_by_id(
        &mut self,
        tag_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.folder.is_none() {
            return Ok(());
        }
        let mut entries = self.read_entries()?;
        entries.retain(|entry| entry.id != tag_id);
        self.write_entries(&entries)
    }

    fn update_tag_orders(
        &mut self,
        tag_orders: &[(Uuid, i64)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if tag_orders.is_empty() || self.folder.is_none() {
            return Ok(());
        }
        let mut ordered = self.read_ordered()?;
        for (id, new_order) in tag_orders {
            if let Some((_, order)) = ordered.iter_mut().find(|(entry, _)| entry.id == *id) {
                *order = *new_order;
            }
        }
        self.write_ordered(ordered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique scratch folder; each test gets its own so they can run in parallel.
    struct TempFolder(PathBuf);

    impl TempFolder {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("frename-tags-{name}-{}", Uuid::new_v4()));
            std::fs::create_dir_all(&path).expect("create temp folder");
            Self(path)
        }

        fn store(&self) -> FolderTagStore {
            FolderTagStore::for_folder(&self.0)
        }

        fn tag_file(&self) -> String {
            std::fs::read_to_string(self.0.join(TAG_FILE_NAME)).expect("read tag file")
        }

        fn write_tag_file(&self, contents: &str) {
            std::fs::write(self.0.join(TAG_FILE_NAME), contents).expect("write tag file");
        }
    }

    impl Drop for TempFolder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn names(store: &FolderTagStore) -> Vec<String> {
        store
            .get_stored_tags()
            .expect("read tags")
            .into_iter()
            .map(|t| t.value().to_string())
            .collect()
    }

    #[test]
    fn folder_without_a_tag_file_gets_the_built_in_tags() {
        let folder = TempFolder::new("defaults");
        let store = folder.store();
        let tags = store.get_stored_tags().expect("read tags");
        assert_eq!(tags.len(), DEFAULT_TAGS.len());
        assert_eq!(tags[0].value(), "pick");
        assert!(tags[0].starred());
        assert!(folder.0.join(TAG_FILE_NAME).is_file(), "file is written out");
    }

    #[test]
    fn file_order_is_tag_order() {
        let folder = TempFolder::new("order");
        folder.write_tag_file(
            r#"{"version":1,"tags":[
                {"id":"00000000-0000-0000-0000-000000000001","name":"nairobi"},
                {"id":"00000000-0000-0000-0000-000000000002","name":"villa"}
            ]}"#,
        );
        let tags = folder.store().get_stored_tags().expect("read tags");
        assert_eq!(names(&folder.store()), vec!["nairobi", "villa"]);
        assert!(tags[0].sort_order() < tags[1].sort_order());
    }

    #[test]
    fn a_hand_written_tag_needs_only_a_name_and_an_id() {
        let folder = TempFolder::new("minimal");
        folder.write_tag_file(
            r#"{"version":1,"tags":[{"id":"00000000-0000-0000-0000-0000000000ff","name":"guangzhou"}]}"#,
        );
        let store = folder.store();
        assert_eq!(names(&store), vec!["guangzhou"]);
        assert!(!store.get_stored_tags().expect("read tags")[0].starred());
        let colors = store.get_tag_color_mapping().expect("read colors");
        assert_eq!(colors.color_index_for("guangzhou"), 0, "no color yet");
    }

    #[test]
    fn saving_a_new_tag_appends_it_with_its_color() {
        let folder = TempFolder::new("save-new");
        folder.write_tag_file(r#"{"version":1,"tags":[]}"#);
        let mut store = folder.store();
        let id = Uuid::new_v4();
        store
            .save_tag(StoredTag::with_all(id, "nairobi", ORDER_GAP, false), 7)
            .expect("save");
        assert_eq!(names(&store), vec!["nairobi"]);
        assert_eq!(
            store.get_tag_color_mapping().expect("colors").color_index_for("nairobi"),
            7
        );
    }

    #[test]
    fn saving_an_existing_tag_replaces_it_in_place() {
        let folder = TempFolder::new("save-existing");
        let mut store = folder.store();
        let first = store.get_stored_tags().expect("read tags")[0].clone();
        store
            .save_tag(
                StoredTag::with_all(first.id(), "keeper", first.sort_order(), true),
                4,
            )
            .expect("save");
        let tags = store.get_stored_tags().expect("read tags");
        assert_eq!(tags.len(), DEFAULT_TAGS.len(), "renamed, not added");
        assert_eq!(tags[0].value(), "keeper");
        assert!(tags[0].starred());
    }

    #[test]
    fn a_tag_saved_with_a_lower_order_moves_to_the_front() {
        let folder = TempFolder::new("save-order");
        let mut store = folder.store();
        let id = Uuid::new_v4();
        store
            .save_tag(StoredTag::with_all(id, "nairobi", 1, false), 3)
            .expect("save");
        assert_eq!(names(&store)[0], "nairobi");
    }

    #[test]
    fn removing_a_tag_drops_its_line() {
        let folder = TempFolder::new("remove");
        let mut store = folder.store();
        let first = store.get_stored_tags().expect("read tags")[0].clone();
        store.remove_stored_tag_by_id(first.id()).expect("remove");
        let remaining = names(&store);
        assert_eq!(remaining.len(), DEFAULT_TAGS.len() - 1);
        assert!(!remaining.contains(&first.value().to_string()));
    }

    #[test]
    fn updating_orders_rewrites_the_file_in_the_new_order() {
        let folder = TempFolder::new("reorder");
        let mut store = folder.store();
        let tags = store.get_stored_tags().expect("read tags");
        let (first, second) = (tags[0].clone(), tags[1].clone());
        store
            .update_tag_orders(&[(first.id(), 20), (second.id(), 10)])
            .expect("reorder");
        let reordered = names(&store);
        assert_eq!(reordered[0], second.value());
        assert_eq!(reordered[1], first.value());
    }

    #[test]
    fn each_tag_is_written_on_its_own_line() {
        let folder = TempFolder::new("layout");
        let _ = folder.store().get_stored_tags().expect("read tags");
        let contents = folder.tag_file();
        let tag_lines = contents.lines().filter(|l| l.trim_start().starts_with("{\"id\"")).count();
        assert_eq!(tag_lines, DEFAULT_TAGS.len());
        assert!(contents.starts_with("{\n  \"version\": 1,"));
    }

    #[test]
    fn a_broken_file_is_reported_and_left_alone() {
        let folder = TempFolder::new("broken");
        folder.write_tag_file("{ not json");
        let store = folder.store();
        assert!(store.get_stored_tags().is_err());
        assert_eq!(folder.tag_file(), "{ not json", "user's file is untouched");
    }

    #[test]
    fn a_written_file_reads_back_identically() {
        let folder = TempFolder::new("roundtrip");
        let before = folder.store().get_stored_tags().expect("read tags");
        let after = folder.store().get_stored_tags().expect("read tags");
        assert_eq!(before, after);
    }

    #[test]
    fn store_without_folder_reads_no_tags_and_drops_writes() {
        let mut store = FolderTagStore::empty();
        assert!(store.folder().is_none());
        assert!(store.get_stored_tags().expect("read").is_empty());
        let tag = StoredTag::with_all(Uuid::from_u128(1), "pick", 0, false);
        assert!(store.save_tag(tag, 0).is_ok());
        assert!(store.get_stored_tags().expect("read").is_empty());
    }

    #[test]
    fn the_tag_file_and_its_scratch_file_are_not_listed_as_footage() {
        assert!(FolderTagStore::is_tag_file(Path::new(r"C:\shoots\kenya\.frename")));
        assert!(FolderTagStore::is_tag_file(Path::new(r"C:\shoots\kenya\.frename.tmp")));
        assert!(!FolderTagStore::is_tag_file(Path::new(r"C:\shoots\kenya\clip.mp4")));
    }
}
