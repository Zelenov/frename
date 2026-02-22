//! Messages for the image viewer sub-feature.

use iced::widget::image;

/// Messages handled by the image viewer.
#[derive(Debug, Clone)]
pub enum Message {
    /// Async decode completed: Ok(handle) on success, Err(description) on failure.
    ImageLoaded(Result<image::Handle, String>),
}
