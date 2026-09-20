//! UI for file workspace: search bar, tag list, file name panel (wrap=true feature) below.
//!
//! Only this module knows the layout of the file workspace region. Folder list keeps using
//! [crate::widgets::file_name_display] with wrap=false.

use iced::keyboard::key::Named;
use iced::widget::text_editor::Binding;
use iced::widget::{column, container, text_editor as text_editor_widget};
use iced::{Element, Length};

use crate::widgets::starred_tags_panel;

use crate::features::{file_name_panel, sync_panel, tag_grid, tag_panel};
use crate::widgets;

use super::{FileWorkspace, Message};

/// Horizontal padding for the file workspace panel (same inset from splitter and window edge).
const PANEL_PADDING_X: f32 = 8.0;

/// Bottom inset so the comment box keeps the same margin as the sides instead of sitting
/// flush against the window edge. The top stays at 0 to line up with the other columns.
const PANEL_PADDING_BOTTOM: f32 = 8.0;

/// Render the file workspace: search bar, tag list, file name panel (feature, wrap=true) below.
pub fn view<'a, S>(
    file_workspace: &'a FileWorkspace<S>,
    tag_panel_state: &'a tag_panel::TagPanelState,
    file_name_panel_state: &'a file_name_panel::FileNamePanelState,
    is_synced: bool,
    sync_locked: bool,
    tag_list: &'a frename_core::TagList<S>,
) -> Element<'a, Message>
where
    S: frename_core::StoredTagStore + Clone,
{
    let file_name =
        file_name_panel::view::view(file_name_panel_state, tag_list).map(Message::FileNamePanel);
    let filter = tag_list.filter_query();
    let on_create = if !filter.trim().is_empty() && !file_workspace.has_tag_with_name(filter.trim()) {
        Some(|text: String| Message::TagPanel(tag_panel::Message::CreateTag(text.trim().to_string())))
    } else {
        None
    };
    let search_bar = widgets::search_bar::view(
        widgets::search_bar::SEARCH_BAR_INPUT_ID,
        filter,
        |s| Message::TagPanel(tag_panel::Message::SetFilter(s)),
        || Message::TagPanel(tag_panel::Message::SetFilter(String::new())),
        on_create,
    );
    let tag_grid = tag_grid::view::view(tag_panel_state, file_workspace.file(), tag_list)
        .map(Message::TagPanel);
    let tag_grid = container(tag_grid)
        .height(Length::Fill)
        .width(Length::Fill);

    let sync: Element<'_, Message> =
        sync_panel::view::view(is_synced, sync_locked).map(Message::SyncPanel);

    // tag_grid → sync → file_name with no gaps so the sync panel visually bridges them.
    let middle = column![tag_grid, sync, file_name]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    let comment_input = text_editor_widget(&file_workspace.comment_content)
        .on_action(Message::CommentAction)
        .placeholder("Comment...")
        .height(80)
        .padding([4, 6])
        // Explicitly capture Enter so the event is not treated as Ignored by Iced,
        // which prevents Windows from playing the system beep for unhandled WM_CHAR(0x0D).
        .key_binding(|kp| {
            if matches!(kp.key, iced::keyboard::Key::Named(Named::Enter)) {
                Some(Binding::Enter)
            } else {
                Binding::from_key_press(kp)
            }
        });

    // Starred panel: shown between search bar and tag grid; unaffected by search filter.
    // Reuses the tag grid's panel_bounds for width (same container column).
    let starred_content_width = tag_panel_state
        .panel_bounds()
        .map(|b| (b.width - 8.0).max(0.0));
    let starred_panel = starred_tags_panel::view(
        tag_list,
        starred_content_width,
        tag_panel_state.selected_tag_id(),
    );

    let mut content_items: Vec<Element<'_, Message>> = Vec::with_capacity(4);
    content_items.push(search_bar);
    if let Some(sp) = starred_panel {
        content_items.push(sp.map(Message::TagPanel));
    }
    content_items.push(middle.into());
    content_items.push(comment_input.into());

    let content = column(content_items)
        .spacing(4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 0.0,
            right: PANEL_PADDING_X,
            bottom: PANEL_PADDING_BOTTOM,
            left: PANEL_PADDING_X,
        })
        .into()
}
