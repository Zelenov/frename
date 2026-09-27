//! The Updates section of the settings window.

use iced::widget::{button, checkbox, column, row, text, tooltip};
use iced::{Alignment, Element};

use super::state::Status;
use super::{Message, UpdatesState};
use crate::theme;

/// The running version, the check button with its outcome, **Update and restart** when a newer
/// version is known, and the start-up check box. `batch_running` holds the update back: the
/// batch job would be cut off by the restart.
pub fn view(state: &UpdatesState, batch_running: bool) -> Element<'_, Message> {
    let version = text(fl!(
        "updates-current-version",
        version = state.current_version()
    ))
    .size(13);
    // The Store build has no updater of its own: nothing to check or to switch on.
    if state.is_store_build() {
        return column![
            version,
            text(fl!("updates-from-store"))
                .size(13)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(8)
        .into();
    }

    let status = state.status();
    let busy = matches!(
        status,
        Status::Checking { .. } | Status::Downloading(_) | Status::Restarting
    );
    let available = state.available_version();

    let check = button(text(fl!("updates-check")).size(13))
        .on_press_maybe((state.installed() && !busy).then_some(Message::CheckNow));

    let note = match (status, &available) {
        _ if !state.installed() => fl!("updates-not-installed"),
        (Status::Checking { .. }, _) => fl!("updates-checking"),
        (Status::Downloading(percent), Some(version)) => fl!(
            "updates-downloading-named",
            version = version.clone(),
            percent = (*percent as i64)
        ),
        (Status::Downloading(percent), None) => {
            fl!("updates-downloading", percent = (*percent as i64))
        }
        (Status::Restarting, _) => fl!("updates-restarting"),
        (Status::CheckFailed(reason), _) => {
            fl!("updates-check-failed", reason = reason.clone())
        }
        (Status::UpdateFailed(reason), _) => {
            fl!("updates-update-failed", reason = reason.clone())
        }
        (_, Some(version)) => fl!("updates-version-available", version = version.clone()),
        (Status::UpToDate, None) => fl!("updates-up-to-date"),
        (Status::Idle, None) => String::new(),
    };

    let mut actions = row![check, text(note).size(13).color(theme::TEXT_MUTED)]
        .spacing(10)
        .align_y(Alignment::Center);
    if available.is_some() {
        let update = button(text(fl!("updates-update-and-restart")).size(13))
            .on_press_maybe((!busy && !batch_running).then_some(Message::UpdateAndRestart));
        actions = actions.push(if batch_running {
            tooltip(
                update,
                text(fl!("updates-wait-for-batch")),
                tooltip::Position::Top,
            )
            .into()
        } else {
            Element::from(update)
        });
    }

    column![
        version,
        actions,
        checkbox(state.check_on_start())
            .label(fl!("updates-check-on-start"))
            .on_toggle_maybe(state.installed().then_some(Message::SetCheckOnStart)),
    ]
    .spacing(8)
    .into()
}
