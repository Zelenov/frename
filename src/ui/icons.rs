//! Icons: Lucide outlines (ISC licence, `assets/icons/`), bundled into the binary so every OS draws
//! the same picture (`docs/design/design-system.md` §7). Each takes the color of the control it
//! sits in; filled icons (`StarFilled`) mean "on" and are the only filled ones.

use std::sync::OnceLock;

use iced::widget::{svg, Svg};
use iced::{Color, Radians};

/// Declares the icons: each variant with its file, so a new icon without its file is a compile
/// error, and [`Icon::ALL`] lists them for the tests.
macro_rules! icons {
    ($($variant:ident => $file:literal,)+) => {
        /// The icons of the design system.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Icon {
            $($variant,)+
        }

        impl Icon {
            /// Every icon.
            #[cfg(test)]
            pub const ALL: &[Icon] = &[$(Icon::$variant,)+];

            /// The icon's handle, made once per icon: a handle made from the same bytes each
            /// frame would be hashed each frame.
            fn handle(self) -> svg::Handle {
                match self {
                    $(Icon::$variant => {
                        static HANDLE: OnceLock<svg::Handle> = OnceLock::new();
                        HANDLE
                            .get_or_init(|| {
                                svg::Handle::from_memory(
                                    include_bytes!(concat!("../../assets/icons/", $file))
                                        .as_slice(),
                                )
                            })
                            .clone()
                    })+
                }
            }

            #[cfg(test)]
            fn file(self) -> &'static str {
                match self {
                    $(Icon::$variant => $file,)+
                }
            }
        }
    };
}

icons! {
    ArrowDown => "arrow-down.svg",
    ArrowUp => "arrow-up.svg",
    Camera => "camera.svg",
    Captions => "captions.svg",
    Check => "check.svg",
    ChevronDown => "chevron-down.svg",
    ChevronLeft => "chevron-left.svg",
    ChevronRight => "chevron-right.svg",
    CircleAlert => "circle-alert.svg",
    CircleCheck => "circle-check.svg",
    CircleDashed => "circle-dashed.svg",
    CircleMinus => "circle-minus.svg",
    CircleX => "circle-x.svg",
    Clapperboard => "clapperboard.svg",
    Copy => "copy.svg",
    Database => "database.svg",
    Ellipsis => "ellipsis.svg",
    ExternalLink => "external-link.svg",
    Eye => "eye.svg",
    FastForward => "fast-forward.svg",
    File => "file.svg",
    FileText => "file-text.svg",
    FileVideoCamera => "file-video-camera.svg",
    Folder => "folder.svg",
    FolderOpen => "folder-open.svg",
    FolderX => "folder-x.svg",
    GripVertical => "grip-vertical.svg",
    Info => "info.svg",
    Languages => "languages.svg",
    ListChecks => "list-checks.svg",
    ListFilter => "list-filter.svg",
    ListOrdered => "list-ordered.svg",
    LoaderCircle => "loader-circle.svg",
    LocateFixed => "locate-fixed.svg",
    Lock => "lock.svg",
    LockOpen => "lock-open.svg",
    MapPin => "map-pin.svg",
    Maximize => "maximize-2.svg",
    MessageSquareText => "message-square-text.svg",
    Minimize => "minimize-2.svg",
    Pause => "pause.svg",
    Pencil => "pencil.svg",
    PencilLine => "pencil-line.svg",
    Play => "play.svg",
    Plus => "plus.svg",
    RefreshCw => "refresh-cw.svg",
    Rewind => "rewind.svg",
    RotateCcw => "rotate-ccw.svg",
    RotateCw => "rotate-cw.svg",
    Scissors => "scissors.svg",
    Search => "search.svg",
    SearchX => "search-x.svg",
    Settings => "settings.svg",
    Sparkles => "sparkles.svg",
    Star => "star.svg",
    StarFilled => "star-filled.svg",
    Tag => "tag.svg",
    TextCursorInput => "text-cursor-input.svg",
    Trash => "trash.svg",
    TriangleAlert => "triangle-alert.svg",
    Volume => "volume-2.svg",
    X => "x.svg",
}

/// `icon` drawn `size` px square in `color`.
pub fn icon<'a>(icon: Icon, size: f32, color: Color) -> Svg<'a> {
    svg(icon.handle())
        .width(size)
        .height(size)
        .style(move |_, _| svg::Style { color: Some(color) })
}

/// Steps of one turn of the spinner, one per tick of the app's spinner clock.
const SPINNER_STEPS: usize = 12;

/// The turning `loader-circle` (§8.15), at `frame` of the app's spinner clock.
pub fn spinner<'a>(frame: usize, size: f32, color: Color) -> Svg<'a> {
    let step = (frame % SPINNER_STEPS) as f32;
    let angle = std::f32::consts::TAU * step / SPINNER_STEPS as f32;
    icon(Icon::LoaderCircle, size, color).rotation(Radians(angle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(icon: Icon) -> String {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
        std::fs::read_to_string(folder.join(icon.file())).unwrap()
    }

    #[test]
    fn every_icon_file_is_an_svg_drawn_in_its_control_color() {
        for &icon in Icon::ALL {
            let text = source(icon);
            assert!(text.contains("<svg"), "{icon:?}");
            assert!(text.contains("currentColor"), "{icon:?}");
        }
    }

    #[test]
    fn only_the_filled_star_is_filled() {
        for &icon in Icon::ALL {
            // The root element's fill; Lucide fills small dots inside some outlines (`tag`).
            let text = source(icon);
            let root = text.split('>').next().unwrap_or_default();
            let filled = root.contains("fill=\"currentColor\"");
            assert_eq!(filled, icon == Icon::StarFilled, "{icon:?}");
        }
    }
}
