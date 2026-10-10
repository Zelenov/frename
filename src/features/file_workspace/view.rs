//! UI for the tags area (design system §13.5): from top to bottom the tag search, the starred
//! strip, the tag grid, the order strip, the file name card, the height handle and the comment.
//!
//! Only this module knows the layout of the tags area. The file list keeps using
//! [crate::widgets::file_name_display].

use iced::keyboard::key::Named;
use iced::widget::{column, container, responsive, stack, text_editor as text_editor_widget, Id};
use iced::{Element, Length, Padding};

use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
use crate::ui::palette::TagPalette;
use crate::ui::tokens::*;
use crate::ui::tooltip::Position;
use crate::ui::{scroll, style};
use crate::widgets::height_handle::HeightHandle;
use crate::widgets::search_bar::{self, SearchBar};
use crate::widgets::starred_tags_panel;

use crate::features::{file_name_panel, sync_panel, tag_grid, tag_panel};

use super::{CommentLayout, FileWorkspace, Message};

/// The scrollable around the comment box, snapped to its end while typing on the last line.
pub const COMMENT_SCROLLABLE_ID: &str = "comment-scrollable";
/// The comment box itself, so a shortcut can tell whether the user is writing in it.
pub const COMMENT_EDITOR_ID: &str = "comment-editor";

/// The area's inset below the search bar: 8 at the sides and the bottom, none on top, so the
/// grid lines up with the search field and the other columns.
const AREA_PADDING: Padding = Padding {
    top: 0.0,
    right: SPACE_S,
    bottom: SPACE_S,
    left: SPACE_S,
};

/// The comment's text keeps clear of the expand button over its top right corner.
const COMMENT_PADDING: Padding = Padding {
    top: CONTROL_PADDING_Y,
    right: ICON_BUTTON_SMALL + SPACE_XS,
    bottom: CONTROL_PADDING_Y,
    left: SPACE_S,
};

/// The expand button's place in the comment's corner, clear of the scroll gutter.
const TOGGLE_PADDING: Padding = Padding {
    top: SPACE_XXS,
    right: SCROLL_GUTTER + SPACE_XXS,
    bottom: 0.0,
    left: 0.0,
};

/// Render the tags area.
pub fn view<'a, S>(
    file_workspace: &'a FileWorkspace<S>,
    tag_panel_state: &'a tag_panel::TagPanelState,
    file_name_panel_state: &'a file_name_panel::FileNamePanelState,
    is_synced: bool,
    sync_locked: bool,
    tag_palette: TagPalette,
    save_status: Option<file_name_panel::SaveStatus>,
) -> Element<'a, Message>
where
    S: frename_core::StoredTagStore + Clone,
{
    let tag_list = file_workspace.tag_list();
    let search = tag_search(file_workspace, tag_list);
    let grid = tag_grid::view::view(
        tag_panel_state,
        file_workspace.file(),
        tag_list,
        tag_palette,
    )
    .map(Message::TagPanel);
    let grid = container(grid).height(Length::Fill).width(Length::Fill);

    // No clip open: the grid's empty state alone; nothing to order, name or comment on.
    if file_workspace.file().is_none() {
        return column![search, container(grid).padding(AREA_PADDING)].into();
    }

    let comment_box = comment(file_workspace);
    // Expanded, the comment box is the whole area.
    if file_workspace.comment_expanded() {
        return container(comment_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(AREA_PADDING)
            .into();
    }

    let columns = tag_grid::view::grid_columns(tag_panel_state, tag_list);
    let starred = starred_tags_panel::view(
        tag_list,
        columns,
        tag_panel_state.selected_tag_id(),
        tag_palette,
    )
    .map(|strip| strip.map(Message::TagPanel));
    // Nothing to order while the folder has no tags.
    let order = tag_list
        .has_tags()
        .then(|| sync_panel::view::view(is_synced, sync_locked).map(Message::SyncPanel));
    let card =
        file_name_panel::view::view(file_name_panel_state, tag_list, tag_palette, save_status)
            .map(Message::FileNamePanel);
    // Dragging the handle up makes the comment box taller.
    let handle = HeightHandle::new(|grow| Message::CommentLayout(CommentLayout::Grow(grow)));

    let content = column![]
        .push(starred)
        .push(grid)
        .push(order)
        .push(card)
        .push(handle)
        .push(comment_box)
        .spacing(SPACE_XS)
        .width(Length::Fill)
        .height(Length::Fill);

    column![
        search,
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(AREA_PADDING),
    ]
    .into()
}

/// The tag search: typing anywhere in the window types here; Enter creates the tag when the
/// text names none, otherwise it only keeps the text.
fn tag_search<'a, S>(
    file_workspace: &'a FileWorkspace<S>,
    tag_list: &'a frename_core::TagList<S>,
) -> Element<'a, Message>
where
    S: frename_core::StoredTagStore + Clone,
{
    let filter = tag_list.filter_query();
    let on_submit = if !filter.trim().is_empty() && !file_workspace.has_tag_with_name(filter.trim())
    {
        Message::TagPanel(tag_panel::Message::CreateTag(filter.trim().to_string()))
    } else {
        Message::TagPanel(tag_panel::Message::SetFilter(filter.to_string()))
    };
    search_bar::view(
        SearchBar {
            input_id: search_bar::SEARCH_BAR_INPUT_ID,
            placeholder: fl!("file-workspace-search-placeholder"),
            value: filter,
            clear_tip: fl!("file-workspace-search-clear"),
            on_clear: Message::TagPanel(tag_panel::Message::SetFilter(String::new())),
            on_submit,
            trailing: None,
        },
        |s| Message::TagPanel(tag_panel::Message::SetFilter(s)),
    )
}

