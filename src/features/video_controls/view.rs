//! The groups of the controls bar (design system §13.3.5) and the timeline. Each control is a
//! [`Command`]: the same one is drawn as an icon button in the bar or as an item of **More**
//! when the pane is too narrow for it (see [`super::Fold`]).

use iced::widget::{button as iced_button, container, mouse_area, row, stack, Row};
use iced::{mouse, Alignment, Element, Length, Padding};

use super::progress_bar::{BarMarker, ProgressBar};
use super::{Message, VideoControlsState};
use crate::ui::icon_button::IconButton;
use crate::ui::icons::{icon, Icon};
use crate::ui::menu::MenuItem;
use crate::ui::style::{self, ButtonKind};
use crate::ui::text;
use crate::ui::tokens::*;
use crate::ui::tooltip::{Position, Tip};

/// What a control shows: an icon, or a word that stays a word (`[`, `]`).
#[derive(Debug, Clone, Copy)]
pub enum Face {
    Icon(Icon),
    Glyph(&'static str),
}

/// A control of the bar, drawn as a button there or as an item of More.
pub struct Command<M> {
    pub face: Face,
    /// Its name, the tooltip's first line and the menu item's label.
    pub label: String,
    /// A second line of the tooltip.
    pub detail: Option<String>,
    pub keys: &'static [&'static str],
    /// A toggle that is on (a list shown, fullscreen).
    pub latched: bool,
    /// `None`: disabled.
    pub on_press: Option<M>,
    /// Held like a key: `down` when pressed, `up` when released. In More a click is a press
    /// and a release.
    pub hold: Option<(M, M)>,
    /// Drawn pressed, with the recording dot: something holds it down.
    pub held: bool,
}

impl<M: Clone> Command<M> {
    fn new(face: Face, label: String, keys: &'static [&'static str], on_press: Option<M>) -> Self {
        Self {
            face,
            label,
            detail: None,
            keys,
            latched: false,
            on_press,
            hold: None,
            held: false,
        }
    }

    pub fn icon(glyph: Icon, label: String, keys: &'static [&'static str], on: Option<M>) -> Self {
        Self::new(Face::Icon(glyph), label, keys, on)
    }

    pub fn latched(mut self, latched: bool) -> Self {
        self.latched = latched;
        self
    }

    pub fn detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// The command with its messages in the parent's type.
    pub fn map<N>(self, f: impl Fn(M) -> N) -> Command<N> {
        Command {
            face: self.face,
            label: self.label,
            detail: self.detail,
            keys: self.keys,
            latched: self.latched,
            on_press: self.on_press.map(&f),
            hold: self.hold.map(|(down, up)| (f(down), f(up))),
            held: self.held,
        }
    }

    /// The icon button in the bar, with its tooltip above it.
    pub fn button<'a>(self) -> Element<'a, M>
    where
        M: 'a,
    {
        let base = match self.face {
            Face::Icon(glyph) => IconButton::new(glyph),
            Face::Glyph(word) => IconButton::glyph(word),
        };
        let mut tip = Tip::new(self.label).keys(self.keys);
        if let Some(detail) = self.detail {
            tip = tip.detail(detail);
        }
        let base = base
            .latched(self.latched)
            .held(self.held)
            .tip(tip, Position::Top);
        let pressable: Element<'a, M> = match (self.hold, self.on_press.is_some()) {
            (Some((down, up)), true) => base.on_hold(down, up).into(),
            _ => base.on_press_maybe(self.on_press).into(),
        };
        if !self.held {
            return pressable;
        }
        // Recording: a dot in the corner of the held button.
        let dot = container(
            container(iced::widget::space())
                .width(DOT_SIZE)
                .height(DOT_SIZE)
                .style(style::dot(ERROR)),
        )
        .width(BAR_HEIGHT)
        .align_right(BAR_HEIGHT)
        .padding(SPACE_XS);
        stack![pressable, dot].into()
    }

    /// The command as an item of More: a click there sends its messages, in order, through
    /// `then`.
    pub fn menu_item(self, then: impl Fn(Vec<M>) -> M) -> MenuItem<M> {
        let messages = match (self.hold, self.on_press) {
            (Some((down, up)), Some(_)) => Some(vec![down, up]),
            (_, on_press) => on_press.map(|m| vec![m]),
        };
        MenuItem {
            icon: match self.face {
                Face::Icon(glyph) => Some(glyph),
                Face::Glyph(_) => None,
            },
            label: self.label,
            keys: self.keys.to_vec(),
            checked: self.latched,
            on_press: messages.map(then),
        }
    }
}

/// Buttons of one group: their 32-px squares touch.
pub fn group<'a, M: Clone + 'a>(commands: impl IntoIterator<Item = Command<M>>) -> Row<'a, M> {
    Row::with_children(commands.into_iter().map(Command::button)).align_y(Alignment::Center)
}

