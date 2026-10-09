//! Icon-only buttons (`docs/design/design-system.md` §8.1, §7): 32 × 32 in toolbars, 24 × 24 in
//! rows and chips, always with a tooltip. The icon takes the button's state: disabled, latched.
//!
//! ```ignore
//! IconButton::new(Icon::Rewind)
//!     .tip(Tip::new(fl!("video-controls-back")).keys(&["F1"]), Position::Top)
//!     .on_press(Message::SeekBack10)
//! ```

use iced::widget::{button, container, mouse_area, stack, text};
use iced::{Color, Element, Length};

use super::icons::{icon, Icon};
use super::style::{self, ButtonKind};
use super::tokens::*;
use super::tooltip::{self, Position, Tip};

/// What a button shows: an icon, or a short word that stays a word (`[`, `]`, `CC`, §7).
#[derive(Debug, Clone, Copy)]
enum Face {
    Icon(Icon),
    Glyph(&'static str),
}

/// An icon button; build it, then turn it into an `Element`.
pub struct IconButton<M> {
    face: Face,
    side: f32,
    icon_size: f32,
    latched: bool,
    held: bool,
    overlay: bool,
    color: Option<Color>,
    dot: bool,
    on_press: Option<M>,
    hold: Option<(M, M)>,
    tip: Option<(Tip, Position)>,
}

impl<M: Clone> IconButton<M> {
    /// A 32 × 32 toolbar button with a 16-px `glyph`.
    pub fn new(glyph: Icon) -> Self {
        Self::with_face(Face::Icon(glyph))
    }

    /// A 32 × 32 toolbar button showing a word instead of an icon.
    pub fn glyph(word: &'static str) -> Self {
        Self::with_face(Face::Glyph(word))
    }

    fn with_face(face: Face) -> Self {
        Self {
            face,
            side: BAR_HEIGHT,
            icon_size: ICON_M,
            latched: false,
            held: false,
            overlay: false,
            color: None,
            dot: false,
            on_press: None,
            hold: None,
            tip: None,
        }
    }

    /// 24 × 24 with a 14-px icon: inside a row or a chip.
    pub fn small(mut self) -> Self {
        self.side = ICON_BUTTON_SMALL;
        self.icon_size = ICON_MARK;
        self
    }

    /// A 28 × 28 button, as high as the fields beside it (the filter next to a search field).
    pub fn control(mut self) -> Self {
        self.side = CONTROL_HEIGHT;
        self
    }

    /// Latched on: a toggle that shows its state (a list shown, fullscreen, batch mode).
    pub fn latched(mut self, latched: bool) -> Self {
        self.latched = latched;
        self
    }

    /// Drawn pressed while something holds it down (a marker being drawn).
    pub fn held(mut self, held: bool) -> Self {
        self.held = held;
        self
    }

    /// Over the video, where the usual hover does not show.
    pub fn overlay(mut self) -> Self {
        self.overlay = true;
        self
    }

    /// The icon in `color` instead of the text color (a mark on a tag chip).
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// The dot in its corner: an update is ready.
    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    pub fn on_press(mut self, message: M) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<M>) -> Self {
        self.on_press = message;
        self
    }

    /// Sends `down` when pressed and `up` when released or left: a button held like a key.
    pub fn on_hold(mut self, down: M, up: M) -> Self {
        self.on_press = Some(up.clone());
        self.hold = Some((down, up));
        self
    }

    pub fn tip(mut self, tip: impl Into<Tip>, position: Position) -> Self {
        self.tip = Some((tip.into(), position));
        self
    }

    /// Whether hovering the button shows a tooltip.
    pub fn has_tip(&self) -> bool {
        self.tip.is_some()
    }

    fn kind(&self) -> ButtonKind {
        if self.overlay {
            ButtonKind::OverlayIcon
        } else {
            ButtonKind::Icon(self.latched)
        }
    }

    fn face_color(&self) -> Color {
        match (self.on_press.is_some(), self.color) {
            (false, _) => TEXT_DISABLED,
            (true, Some(color)) => color,
            (true, None) if self.latched => ACCENT_TEXT,
            (true, None) => TEXT,
        }
    }
}

impl<'a, M: Clone + 'a> From<IconButton<M>> for Element<'a, M> {
    fn from(b: IconButton<M>) -> Self {
        let color = b.face_color();
        let face: Element<'a, M> = match b.face {
            Face::Icon(glyph) => icon(glyph, b.icon_size, color).into(),
            Face::Glyph(word) => text(word)
                .size(TEXT_BODY)
                .font(FONT_STRONG)
                .color(color)
                .into(),
        };
        let face: Element<'a, M> = container(face).center(Length::Fill).into();
        let face = match b.hold.clone() {
            Some((down, up)) => mouse_area(face)
                .on_press(down)
                .on_release(up.clone())
                .on_exit(up)
                .into(),
            None => face,
        };
        let kind = b.kind();
        let held = b.held;
        let pressable = button(face)
            .width(b.side)
            .height(b.side)
            .padding(0)
            .on_press_maybe(b.on_press)
            .style(move |theme, status| {
                let status = if held {
                    button::Status::Pressed
                } else {
                    status
                };
                style::button(kind)(theme, status)
            });
        let body: Element<'a, M> = if b.dot {
            stack![
                pressable,
                container(
                    container(iced::widget::space())
                        .width(DOT_SIZE)
                        .height(DOT_SIZE)
                        .style(style::dot(ACCENT_TEXT))
                )
                .width(b.side)
                .align_right(b.side)
                .padding(SPACE_XS)
            ]
            .into()
        } else {
            pressable.into()
        };
        match b.tip {
            Some((tip, position)) => tooltip::tip(body, tip, position),
            None => body,
        }
    }
}
