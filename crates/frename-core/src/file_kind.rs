//! File-kind classification based on file extension.

/// Media type classification for a file, based on its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// Video file (.mp4, .mkv, .avi, …)
    Video,
    /// Static image file (.jpg, .jpeg, .heic, .heif, .png, …)
    Image,
    /// Everything else — not shown in the file list
    Other,
}

impl FileKind {
    /// Classify an extension string (without the leading dot, e.g. `"mp4"`).
    /// Comparison is case-insensitive.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "m4v" | "flv" | "webm" | "ts" | "m2ts"
            | "mpg" | "mpeg" | "3gp" => Self::Video,
            "jpg" | "jpeg" | "heic" | "heif" | "png" | "webp" | "bmp" => Self::Image,
            _ => Self::Other,
        }
    }

    /// Returns `true` for `Video` and `Image`; `false` for `Other`.
    pub fn is_media(self) -> bool {
        !matches!(self, Self::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_is_image() {
        assert_eq!(FileKind::from_extension("jpg"), FileKind::Image);
    }

    #[test]
    fn jpeg_uppercase_is_image() {
        assert_eq!(FileKind::from_extension("JPEG"), FileKind::Image);
    }

    #[test]
    fn heic_is_image() {
        assert_eq!(FileKind::from_extension("heic"), FileKind::Image);
    }

    #[test]
    fn heif_is_image() {
        assert_eq!(FileKind::from_extension("heif"), FileKind::Image);
    }

    #[test]
    fn png_is_image() {
        assert_eq!(FileKind::from_extension("png"), FileKind::Image);
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
    fn is_media_image() {
        assert!(FileKind::Image.is_media());
    }

    #[test]
    fn is_media_other_is_false() {
        assert!(!FileKind::Other.is_media());
    }
}
