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

impl Icon {
    const ALL: [Icon; 9] = [
        Icon::Info,
        Icon::CircleCheck,
        Icon::CircleAlert,
        Icon::TriangleAlert,
        Icon::Languages,
        Icon::Folder,
        Icon::Sparkles,
        Icon::Captions,
        Icon::RefreshCw,
    ];

    /// The icon's SVG file; the match makes a new icon without a file a compile error.
    fn bytes(self) -> &'static [u8] {
        match self {
            Icon::Info => include_bytes!("../../assets/icons/info.svg"),
            Icon::CircleCheck => include_bytes!("../../assets/icons/circle-check.svg"),
            Icon::CircleAlert => include_bytes!("../../assets/icons/circle-alert.svg"),
            Icon::TriangleAlert => include_bytes!("../../assets/icons/triangle-alert.svg"),
            Icon::Languages => include_bytes!("../../assets/icons/languages.svg"),
            Icon::Folder => include_bytes!("../../assets/icons/folder.svg"),
            Icon::Sparkles => include_bytes!("../../assets/icons/sparkles.svg"),
            Icon::Captions => include_bytes!("../../assets/icons/captions.svg"),
            Icon::RefreshCw => include_bytes!("../../assets/icons/refresh-cw.svg"),
        }
    }

    /// One handle per icon, made once: a handle made from the same bytes each frame would be
    /// hashed each frame.
    fn handle(self) -> svg::Handle {
        static HANDLES: OnceLock<Vec<svg::Handle>> = OnceLock::new();
        HANDLES.get_or_init(|| {
            Icon::ALL
                .iter()
                .map(|icon| svg::Handle::from_memory(icon.bytes()))
                .collect()
        })[self as usize]
            .clone()
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
    fn every_icon_is_an_svg_listed_in_its_own_place() {
        for (index, icon) in Icon::ALL.into_iter().enumerate() {
            assert_eq!(icon as usize, index, "{icon:?} is out of order in ALL");
            let text = std::str::from_utf8(icon.bytes()).unwrap();
            assert!(text.contains("<svg"), "{icon:?}");
        }
    }
}
