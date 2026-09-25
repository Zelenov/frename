//! UI for the folder list. Only this module knows the list is scrollable and how rows look.
//!
//! Receives only data (directory, loading, tag color mapping) from workspace; selection from directory; no parent knows our layout or widgets.

use iced::widget::{
    button, checkbox, column, container, mouse_area, row, scrollable, text, text_input, tooltip,
};
use iced::{mouse, Element, Length};

use crate::features::batch::{BatchState, ItemStatus};
use crate::tag_colors::TagPalette;
use crate::theme;
use crate::widgets;
use crate::widgets::search_bar::FILE_SEARCH_BAR_INPUT_ID;

use super::Message;
use super::{InlineRename, FOLDER_LIST_SCROLLABLE_ID, FOLDER_RENAME_INPUT_ID, FOLDER_ROW_HEIGHT};

const SUBTITLES_MARKER_WIDTH: f32 = 28.0;
/// Width of the check box column in batch mode.
const CHECK_WIDTH: f32 = 28.0;
/// Size of the check boxes, so the header box lines up with the rows' boxes.
const CHECK_SIZE: f32 = 16.0;

/// Render the folder panel: a scrollable list of file names (tag chips + name.extension, no wrap).
/// Selection comes from the directory; view emits SelectFile/Previous/Next.
/// `batch` is the batch state while batch mode is on: rows get a check box, or the file's
/// outcome while a job has it.
pub fn view<'a>(
    directory: Option<&'a crate::features::folder_workspace::Directory>,
    loading: bool,
    tag_color_mapping: &frename_core::TagColorMapping,
    tag_palette: TagPalette,
    rename: Option<&'a InlineRename>,
    spinner_frame: usize,
    batch: Option<&'a BatchState>,
) -> Element<'a, Message> {
    let placeholder_icon = |icon: &'static str| {
        container(text(icon).size(48).color(theme::TEXT_MUTED))
            .padding([8, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::panel_container_style)
    };

    if loading {
        return placeholder_icon("⏳").into();
    }

    let Some(dir) = directory else {
        return placeholder_icon("📂").into();
    };

    if dir.is_empty() {
        return placeholder_icon("📭").into();
    }

    let selected_index = dir.selected_index();
    // A running job locks the list: no other file may open while files are written.
    let locked = batch.is_some_and(|b| b.is_running());

    let items: Vec<Element<'_, Message>> = dir
        .files_in_order()
        .enumerate()
        .map(|(index, file_info)| {
            let is_selected = selected_index == Some(index);
            let name_display = widgets::file_name_display::view(
                file_info.snapshot(),
                tag_color_mapping,
                tag_palette,
                false,
            );

            // On the name line, next to the tags: centred on the whole row it would float
            // between the name and the comment line.
            let subtitles_icon: Element<'_, Message> = if file_info.has_subtitles() {
                container(text("SRT").size(9).color(theme::ACCENT))
                    .center_x(Length::Fixed(SUBTITLES_MARKER_WIDTH))
                    .into()
            } else {
                container(iced::widget::Space::new())
                    .width(Length::Fixed(SUBTITLES_MARKER_WIDTH))
                    .into()
            };

            // In batch mode the check box leads the name line, level with the tags whether or
            // not a comment line follows.
            let check_width = if batch.is_some() { CHECK_WIDTH } else { 0.0 };
            let mut name_line = row![].align_y(iced::Alignment::Center);
            if let Some(batch) = batch {
                name_line = name_line.push(check_cell(batch, file_info.id(), locked));
            }
            let name_line = name_line.push(subtitles_icon).push(name_display);
            // Indented like the name, so the comment starts under it and not under the marker.
            let comment_line = row![
                iced::widget::Space::new()
                    .width(Length::Fixed(check_width + SUBTITLES_MARKER_WIDTH)),
                comment_line(file_info.snapshot(), spinner_frame),
            ];

            let editing = rename.filter(|r| r.id == file_info.id() && batch.is_none());
            let name_area: Element<'_, Message> = if let Some(rename) = editing {
                row![
                    iced::widget::Space::new().width(Length::Fixed(SUBTITLES_MARKER_WIDTH)),
                    rename_editor(rename),
                ]
                .into()
            } else {
                let area = mouse_area(
                    container(column![name_line, comment_line].spacing(2))
                        .padding(iced::Padding {
                            top: 4.0,
                            right: 8.0,
                            bottom: 4.0,
                            left: 0.0,
                        })
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_y(Length::Fill)
                        .clip(true),
                );
                match (locked, batch.is_some()) {
                    (true, _) => area.into(),
                    // Batch mode previews files but does not edit them, renaming included.
                    (false, true) => area
                        .on_press(Message::SelectFile(index))
                        .interaction(mouse::Interaction::Pointer)
                        .into(),
                    (false, false) => area
                        .on_press(Message::SelectFile(index))
                        .on_double_click(Message::StartRename(index))
                        .interaction(mouse::Interaction::Pointer)
                        .into(),
                }
            };

            container(
                row![name_area]
                    .align_y(iced::Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fixed(FOLDER_ROW_HEIGHT))
            .style(move |theme: &iced::Theme| theme::selectable_row_style(theme, is_selected))
            .into()
        })
        .collect();

    // A filter hides everything: say so instead of showing an empty scrollable.
    let body: Element<'_, Message> = if items.is_empty() {
        let icon = if dir.name_filter().trim().is_empty() {
            "✓"
        } else {
            "🔍"
        };
        container(text(icon).size(48).color(theme::TEXT_MUTED))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else {
        scrollable(column(items).width(Length::Fill))
            .id(iced::widget::Id::new(FOLDER_LIST_SCROLLABLE_ID))
            .height(Length::Fill)
            // Scrollbar beside the rows, not over them: long names and the rename field
            // would otherwise run underneath it.
            .spacing(2)
            .on_scroll(|viewport| {
                let offset = viewport.absolute_offset();
                Message::Scrolled {
                    scroll_y: offset.y,
                    viewport_height: viewport.bounds().height,
                }
            })
            .style(theme::dark_scrollable_style)
            .into()
    };

    let search = widgets::search_bar::view(
        FILE_SEARCH_BAR_INPUT_ID,
        dir.name_filter(),
        Message::SetNameFilter,
        || Message::SetNameFilter(String::new()),
        None::<fn(String) -> Message>,
    );
    let mut content = column![search].spacing(4);
    if let Some(batch) = batch {
        content = content.push(batch_header(dir, batch, locked));
    }

    container(content.push(body))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_container_style)
        .into()
}

