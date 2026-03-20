//! Core logic for frename - file renaming utility.

mod db;
mod directory;
mod file_kind;
mod folder_file;
mod file;
mod ordered;
mod tags;
pub mod undo;

pub use db::{AppDatabase, AppStateStore, Initializable, LoggingAppStateStore, StoredTagStore};
pub use directory::Directory;
pub use file_kind::FileKind;
pub use folder_file::FolderAndFile;
pub use file::{File, FileId};
pub use ordered::{OrderableEntry, OrderedCollection, OrderKey, OrderedThing};
pub use tags::{
    install_file_tagger,
    FileSnapshot, FileTagger, FileTaggerBackend, LoggingFileTagger,
    InMemoryFileTagger, ProductionFileTagger,
    SaveAndReparse, StoredTag, Tag, TagColorMapping, TagId, TagList,
};
pub use undo::{
    History, UndoContext, UndoError,
    NavigateFileCommand, ReorderTagCommand, ToggleTagCommand, PasteTagsCommand,
    DeleteTagCommand, CreateTagCommand, SaveTagCommand, StarTagCommand,
};
