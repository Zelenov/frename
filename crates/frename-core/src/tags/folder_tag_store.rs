//! Tag store backed by a `.frename` file in the folder being worked on.
//!
//! Tags belong to the folder, not to the application: a Kenya shoot wants `nairobi`, a China
//! trip wants `guangzhou`. Each folder therefore carries its own tag file, which travels with
//! the footage and can be read and edited by hand or by an AI. It is TOML:
//!
//! ```toml
//! version = 3
//!
//! tags = [
//!   { id = "00000000-0000-0000-0000-000000000000", name = "pick", color = 9, starred = true },
//!   { id = "00000000-0000-0000-0000-000000000003", name = "wide", color = 3 },
//! ]
//!
//! [[files]]
//! name = "Food.Commented.IMG_0424.MOV"
//! size = 10568817
//! modified_ms = 1753632259000
//! comment = """
//! 00-00-02-909: Goat
//! Second line of the comment"""
//! in = 1.0
//! out = 2.5
//!
//! [[files]]
//! name = "IMG_0425.MOV"
//! size = 8812342
//! modified_ms = 1753632300000
//! ```
//!
//! Array order is the tag order — there are no sort keys in the file, so reordering tags means
//! moving lines. One tag per line keeps that edit, and its diff, to a single line.
//!
//! The `[[files]]` blocks list the folder's videos with the comment and in/out points stored
//! inside each video. Comments are multi-line, so they are written as `"""` strings: the text
//! between the quotes is the comment, line breaks and all.
//!
//! Opening a video costs about 20 ms on a synced drive, so a folder scan cannot open every one
//! for its comment; it reads this list instead. The video stays the truth: a line counts only while
//! `name`, `size` and `modified_ms` still match the file, and anything else is read again. A
//! comment or in/out edited here is shown by frename and written into the video when that file
//! is next saved. `comment`, `in` and `out` are omitted when empty; `in`/`out` are seconds.
//!
//! Earlier versions wrote JSON; such a file no longer parses and is left untouched.
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
/// 3 is the first TOML version; 1 and 2 were JSON.
const FORMAT_VERSION: u32 = 3;

/// Serializes read-modify-write of tag files: the tag list and the file list are written from
/// different threads (the UI and the background comment loader), and each keeps the other's part.
static TAG_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Gap between the order keys derived from adjacent lines, leaving room to insert a tag
/// between two neighbours without respreading every key.
const ORDER_GAP: i64 = 1_000_000;

/// The tag file as it is stored on disk.
#[derive(Debug, Serialize, Deserialize)]
struct TagFile {
    version: u32,
    tags: Vec<TagEntry>,
    #[serde(default)]
    files: Vec<CachedFile>,
}

