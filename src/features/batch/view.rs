//! The batch panel (`docs/design/design-system.md` §13.6.1–13.6.4): its header, the action list,
//! and the chosen action's page with its button bar. While a job runs or its result is shown,
//! the job's page ([`super::job_view`]) takes the page's place; the header and the list stay.

use frename_core::{File, FileId};
use iced::widget::{column, container, row, text::IntoFragment, Column};
use iced::{Alignment, Element, Length, Padding};

use crate::features::folder_workspace::Directory;
use crate::ui::badge::{badge, BadgeKind};
use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
use crate::ui::tokens::*;
use crate::ui::tooltip::{Position, Tip};
use crate::ui::{button, layout, scroll, style, text};

use super::actions::Group;
use super::{job_view, Action, BatchState, Message};

/// The page's inset: 16 above and below, 24 at the sides.
pub(super) const PAGE_PADDING: Padding = Padding {
    top: SPACE_L,
    bottom: SPACE_L,
    left: SPACE_XL,
    right: SPACE_XL,
};

/// Render the batch panel. `directory` names the files of the job and lists the files the
/// panel can check.
pub fn view<'a>(state: &'a BatchState, directory: Option<&'a Directory>) -> Element<'a, Message> {
    let page = match state.progress() {
        Some(progress) => job_view::view(state, progress, directory),
        None => action_page(state, directory),
    };
    container(column![
        header(state),
        layout::horizontal_line(),
        row![action_list(state), layout::vertical_line(), page].height(Length::Fill),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(style::panel)
    .into()
}

/// "Batch actions", how many files are checked, and the way back to the open file.
fn header(state: &BatchState) -> Element<'_, Message> {
    let back = IconButton::new(Icon::X)
        .small()
        .tip(
            Tip::new(if state.is_running() {
                fl!("batch-back-while-running")
            } else {
                fl!("folder-controls-batch-back")
            }),
            Position::Left,
        )
        .on_press_maybe((!state.is_running()).then_some(Message::SetActive(false)));
    container(
        row![
            text::heading(fl!("batch-title")),
            text::secondary(fl!(
                "batch-checked-count",
                count = (state.checked_count() as i64)
            )),
            iced::widget::space::horizontal(),
            back,
        ]
        .spacing(SPACE_S)
        .align_y(Alignment::Center),
    )
    .padding(Padding {
        left: SPACE_L,
        right: SPACE_S,
        ..Padding::ZERO
    })
    .center_y(BATCH_HEADER_HEIGHT)
    .into()
}

/// Every action, under its group's caption; locked while a job runs (the running action keeps
/// its selection).
fn action_list(state: &BatchState) -> Element<'_, Message> {
    let locked = state.is_running();
    let mut list = Column::new().spacing(SPACE_XXS);
    let mut group = None;
    for action in Action::ALL {
        if group != Some(action.group()) {
            group = Some(action.group());
            list = list.push(group_caption(
                action.group(),
                group != Some(Group::MoveBetweenPlaces),
            ));
        }
        let service = state.actions().service(action).map(|(name, key_missing)| {
            if key_missing {
                badge(BadgeKind::Warning, fl!("batch-no-key"))
            } else {
                badge(BadgeKind::Neutral, name)
            }
        });
        list = list.push(layout::nav_item_with(
            action.icon(),
            action.label(),
            action == state.action(),
            service,
            (!locked).then_some(Message::SelectAction(action)),
        ));
    }
    container(scroll::vertical(list))
        .width(ACTION_LIST_WIDTH)
        .height(Length::Fill)
        .padding(SPACE_S)
        .into()
}

/// A group's caption; groups after the first get room above them.
fn group_caption<'a>(group: Group, spaced: bool) -> Element<'a, Message> {
    container(text::caption(group.label()))
        .padding(Padding {
            top: if spaced { SPACE_M } else { 0.0 },
            bottom: SPACE_XS,
            left: SPACE_S,
            right: 0.0,
        })
        .into()
}

/// A page's content, scrolled and no wider than a readable line.
pub(super) fn page_body<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    scroll::vertical(
        container(content)
            .max_width(PAGE_MAX_WIDTH + PAGE_PADDING.left + PAGE_PADDING.right)
            .padding(PAGE_PADDING),
    )
    .into()
}

/// The page, then the button bar, as every page of the panel has them.
pub(super) fn with_button_bar<'a>(
    body: Element<'a, Message>,
    hint: impl IntoFragment<'a>,
    buttons: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        container(body).height(Length::Fill),
        layout::button_bar(hint, buttons)
    ]
    .width(Length::Fill)
    .into()
}

/// The chosen action's page for the checked files, and its button bar: why Run is off (with
/// the button that fixes it) on the left, Run on the right.
fn action_page<'a>(
    state: &'a BatchState,
    directory: Option<&'a Directory>,
) -> Element<'a, Message> {
    let checked: Vec<&File> = directory.map_or_else(Vec::new, |dir| {
        dir.all_files()
            .filter(|f| state.is_checked(f.id()))
            .collect()
    });
    let panel = state.actions().panel(state.action(), &checked);
    // Clip lengths still being read hold their files open, which a rename would fail on.
    let reading = state.actions().is_reading_files();
    let reason = if checked.is_empty() {
        Some(fl!("batch-reason-none-checked"))
    } else if reading && panel.reason.is_none() {
        Some(fl!("batch-reason-reading"))
    } else {
        panel.reason
    };
    let can_run = panel.ready && !reading && !state.is_running();
    let listed: Vec<FileId> = directory
        .map(|dir| dir.files_in_order().map(|f| f.id()).collect())
        .unwrap_or_default();
    let check_all = (checked.is_empty() && !listed.is_empty()).then(|| {
        button::secondary(fl!("batch-check-all", count = (listed.len() as i64)))
            .on_press(Message::CheckAll(listed))
            .into()
    });
    let run = button::primary(panel.run)
        .on_press_maybe(can_run.then_some(Message::Run))
        .into();
    with_button_bar(
        page_body(panel.page.map(Message::Action)),
        reason.unwrap_or_default(),
        check_all.into_iter().chain([run]),
    )
}