/// Back 10 s, play or pause, forward 10 s. The play button shows what a click will do.
pub fn transport(state: &VideoControlsState) -> [Command<Message>; 3] {
    let (play_icon, play_label) = if state.is_playing() {
        (Icon::Pause, fl!("video-controls-pause"))
    } else {
        (Icon::Play, fl!("video-controls-play"))
    };
    [
        Command::icon(
            Icon::Rewind,
            fl!("video-controls-back"),
            &["F1"],
            Some(Message::SeekBack10),
        ),
        Command::icon(
            play_icon,
            play_label,
            &["Space"],
            Some(Message::TogglePlayPause),
        ),
        Command::icon(
            Icon::FastForward,
            fl!("video-controls-forward"),
            &["F3"],
            Some(Message::SeekForward10),
        ),
    ]
}

/// The in and out points: words, as they are on the keys.
pub fn in_out() -> [Command<Message>; 2] {
    [
        Command::new(
            Face::Glyph("["),
            fl!("video-controls-set-in"),
            &["["],
            Some(Message::SetSegmentStart),
        ),
        Command::new(
            Face::Glyph("]"),
            fl!("video-controls-set-out"),
            &["]"],
            Some(Message::SetSegmentEnd),
        ),
    ]
}

/// Save this frame, and add a marker. `can_add_markers` is false when the file cannot hold
/// markers; `marker_held` while `F2` or the button draws a range.
pub fn mark(can_add_markers: bool, marker_held: bool) -> [Command<Message>; 2] {
    let marker = if can_add_markers {
        let mut marker = Command::icon(
            Icon::MapPin,
            fl!("video-controls-add-marker"),
            &["F2"],
            Some(Message::MarkerKeyReleased),
        )
        .detail(fl!("video-controls-add-marker-hold"));
        // Held like `F2`: pressed, a marker starts; released (or left), it ends there, so
        // holding the button while the clip plays marks a range.
        marker.hold = Some((Message::MarkerKeyPressed, Message::MarkerKeyReleased));
        marker.held = marker_held;
        marker
    } else {
        Command::icon(
            Icon::MapPin,
            fl!("video-controls-cannot-hold-markers"),
            &[],
            None,
        )
    };
    [
        Command::icon(
            Icon::Camera,
            fl!("video-controls-screenshot"),
            &["F12"],
            Some(Message::TakeScreenshot),
        ),
        marker,
    ]
}

/// Turn the video left or right. `cannot_rotate` says why the file cannot be turned: they are
/// off then, with it as their tooltip's second line.
pub fn rotate(cannot_rotate: Option<String>) -> [Command<Message>; 2] {
    let turn = |glyph, label: String, keys, quarter_turns| {
        let command = Command::icon(
            glyph,
            label,
            keys,
            cannot_rotate
                .is_none()
                .then_some(Message::Rotate(quarter_turns)),
        );
        match &cannot_rotate {
            Some(reason) => command.detail(reason.clone()),
            None => command,
        }
    };
    [
        turn(
            Icon::RotateCcw,
            fl!("video-controls-rotate-left"),
            &["Ctrl", "Alt", "←"],
            -1,
        ),
        turn(
            Icon::RotateCw,
            fl!("video-controls-rotate-right"),
            &["Ctrl", "Alt", "→"],
            1,
        ),
    ]
}

/// `00:10 / 00:30`, fixed wide so it never moves.
pub fn time_readout<'a>(state: &VideoControlsState, position_secs: f32) -> Element<'a, Message> {
    let readout = format!(
        "{} / {}",
        clock(position_secs),
        clock(state.duration_secs())
    );
    container(text::mono(readout).wrapping(iced::widget::text::Wrapping::None))
        .width(TIME_READOUT_WIDTH)
        .align_right(TIME_READOUT_WIDTH)
        .into()
}

/// `mm:ss`, or `h:mm:ss` from an hour on.
pub fn clock(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// The volume slider alone: in the bar after its icon, or in More.
pub fn volume_slider(state: &VideoControlsState) -> Element<'_, Message> {
    container(
        ProgressBar::new(0.0..=1.0, state.volume(), Message::SetVolume)
            .fill_color(TEXT_SECONDARY)
            .without_playhead(),
    )
    .width(VOLUME_WIDTH)
    .into()
}

/// The volume: its icon (not a button) and the slider. The wheel over either changes it too.
pub fn volume(state: &VideoControlsState) -> Element<'_, Message> {
    let volume = state.volume();
    mouse_area(
        row![
            icon(Icon::Volume, ICON_M, TEXT_SECONDARY),
            volume_slider(state)
        ]
        .spacing(SPACE_S)
        .align_y(Alignment::Center),
    )
    .on_scroll(move |delta| Message::SetVolume(volume_after_scroll(volume, delta)))
    .into()
}

