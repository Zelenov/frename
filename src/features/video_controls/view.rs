//! UI rendering for video controls feature

use iced::widget::{button, container, row, text};
use iced::{Background, Color, Element};

use super::progress_bar::ProgressBar;
use super::{Message, VideoControlsState};

const CONTROLS_HEIGHT: f32 = 32.0;
const CONTROLS_BG: Color = Color::from_rgb(0.12, 0.12, 0.12);

/// Render the video player controls
pub fn view(state: &VideoControlsState) -> Element<'_, Message> {
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
                button::Status::Hovered => Color::from_rgb(0.25, 0.25, 0.25),
                button::Status::Pressed => Color::from_rgb(0.3, 0.3, 0.3),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::WHITE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            }
        });

    let duration = state.duration_secs();
    let progress = if duration > 0.0 {
        state.position_secs() / duration
    } else {
        0.0
    };

    let bar = ProgressBar::new(progress, move |fraction| {
        Message::Seek(fraction * duration)
    })
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
            background: Some(Background::Color(CONTROLS_BG)),
            ..container::Style::default()
        })
        .into()
}
