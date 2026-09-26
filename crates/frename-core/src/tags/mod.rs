//! Tag-related types: file tag list (tags + name + extension), snapshots, tagger, stored tags, tag color mapping, and tag list UI model.

mod default_tags;
mod file_snapshot;
mod file_tagger;
mod file_tagger_backend;
mod file_tagger_logging;
mod folder_info;
mod folder_tag_store;
mod in_memory_file_tagger;
mod production_file_tagger;
pub(crate) use production_file_tagger::screenshot_path;
mod screenshot;
mod stored_tag;
mod tag;
mod tag_color_mapping;
mod tag_list;

pub use default_tags::{DefaultTag, DEFAULT_TAGS};
pub use file_snapshot::{set_space_after_tags, space_after_tags, FileSnapshot};
pub use file_tagger::{install_file_tagger, FileTagger, SaveAndReparse};
pub use file_tagger_backend::FileTaggerBackend;
pub use file_tagger_logging::LoggingFileTagger;
pub use folder_info::FolderInfo;
pub use folder_tag_store::{CachedFile, FolderTagStore};
pub use in_memory_file_tagger::InMemoryFileTagger;
pub use production_file_tagger::ProductionFileTagger;
pub use screenshot::Screenshot;
pub use stored_tag::StoredTag;
pub use tag::{Tag, TagId};
pub use tag_color_mapping::TagColorMapping;
pub use tag_list::TagList;
