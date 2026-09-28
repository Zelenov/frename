//! UI for file workspace: search bar, tag list, file name panel (wrap=true feature) below.
//!
//! Only this module knows the layout of the file workspace region. Folder list keeps using
//! [crate::widgets::file_name_display] with wrap=false.

use iced::keyboard::key::Named;
use iced::widget::text_editor::Binding;
use iced::widget::{
    button, column, container, responsive, scrollable, stack, text,
    text_editor as text_editor_widget, tooltip,
};
use iced::{Element, Length};

use crate::theme;
use crate::ui::palette::TagPalette;
use crate::widgets::height_handle::HeightHandle;
use crate::widgets::starred_tags_panel;

use crate::features::{file_name_panel, sync_panel, tag_grid, tag_panel};
use crate::widgets;

use super::{CommentLayout, FileWorkspace, Message};

/// The scrollable around the comment box, snapped to its end while typing on the last line.
pub const COMMENT_SCROLLABLE_ID: &str = "comment-scrollable";
/// The comment box itself, so a shortcut can tell whether the user is writing in it.
pub const COMMENT_EDITOR_ID: &str = "comment-editor";

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
    tag_palette: TagPalette,
) -> Element<'a, Message>
where
    S: frename_core::StoredTagStore + Clone,
{
    let file_name = file_name_panel::view::view(file_name_panel_state, tag_list, tag_palette)
        .map(Message::FileNamePanel);
    let filter = tag_list.filter_query();
    let on_create = if !filter.trim().is_empty() && !file_workspace.has_tag_with_name(filter.trim())
    {
        Some(|text: String| {
            Message::TagPanel(tag_panel::Message::CreateTag(text.trim().to_string()))
        })
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
    let tag_grid = tag_grid::view::view(
        tag_panel_state,
        file_workspace.file(),
        tag_list,
        tag_palette,
    )
    .map(Message::TagPanel);
    let tag_grid = container(tag_grid).height(Length::Fill).width(Length::Fill);

    let sync: Element<'_, Message> =
        sync_panel::view::view(is_synced, sync_locked).map(Message::SyncPanel);

    // tag_grid → sync → file_name with no gaps so the sync panel visually bridges them.
    let middle = column![tag_grid, sync, file_name]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    let expanded = file_workspace.comment_expanded();
    // The editor grows with its text inside a scrollable, which shows the scroll bar the
    // editor itself does not draw (unbounded, it also leaves the wheel to the scrollable). It
    // is at least as tall as the box, expanded or not, so a click anywhere in it lands in the
    // editor.
    let comment_scroll = responsive(move |size| {
        let comment_editor = text_editor_widget(&file_workspace.comment_content)
            .id(iced::widget::Id::new(COMMENT_EDITOR_ID))
            .on_action(Message::CommentAction)
            .placeholder(fl!("file-workspace-comment-placeholder"))
            .height(Length::Shrink)
            .min_height(size.height)
            // Room on the right for the expand button over the box's corner.
            .padding(iced::Padding {
                top: 4.0,
                right: 26.0,
                bottom: 4.0,
                left: 6.0,
            })
            // Explicitly capture Enter so the event is not treated as Ignored by Iced,
            // which prevents Windows from playing the system beep for unhandled WM_CHAR(0x0D).
            .key_binding(|kp| {
                if matches!(kp.key, iced::keyboard::Key::Named(Named::Enter)) {
                    Some(Binding::Enter)
                } else {
                    Binding::from_key_press(kp)
                }
            });
        scrollable(comment_editor)
            .id(iced::widget::Id::new(COMMENT_SCROLLABLE_ID))
            .width(Length::Fill)
            .height(Length::Fill)
            // The bar beside the text, not over it.
            .spacing(2)
            .style(theme::dark_scrollable_style)
            .into()
    });
    // Expand / collapse sits over the box's top right corner, clear of the scroll bar.
    let toggle = tooltip(
        button(text(if expanded { "⊡" } else { "⛶" }).size(13))
            .on_press(Message::CommentLayout(CommentLayout::ToggleExpanded))
            .padding([0, 4])
            .style(theme::icon_button_style(true)),
        text(if expanded {
            fl!("file-workspace-comment-collapse")
        } else {
            fl!("file-workspace-comment-expand")
        })
        .size(12),
        tooltip::Position::Left,
    );
    let comment_box = container(stack![
        comment_scroll,
        container(toggle)
            .align_right(Length::Fill)
            .padding(iced::Padding {
                top: 2.0,
                right: 14.0,
                bottom: 0.0,
                left: 0.0,
            }),
    ])
    .width(Length::Fill)
    .height(if expanded {
        Length::Fill
    } else {
        Length::Fixed(file_workspace.comment_height())
    });

    // Expanded, the comment box is the whole panel.
    if expanded {
        return container(comment_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: 0.0,
                right: PANEL_PADDING_X,
                bottom: PANEL_PADDING_BOTTOM,
                left: PANEL_PADDING_X,
            })
            .into();
    }

    // Starred panel: shown between search bar and tag grid; unaffected by search filter.
    // Reuses the tag grid's panel_bounds for width (same container column).
    let starred_content_width = tag_panel_state
        .panel_bounds()
        .map(|b| (b.width - 8.0).max(0.0));
    let starred_panel = starred_tags_panel::view(
        tag_list,
        starred_content_width,
        tag_panel_state.selected_tag_id(),
        tag_palette,
    );

    let mut content_items: Vec<Element<'_, Message>> = Vec::with_capacity(4);
    content_items.push(search_bar);
    if let Some(sp) = starred_panel {
        content_items.push(sp.map(Message::TagPanel));
    }
    content_items.push(middle.into());
    // Dragging the handle up makes the comment box taller.
    content_items
        .push(HeightHandle::new(|grow| Message::CommentLayout(CommentLayout::Grow(grow))).into());
    content_items.push(comment_box.into());

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
