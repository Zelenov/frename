//! UI rendering for the folder feature

use iced::widget::{column, container, mouse_area, row, scrollable, text};
use iced::{mouse, Background, Element, Length};

use crate::theme;
use super::{FolderState, Message};

/// Render the folder panel: a scrollable list of file names
pub fn view(state: &FolderState) -> Element<'_, Message> {
    let placeholder = |s: String| {
        container(
            text(s)
                .size(14)
                .color(theme::TEXT_MUTED),
        )
        .padding([8, 8])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..Default::default()
        })
    };

    if state.is_loading() {
        return placeholder("Scanning folder...".to_string()).into();
    }

    let Some(directory) = state.directory() else {
        return placeholder("Drop a file to open its folder".to_string()).into();
    };

    if directory.is_empty() {
        return placeholder("Folder is empty".to_string()).into();
    }

    let selected = directory.selected_index();
    let applying = state.applying_index();

    let items: Vec<Element<'_, Message>> = directory
        .files()
        .iter()
        .enumerate()
        .map(|(index, file_info)| {
            let name = file_info.initial_filename();
            let is_selected = selected == Some(index);
            let is_applying = applying == Some(index);

            let label = text(name)
                .size(14)
                .color(theme::TEXT);
            let spinner_text = if is_applying { " ⟳" } else { "" };
            let row_content = row![
                label,
                text(spinner_text)
                    .size(14)
                    .color(theme::ACCENT),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center);

            let row = container(row_content)
                .padding([4, 8])
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme| container::Style {
                    background: Some(Background::Color(if is_selected {
                        theme::ACCENT_SELECTED
                    } else {
                        iced::Color::TRANSPARENT
                    })),
                    ..Default::default()
                });

            mouse_area(row)
                .on_press(Message::SelectFile(index))
                .interaction(mouse::Interaction::Pointer)
                .into()
        })
        .collect();

    let list = scrollable(column(items).width(Length::Fill))
        .id(state.scrollable_id().clone())
        .height(Length::Fill)
        .style(theme::dark_scrollable_style);

    container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..Default::default()
        })
        .into()
}
