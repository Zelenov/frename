//! Keeping the tag file's file list (see [`FolderTagStore`]) in line with the videos' XMP,
//! so a folder scan can take comments, in/out points and marker counts from it instead of
//! opening every file. Clip markers always live in the video, so the list is kept whatever the
//! comment and in/out storage.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{xmp, MetadataStorage, XmpSource};
use crate::tags::{CachedFile, FileSnapshot, FolderTagStore};

/// A modification time as the file list stores it: milliseconds since the Unix epoch.
pub fn modified_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// The file list line for the file at `path`: its size and time now, and the XMP it holds.
/// `None` when the file cannot be read, cannot hold XMP, or is a cloud placeholder: those are
/// not cached, so they are read again once they can be.
fn line_for(path: &Path) -> Option<(CachedFile, xmp::XmpFields)> {
    let fields = xmp::probe(path)?;
    let metadata = std::fs::metadata(path).ok()?;
    let line = CachedFile {
        name: path.file_name()?.to_str()?.to_string(),
        size: metadata.len(),
        modified_ms: modified_ms(metadata.modified().ok()?),
        comment: fields.comment.clone(),
        start: fields.segment.start,
        end: fields.segment.end,
        markers: Some(fields.markers),
    };
    Some((line, fields))
}

/// After a save turned `old` into `new`: re-read `new`'s XMP and put its line in the file
/// list, dropping `old`'s line when the file was renamed. The save may have changed the XMP
/// (and with it the file's size) or the name, and either would make the old line miss.
pub(crate) fn refresh_after_save(old: &Path, new: &Path) {
    let Some(folder) = new.parent() else { return };
    let old_name = old.file_name().and_then(|n| n.to_str()).map(str::to_string);
    let remove: Vec<String> = old_name.filter(|_| old != new).into_iter().collect();
    let upsert: Vec<CachedFile> = line_for(new).map(|(line, _)| line).into_iter().collect();
    if !remove.is_empty() || !upsert.is_empty() {
        FolderTagStore::update_file_cache(folder, &remove, upsert);
    }
}

/// Drop the file list's line for the file at `path` and read one from the file again. Returns
/// whether the line changed: it was missing, or no longer matched the file.
pub(crate) fn reload_line(path: &Path) -> bool {
    let (Some(folder), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str()))
    else {
        return false;
    };
    let old = FolderTagStore::read_file_cache(folder)
        .into_iter()
        .find(|line| line.name == name);
    let new = line_for(path).map(|(line, _)| line);
    if old == new {
        return false;
    }
    FolderTagStore::update_file_cache(folder, &[name.to_string()], new.into_iter().collect());
    true
}

/// Read the XMP a folder scan deferred for each `(path, snapshot)`, once per file, and record
/// what was read in the file list in one write per folder. Returns the resolved snapshots in
/// the same order.
pub(crate) fn resolve_batch(
    items: &[(PathBuf, FileSnapshot)],
    storage: MetadataStorage,
) -> Vec<FileSnapshot> {
    let mut lines: Vec<(PathBuf, CachedFile)> = Vec::new();
    let resolved = items
        .iter()
        .map(|(path, snapshot)| {
            let mut resolved = snapshot.clone();
            if !snapshot.comment_loading() {
                return resolved;
            }
            resolved.set_comment_loading(false);
            let has_text_file = crate::comment::comment_path(path).is_file();
            match line_for(path) {
                Some((line, _)) => {
                    super::load(
                        path,
                        has_text_file,
                        &mut resolved,
                        storage,
                        XmpSource::Cached(&line),
                    );
                    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
                    lines.push((folder, line));
                }
                None => super::load(path, has_text_file, &mut resolved, storage, XmpSource::Read),
            }
            resolved
        })
        .collect();

    let mut folders: Vec<PathBuf> = lines.iter().map(|(folder, _)| folder.clone()).collect();
    folders.dedup();
    for folder in folders {
        let upsert = lines
            .iter()
            .filter(|(f, _)| *f == folder)
            .map(|(_, line)| line.clone())
            .collect();
        FolderTagStore::update_file_cache(&folder, &[], upsert);
    }
    resolved
}
