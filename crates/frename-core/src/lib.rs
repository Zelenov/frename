//! Core logic for frename - file renaming utility.

mod file;
mod tags;

pub use file::{FileInfo, RenameCore, scan_directory};
pub use tags::{Tag, TagList};
