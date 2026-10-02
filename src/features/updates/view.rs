//! The Version row of the settings window's Updates page.

use iced::Element;

use super::state::Status;
use super::{Control, Message, UpdatesState};
use crate::ui::layout;
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
    let status = state.status();
    let busy = state.is_busy();
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
            .on_press_maybe((state.installed() && !busy).then_some(Message::CheckNow)),
        focused(Control::Check),
    );
    let update = available.is_some().then(|| {
        let update = ring(
            button::primary(fl!("updates-update-and-restart"))
                .on_press_maybe((!busy && !batch_running).then_some(Message::UpdateAndRestart)),
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
        [text::strong(fl!(
            "updates-current-version",
            version = state.current_version()
        ))
        .into()]
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
