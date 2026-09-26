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
    let status = state.status();
    let busy = matches!(
        status,
        Status::Checking { .. } | Status::Downloading(_) | Status::Restarting
    );
    let available = state.available_version();

    let check = button(text("Check for updates").size(13))
        .on_press_maybe((state.installed() && !busy).then_some(Message::CheckNow));

    let note = match (status, &available) {
        _ if !state.installed() => "Updates work in the installed version".to_string(),
        (Status::Checking { .. }, _) => "Checking…".to_string(),
        (Status::Downloading(percent), Some(version)) => {
            format!("Downloading {version}… {percent}%")
        }
        (Status::Downloading(percent), None) => format!("Downloading… {percent}%"),
        (Status::Restarting, _) => "Restarting…".to_string(),
        (Status::CheckFailed(reason), _) => format!("Could not check for updates: {reason}"),
        (Status::UpdateFailed(reason), _) => format!("Could not update: {reason}"),
        (_, Some(version)) => format!("Version {version} is available"),
        (Status::UpToDate, None) => "frename is up to date".to_string(),
        (Status::Idle, None) => String::new(),
    };

    let mut actions = row![check, text(note).size(13).color(theme::TEXT_MUTED)]
        .spacing(10)
        .align_y(Alignment::Center);
    if available.is_some() {
        let update = button(text("Update and restart").size(13))
            .on_press_maybe((!busy && !batch_running).then_some(Message::UpdateAndRestart));
        actions = actions.push(if batch_running {
            tooltip(
                update,
                text("Wait for the batch to finish"),
                tooltip::Position::Top,
            )
            .into()
        } else {
            Element::from(update)
        });
    }

    column![
        text(format!("frename {}", state.current_version())).size(13),
        actions,
        checkbox(state.check_on_start())
            .label("Check for updates when frename starts")
            .on_toggle_maybe(state.installed().then_some(Message::SetCheckOnStart)),
    ]
    .spacing(8)
    .into()
}
