//! UI rendering for the folder feature

use iced::widget::{column, container, mouse_area, scrollable, text};
use iced::{mouse, Background, Color, Element, Length};

use super::{FolderState, Message};

/// Render the folder panel: a scrollable list of file names
pub fn view(state: &FolderState) -> Element<'_, Message> {
    if state.is_loading() {
        return container(text("Scanning folder...").size(13))
            .padding([8, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    if state.files().is_empty() {
        return container(text("Drop a file to open its folder").size(13))
            .padding([8, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let selected = state.selected_index();

    let items: Vec<Element<'_, Message>> = state
        .files()
        .iter()
        .enumerate()
        .map(|(index, file_info)| {
            let name = file_info
                .file_path()
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("???");

            let is_selected = selected == Some(index);

            let label = text(name).size(13);

            let row = container(label)
                .padding([3, 8])
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme| {
                    if is_selected {
                        container::Style {
                            background: Some(Background::Color(Color::from_rgba(
                                0.3, 0.5, 0.8, 0.35,
                            ))),
                            ..Default::default()
                        }
                    } else {
                        container::Style::default()
                    }
                });

            mouse_area(row)
                .on_press(Message::SelectFile(index))
                .interaction(mouse::Interaction::Pointer)
                .into()
        })
        .collect();

    let list = scrollable(column(items).width(Length::Fill))
        .id(state.scrollable_id().clone())
        .height(Length::Fill);

    container(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
