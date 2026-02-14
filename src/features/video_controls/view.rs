//! UI rendering for video controls feature

use iced::widget::{button, container, text};
use iced::Element;

use super::{Message, VideoControlsState};

const BUTTON_SIZE: f32 = 40.0;

/// Render the video player controls
pub fn view(state: &VideoControlsState) -> Element<'_, Message> {
    let play_pause_label = if state.is_playing() { "⏸" } else { "▶" };
    let play_pause_btn = button(text(play_pause_label).size(20))
        .on_press(Message::TogglePlayPause)
        .width(BUTTON_SIZE)
        .height(BUTTON_SIZE)
        .padding(0);

    container(play_pause_btn)
        .padding(8)
        .width(iced::Length::Fill)
        .into()
}
