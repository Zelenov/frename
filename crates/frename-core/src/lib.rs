//! Core logic for frename - file renaming utility.

mod db;
mod directory;
pub(crate) mod transliteration;
mod file_kind;
mod folder_file;
mod file;
mod ordered;
mod tags;
pub(crate) mod comment;
mod subtitles;
pub mod undo;

pub use db::{AppDatabase, AppSettings, AppStateStore, Initializable, LoggingAppStateStore, StoredTagStore, VideoSettings, WindowGeometry};
pub use directory::Directory;
pub use file_kind::FileKind;
pub use folder_file::FolderAndFile;
pub use subtitles::{load_subtitles, subtitle_path, SubtitleCue, Subtitles};
pub use file::{File, FileId};
pub use ordered::{OrderableEntry, OrderedCollection, OrderKey, OrderedThing};
pub use tags::{
    install_file_tagger,
    DefaultTag, FileSnapshot, FileTagger, FileTaggerBackend, FolderInfo, FolderTagStore,
    LoggingFileTagger, InMemoryFileTagger, ProductionFileTagger,
    SaveAndReparse, Screenshot, StoredTag, Tag, TagColorMapping, TagId, TagList, DEFAULT_TAGS,
};
pub use undo::{
    History, UndoContext, UndoError,
    NavigateFileCommand, ReorderTagCommand, ToggleTagCommand, PasteTagsCommand,
    DeleteTagCommand, CreateTagCommand, SaveTagCommand, StarTagCommand,
    SetSegmentStartCommand, SetSegmentEndCommand,
};
