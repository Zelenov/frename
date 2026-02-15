//! State for file workspace: the file currently being edited and its tag selection.
//!
//! Public interface: set the file to work on (None or Some). The component owns loading
//! and all internal behaviour; callers only use `set_file` and `file()`.

use frename_core::File;

/// File workspace: operates on the currently selected file. You set the file (None or Some);
/// the component does the rest internally (loading, etc.). Only public API: `set_file`, `file`.
#[derive(Default)]
pub struct FileWorkspace {
    /// Internal: true while loading the file (e.g. async load). Not exposed.
    loading: bool,
    /// Current file when ready. None while loading or when nothing set.
    file: Option<File>,
}

impl FileWorkspace {
    /// Set the file to work on. None = clear. Some(file) = load and work on it; internals handle loading.
    pub fn set_file(&mut self, file: Option<File>) {
        match file {
            None => {
                self.loading = false;
                self.file = None;
            }
            Some(f) => {
                let already_loaded = self
                    .file
                    .as_ref()
                    .map_or(false, |current| current.file_path() == f.file_path());
                if already_loaded {
                    return;
                }
                self.loading = true;
                self.file = None;
                // Internal: "load" (sync for now; can become async later). When ready, set file and clear loading.
                self.file = Some(f);
                self.loading = false;
            }
        }
    }

    /// The current file when ready. Returns None if nothing set or still loading.
    pub fn file(&self) -> Option<&File> {
        if self.loading {
            return None;
        }
        self.file.as_ref()
    }

    /// Toggle a tag by index and rebuild the displayed file name.
    pub fn toggle_tag(&mut self, index: usize) {
        if let Some(file) = self.file.as_mut() {
            file.toggle_tag(index);
        }
    }
}
