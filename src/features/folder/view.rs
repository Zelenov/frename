//! UI for the file list (`docs/design/design-system.md` §13.4, §13.6.2, §13.7). Only this module
//! knows the list is scrollable and how rows look; the workspace passes the data in
//! [`ListProps`].

use std::collections::HashMap;

use frename_core::{FileId, Marker, TagColorMapping};
use iced::widget::{column, container, row, space, stack, Column};
use iced::{Alignment, Element, Length, Padding};

use crate::features::batch::BatchState;
use crate::features::folder_workspace::Directory;
use crate::ui::button;
use crate::ui::empty;
use crate::ui::icons::{icon, spinner, Icon};
use crate::ui::palette::TagPalette;
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position};
use crate::ui::{form, scroll, style, text};
use crate::widgets::search_bar::{self, SearchBar, FILE_SEARCH_BAR_INPUT_ID};

use super::row::{self as file_row, RowState};
use super::{filter, InlineRename, Message, FOLDER_LIST_SCROLLABLE_ID};

/// What the file list shows.
/// `'m` is the colour mapping's borrow, which only lasts while the rows are built.
pub struct ListProps<'a, 'm> {
    pub directory: Option<&'a Directory>,
    /// A folder is being opened.
    pub loading: bool,
    pub tag_color_mapping: &'m TagColorMapping,
    pub tag_palette: TagPalette,
    pub rename: Option<&'a InlineRename>,
    /// The app's spinner clock.
    pub spinner_frame: usize,
    /// The batch state while batch mode is on: rows get the check column.
    pub batch: Option<&'a BatchState>,
    /// Files whose markers could not be written.
    pub markers_not_saved: &'a HashMap<FileId, Vec<Marker>>,
    /// The filter menu is open over the list.
    pub filter_menu_open: bool,
    /// The list's width, which the names are fitted to.
    pub width: f32,
}

/// The inset of the lock line and the batch header, level with the rows' content.
const STRIP_PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_S,
    right: SPACE_S,
};

/// Render the file list: the lock line while a job runs, the search bar, the batch header in
/// batch mode, and the rows (or what is missing).
pub fn view<'a>(props: ListProps<'a, '_>) -> Element<'a, Message> {
    let body = match props.directory {
        _ if props.loading => opening(props.spinner_frame),
        None => space().into(),
        Some(dir) => list(&props, dir),
    };
    let mut top = Column::new();
    let locked_by = props.batch.filter(|b| b.is_running());
    if let Some(batch) = locked_by {
        top = top.push(lock_line(batch));
    }
    if let Some(dir) = props.directory.filter(|_| !props.loading) {
        top = top.push(search(dir, props.filter_menu_open));
        // The batch header joins the top column rather than the outer one, so the list keeps
        // its place in the widget tree and with it its scroll position when batch mode turns
        // on or off.
        if let Some(batch) = props.batch {
            top = top.push(batch_header(dir, batch, locked_by.is_some()));
        }
    }
    // The filter menu floats over the rows. The layers are always there (a space when the menu
    // is closed), so the list keeps its place in the widget tree and its scroll position.
    let menu: Element<'a, Message> = match props.directory {
        Some(dir) if props.filter_menu_open && !props.loading => filter::menu(dir),
        _ => space().into(),
    };
    container(column![top, stack![body, menu]])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::panel)
        .into()
}

/// The rows, or why there are none.
fn list<'a>(props: &ListProps<'a, '_>, dir: &'a Directory) -> Element<'a, Message> {
    if dir.is_empty() {
        return empty::pane(
            Icon::FolderX,
            fl!("folder-empty-title"),
            Some(fl!("folder-empty-line")),
            Some(
                button::secondary(fl!("folder-open-another"))
                    .on_press(Message::OpenFolder)
                    .into(),
            ),
        );
    }
    if dir.listed_count() == 0 {
        return empty::pane(
            Icon::SearchX,
            fl!("folder-no-match"),
            None,
            Some(
                button::secondary(fl!("folder-show-all"))
                    .on_press(Message::ShowAll)
                    .into(),
            ),
        );
    }
    let selected = dir.selected_index();
    // A running job locks the list: no other file may open while files are written.
    let locked = props.batch.is_some_and(|b| b.is_running());
    let failures: HashMap<FileId, &str> = props
        .batch
        .map(|b| {
            b.failed()
                .into_iter()
                .filter_map(|(id, reason)| reason.map(|r| (id, r)))
                .collect()
        })
        .unwrap_or_default();
    let rows = dir.files_in_order().enumerate().map(|(index, file)| {
        file_row::view(
            props,
            file,
            RowState {
                index,
                selected: selected == Some(index),
                locked,
                failure: failures.get(&file.id()).copied(),
            },
        )
    });
    scroll::vertical_with_id(FOLDER_LIST_SCROLLABLE_ID, Column::with_children(rows))
        .on_scroll(|viewport| Message::Scrolled {
            scroll_y: viewport.absolute_offset().y,
            viewport_height: viewport.bounds().height,
        })
        .into()
}

