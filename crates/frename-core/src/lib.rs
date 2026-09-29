//! Core logic for frename - file renaming utility.

pub mod ai;
mod app_dir;
pub(crate) mod comment;
mod db;
pub mod demo;
mod directory;
mod file;
mod file_kind;
mod folder_file;
mod markers;
mod metadata;
pub mod old_settings;
mod ordered;
mod search;
mod subtitles;
mod tags;
pub(crate) mod transliteration;
pub mod undo;

pub use app_dir::{app_data_dir, log_path, set_app_data_dir, DATA_DIR_VAR};
pub use db::{
    AppDatabase, AppSettings, AppStateStore, BatchRun, Initializable, LoggingAppStateStore,
    StoredTagStore, UpdateCheckState, VideoSettings, WindowGeometry,
};
pub use directory::Directory;
pub use file::{File, FileId};
pub use file_kind::FileKind;
pub use folder_file::{choose_dropped_path, FolderAndFile};
pub use markers::{
    comment_to_markers, format_marker_line, format_marker_time, markers_from_comment,
    markers_into_comment, markers_to_comment, parse_ai_line, parse_marker_line, replace_ai_markers,
    sort_markers, CommentToMarkers, Marker, MarkerColor, MarkerLine, AI_MARKER_COLOR,
    MARKER_SNAP_MS,
};
pub use metadata::{
    active_commented_tag, cache::modified_ms, clean_commented_tag, commented_tag,
    format_in_out_range, marker_storage, metadata_storage, set_comment_storage, set_commented_tag,
    set_in_out_storage, set_marker_storage, CommentStorage, InOutStorage, MarkerStorage,
    MarkersError, MetadataMove, MetadataStorage, MoveOutcome, Rotation, RotationError, Segment,
    DEFAULT_COMMENTED_TAG,
};
pub use ordered::{OrderKey, OrderableEntry, OrderedCollection, OrderedThing};
pub use search::CommentFragment;
pub use subtitles::{
    load_subtitles, subtitle_path, transcript_path, CueLength, SubtitleCue, Subtitles,
    DEFAULT_SUBTITLE_LANGUAGES,
};
pub use tags::{
    install_file_tagger, set_space_after_tags, space_after_tags, CachedFile, DefaultTag,
    FileSnapshot, FileTagger, FileTaggerBackend, FolderInfo, FolderTagStore, InMemoryFileTagger,
    LoggingFileTagger, ProductionFileTagger, SaveAndReparse, Screenshot, StoredTag, Tag,
    TagColorMapping, TagId, TagList, DEFAULT_TAGS,
};
pub use undo::{
    AddMarkerCommand, CreateTagCommand, DeleteMarkerCommand, DeleteTagCommand, History,
    NavigateFileCommand, PasteTagsCommand, ReorderTagCommand, RotateVideoCommand, SaveTagCommand,
    SetMarkerColorCommand, SetMarkerDurationCommand, SetMarkerSpanCommand, SetSegmentEndCommand,
    SetSegmentStartCommand, StarTagCommand, ToggleTagCommand, UndoContext, UndoError,
};
