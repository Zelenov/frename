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
    /// The icon's handle, made once per icon: a handle made from the same bytes each frame would
    /// be hashed each frame. The match makes a new icon without its file a compile error.
    fn handle(self) -> svg::Handle {
        macro_rules! file {
            ($name:literal) => {{
                static HANDLE: OnceLock<svg::Handle> = OnceLock::new();
                HANDLE
                    .get_or_init(|| {
                        svg::Handle::from_memory(
                            include_bytes!(concat!("../../assets/icons/", $name)).as_slice(),
                        )
                    })
                    .clone()
            }};
        }
        match self {
            Icon::Info => file!("info.svg"),
            Icon::CircleCheck => file!("circle-check.svg"),
            Icon::CircleAlert => file!("circle-alert.svg"),
            Icon::TriangleAlert => file!("triangle-alert.svg"),
            Icon::Languages => file!("languages.svg"),
            Icon::Folder => file!("folder.svg"),
            Icon::Sparkles => file!("sparkles.svg"),
            Icon::Captions => file!("captions.svg"),
            Icon::RefreshCw => file!("refresh-cw.svg"),
        }
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
    #[test]
    fn every_icon_file_is_an_svg() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
        let source = include_str!("icons.rs");
        let names: Vec<&str> = source
            .split("file!(\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|name| name.ends_with(".svg"))
            .collect();
        assert!(names.len() >= 9, "{names:?}");
        for name in names {
            let text = std::fs::read_to_string(folder.join(name)).unwrap();
            assert!(text.contains("<svg"), "{name}");
        }
    }
}
