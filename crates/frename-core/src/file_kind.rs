//! File-kind classification based on file extension.

/// Media type classification for a file, based on its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// Video file (.mp4, .mkv, .avi, …)
    Video,
    /// Everything else, images included — not shown in the file list
    Other,
}

impl FileKind {
    /// Classify an extension string (without the leading dot, e.g. `"mp4"`).
    /// Comparison is case-insensitive.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "m4v" | "flv" | "webm" | "ts" | "m2ts"
            | "mpg" | "mpeg" | "3gp" => Self::Video,
            _ => Self::Other,
        }
    }

    /// Returns `true` for `Video`; `false` for `Other`.
    pub fn is_media(self) -> bool {
        !matches!(self, Self::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_are_other() {
        for ext in [
            "jpg", "jpeg", "JPEG", "heic", "HEIC", "heif", "png", "webp", "bmp",
        ] {
            assert_eq!(FileKind::from_extension(ext), FileKind::Other, "{ext}");
        }
    }

    #[test]
    fn images_are_not_media() {
        assert!(!FileKind::from_extension("jpg").is_media());
        assert!(!FileKind::from_extension("heic").is_media());
    }

    #[test]
    fn mp4_is_video() {
        assert_eq!(FileKind::from_extension("mp4"), FileKind::Video);
    }

    #[test]
    fn mkv_is_video() {
        assert_eq!(FileKind::from_extension("mkv"), FileKind::Video);
    }

    #[test]
    fn txt_is_other() {
        assert_eq!(FileKind::from_extension("txt"), FileKind::Other);
    }

    #[test]
    fn empty_is_other() {
        assert_eq!(FileKind::from_extension(""), FileKind::Other);
    }

    #[test]
    fn is_media_video() {
        assert!(FileKind::Video.is_media());
    }

    #[test]
    fn is_media_other_is_false() {
        assert!(!FileKind::Other.is_media());
    }
}
