//! Moving a file's comment or in/out points from one of their homes to the other.
//!
//! Normal parsing reads only the chosen storage, so after switching a setting the files
//! still stored the old way look empty. A move reads both homes of the file, writes the
//! destination and removes the old copy, and leaves everything else where it is. It goes
//! through the installed file tagger, so a build that must not touch the disk moves nothing.

use std::path::{Path, PathBuf};

use super::{xmp, CommentStorage, InOutStorage, MetadataStorage, Segment};
use crate::tags::FileSnapshot;

/// What a move takes to where. The other kind of metadata stays in its current home.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataMove {
    /// Move the comment into this home.
    Comments(CommentStorage),
    /// Move the in/out points into this home.
    InOut(InOutStorage),
}

impl MetadataMove {
    /// `storage` with the moved kind set to its destination.
    fn applied_to(self, storage: MetadataStorage) -> MetadataStorage {
        match self {
            Self::Comments(comment) => MetadataStorage { comment, ..storage },
            Self::InOut(in_out) => MetadataStorage { in_out, ..storage },
        }
    }
}

/// What moving one file's metadata did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveOutcome {
    /// Everything moved; the file's path afterwards.
    Moved(PathBuf),
    /// The file already keeps it in the destination, or has none.
    NothingToMove,
    /// Something is still outside the destination, e.g. because the format cannot hold XMP
    /// or the file could not be written; the details are in the log. The file's path
    /// afterwards, since part of the move may have renamed it.
    Failed(PathBuf),
}

/// What one file keeps outside a storage, i.e. what moving it there would move.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FileConversion {
    pub comment: bool,
    pub in_out: bool,
}

impl FileConversion {
    /// Whether converting the file would change anything.
    pub fn is_needed(self) -> bool {
        self.comment || self.in_out
    }
}

/// Where a file's comment and in/out points are on disk right now.
#[derive(Debug, Clone, Default)]
pub(crate) struct Inspection {
    has_text_file: bool,
    name_has_in_out: bool,
    /// `None` when the file cannot hold XMP (or is a cloud placeholder).
    xmp: Option<xmp::XmpFields>,
}

impl Inspection {
    /// Read both homes of the file at `path`.
    pub(crate) fn of(path: &Path) -> Self {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let from_name = FileSnapshot::parse(name);
        Self {
            has_text_file: crate::comment::comment_path(path).is_file(),
            name_has_in_out: from_name.segment_start().is_some() || from_name.segment_end().is_some(),
            xmp: xmp::probe(path),
        }
    }

    /// The storage the file uses now: a text file or in/out in the name win over XMP, as in
    /// parsing, and a file that cannot hold XMP keeps both outside it.
    fn current_storage(&self) -> MetadataStorage {
        let can_hold_xmp = self.xmp.is_some();
        MetadataStorage {
            comment: if self.has_text_file || !can_hold_xmp { CommentStorage::TextFile } else { CommentStorage::InVideo },
            in_out: if self.name_has_in_out || !can_hold_xmp { InOutStorage::FileName } else { InOutStorage::InVideo },
        }
    }

    /// The storage the file has after `what`: its destination for the moved kind, the
    /// current home for the other.
    pub(crate) fn storage_after(&self, what: MetadataMove) -> MetadataStorage {
        what.applied_to(self.current_storage())
    }

    /// What moving `what` would move. The file's tags stay as they are: a move does not add
    /// or remove a comment, so the commented tag has no reason to change.
    pub(crate) fn moved_by(&self, what: MetadataMove) -> FileConversion {
        let needed = self.needed(self.storage_after(what));
        match what {
            MetadataMove::Comments(_) => FileConversion { in_out: false, ..needed },
            MetadataMove::InOut(_) => FileConversion { comment: false, ..needed },
        }
    }

    /// What converting the file to `storage` would move. A text file or in/out in the name
    /// wins over XMP, as in parsing, so XMP is only moved out when the other home is empty;
    /// otherwise the XMP copy is left alone rather than lost.
    fn needed(&self, storage: MetadataStorage) -> FileConversion {
        let can_hold_xmp = self.xmp.is_some();
        let xmp = self.xmp.clone().unwrap_or_default();
        FileConversion {
            comment: match storage.comment {
                CommentStorage::InVideo => self.has_text_file && can_hold_xmp,
                CommentStorage::TextFile => !self.has_text_file && !xmp.comment.is_empty(),
            },
            in_out: match storage.in_out {
                InOutStorage::InVideo => self.name_has_in_out && can_hold_xmp,
                InOutStorage::FileName => !self.name_has_in_out && !xmp.segment.is_empty(),
            },
        }
    }
}

/// After a move to text file or file name saved the file at `path`, remove the XMP
/// copies that were moved out, so Premiere does not keep showing stale values. Only removes
/// what now exists in the new home.
pub(crate) fn clear_moved_xmp(path: &Path, moved: FileConversion, storage: MetadataStorage) {
    let clear_comment = moved.comment
        && storage.comment == CommentStorage::TextFile
        && crate::comment::comment_path(path).is_file();
    let clear_in_out = moved.in_out
        && storage.in_out == InOutStorage::FileName
        && Inspection::of(path).name_has_in_out;
    if !clear_comment && !clear_in_out {
        return;
    }
    let comment = clear_comment.then_some("");
    let segment = clear_in_out.then_some(Segment::default());
    if let Err(e) = xmp::write(path, comment, segment) {
        log::warn!("metadata: could not remove moved XMP from {:?}: {}", path, e);
    }
}
