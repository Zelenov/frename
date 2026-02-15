//! Messages from the folder list (user actions; workspace handles selection and notifies panels)

/// User actions in the folder list; workspace updates selected file and panels react.
#[derive(Debug, Clone)]
pub enum Message {
    /// User selected a file by its index in the list
    SelectFile(usize),
    /// User pressed previous file (folder controls)
    PreviousFile,
    /// User pressed next file (folder controls)
    NextFile,
}
