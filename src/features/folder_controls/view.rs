//! UI for folder controls (prev/next). Only this module knows they are buttons; receives only booleans.

use iced::widget::{button, checkbox, container, mouse_area, row, text, tooltip};
use iced::Element;

use crate::features::folder;
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;

/// Dark checkbox style for the untagged filter: dark background, accent when checked.
fn dark_checkbox_style(
    _theme: &iced::Theme,
    status: iced::widget::checkbox::Status,
) -> iced::widget::checkbox::Style {
    let (is_checked, hovered) = match status {
        iced::widget::checkbox::Status::Hovered { is_checked } => (is_checked, true),
        iced::widget::checkbox::Status::Active { is_checked }
        | iced::widget::checkbox::Status::Disabled { is_checked } => (is_checked, false),
    };
    let background = match (is_checked, hovered) {
        (true, _) => theme::ACCENT,
        (false, true) => theme::SPLITTER_ACTIVE,
        (false, false) => theme::TRACK,
    };
    iced::widget::checkbox::Style {
        background: iced::Background::Color(background),
        icon_color: theme::TEXT,
        border: iced::Border {
            radius: 2.0.into(),
            width: 1.0,
            color: theme::TEXT_MUTED,
        },
        text_color: Some(theme::TEXT),
    }
}

/// "Untagged only" toggle plus the number of files still without tags.
/// Sits at the right end of the controls bar, in the space the buttons leave free.
fn untagged_filter(untagged_only: bool, untagged_count: usize) -> Element<'static, folder::Message> {
    let label_color = if untagged_only { theme::TEXT } else { theme::TEXT_MUTED };
    let toggle = checkbox(untagged_only)
        .on_toggle(folder::Message::SetUntaggedOnly)
        .size(14)
        .spacing(0)
        .style(dark_checkbox_style);

    // The label toggles too; the checkbox itself is excluded so one click is one toggle.
    let label = mouse_area(
        row![
            text("Untagged").size(12).color(label_color),
            text(untagged_count.to_string()).size(12).color(theme::TEXT_MUTED),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .on_press(folder::Message::SetUntaggedOnly(!untagged_only))
    .interaction(iced::mouse::Interaction::Pointer);

    tooltip(
        row![toggle, label]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        text("Show only files without tags"),
        iced::widget::tooltip::Position::Top,
    )
    .into()
}

/// Render the folder controls: Previous File, Next File, and Scroll-to-Selected buttons.
/// Buttons are enabled only when applicable.
pub fn view(
    has_previous: bool,
    has_next: bool,
    has_selected: bool,
    untagged_only: bool,
    untagged_count: usize,
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

    let controls = row![
        prev_btn,
        next_btn,
        scroll_btn,
        open_btn,
        container(iced::widget::Space::new()).width(iced::Length::Fill),
        untagged_filter(untagged_only, untagged_count),
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
