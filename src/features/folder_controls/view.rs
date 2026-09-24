//! UI for folder controls (prev/next). Only this module knows they are buttons; receives only booleans.

use iced::widget::{button, container, pick_list, row, text, tooltip};
use iced::Element;

use crate::features::folder;
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;

/// The folder list filters: whether each is on, and how many files in the folder match it.
#[derive(Debug, Clone, Copy, Default)]
pub struct ListFilters {
    pub untagged_only: bool,
    pub untagged_count: usize,
    pub subtitled_only: bool,
    pub subtitled_count: usize,
    pub commented_only: bool,
    pub commented_count: usize,
}

/// Which filter a dropdown row controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterKind {
    Untagged,
    Subtitles,
    Comments,
}

impl FilterKind {
    fn label(self) -> &'static str {
        match self {
            Self::Untagged => "Untagged",
            Self::Subtitles => "Subtitles",
            Self::Comments => "Comments",
        }
    }

    fn set(self, on: bool) -> folder::Message {
        match self {
            Self::Untagged => folder::Message::SetUntaggedOnly(on),
            Self::Subtitles => folder::Message::SetSubtitledOnly(on),
            Self::Comments => folder::Message::SetCommentedOnly(on),
        }
    }
}

/// One row of the filter dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilterItem {
    kind: FilterKind,
    active: bool,
    count: usize,
}

impl std::fmt::Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mark = if self.active { "✓" } else { "   " };
        write!(f, "{mark} {}  {}", self.kind.label(), self.count)
    }
}

impl ListFilters {
    fn items(self) -> [FilterItem; 3] {
        [
            FilterItem { kind: FilterKind::Untagged, active: self.untagged_only, count: self.untagged_count },
            FilterItem { kind: FilterKind::Subtitles, active: self.subtitled_only, count: self.subtitled_count },
            FilterItem { kind: FilterKind::Comments, active: self.commented_only, count: self.commented_count },
        ]
    }
}

/// Filter dropdown: one row per filter with its match count; picking a row toggles it.
/// The button shows how many filters are on. Sits at the right end of the controls bar.
fn filter_dropdown(filters: ListFilters) -> Element<'static, folder::Message> {
    let items = filters.items();
    let active = items.iter().filter(|item| item.active).count();
    let placeholder = if active == 0 { "Filter".to_string() } else { format!("Filter ({active})") };
    // No tooltip: it would draw over the open list.
    pick_list(items.to_vec(), None::<FilterItem>, |item| item.kind.set(!item.active))
        .placeholder(placeholder)
        .text_size(12)
        .padding([2, 8])
        .into()
}

/// Render the folder controls: Previous File, Next File, Scroll-to-Selected, Open and Settings buttons.
/// Buttons are enabled only when applicable.
pub fn view(
    has_previous: bool,
    has_next: bool,
    has_selected: bool,
    filters: ListFilters,
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

    let settings_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("⚙").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(folder::Message::OpenSettings)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(true)),
        text("Settings"),
        iced::widget::tooltip::Position::Top,
    )
        .into();

    let controls = row![
        prev_btn,
        next_btn,
        scroll_btn,
        open_btn,
        settings_btn,
        container(iced::widget::Space::new()).width(iced::Length::Fill),
        filter_dropdown(filters),
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
