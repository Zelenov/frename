//! UI rendering for video controls feature

use iced::widget::{button, container, row, text};
use iced::{Background, Color, Element};

use crate::theme;
use super::progress_bar::ProgressBar;
use super::{Message, VideoControlsState};

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the video player controls.
/// `position_secs` is the live playback position read from the video at view time.
pub fn view(state: &VideoControlsState, position_secs: f32) -> Element<'_, Message> {
    let play_pause_label = if state.is_playing() { "⏸" } else { "▶" };
    let play_pause_btn = button(
        container(text(play_pause_label).size(16))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill),
    )
        .on_press(Message::TogglePlayPause)
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => theme::TRACK,
                button::Status::Pressed => theme::SPLITTER_ACTIVE,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: theme::TEXT,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            }
        });

    let duration = state.duration_secs();
    // While seeking, show the drag position; otherwise use live video position
    let current_pos = if state.is_seeking() {
        state.seek_position_secs()
    } else {
        position_secs
    };

    let bar = ProgressBar::new(0.0..=duration, current_pos, Message::Seek)
        .on_release(Message::SeekReleased);

    let controls = row![play_pause_btn, bar]
        .spacing(8)
        .height(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    container(controls)
        .padding([0, 8])
        .width(iced::Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..container::Style::default()
        })
        .into()
}