/// Batch mode header: check or uncheck every listed file, invert, and how many are checked.
fn batch_header<'a>(
    dir: &'a crate::features::folder_workspace::Directory,
    batch: &'a BatchState,
    locked: bool,
) -> Element<'a, Message> {
    let mut listed = dir.files_in_order().peekable();
    let any_listed = listed.peek().is_some();
    let all_checked = any_listed && listed.all(|f| batch.is_checked(f.id()));
    let mut all = checkbox(all_checked)
        .label("All")
        .text_size(12)
        .size(CHECK_SIZE);
    if !locked {
        all = all.on_toggle(|_| Message::ToggleAllChecked);
    }
    let invert = button(text("Invert").size(12))
        .on_press_maybe((!locked).then_some(Message::InvertChecks))
        .padding([2, 8])
        .style(theme::icon_button_style(!locked));
    // The box sits where the rows' boxes sit: centred in the same first column.
    let inset = (CHECK_WIDTH - CHECK_SIZE) / 2.0;
    row![
        container(all).padding(iced::Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: inset
        }),
        invert,
        iced::widget::Space::new().width(Length::Fill),
        text(format!("{} checked", batch.checked_count()))
            .size(12)
            .color(theme::TEXT_MUTED),
    ]
    .spacing(8)
    .padding(iced::Padding {
        top: 0.0,
        right: 8.0,
        bottom: 0.0,
        left: 0.0,
    })
    .align_y(iced::Alignment::Center)
    .into()
}

