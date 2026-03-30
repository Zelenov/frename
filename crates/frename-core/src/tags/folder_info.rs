//! Folder-level metadata used during bulk file parsing.

#[derive(Debug, Clone, Default)]
pub struct FolderInfo {
    file_names: Vec<String>,
}

impl FolderInfo {
    pub fn new(file_names: Vec<String>) -> Self {
        Self { file_names }
    }

    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }
}
