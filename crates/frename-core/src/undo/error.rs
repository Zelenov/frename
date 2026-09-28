use std::fmt;
use std::path::PathBuf;

use crate::{RotationError, TagId};

#[derive(Debug)]
pub enum UndoError {
    FileNotFound(PathBuf),
    TagNotFound(TagId),
    Io(std::io::Error),
    /// A video could not be turned (back).
    Rotation(PathBuf, RotationError),
}

impl fmt::Display for UndoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UndoError::FileNotFound(p) => write!(f, "File not found: {}", p.display()),
            UndoError::TagNotFound(id) => write!(f, "Tag not found: {:?}", id),
            UndoError::Io(e) => write!(f, "I/O: {}", e),
            UndoError::Rotation(p, e) => write!(f, "{} not turned: {}", p.display(), e),
        }
    }
}

impl std::error::Error for UndoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UndoError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for UndoError {
    fn from(e: std::io::Error) -> Self {
        UndoError::Io(e)
    }
}