/// How far one notch of a mouse wheel moves the volume (out of 1). A trackpad's pixel delta is
/// scaled to it over `VOLUME_WHEEL_PIXELS_PER_STEP`, so a gentle swipe (many small deltas) moves
/// it about as far as a deliberate one, not a full step each.
const VOLUME_SCROLL_STEP: f32 = 0.05;
/// Trackpad pixels worth one full `VOLUME_SCROLL_STEP`.
const VOLUME_WHEEL_PIXELS_PER_STEP: f32 = 20.0;

/// The volume after the wheel moved by `delta` over the volume control, kept within 0..=1.
pub fn volume_after_scroll(current: f32, delta: mouse::ScrollDelta) -> f32 {
    // `f32::signum` is 1.0 for a zero of either sign, so a still wheel needs its own case.
    let step = match delta {
        mouse::ScrollDelta::Lines { y: 0.0, .. } => 0.0,
        mouse::ScrollDelta::Lines { y, .. } => y.signum() * VOLUME_SCROLL_STEP,
        mouse::ScrollDelta::Pixels { y, .. } => {
            (y / VOLUME_WHEEL_PIXELS_PER_STEP).clamp(-1.0, 1.0) * VOLUME_SCROLL_STEP
        }
    };
    (current + step).clamp(0.0, 1.0)
}

/// The seek bar with the in/out range, the clip markers as pins and the label of the marker the
/// playhead is on, filling the width it is given.
pub fn progress_bar<'a>(
    state: &'a VideoControlsState,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: Vec<BarMarker>,
    marker_label: Option<MarkerLabel<'a>>,
    label_lane: bool,
) -> Element<'a, Message> {
    // While seeking, show the drag position; otherwise use live video position
    let current_pos = if state.is_seeking() {
        state.seek_position_secs()
    } else {
        position_secs
    };
    ProgressBar::new(0.0..=state.duration_secs(), current_pos, Message::Seek)
        .on_release(Message::SeekReleased)
        .segment_range(segment_start, segment_end)
        .markers(markers)
        .on_marker_span(Message::SetMarkerSpan)
        .on_new_range(!state.is_playing(), Message::AddRange)
        .on_play_range(Message::PlayRange)
        .label(marker_label.map(|label| (label.at, marker_label_button(label))))
        .label_right_edge(marker_label.and_then(|label| label.right_edge))
        .label_lane(label_lane)
        .into()
}

/// The marker the playhead is on, as the progress bar labels it.
#[derive(Debug, Clone, Copy)]
pub struct MarkerLabel<'a> {
    /// Where the label is centred, in seconds: a point's tick, or the middle of a range.
    pub at: f32,
    pub name: &'a str,
    /// `None` for a marker frename cannot change (it has no GUID).
    pub guid: Option<&'a str>,
    /// Its pin's color: the label is the pin's head, outlined in it.
    pub color: iced::Color,
    /// The player's right edge (window x) the label stays left of; `None` in fullscreen,
    /// where the player is the whole window.
    pub right_edge: Option<f32>,
}

/// Room the label's padding, edge and pencil take besides the name, and its margins.
const LABEL_CHROME: f32 = 2.0 * SPACE_S + SPACE_TIGHT + ICON_S + 2.0 * RING + 2.0 * SPACE_XS;
/// A generous average advance of a character of the label's 12 px text.
const LABEL_CHAR_ADVANCE: f32 = 6.8;
/// A name is never cut shorter than this many characters.
const LABEL_MIN_CHARS: usize = 12;
/// The label's edge in its marker's color: a little heavier than a line, so the color reads.
const LABEL_EDGE: f32 = LINE * 1.5;

