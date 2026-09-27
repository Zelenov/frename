//! Icons: Lucide outlines (ISC licence, `assets/icons/`), bundled into the binary so every OS draws
//! the same picture. Each takes the color of the control it sits in.

use std::sync::OnceLock;

use iced::widget::{svg, Svg};
use iced::Color;

/// The icons the design system uses so far; #44 adds the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Info,
    CircleCheck,
    CircleAlert,
    TriangleAlert,
    Languages,
    Folder,
    Sparkles,
    Captions,
    RefreshCw,
}

const ICONS: [(Icon, &[u8]); 9] = [
    (Icon::Info, include_bytes!("../../assets/icons/info.svg")),
    (
        Icon::CircleCheck,
        include_bytes!("../../assets/icons/circle-check.svg"),
    ),
    (
        Icon::CircleAlert,
        include_bytes!("../../assets/icons/circle-alert.svg"),
    ),
    (
        Icon::TriangleAlert,
        include_bytes!("../../assets/icons/triangle-alert.svg"),
    ),
    (
        Icon::Languages,
        include_bytes!("../../assets/icons/languages.svg"),
    ),
    (
        Icon::Folder,
        include_bytes!("../../assets/icons/folder.svg"),
    ),
    (
        Icon::Sparkles,
        include_bytes!("../../assets/icons/sparkles.svg"),
    ),
    (
        Icon::Captions,
        include_bytes!("../../assets/icons/captions.svg"),
    ),
    (
        Icon::RefreshCw,
        include_bytes!("../../assets/icons/refresh-cw.svg"),
    ),
];

/// One handle per icon, made once: a handle made from the same bytes each frame would be hashed
/// each frame.
fn handles() -> &'static [(Icon, svg::Handle)] {
    static HANDLES: OnceLock<Vec<(Icon, svg::Handle)>> = OnceLock::new();
    HANDLES.get_or_init(|| {
        ICONS
            .iter()
            .map(|(icon, bytes)| (*icon, svg::Handle::from_memory(*bytes)))
            .collect()
    })
}

impl Icon {
    fn handle(self) -> svg::Handle {
        handles()
            .iter()
            .find(|(icon, _)| *icon == self)
            .map(|(_, handle)| handle.clone())
            .unwrap_or_else(|| svg::Handle::from_memory(&[][..]))
    }
}

/// `icon` drawn `size` px square in `color`.
pub fn icon<'a>(icon: Icon, size: f32, color: Color) -> Svg<'a> {
    svg(icon.handle())
        .width(size)
        .height(size)
        .style(move |_, _| svg::Style { color: Some(color) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_an_svg() {
        for (icon, bytes) in ICONS {
            let text = std::str::from_utf8(bytes).unwrap();
            assert!(text.contains("<svg"), "{icon:?}");
        }
    }
}
