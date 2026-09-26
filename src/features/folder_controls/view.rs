//! UI for folder controls (prev/next). Only this module knows they are buttons; receives only booleans.

use iced::widget::{button, container, row, text, tooltip};
use iced::Element;

use crate::features::folder;
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the folder controls: Previous File, Next File, Scroll-to-Selected, Open, Settings and
/// Batch mode buttons. Buttons are enabled only when applicable. `batch_mode` highlights the
/// batch button; `batch_running` locks it while a job runs. `update_available`, a newer frename
/// version, puts a dot on the Settings button, where the update is.
pub fn view(
    has_previous: bool,
    has_next: bool,
    has_selected: bool,
    batch_mode: bool,
    batch_running: bool,
    update_available: Option<String>,
) -> Element<'static, folder::Message> {
    let prev_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("◀").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press(folder::Message::PreviousFile)
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::icon_button_style(has_previous)),
        text("Page Up"),
        iced::widget::tooltip::Position::Top,
    )
    .into();

    let next_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("▶").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press(folder::Message::NextFile)
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::icon_button_style(has_next)),
        text("Page Down"),
        iced::widget::tooltip::Position::Top,
    )
    .into();

    let scroll_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("⊙").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press(folder::Message::ScrollToSelected)
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::icon_button_style(has_selected)),
        text("Scroll to file"),
        iced::widget::tooltip::Position::Top,
    )
    .into();

    let open_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("📂").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press(folder::Message::OpenFolder)
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::icon_button_style(true)),
        text("Open file"),
        iced::widget::tooltip::Position::Top,
    )
    .into();

    let settings_button = button(
        container(text("⚙").size(16))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill),
    )
    .on_press(folder::Message::OpenSettings)
    .width(CONTROLS_HEIGHT)
    .height(iced::Length::Fill)
    .padding(0)
    .style(theme::icon_button_style(true));
    let (settings_face, settings_hint): (Element<'_, folder::Message>, String) =
        match update_available {
            Some(version) => (
                iced::widget::stack![settings_button, update_dot()].into(),
                format!("Update available: {version}"),
            ),
            None => (settings_button.into(), "Settings".to_string()),
        };
    let settings_btn: Element<'_, folder::Message> = tooltip(
        settings_face,
        text(settings_hint),
        iced::widget::tooltip::Position::Top,
    )
    .into();

    let batch_hint = if batch_mode {
        "Back to the open file"
    } else {
        "Batch actions on checked files"
    };
    let batch_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("☑").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press_maybe((!batch_running).then_some(folder::Message::SetBatchMode(!batch_mode)))
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::list_item_button_style(batch_mode, !batch_running)),
        text(batch_hint),
        iced::widget::tooltip::Position::Top,
    )
    .into();

    let controls = row![
        prev_btn,
        next_btn,
        scroll_btn,
        open_btn,
        settings_btn,
        batch_btn,
    ]
    .spacing(8)
    .height(iced::Length::Fill)
    .align_y(iced::Alignment::Center);

    container(controls)
        .padding([0, 8])
        .width(iced::Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style)
        .into()
}

/// The accent dot in the Settings button's top right corner: a newer frename is available.
fn update_dot() -> Element<'static, folder::Message> {
    const DOT: f32 = 7.0;
    container(
        container(iced::widget::Space::new())
            .width(DOT)
            .height(DOT)
            .style(|_| container::Style {
                background: Some(theme::ACCENT.into()),
                border: iced::Border {
                    radius: (DOT / 2.0).into(),
                    ..iced::Border::default()
                },
                ..container::Style::default()
            }),
    )
    .width(CONTROLS_HEIGHT)
    .align_right(CONTROLS_HEIGHT)
    .padding(3)
    .into()
}