/// One video in the tag file's `files` list: the comment and in/out points stored inside the
/// video, valid while the name, size and modification time still match it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedFile {
    pub name: String,
    pub size: u64,
    /// Modification time, milliseconds since the Unix epoch.
    pub modified_ms: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub comment: String,
    /// In point in seconds.
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub start: Option<f32>,
    /// Out point in seconds.
    #[serde(default, rename = "out", skip_serializing_if = "Option::is_none")]
    pub end: Option<f32>,
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
        let file = parse(&contents).map_err(|e| with_path("parse tag file", &path, e))?;
        if file.version > FORMAT_VERSION {
            log::warn!(
                "Tag file {} is version {}, newer than the supported {FORMAT_VERSION}; reading it anyway",
                path.display(),
                file.version
            );
        }
        Ok(file.tags)
    }

    /// Writes the tags, replacing them in the file and keeping its file list. Does nothing
    /// when no folder is open.
    fn write_entries(
        &self,
        entries: &[TagEntry],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(folder) = self.folder.as_ref() else {
            return Ok(());
        };
        let _lock = TAG_FILE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let files = read_file(folder).map(|f| f.files).unwrap_or_default();
        write_file(folder, entries, &files)
    }

    /// The folder's file list (see the module docs), in file order. Empty when there is no tag
    /// file or it cannot be parsed.
    pub fn read_file_cache(folder: &Path) -> Vec<CachedFile> {
        read_file(folder).map(|f| f.files).unwrap_or_default()
    }

    /// Change the folder's file list: drop the lines named in `remove`, then replace or add the
    /// lines in `upsert` by name, keeping the tags. A folder without a tag file gets one with
    /// the built-in tags. Failures are logged; the list is only a cache.
    pub fn update_file_cache(folder: &Path, remove: &[String], upsert: Vec<CachedFile>) {
        let _lock = TAG_FILE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let (tags, mut files) = match read_file(folder) {
            Some(file) => (file.tags, file.files),
            None => (Self::default_entries(), Vec::new()),
        };
        let replaced: std::collections::HashSet<&str> = upsert.iter().map(|f| f.name.as_str()).collect();
        files.retain(|f| !remove.contains(&f.name) && !replaced.contains(f.name.as_str()));
        files.extend(upsert);
        files.sort_by(|a, b| a.name.cmp(&b.name));
        let _ = write_file(folder, &tags, &files);
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

fn parse(contents: &str) -> Result<TagFile, toml::de::Error> {
    toml::from_str(contents)
}

/// The folder's tag file, or `None` when there is none or it cannot be read or parsed.
fn read_file(folder: &Path) -> Option<TagFile> {
    let contents = std::fs::read_to_string(folder.join(TAG_FILE_NAME)).ok()?;
    parse(&contents).ok()
}

/// Write the tag file through a scratch file renamed over the real one, which is atomic on
/// Windows and POSIX alike: an interrupted write leaves the previous file intact.
fn write_file(
    folder: &Path,
    tags: &[TagEntry],
    files: &[CachedFile],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = folder.join(TAG_FILE_NAME);
    let temp_path = folder.join(TAG_FILE_TEMP_NAME);
    std::fs::write(&temp_path, render(tags, files)?)
        .map_err(|e| with_path("write tag file", &temp_path, e))?;
    std::fs::rename(&temp_path, &path).map_err(|e| {
        // The scratch file would otherwise be left behind next to the user's footage.
        let _ = std::fs::remove_file(&temp_path);
        with_path("replace tag file", &path, e)
    })?;
    Ok(())
}

/// Renders the file by hand rather than through a TOML serializer, for a layout that reads
/// and diffs well: one tag per line, so reordering a tag is a one-line edit, and one block per
/// video with its comment as a multi-line string, the way a person would write it.
fn render(tags: &[TagEntry], files: &[CachedFile]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut out = format!("version = {FORMAT_VERSION}\n\ntags = [\n");
    for tag in tags {
        let mut fields = vec![format!("id = {}", basic_string(&tag.id.to_string())), format!("name = {}", basic_string(&tag.name))];
        if let Some(color) = tag.color {
            fields.push(format!("color = {color}"));
        }
        if tag.starred {
            fields.push("starred = true".to_string());
        }
        out.push_str(&format!("  {{ {} }},\n", fields.join(", ")));
    }
    out.push_str("]\n");
    for file in files {
        out.push_str(&format!(
            "\n[[files]]\nname = {}\nsize = {}\nmodified_ms = {}\n",
            basic_string(&file.name),
            file.size,
            file.modified_ms
        ));
        if !file.comment.is_empty() {
            out.push_str(&format!("comment = {}\n", text_string(&file.comment)));
        }
        if let Some(start) = file.start {
            out.push_str(&format!("in = {start:?}\n"));
        }
        if let Some(end) = file.end {
            out.push_str(&format!("out = {end:?}\n"));
        }
    }
    Ok(out)
}

/// A TOML basic string: `"..."`, with quotes, backslashes and control characters escaped.
fn basic_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A comment as TOML: a multi-line `"""` string when it has line breaks, so each line of the
/// comment is a line of the file, otherwise a basic string. The line break right after the
/// opening quotes is not part of the value (TOML trims it).
fn text_string(value: &str) -> String {
    if !value.contains('\n') {
        return basic_string(value);
    }
    let mut out = String::from("\"\"\"\n");
    for c in value.chars() {
        match c {
            // Escaping every quote rules out an accidental closing `"""`.
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\r' => out.push_str("\\r"),
            '\n' | '\t' => out.push(c),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push_str("\"\"\"");
    out
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
            r#"version = 3
tags = [
  { id = "00000000-0000-0000-0000-000000000001", name = "nairobi" },
  { id = "00000000-0000-0000-0000-000000000002", name = "villa" },
]
"#,
        );
        let tags = folder.store().get_stored_tags().expect("read tags");
        assert_eq!(names(&folder.store()), vec!["nairobi", "villa"]);
        assert!(tags[0].sort_order() < tags[1].sort_order());
    }

    #[test]
    fn a_hand_written_tag_needs_only_a_name_and_an_id() {
        let folder = TempFolder::new("minimal");
        folder.write_tag_file(
            r#"version = 3
tags = [{ id = "00000000-0000-0000-0000-0000000000ff", name = "guangzhou" }]
"#,
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
        folder.write_tag_file("version = 3\ntags = []\n");
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
        let tag_lines = contents.lines().filter(|l| l.trim_start().starts_with("{ id = ")).count();
        assert_eq!(tag_lines, DEFAULT_TAGS.len());
        assert!(contents.starts_with("version = 3\n"));
    }

    #[test]
    fn a_broken_file_is_reported_and_left_alone() {
        let folder = TempFolder::new("broken");
        folder.write_tag_file("tags = [ not toml");
        let store = folder.store();
        assert!(store.get_stored_tags().is_err());
        assert_eq!(folder.tag_file(), "tags = [ not toml", "user's file is untouched");
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

    #[test]
    fn the_file_list_and_the_tags_keep_each_other() {
        let folder = TempFolder::new("file-list");
        let mut store = folder.store();
        let _ = names(&store); // writes the built-in tags
        let entry = CachedFile {
            name: "IMG_1.MOV".into(),
            size: 10,
            modified_ms: 20,
            comment: "Goat".into(),
            start: Some(1.0),
            end: None,
        };
        FolderTagStore::update_file_cache(&folder.0, &[], vec![entry.clone()]);
        assert_eq!(FolderTagStore::read_file_cache(&folder.0), vec![entry.clone()]);

        // A tag change keeps the file list, and the file list keeps the tags.
        let before = names(&store);
        store.remove_stored_tag_by_id(store.get_stored_tags().expect("tags")[0].id()).expect("remove");
        assert_eq!(FolderTagStore::read_file_cache(&folder.0), vec![entry.clone()]);
        assert_eq!(names(&store).len(), before.len() - 1);

        let text = folder.tag_file();
        assert!(
            text.contains("[[files]]\nname = \"IMG_1.MOV\"\nsize = 10\nmodified_ms = 20\ncomment = \"Goat\"\nin = 1.0\n"),
            "{text}"
        );

        FolderTagStore::update_file_cache(&folder.0, &["IMG_1.MOV".to_string()], Vec::new());
        assert!(FolderTagStore::read_file_cache(&folder.0).is_empty());
        assert!(!folder.tag_file().contains("[[files]]"));
    }

    #[test]
    fn a_multi_line_comment_is_written_as_its_lines_and_reads_back_exactly() {
        let folder = TempFolder::new("multiline");
        let _ = names(&folder.store());
        let comment = "00-00-01-140: АФРИКАНСКИЙ размер\n\nQuote \" and \"\"\" and back\\slash\n\tindented";
        let entry = CachedFile {
            name: "Ad.MP4".into(),
            size: 1,
            modified_ms: 2,
            comment: comment.into(),
            start: None,
            end: Some(2.5),
        };
        FolderTagStore::update_file_cache(&folder.0, &[], vec![entry.clone()]);

        let text = folder.tag_file();
        assert!(text.contains("comment = \"\"\"\n00-00-01-140: АФРИКАНСКИЙ размер\n\nQuote"), "{text}");
        assert_eq!(FolderTagStore::read_file_cache(&folder.0), vec![entry]);
    }

    #[test]
    fn a_file_list_written_by_hand_or_by_an_ai_is_read() {
        let folder = TempFolder::new("hand-files");
        folder.write_tag_file(
            r#"version = 3
tags = []

[[files]]
name = "IMG_1.MOV"
size = 10
modified_ms = 20
comment = '''
Written by hand,
over two lines'''
out = 3.0
"#,
        );
        let files = FolderTagStore::read_file_cache(&folder.0);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].comment, "Written by hand,\nover two lines");
        assert_eq!((files[0].start, files[0].end), (None, Some(3.0)));
    }
}
