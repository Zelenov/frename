//! UI for the file name card: its chips, the trash, the timecodes and the name line.

use iced::widget::text::Wrapping;
use iced::widget::{column, container, mouse_area, row, Row};
use iced::{Alignment, Element, Length};

use frename_core::{StoredTagStore, TagList};

use crate::ui::badge;
use crate::ui::icons::{icon, Icon};
use crate::ui::palette::TagPalette;
use crate::ui::style;
use crate::ui::text;
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Tip};

use super::chips_panel;
use super::file_name_line;
use super::trash_zone;
use super::{
    pending_tag_suggestions, suggested_in_out, tag_ideas, FileNamePanelState, Message, SaveStatus,
};

/// Renders the file name card (design system §13.5.6):
/// - first line: the checked tags as chips in the file's order | the trash
/// - second line: the IN and OUT timecodes (when set), the In/Out the AI suggests (while it
///   differs from them) | the name without the tags
/// - third line, while the AI description suggests tags the clip does not have yet or has tag
///   ideas: the suggested tags (a click adds one), "Add all", and the ideas
pub fn view<'a, S>(
    state: &'a FileNamePanelState,
    tag_list: &'a TagList<S>,
    tag_palette: TagPalette,
    save_status: Option<SaveStatus>,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let snapshot = tag_list.file_snapshot();
    let name = snapshot.name_without_extension().to_string();
    let ext = snapshot.extension().to_string();
    let name_ext = file_name_line::name_ext_from_parts(&name, &ext);

    let chips = chips_panel::view(state, tag_list, tag_palette);
    let trash = trash_zone::view(state, tag_list, tag_palette);
    let top_row = row![chips, trash]
        .spacing(SPACE_S)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let seg_start = tag_list.segment_start_secs();
    let seg_end = tag_list.segment_end_secs();

    let mut bottom_items: Vec<Element<'_, Message>> = Vec::new();
    if let Some(s) = seg_start {
        bottom_items.push(badge::timecode(
            "IN",
            file_name_line::fmt_timecode(s),
            Some((Message::ClearSegmentStart, fl!("file-name-panel-clear-in"))),
        ));
    }
    if let Some(e) = seg_end {
        bottom_items.push(badge::timecode(
            "OUT",
            file_name_line::fmt_timecode(e),
            Some((Message::ClearSegmentEnd, fl!("file-name-panel-clear-out"))),
        ));
    }
    if let Some(suggested) = suggested_in_out(tag_list) {
        bottom_items.push(badge::suggested_timecode(
            fl!("file-name-panel-suggested-in-out"),
            format!(
                "{} – {}",
                suggested
                    .start
                    .map(file_name_line::fmt_timecode)
                    .unwrap_or_default(),
                suggested
                    .end
                    .map(file_name_line::fmt_timecode)
                    .unwrap_or_default(),
            ),
            Message::ApplySuggestedInOut,
            fl!("file-name-panel-apply-suggested-in-out"),
        ));
    }
    bottom_items.push(file_name_line::view(name_ext));

    let bottom_row = row(bottom_items)
        .spacing(SPACE_TIGHT)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let inner = container(
        column![top_row, bottom_row]
            .push(suggested_tags_row(tag_list))
            .push(save_status.map(save_status_line))
            .spacing(SPACE_S)
            .width(Length::Fill),
    )
    .padding(SPACE_S)
    .width(Length::Fill)
    .style(style::card);

    // A right-click on the file's name (its chips or the name line) opens the file menu.
    mouse_area(container(inner).width(Length::Fill))
        .on_right_press(Message::OpenFileMenu)
        .into()
}

/// The card's last line: whether the clip's edits are in its file or wait in the recovery
/// journal (§13.5.6). An icon and a `caption`, quiet: the editor looks for it only to be sure.
fn save_status_line<'a>(status: SaveStatus) -> Element<'a, Message> {
    let (glyph, color, label) = match status {
        SaveStatus::Saved => (Icon::CircleCheck, SUCCESS, fl!("file-name-panel-saved")),
        SaveStatus::InRecovery => (
            Icon::CircleDashed,
            TEXT_SECONDARY,
            fl!("file-name-panel-saved-to-recovery"),
        ),
    };
    row![icon(glyph, ICON_S, color), text::caption(label)]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center)
        .into()
}

/// The tags the AI suggests for the clip, most likely first (F6 adds that one), "Add all" when
/// there are several (Shift+F6), and the tags it noticed that the folder does not have, which
/// are only shown. `None` when there is nothing of the kind.
fn suggested_tags_row<'a, S>(tag_list: &TagList<S>) -> Option<Element<'a, Message>>
where
    S: StoredTagStore + Clone,
{
    let suggested = pending_tag_suggestions(tag_list);
    let ideas = tag_ideas(tag_list);
    if suggested.is_empty() && ideas.is_empty() {
        return None;
    }
    let lead = row![
        icon(Icon::Sparkles, ICON_S, ACCENT_TEXT),
        text::caption(fl!("file-name-panel-suggested-tags"))
            .color(ACCENT_TEXT)
            .wrapping(Wrapping::None),
    ]
    .spacing(SPACE_XS)
    .align_y(Alignment::Center);
    let several = suggested.len() > 1;
    let mut items: Vec<Element<'a, Message>> = vec![lead.into()];
    for (index, tag) in suggested.into_iter().enumerate() {
        let percent = (tag.percent > 0).then_some(tag.percent);
        let label = match percent {
            Some(percent) => fl!(
                "file-name-panel-add-suggested-tag",
                tag = tag.name.clone(),
                percent = (percent as i64)
            ),
            None => fl!(
                "file-name-panel-add-suggested-tag-unsure",
                tag = tag.name.clone()
            ),
        };
        let tip = if index == 0 {
            Tip::new(label).keys(&["F6"])
        } else {
            Tip::new(label)
        };
        items.push(badge::suggested_tag(
            tag.name.clone(),
            percent,
            Message::AddSuggestedTag(tag.name),
            tip,
        ));
    }
    if several {
        items.push(badge::apply_all(
            fl!("file-name-panel-add-all-suggested-tags"),
            Message::AddAllSuggestedTags,
            Tip::new(fl!("file-name-panel-add-all-suggested-tags-tip")).keys(&["Shift", "F6"]),
        ));
    }
    if !ideas.is_empty() {
        items.push(tooltip::tip_text(
            text::secondary(fl!("file-name-panel-tag-ideas", ideas = ideas.join(", "))),
            fl!("file-name-panel-tag-ideas-tip"),
            tooltip::Position::Top,
        ));
    }
    Some(
        Row::with_children(items)
            .spacing(SPACE_TIGHT)
            .align_y(Alignment::Center)
            .wrap()
            .into(),
    )
}