/// The check box leading a row in batch mode. A file the last job finished keeps its box, tinted
/// with the outcome until it is clicked (which unchecks it) or the report is closed: green when
/// the file now is as the action wants it, red with a cross when it failed.
fn check_cell(
    batch: &BatchState,
    id: frename_core::FileId,
    locked: bool,
) -> Element<'static, Message> {
    // The file in work keeps its plain box: most files take milliseconds, so anything shown
    // for them would only flicker. The job panel names the file in work.
    let outcome = match batch.status(id) {
        Some(ItemStatus::Done) => Some((theme::VOLUME, None, "Changed")),
        Some(ItemStatus::Skipped) => Some((theme::VOLUME, None, "Nothing to change")),
        Some(ItemStatus::Failed) => Some((theme::ERROR, Some('✕'), "Failed, see the log")),
        Some(ItemStatus::Pending | ItemStatus::Running) | None => None,
    };
    let mut check = checkbox(batch.is_checked(id)).size(CHECK_SIZE);
    if !locked {
        check = check.on_toggle(move |_| Message::ToggleChecked(id));
    }
    let content: Element<'static, Message> = match outcome {
        None => check.into(),
        Some((color, mark, hint)) => {
            let mut check = check.style(theme::outcome_checkbox_style(color));
            if let Some(mark) = mark {
                check = check.icon(checkbox::Icon {
                    font: iced::Font::DEFAULT,
                    code_point: mark,
                    size: None,
                    line_height: text::LineHeight::default(),
                    shaping: text::Shaping::Advanced,
                });
            }
            tooltip(
                check,
                container(text(hint))
                    .padding([2, 6])
                    .style(theme::elevated_container_style),
                tooltip::Position::Right,
            )
            .into()
        }
    };
    container(content)
        .center_x(Length::Fixed(CHECK_WIDTH))
        .into()
}

/// Spinner frames for rows whose comment is still loading.
const SPINNER: [&str; 4] = ["◐", "◓", "◑", "◒"];

/// The line under a file's name: the first line of its comment, a spinner while the comment
/// is still loading, or nothing. Always one line high, so every row
/// keeps [`FOLDER_ROW_HEIGHT`] and the comment never wraps into the next row.
fn comment_line(
    snapshot: &frename_core::FileSnapshot,
    spinner_frame: usize,
) -> Element<'_, Message> {
    let line = if snapshot.comment_loading() {
        SPINNER[spinner_frame % SPINNER.len()]
    } else {
        snapshot.comment().lines().next().unwrap_or_default()
    };
    text(line)
        .size(11)
        .color(theme::TEXT_MUTED)
        .wrapping(iced::widget::text::Wrapping::None)
        .into()
}

/// The in-place rename editor that replaces a row's name: the whole file name in a text
/// field, with a red border and the reason next to it when Enter was refused.
fn rename_editor(rename: &InlineRename) -> Element<'_, Message> {
    let border = if rename.error.is_some() {
        theme::ERROR
    } else {
        theme::ACCENT
    };
    let input = text_input("", &rename.text)
        .id(iced::widget::Id::from(FOLDER_RENAME_INPUT_ID))
        .on_input(Message::RenameInput)
        .on_submit(Message::SubmitRename)
        .size(14)
        .padding([4, 6])
        .style(
            move |_theme: &iced::Theme, _status: text_input::Status| text_input::Style {
                background: iced::Background::Color(theme::BG_ELEVATED),
                border: iced::Border {
                    radius: 2.0.into(),
                    width: 1.0,
                    color: border,
                },
                icon: theme::TEXT_MUTED,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT,
                selection: theme::ACCENT_SELECTED,
            },
        );
    let mut content = row![input].spacing(6).align_y(iced::Alignment::Center);
    if let Some(error) = rename.error {
        content = content.push(text(error).size(11).color(theme::ERROR));
    }
    container(content)
        .padding(iced::Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
