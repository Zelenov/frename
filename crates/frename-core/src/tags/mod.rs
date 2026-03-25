//! Tag-related types: file tag list (tags + name + extension), snapshots, tagger, stored tags, tag color mapping, and tag list UI model.

mod file_snapshot;
mod screenshot;
mod file_tagger_backend;
mod file_tagger;
mod file_tagger_logging;
mod in_memory_file_tagger;
mod production_file_tagger;
mod stored_tag;
mod tag;
mod tag_color_mapping;
mod tag_list;

pub use file_snapshot::FileSnapshot;
pub use screenshot::Screenshot;
pub use file_tagger::{install_file_tagger, FileTagger, SaveAndReparse};
pub use file_tagger_backend::FileTaggerBackend;
pub use file_tagger_logging::LoggingFileTagger;
pub use in_memory_file_tagger::InMemoryFileTagger;
pub use production_file_tagger::ProductionFileTagger;
pub use stored_tag::StoredTag;
pub use tag::{Tag, TagId};
pub use tag_color_mapping::TagColorMapping;
pub use tag_list::TagList;
