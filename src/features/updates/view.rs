//! The Version row of the settings window's Updates page.

use iced::widget::{tooltip, Row};
use iced::Element;

use super::state::Status;
use super::{Message, UpdatesState};
use crate::ui::layout;
use crate::ui::tokens::SPACE_S;
use crate::ui::{button, form, text};

/// The running version, one status line, **Check for updates** and, when a newer version is known,
/// **Update and restart**, then the start-up checkbox. `batch_running` holds the update back: the
/// batch job would be cut off by the restart.
pub fn view(state: &UpdatesState, batch_running: bool) -> Element<'_, Message> {
    let status = state.status();
    let busy = matches!(
        status,
        Status::Checking { .. } | Status::Downloading(_) | Status::Restarting
    );
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

    let check = button::secondary(fl!("updates-check"))
        .on_press_maybe((state.installed() && !busy).then_some(Message::CheckNow));
    let update = available.is_some().then(|| {
        let update = button::primary(fl!("updates-update-and-restart"))
            .on_press_maybe((!busy && !batch_running).then_some(Message::UpdateAndRestart));
        if batch_running {
            layout::with_tooltip(
                update,
                fl!("updates-wait-for-batch"),
                tooltip::Position::Top,
            )
        } else {
            update.into()
        }
    });
    let actions = Row::with_children(std::iter::once(check.into()).chain(update)).spacing(SPACE_S);

    layout::choices(
        [text::strong(fl!(
            "updates-current-version",
            version = state.current_version()
        ))
        .into()]
        .into_iter()
        .chain(note.map(Element::from))
        .chain([
            actions.into(),
            form::checkbox(fl!("updates-check-on-start"), state.check_on_start())
                .on_toggle_maybe(state.installed().then_some(Message::SetCheckOnStart))
                .into(),
        ]),
    )
    .into()
}
