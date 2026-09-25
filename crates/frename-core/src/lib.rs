//! Core logic for frename - file renaming utility.

pub(crate) mod comment;
mod db;
mod directory;
mod file;
mod file_kind;
mod folder_file;
mod metadata;
mod ordered;
mod subtitles;
mod tags;
pub(crate) mod transliteration;
pub mod undo;

pub use db::{
    AppDatabase, AppSettings, AppStateStore, Initializable, LoggingAppStateStore, StoredTagStore,
    VideoSettings, WindowGeometry,
};
pub use directory::Directory;
pub use file::{File, FileId};
pub use file_kind::FileKind;
pub use folder_file::FolderAndFile;
pub use metadata::{
    active_commented_tag, cache::modified_ms, clean_commented_tag, commented_tag, metadata_storage,
    set_comment_storage, set_commented_tag, set_in_out_storage, CommentStorage, InOutStorage,
    MetadataMove, MetadataStorage, MoveOutcome, DEFAULT_COMMENTED_TAG,
};
pub use ordered::{OrderKey, OrderableEntry, OrderedCollection, OrderedThing};
pub use subtitles::{load_subtitles, subtitle_path, SubtitleCue, Subtitles};
pub use tags::{
    install_file_tagger, CachedFile, DefaultTag, FileSnapshot, FileTagger, FileTaggerBackend,
    FolderInfo, FolderTagStore, InMemoryFileTagger, LoggingFileTagger, ProductionFileTagger,
    SaveAndReparse, Screenshot, StoredTag, Tag, TagColorMapping, TagId, TagList, DEFAULT_TAGS,
};
pub use undo::{
    CreateTagCommand, DeleteTagCommand, History, NavigateFileCommand, PasteTagsCommand,
    ReorderTagCommand, SaveTagCommand, SetSegmentEndCommand, SetSegmentStartCommand,
    StarTagCommand, ToggleTagCommand, UndoContext, UndoError,
};