/// A folder is being opened.
fn opening<'a>(spinner_frame: usize) -> Element<'a, Message> {
    container(
        column![
            spinner(spinner_frame, ICON_L, TEXT_SECONDARY),
            text::secondary(fl!("folder-opening"))
        ]
        .spacing(SPACE_S)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

fn search(dir: &Directory, menu_open: bool) -> Element<'_, Message> {
    search_bar::view(
        SearchBar {
            input_id: FILE_SEARCH_BAR_INPUT_ID,
            placeholder: fl!("folder-search-placeholder"),
            value: dir.name_filter(),
            clear_tip: fl!("folder-search-clear"),
            on_clear: Message::SetNameFilter(String::new()),
            on_submit: Message::SetNameFilter(dir.name_filter().to_string()),
            trailing: Some(filter::button(dir, menu_open)),
        },
        Message::SetNameFilter,
    )
}

/// The line over the list while a job runs: why the rows take no clicks (§13.2 "Locks").
fn lock_line<'a>(batch: &BatchState) -> Element<'a, Message> {
    let action = batch.job_action().unwrap_or(batch.action()).label();
    container(
        row![
            icon(Icon::Lock, ICON_MARK, TEXT_SECONDARY),
            text::secondary(fl!("folder-locked", action = action))
        ]
        .spacing(SPACE_S)
        .align_y(Alignment::Center),
    )
    .padding(STRIP_PADDING)
    .width(Length::Fill)
    .center_y(LOCK_LINE_HEIGHT)
    .style(style::raised)
    .into()
}

/// How many checked files the search or a filter hides: they are in the job too.
fn hidden_checked(checked: usize, listed_checked: usize) -> usize {
    checked.saturating_sub(listed_checked)
}

/// Batch mode header (§13.6.2): check or uncheck every listed file, invert, and how many are
/// checked, hidden ones too.
fn batch_header<'a>(
    dir: &'a Directory,
    batch: &'a BatchState,
    locked: bool,
) -> Element<'a, Message> {
    let listed = dir.listed_count();
    let listed_checked = dir
        .files_in_order()
        .filter(|f| batch.is_checked(f.id()))
        .count();
    let all_checked = listed > 0 && listed_checked == listed;
    let all = form::checkbox(fl!("folder-all"), all_checked);
    let all = if locked {
        all
    } else {
        all.on_toggle(|_| Message::ToggleAllChecked)
    };
    let invert = button::ghost(fl!("folder-invert"))
        .on_press_maybe((!locked).then_some(Message::InvertChecks));
    let count = batch.checked_count() as i64;
    let hidden = hidden_checked(batch.checked_count(), listed_checked);
    let checked: Element<'a, Message> = if hidden == 0 {
        text::secondary(fl!("folder-checked", count = count)).into()
    } else {
        let hidden = hidden as i64;
        tooltip::tip_text(
            text::secondary(fl!("folder-checked-hidden", count = count, hidden = hidden)),
            fl!("folder-checked-hidden-tip", hidden = hidden),
            Position::Bottom,
        )
    };
    // The box sits where the rows' boxes sit: centred in the same first column.
    let inset = (CHECK_COLUMN - CHECK_SIZE) / 2.0;
    container(
        row![
            container(all).padding(Padding {
                left: inset,
                ..Padding::ZERO
            }),
            invert,
            space::horizontal(),
            checked,
        ]
        .spacing(SPACE_S)
        .align_y(Alignment::Center),
    )
    .padding(STRIP_PADDING)
    .width(Length::Fill)
    .center_y(ROW_HEIGHT)
    .into()
}

#[cfg(test)]
mod tests {
    use super::hidden_checked;

    #[test]
    fn checked_files_outside_the_list_count_as_hidden() {
        assert_eq!(hidden_checked(12, 9), 3);
        assert_eq!(hidden_checked(4, 4), 0);
        assert_eq!(hidden_checked(0, 0), 0);
    }
}
