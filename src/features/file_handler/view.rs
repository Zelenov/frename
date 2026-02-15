//! UI rendering for file handler feature

use iced::Element;

use crate::features::video_player;

use super::{FileHandlerState, Message};

/// Render the file handler view (delegates to the appropriate viewer)
pub fn view(state: &FileHandlerState) -> Element<'_, Message> {
    video_player::view::view(state.video_player()).map(Message::VideoPlayer)
}
