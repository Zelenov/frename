//! Widget: file name + extension text line (no dots between name and extension).

use iced::widget::{container, text};
use iced::{Element, Length};

use crate::theme;

use super::Message;

/// Renders the file name line: name and extension concatenated (extension already includes dot when needed).
pub fn view(name_ext: impl Into<String>) -> Element<'static, Message> {
    container(text(name_ext.into()).size(14).color(theme::TEXT))
        .width(Length::Fill)
        .padding([6, 0])
        .into()
}

/// Builds display string from name and extension parts (no extra dot).
pub fn name_ext_from_parts(name: &str, ext: &str) -> String {
    if name.is_empty() {
        ext.to_string()
    } else if ext.is_empty() {
        name.to_string()
    } else {
        format!("{}{}", name, ext)
    }
}
