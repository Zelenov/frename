//! The Version row of the settings window's Updates page.

use iced::widget::{column, row};
use iced::{Element, Length};

use super::notes::{self, Style};
use super::state::{Heading, Section, Shown, Status};
use super::{Control, Message, UpdatesState};
use crate::ui::layout;
use crate::ui::text::Run;
use crate::ui::tokens::{CHOICE_INDENT, SPACE_M, SPACE_S, SPACE_XS};
use crate::ui::tooltip::{self, Position};
use crate::ui::{button, form, text};
use crate::widgets::focus_ring::ring;

/// The running version, one status line, **Check for updates** and, when a newer version is known,
/// **Update and restart**, then the start-up checkbox. `batch_running` holds the update back: the
/// batch job would be cut off by the restart. `focus` is the control Settings' keyboard focus is
/// on.
pub fn view(
    state: &UpdatesState,
    batch_running: bool,
    focus: Option<Control>,
) -> Element<'_, Message> {
    let version = || {
        text::strong(fl!(
            "updates-current-version",
            version = state.current_version()
        ))
        .into()
    };
    match state.section() {
        // The Store build has no updater of its own: nothing to check or to switch on.
        Section::Store => {
            return layout::aligned([version(), text::secondary(fl!("updates-from-store")).into()])
                .into();
        }
        // The Mac download and the Linux AppImage: new versions are downloaded by hand.
        Section::ReleasesPage => {
            let line = row![
                text::secondary(fl!("updates-from-releases")),
                ring(
                    button::link(fl!("updates-open-releases")).on_press(Message::OpenReleases),
                    focus == Some(Control::OpenReleases),
                ),
            ]
            .spacing(SPACE_S)
            .wrap();
            return layout::aligned([version(), line.into()]).into();
        }
        Section::Updater => {}
    }

    let status = state.status();
    let focused = |control| focus == Some(control);
    let available = state.available_version();

    let note = match (status, &available) {
        _ if !state.installed() => Some(text::secondary(fl!("updates-not-installed"))),
        (Status::Checking { .. }, _) => Some(text::secondary(fl!("updates-checking"))),
        (Status::Downloading(percent), Some(version)) => Some(text::secondary(fl!(
            "updates-downloading-named",
            version = version.clone(),
            percent = (*percent as i64)
        ))),
        (Status::Downloading(percent), None) => Some(text::secondary(fl!(
            "updates-downloading",
            percent = (*percent as i64)
        ))),
        (Status::Restarting, _) => Some(text::secondary(fl!("updates-restarting"))),
        (Status::CheckFailed(reason), _) => Some(text::error(fl!(
            "updates-check-failed",
            reason = reason.clone()
        ))),
        (Status::UpdateFailed(reason), _) => Some(text::error(fl!(
            "updates-update-failed",
            reason = reason.clone()
        ))),
        (_, Some(version)) => Some(text::secondary(fl!(
            "updates-version-available",
            version = version.clone()
        ))),
        (Status::UpToDate, None) => Some(text::secondary(fl!("updates-up-to-date"))),
        (Status::Idle, None) => None,
    };

    let check = ring(
        button::secondary(fl!("updates-check"))
            .on_press_maybe(state.check_enabled().then_some(Message::CheckNow)),
        focused(Control::Check),
    );
    let update = available.is_some().then(|| {
        let update = ring(
            button::primary(fl!("updates-update-and-restart")).on_press_maybe(
                state
                    .update_enabled(batch_running)
                    .then_some(Message::UpdateAndRestart),
            ),
            focused(Control::UpdateAndRestart),
        );
        if batch_running {
            tooltip::tip_text(update, fl!("updates-wait-for-batch"), Position::Top)
        } else {
            update
        }
    });
    let actions = layout::buttons(std::iter::once(check).chain(update));

    layout::aligned(
        [version()]
            .into_iter()
            .chain(note.map(Element::from))
            .chain([
                actions,
                ring(
                    form::checkbox(fl!("updates-check-on-start"), state.check_on_start())
                        .on_toggle_maybe(state.installed().then_some(Message::SetCheckOnStart)),
                    focused(Control::CheckOnStart),
                ),
            ]),
    )
    .into()
}

/// The Release notes row of the Updates page: the button for the whole history, the link to all
/// releases, and the notes block under them (the update's, what is new, or the history).
pub fn notes(state: &UpdatesState, focus: Option<Control>) -> Element<'_, Message> {
    let focused = |control| focus == Some(control);
    let toggle = if state.history_open() {
        fl!("updates-notes-hide")
    } else {
        fl!("updates-notes-show")
    };
    let buttons = layout::buttons([
        ring(
            button::secondary(toggle).on_press(Message::ToggleHistory),
            focused(Control::ReleaseNotes),
        ),
        ring(
            button::link(fl!("updates-notes-all-releases")).on_press(Message::OpenReleases),
            focused(Control::AllReleases),
        ),
    ]);
    let mut parts: Vec<Element<'_, Message>> = vec![buttons];
    if let Some(shown) = state.shown() {
        parts.push(block(&shown, focused(Control::ShowMore)));
    }
    layout::aligned(parts).into()
}

/// The heading, what is wrong with the notes (still loading, not fetched), then one block per
/// version.
fn block<'a>(shown: &Shown<'a>, show_more_focused: bool) -> Element<'a, Message> {
    let heading = match &shown.heading {
        Heading::History => fl!("updates-notes-history"),
        Heading::Update(version) => fl!("updates-notes-update", version = version.clone()),
        Heading::WhatsNew(version) => fl!("updates-notes-whats-new", version = version.clone()),
    };
    let mut blocks: Vec<Element<'a, Message>> = vec![text::strong(heading).into()];
    if shown.versions.is_empty() {
        if shown.loading {
            blocks.push(text::secondary(fl!("updates-notes-loading")).into());
        } else if shown.failed {
            blocks.push(text::secondary(fl!("updates-notes-unavailable")).into());
        }
    } else if shown.failed {
        // Some of the versions are missing: say so above them.
        blocks.push(text::secondary(fl!("updates-notes-unavailable")).into());
    }
    blocks.extend(shown.versions.iter().map(|version| {
        let mut lines: Vec<Element<'a, Message>> = vec![text::strong(version.label()).into()];
        for group in &version.groups {
            if !group.title.is_empty() {
                lines.push(text::secondary(group.title.clone()).into());
            }
            lines.extend(group.items.iter().map(bullet));
        }
        column(lines).spacing(SPACE_XS).width(Length::Fill).into()
    }));
    if shown.more {
        blocks.push(ring(
            button::secondary(fl!("updates-notes-more")).on_press(Message::ShowMore),
            show_more_focused,
        ));
    }
    column(blocks).spacing(SPACE_M).width(Length::Fill).into()
}

/// One bullet: a dot, then its text with bold and code drawn.
fn bullet<'a>(item: &notes::Item) -> Element<'a, Message> {
    let runs = notes::inline(&item.text)
        .into_iter()
        .map(|(style, content)| {
            let run = match style {
                Style::Plain => Run::Plain,
                Style::Bold => Run::Strong,
                Style::Code => Run::Mono,
            };
            (run, content)
        });
    let indent = if item.nested { CHOICE_INDENT } else { 0.0 };
    row![
        iced::widget::Space::new().width(indent),
        text::secondary("•"),
        text::body_runs(runs),
    ]
    .spacing(SPACE_S)
    .into()
}
