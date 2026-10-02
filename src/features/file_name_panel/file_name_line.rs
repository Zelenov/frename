//! The file name part without the tags, on the card's second line, in mono.

use iced::widget::container;
use iced::{Element, Length};

use crate::ui::text;
use crate::ui::tokens::TEXT;

use super::Message;

/// Renders the file name line: name and extension concatenated (extension already includes dot when needed).
pub fn view(name_ext: impl Into<String>) -> Element<'static, Message> {
    container(text::mono(name_ext.into()).color(TEXT))
        .width(Length::Fill)
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

/// Seconds as `MM:SS`, or `HH:MM:SS` when there are hours.
pub fn fmt_timecode(secs: f32) -> String {
    let total = secs as u32;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h == 0 {
        format!("{m:02}:{s:02}")
    } else {
        format!("{h:02}:{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timecodes_show_hours_only_when_there_are_some() {
        assert_eq!(fmt_timecode(65.9), "01:05");
        assert_eq!(fmt_timecode(3_725.0), "01:02:05");
    }
}
