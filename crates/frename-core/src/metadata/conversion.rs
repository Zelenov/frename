//! Moving comments and in/out points between their two homes, a whole folder at a time.
//!
//! Normal parsing reads only the chosen storage, so after switching a setting the files
//! still stored the old way look empty. A conversion reads both homes of each file, writes
//! the chosen one and removes the old copy. It goes through the installed file tagger, so
//! a build that must not touch the disk plans and converts nothing.

use std::path::{Path, PathBuf};

use super::{xmp, CommentStorage, InOutStorage, MetadataStorage, Segment};
use crate::tags::{FileSnapshot, FileTagger};

/// What one file keeps outside the chosen storage, i.e. what converting it would move.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileConversion {
    pub comment: bool,
    pub in_out: bool,
}

impl FileConversion {
    /// Whether converting the file would change anything.
    pub fn is_needed(self) -> bool {
        self.comment || self.in_out
    }
}

/// A folder conversion before it runs: which files it touches and what moves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionPlan {
    /// The storage the files would be converted to.
    pub storage: MetadataStorage,
    /// Files that need converting.
    pub files: Vec<PathBuf>,
    /// Comments to move.
    pub comments: usize,
    /// In/out points to move. Each one renames its file.
    pub in_outs: usize,
}

impl ConversionPlan {
    /// Whether the folder already uses the storage throughout.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// What a folder conversion did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionReport {
    /// Files now fully in the chosen storage.
    pub converted: usize,
    /// Files renamed on the way, old path to new path.
    pub renamed: Vec<(PathBuf, PathBuf)>,
    /// Files that still keep something outside the chosen storage, e.g. because the format
    /// cannot hold XMP or the file could not be written; the details are in the log.
    pub not_converted: Vec<PathBuf>,
}

/// Look at `paths` and list what converting them to `storage` would move.
pub fn plan_conversion(paths: &[PathBuf], storage: MetadataStorage) -> ConversionPlan {
    let mut plan = ConversionPlan { storage, ..ConversionPlan::default() };
    for path in paths {
        let needed = FileTagger::metadata_conversion(path, storage);
        if needed.is_needed() {
            plan.files.push(path.clone());
            plan.comments += usize::from(needed.comment);
            plan.in_outs += usize::from(needed.in_out);
        }
    }
    plan
}

/// Convert the files of `plan` to its storage.
pub fn convert(plan: &ConversionPlan) -> ConversionReport {
    let mut report = ConversionReport::default();
    for path in &plan.files {
        let new_path = FileTagger::convert_metadata(path, plan.storage);
        if FileTagger::metadata_conversion(&new_path, plan.storage).is_needed() {
            report.not_converted.push(new_path.clone());
        } else {
            report.converted += 1;
        }
        if new_path != *path {
            report.renamed.push((path.clone(), new_path));
        }
    }
    report
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

    /// What converting the file to `storage` would move. A text file or in/out in the name
    /// wins over XMP, as in parsing, so XMP is only moved out when the other home is empty;
    /// otherwise the XMP copy is left alone rather than lost.
    pub(crate) fn needed(&self, storage: MetadataStorage) -> FileConversion {
        let can_hold_xmp = self.xmp.is_some();
        let xmp = self.xmp.clone().unwrap_or_default();
        FileConversion {
            comment: match storage.comment {
                CommentStorage::Xmp => self.has_text_file && can_hold_xmp,
                CommentStorage::TextFile => !self.has_text_file && !xmp.comment.is_empty(),
            },
            in_out: match storage.in_out {
                InOutStorage::Xmp => self.name_has_in_out && can_hold_xmp,
                InOutStorage::FileName => !self.name_has_in_out && !xmp.segment.is_empty(),
            },
        }
    }
}

/// After a conversion to text file or file name saved the file at `path`, remove the XMP
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