/// `name`, cut with "…" so the label fits a player `width` px wide.
fn fit_label(name: &str, width: f32) -> String {
    let fits = ((width - LABEL_CHROME) / LABEL_CHAR_ADVANCE).max(1.0) as usize;
    let room = fits.max(LABEL_MIN_CHARS);
    if name.chars().count() <= room {
        return name.to_string();
    }
    let cut: String = name.chars().take(room.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

const LABEL_PADDING: Padding = Padding {
    top: SPACE_XXS,
    bottom: SPACE_XXS,
    left: SPACE_S,
    right: SPACE_S,
};

/// The label over the marker's tick: its name and a pencil. A click opens the marker's row in
/// the marker list with the name field focused; a read-only marker's label has no pencil and
/// does nothing.
fn marker_label_button(label: MarkerLabel<'_>) -> Element<'_, Message> {
    let name = if label.name.trim().is_empty() {
        text::tooltip(fl!("video-controls-add-a-name")).color(TEXT_SECONDARY)
    } else {
        let fitted = match label.right_edge {
            Some(width) => fit_label(label.name, width),
            None => label.name.to_string(),
        };
        text::tooltip(fitted).wrapping(iced::widget::text::Wrapping::None)
    };
    let content = row![name]
        .push(
            label
                .guid
                .map(|_| icon(Icon::Pencil, ICON_S, TEXT_SECONDARY)),
        )
        .spacing(SPACE_TIGHT)
        .align_y(Alignment::Center);
    let color = label.color;
    iced_button(content)
        .on_press_maybe(label.guid.map(|guid| Message::EditMarker(guid.to_string())))
        .padding(LABEL_PADDING)
        .style(move |theme, status| {
            let hovered = matches!(
                status,
                iced_button::Status::Hovered | iced_button::Status::Pressed
            );
            iced_button::Style {
                background: Some(iced::Background::Color(if hovered {
                    over(BG_OVERLAY, HOVER)
                } else {
                    BG_OVERLAY
                })),
                text_color: TEXT,
                border: iced::Border {
                    color,
                    width: LABEL_EDGE,
                    radius: RADIUS_M.into(),
                },
                ..style::button(ButtonKind::Ghost)(theme, status)
            }
        })
        .width(Length::Shrink)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_up_raises_volume_and_down_lowers_it_clamped() {
        let up = mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
        let down = mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 };
        assert_eq!(volume_after_scroll(0.5, up), 0.5 + VOLUME_SCROLL_STEP);
        assert_eq!(volume_after_scroll(0.5, down), 0.5 - VOLUME_SCROLL_STEP);
        assert_eq!(volume_after_scroll(1.0, up), 1.0, "clamped at the top");
        assert_eq!(volume_after_scroll(0.0, down), 0.0, "clamped at the bottom");
        let still = mouse::ScrollDelta::Lines { x: 0.0, y: 0.0 };
        assert_eq!(
            volume_after_scroll(0.5, still),
            0.5,
            "a still wheel is a no-op"
        );
        let no_move = mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 };
        assert_eq!(volume_after_scroll(0.5, no_move), 0.5);
    }

    /// A trackpad's inertial scroll sends many small `Pixels` deltas per gesture: each moves the
    /// volume only as far as its own size warrants, and never more than one step (#96).
    #[test]
    fn a_small_trackpad_scroll_moves_the_volume_less_than_a_full_step() {
        let gentle = mouse::ScrollDelta::Pixels { x: 0.0, y: 2.0 };
        let after = volume_after_scroll(0.5, gentle);
        assert!(after > 0.5 && after < 0.5 + VOLUME_SCROLL_STEP, "{after}");
        let large = mouse::ScrollDelta::Pixels {
            x: 0.0,
            y: VOLUME_WHEEL_PIXELS_PER_STEP * 10.0,
        };
        assert_eq!(volume_after_scroll(0.5, large), 0.5 + VOLUME_SCROLL_STEP);
        let gentle_down = mouse::ScrollDelta::Pixels { x: 0.0, y: -2.0 };
        let after_down = volume_after_scroll(0.5, gentle_down);
        assert!(
            after_down < 0.5 && after_down > 0.5 - VOLUME_SCROLL_STEP,
            "{after_down}"
        );
    }

    #[test]
    fn a_long_marker_name_is_cut_to_the_player() {
        assert_eq!(fit_label("Lion", 400.0), "Lion");
        let long = "Close-up of the blue Turkish Airlines sign hanging from the ceiling";
        let fitted = fit_label(long, 300.0);
        assert!(fitted.ends_with('…'), "{fitted}");
        assert!(fitted.chars().count() as f32 * LABEL_CHAR_ADVANCE + LABEL_CHROME <= 300.0);
    }

    #[test]
    fn a_name_keeps_at_least_twelve_characters() {
        let fitted = fit_label("Close-up of the blue sign", 10.0);
        assert_eq!(fitted.chars().count(), LABEL_MIN_CHARS);
    }

    #[test]
    fn the_clock_shows_hours_only_from_an_hour_on() {
        assert_eq!(clock(10.4), "00:10");
        assert_eq!(clock(3_725.0), "1:02:05");
        assert_eq!(clock(-1.0), "00:00");
    }

    #[test]
    fn a_held_button_sends_its_press_and_release_from_more() {
        let [_, marker] = mark(true, false);
        let item = marker.menu_item(|messages| Message::SetVolume(messages.len() as f32));
        assert!(matches!(item.on_press, Some(Message::SetVolume(n)) if n == 2.0));
        let [_, marker] = mark(false, false);
        assert!(marker.menu_item(|_| Message::SeekBack10).on_press.is_none());
    }
}