/// The comment (§13.5.7): a multi-line field with the expand button in its top right corner.
fn comment<'a, S>(file_workspace: &'a FileWorkspace<S>) -> Element<'a, Message>
where
    S: frename_core::StoredTagStore + Clone,
{
    let expanded = file_workspace.comment_expanded();
    // The editor grows with its text inside a scrollable, which shows the scroll bar the
    // editor itself does not draw (unbounded, it also leaves the wheel to the scrollable). It
    // is at least as tall as the box, expanded or not, so a click anywhere in it lands in the
    // editor.
    let comment_scroll = responsive(move |size| {
        let comment_editor = text_editor_widget(&file_workspace.comment_content)
            .id(Id::new(COMMENT_EDITOR_ID))
            .on_action(Message::CommentAction)
            .placeholder(fl!("file-workspace-comment-placeholder"))
            .height(Length::Shrink)
            // iced adds the padding after the minimum: without taking it off, the editor is a
            // little taller than the box, scrolls, and its top edge goes out of sight.
            .min_height((size.height - COMMENT_PADDING.y()).max(0.0))
            .size(TEXT_BODY)
            .padding(COMMENT_PADDING)
            .style(style::bare_text_editor)
            // Explicitly capture Enter so the event is not treated as Ignored by Iced,
            // which prevents Windows from playing the system beep for unhandled WM_CHAR(0x0D).
            // Only while the box has the keys: otherwise Enter would type a newline into the
            // comment from anywhere, and never reach the window's Enter (the recent folders
            // list takes it while it is open).
            .key_binding(comment_key_binding);
        scroll::vertical_with_id(COMMENT_SCROLLABLE_ID, comment_editor).into()
    });
    let (glyph, tip) = if expanded {
        (Icon::Minimize, fl!("file-workspace-comment-collapse"))
    } else {
        (Icon::Maximize, fl!("file-workspace-comment-expand"))
    };
    let toggle = IconButton::new(glyph)
        .small()
        .tip(tip, Position::Left)
        .on_press(Message::CommentLayout(CommentLayout::ToggleExpanded));
    // The box draws the field's fill and edge (and the focus ring) outside the scroll area, so
    // the edge stays put while the text scrolls, and the scrollbar sits inside the box. It keeps
    // the ring's width free inside, so the ring never covers the text.
    container(stack![
        comment_scroll,
        container(toggle)
            .align_right(Length::Fill)
            .padding(TOGGLE_PADDING),
    ])
    .padding(RING)
    .style(style::field_box(file_workspace.comment_focused()))
    .width(Length::Fill)
    .height(if expanded {
        Length::Fill
    } else {
        Length::Fixed(file_workspace.comment_height())
    })
    .into()
}

/// The comment box's keys: Enter is captured only while the box has the keys (see the comment
/// where it is used), everything else is iced's own.
fn comment_key_binding<M>(
    press: iced::widget::text_editor::KeyPress,
) -> Option<iced::widget::text_editor::Binding<M>> {
    use iced::widget::text_editor::{Binding, Status};
    if matches!(press.key, iced::keyboard::Key::Named(Named::Enter))
        && matches!(press.status, Status::Focused { .. })
    {
        Some(Binding::Enter)
    } else {
        Binding::from_key_press(press)
    }
}

#[cfg(test)]
mod key_tests {
    use super::*;
    use iced::widget::text_editor::{Binding, KeyPress, Status};

    fn enter(status: Status) -> KeyPress {
        let key = iced::keyboard::Key::Named(Named::Enter);
        KeyPress {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Enter),
            modifiers: iced::keyboard::Modifiers::empty(),
            text: None,
            status,
        }
    }

    #[test]
    fn enter_is_the_boxs_only_while_it_has_the_keys() {
        let focused = Status::Focused { is_hovered: false };
        assert!(matches!(
            comment_key_binding::<()>(enter(focused)),
            Some(Binding::Enter)
        ));
        assert!(comment_key_binding::<()>(enter(Status::Active)).is_none());
        assert!(comment_key_binding::<()>(enter(Status::Hovered)).is_none());
    }
}
