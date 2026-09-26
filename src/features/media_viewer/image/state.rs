//! State for the image viewer sub-feature.

use iced::widget::image;
use iced::Task;
use std::path::{Path, PathBuf};

use super::Message;

/// Image viewer state: holds a decoded image handle once loading completes.
#[derive(Default)]
pub struct ImageViewerState {
    current_handle: Option<image::Handle>,
    loading: bool,
    load_failed: bool,
}

impl ImageViewerState {
    /// Begin loading and decoding an image asynchronously.
    /// The file handle is released as soon as decoding completes.
    pub fn load_image(&mut self, path: PathBuf) -> Task<Message> {
        self.loading = true;
        self.load_failed = false;
        self.current_handle = None;

        Task::future(async move {
            let result = tokio::task::spawn_blocking(move || decode_image(&path))
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            Message::ImageLoaded(result)
        })
    }

    /// Handle image messages.
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ImageLoaded(Ok(handle)) => {
                self.loading = false;
                self.current_handle = Some(handle);
            }
            Message::ImageLoaded(Err(e)) => {
                self.loading = false;
                self.load_failed = true;
                log::error!("Image load failed: {e}");
            }
            // Intercepted by media_viewer/folder_workspace; no-op here.
            Message::ToggleFullscreen => {}
        }
        Task::none()
    }

    /// Clear the current image and reset all state.
    pub fn unload(&mut self) {
        *self = Self::default();
    }

    pub fn current_handle(&self) -> Option<&image::Handle> {
        self.current_handle.as_ref()
    }
    pub fn is_loading(&self) -> bool {
        self.loading
    }
    pub fn load_failed(&self) -> bool {
        self.load_failed
    }
}

// ---------------------------------------------------------------------------
// Decoding (blocking — runs on the thread pool)
// ---------------------------------------------------------------------------

fn decode_image(path: &Path) -> Result<image::Handle, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" => decode_via_image_crate(path),
        #[cfg(feature = "heic")]
        "heic" | "heif" => decode_heic(path),
        #[cfg(not(feature = "heic"))]
        "heic" | "heif" => {
            Err("HEIC/HEIF support is not compiled in (enable the 'heic' feature)".to_string())
        }
        _ => Err(format!("Unsupported image extension: {ext}")),
    }
}

fn decode_via_image_crate(path: &Path) -> Result<image::Handle, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let img = image::Handle::from_bytes(bytes);
    // iced's Handle::from_bytes accepts any format the image crate supports
    // (JPEG, PNG, WebP, BMP) and decodes lazily at render time.
    Ok(img)
}

#[cfg(feature = "heic")]
fn decode_heic(path: &Path) -> Result<image::Handle, String> {
    use libheif_rs::{ColorSpace, LibHeif, RgbChroma};

    let ctx = LibHeif::new();
    let handle = ctx
        .read_from_file(path.to_str().ok_or("Path is not valid UTF-8")?)
        .map_err(|e| e.to_string())?;
    let decoded = ctx
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)
        .map_err(|e| e.to_string())?;

    let planes = decoded.planes();
    let interleaved = planes
        .interleaved
        .ok_or("No interleaved RGBA plane in HEIC image")?;
    let width = decoded.width();
    let height = decoded.height();
    let pixels = interleaved.data.to_vec();

    Ok(image::Handle::from_rgba(width, height, pixels))
}
