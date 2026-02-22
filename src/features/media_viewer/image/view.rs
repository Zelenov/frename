//! View for the image viewer sub-feature.

use iced::widget::{container, image, text};
use iced::{ContentFit, Element, Length};

use crate::theme;
use super::{ImageViewerState, Message};

/// Render the image viewer: image when loaded, placeholder while loading or idle.
pub fn view(state: &ImageViewerState) -> Element<'_, Message> {
    if let Some(handle) = state.current_handle() {
        image(handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Contain)
            .into()
    } else if state.is_loading() {
        container(text("⏳").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into()
    } else if state.load_failed() {
        container(text("✕").size(80).color(theme::ERROR))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::panel_container_style)
            .into()
    } else {
        container(text("🖼").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into()
    }
}
