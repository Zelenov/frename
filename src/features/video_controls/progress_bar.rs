//! Custom rectangular progress bar widget with click-to-seek and drag support. Clip markers
//! are pins in their colors: a needle through the bar with a round head above it, or, for the
//! marker the playhead is on, with its name label as the head. A ranged marker also draws a
//! band under the bar; overlapping bands go into lanes. With Shift held, a seek snaps to the
//! nearest marker.
//!
//! Ranges are edited on the bar: the active range shows handles at both ends that drag its
//! ends, `Alt`+click on a band turns it back into a point, a click on a band plays it, and
//! (while paused) `Alt`+drag on the bar draws a new range.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, overlay, renderer, Clipboard, Shell};
use iced::{keyboard, mouse};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector};

use crate::theme;

const BAR_HEIGHT: f32 = 8.0;
const HIT_HEIGHT: f32 = 24.0;
/// Top of the bar within the widget; the widget grows downwards when bands need more lanes.
const BAR_TOP: f32 = (HIT_HEIGHT - BAR_HEIGHT) / 2.0;
const BORDER_RADIUS: f32 = 4.0;
/// How far (px) a pin's needle reaches below the bar.
const TICK_OVERHANG: f32 = 3.0;
/// Diameter of a pin's round head, at the top of the widget.
const PIN_HEAD: f32 = 7.0;
const NEEDLE_WIDTH: f32 = 2.0;
/// Height of the band a ranged marker draws under the bar.
const BAND_HEIGHT: f32 = 3.0;
/// From one lane of bands to the next.
const LANE_PITCH: f32 = BAND_HEIGHT + 1.0;
/// Overlapping ranges stack into at most this many lanes; the rest share the last one.
pub const MAX_LANES: usize = 3;
/// Width of the handles at the ends of the active range.
const HANDLE_WIDTH: f32 = 6.0;
/// How far a handle reaches above and below its band, so it reads as something to grab.
const HANDLE_OVERHANG: f32 = 3.0;
/// How far (px) around a band or a handle a press still hits it.
const HIT_SLACK: f32 = 3.0;
/// A Shift seek snaps to a marker this close to the cursor (px).
const SNAP_DISTANCE: f32 = 8.0;
/// How far into the widget the label (the active pin's head) reaches: its bottom sits where
/// a round head would be, and the needle runs from it.
const LABEL_DIP: f32 = 2.0;
/// Least room between the label and the edge of the window or the player.
const LABEL_MARGIN: f32 = 4.0;
/// Bands that are not the active one are drawn this opaque, so the active one stands out.
const IDLE_BAND_ALPHA: f32 = 0.6;

/// A clip marker as the bar draws it, in the bar's unit.
#[derive(Debug, Clone, PartialEq)]
pub struct BarMarker {
    pub start: f32,
    /// Equal to `start` for a point marker.
    pub end: f32,
    pub color: Color,
    /// The playhead is on it: the label over the bar is its head, not a round one.
    pub active: bool,
    /// `None` for a marker frename cannot change: it gets no handles.
    pub guid: Option<String>,
}

impl BarMarker {
    fn is_range(&self) -> bool {
        self.end > self.start
    }
}

/// What a press on the bar started.
#[derive(Debug, Clone, Default, PartialEq)]
enum Drag {
    #[default]
    None,
    /// Seeking, as a plain click and drag always did.
    Seek,
    /// Moving one end of a range: `fixed` stays, `moving` follows the cursor.
    Handle {
        guid: String,
        fixed: f32,
        moving: f32,
    },
    /// `Alt`+drag drawing a new range.
    NewRange { from: f32, to: f32 },
    /// A band was clicked: nothing follows until the release.
    Band,
}

/// Internal widget state for tracking drag
#[derive(Default)]
struct State {
    drag: Drag,
    /// Whether Shift is held: seeks and handles snap to markers.
    shift: bool,
    /// Whether Alt is held: a band click makes a point, a drag on the bar a new range.
    alt: bool,
}

/// Callbacks for editing ranges on the bar.
type SpanFn<'a, Message> = Box<dyn Fn(String, f32, f32) -> Message + 'a>;
type RangeFn<'a, Message> = Box<dyn Fn(f32, f32) -> Message + 'a>;

/// A rectangular progress bar that supports click and drag to seek.
/// Works with absolute values in a range, like `Slider`.
pub struct ProgressBar<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    /// Minimum value
    min: f32,
    /// Maximum value
    max: f32,
    /// Current value within min..=max
    value: f32,
    /// Called with value in min..=max when user clicks or drags
    on_seek: Box<dyn Fn(f32) -> Message + 'a>,
    /// Called when user releases the mouse after seeking
    on_release: Option<Message>,
    /// Optional segment start in seconds (for the highlighted range).
    segment_start: Option<f32>,
    /// Optional segment end in seconds (for the highlighted range).
    segment_end: Option<f32>,
    /// Fill color for the progress portion (defaults to theme::ACCENT).
    fill_color: Option<Color>,
    /// Clip markers to draw as colored ticks (ranged ones with a band) and snap to.
    markers: Vec<BarMarker>,
    /// The lane of each marker's band (0 for points), and how many lanes are in use.
    lanes: Vec<usize>,
    lane_count: usize,
    /// Shown above the bar at a value: the name of the marker the playhead is on.
    label: Option<(f32, Element<'a, Message, Theme, Renderer>)>,
    /// Right edge (window x) the label stays left of; `None` for the window's.
    label_right_edge: Option<f32>,
    /// A range's ends were dragged (or `Alt`+click made it a point): its GUID, start and end.
    on_marker_span: Option<SpanFn<'a, Message>>,
    /// `Alt`+drag drew a new range from start to end; `None` turns `Alt`+drag off.
    on_new_range: Option<RangeFn<'a, Message>>,
    /// A band was clicked: play from its start to its end.
    on_play_range: Option<RangeFn<'a, Message>>,
}

impl<'a, Message, Theme, Renderer> ProgressBar<'a, Message, Theme, Renderer> {
    /// Create a new progress bar with a range and current value.
    /// Usage: `ProgressBar::new(0.0..=duration, position, Message::Seek)`
    pub fn new(
        range: std::ops::RangeInclusive<f32>,
        value: f32,
        on_seek: impl Fn(f32) -> Message + 'a,
    ) -> Self {
        let min = *range.start();
        let max = *range.end();
        Self {
            min,
            max,
            value: value.clamp(min, max),
            on_seek: Box::new(on_seek),
            on_release: None,
            segment_start: None,
            segment_end: None,
            fill_color: None,
            markers: Vec::new(),
            lanes: Vec::new(),
            lane_count: 0,
            label: None,
            label_right_edge: None,
            on_marker_span: None,
            on_new_range: None,
            on_play_range: None,
        }
    }

    /// Override the fill color for the progress portion.
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// Message to emit when the user releases after seeking
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// Set optional segment start/end positions (in seconds) to highlight on the bar.
    pub fn segment_range(mut self, start: Option<f32>, end: Option<f32>) -> Self {
        self.segment_start = start;
        self.segment_end = end;
        self
    }

    /// Set the clip markers (in the same unit as the range).
    pub fn markers(mut self, markers: impl IntoIterator<Item = BarMarker>) -> Self {
        self.markers = markers.into_iter().collect();
        (self.lanes, self.lane_count) = assign_lanes(&self.markers);
        self
    }

    /// Show `content` above the bar, centred on `value` (in the range's unit) and kept within
    /// the bar's width. It is an overlay: it reaches above the widget, over the picture, and
    /// takes the clicks there instead of what it covers.
    pub fn label(mut self, label: Option<(f32, Element<'a, Message, Theme, Renderer>)>) -> Self {
        self.label = label;
        self
    }

    /// Keep the label left of this window x (the player's right edge) instead of the
    /// window's right edge. On the left it stays within the window: the player starts there.
    pub fn label_right_edge(mut self, right_edge: Option<f32>) -> Self {
        self.label_right_edge = right_edge;
        self
    }

    /// Let the active range's handles and `Alt`+click on a band change a marker's span.
    pub fn on_marker_span(mut self, f: impl Fn(String, f32, f32) -> Message + 'a) -> Self {
        self.on_marker_span = Some(Box::new(f));
        self
    }

    /// Let `Alt`+drag on the bar draw a new range, when `enabled`.
    pub fn on_new_range(mut self, enabled: bool, f: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_new_range = enabled.then(|| Box::new(f) as RangeFn<'a, Message>);
        self
    }

    /// Play a band's range when it is clicked.
    pub fn on_play_range(mut self, f: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_play_range = Some(Box::new(f));
        self
    }

    /// Current progress as a fraction 0.0..=1.0
    fn fraction(&self) -> f32 {
        let span = self.max - self.min;
        if span > 0.0 {
            (self.value - self.min) / span
        } else {
            0.0
        }
    }

    fn height(&self) -> f32 {
        height_for_lanes(self.lane_count)
    }

    /// Convert cursor position to a value in min..=max, not snapped.
    fn raw_value(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<f32> {
        let x = cursor.position()?.x;
        let fraction = ((x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        Some(self.min + fraction * (self.max - self.min))
    }

    /// Convert cursor position to a value in min..=max; with `snap`, the start of the nearest
    /// marker within [`SNAP_DISTANCE`] instead, when there is one.
    fn value_from_cursor(
        &self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        snap: bool,
    ) -> Option<f32> {
        let value = self.raw_value(bounds, cursor)?;
        if !snap {
            return Some(value);
        }
        Some(snap_to(
            value,
            self.markers.iter().map(|m| m.start),
            self.px_per_unit(bounds.width),
        ))
    }

    /// Where a dragged range end lands: with `snap`, on the nearest start or end of another
    /// marker, or on the in/out points.
    fn handle_value(
        &self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        snap: bool,
        guid: &str,
    ) -> Option<f32> {
        let value = self.raw_value(bounds, cursor)?;
        if !snap {
            return Some(value);
        }
        let others = self
            .markers
            .iter()
            .filter(|m| m.guid.as_deref() != Some(guid))
            .flat_map(|m| [m.start, m.end]);
        let in_out = self.segment_start.into_iter().chain(self.segment_end);
        Some(snap_to(
            value,
            others.chain(in_out),
            self.px_per_unit(bounds.width),
        ))
    }

    fn px_per_unit(&self, width: f32) -> f32 {
        let span = self.max - self.min;
        if span > 0.0 {
            width / span
        } else {
            0.0
        }
    }

    /// X of `value` within `bounds`, clamped to the bar.
    fn x_of(&self, bounds: Rectangle, value: f32) -> f32 {
        let span = self.max - self.min;
        if span <= 0.0 {
            return bounds.x;
        }
        bounds.x + ((value - self.min) / span).clamp(0.0, 1.0) * bounds.width
    }

    /// Top of the band in `lane`.
    fn band_y(bounds: Rectangle, lane: usize) -> f32 {
        bounds.y + BAR_TOP + BAR_HEIGHT + 1.0 + lane as f32 * LANE_PITCH
    }

    /// The active editable range whose handle is under the cursor: its GUID, the end that
    /// stays and the end that moves.
    fn handle_at(&self, bounds: Rectangle, at: Point) -> Option<(String, f32, f32)> {
        self.on_marker_span.as_ref()?;
        self.markers
            .iter()
            .zip(&self.lanes)
            .filter(|(m, _)| m.active && m.is_range())
            .find_map(|(m, &lane)| {
                let guid = m.guid.clone()?;
                let y = Self::band_y(bounds, lane);
                if at.y < y - HIT_SLACK || at.y > y + BAND_HEIGHT + HIT_SLACK {
                    return None;
                }
                let reach = HANDLE_WIDTH / 2.0 + HIT_SLACK;
                let (x0, x1) = (self.x_of(bounds, m.start), self.x_of(bounds, m.end));
                if (at.x - x1).abs() <= reach {
                    Some((guid, m.start, m.end))
                } else if (at.x - x0).abs() <= reach {
                    Some((guid, m.end, m.start))
                } else {
                    None
                }
            })
    }

    /// The band under the cursor: the one drawn last where bands share a lane.
    fn band_at(&self, bounds: Rectangle, at: Point) -> Option<&BarMarker> {
        self.markers
            .iter()
            .zip(&self.lanes)
            .rev()
            .filter(|(m, _)| m.is_range())
            .find(|(m, &lane)| {
                let y = Self::band_y(bounds, lane);
                let (x0, x1) = (self.x_of(bounds, m.start), self.x_of(bounds, m.end));
                (y - HIT_SLACK..=y + BAND_HEIGHT + HIT_SLACK).contains(&at.y)
                    && (x0 - HIT_SLACK..=x1 + HIT_SLACK).contains(&at.x)
            })
            .map(|(m, _)| m)
    }
}

/// `value`, or the nearest of `targets` at most [`SNAP_DISTANCE`] px away.
fn snap_to(value: f32, targets: impl Iterator<Item = f32>, px_per_unit: f32) -> f32 {
    targets
        .map(|t| (t, ((t - value) * px_per_unit).abs()))
        .filter(|(_, distance)| *distance <= SNAP_DISTANCE)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(value, |(t, _)| t)
}

/// `value`, or the start of the nearest marker at most [`SNAP_DISTANCE`] px away.
#[cfg(test)]
fn snap_to_marker(value: f32, markers: &[BarMarker], px_per_unit: f32) -> f32 {
    snap_to(value, markers.iter().map(|m| m.start), px_per_unit)
}

/// The widget's height: the bar, and a lane of bands under it for each lane beyond one.
fn height_for_lanes(lanes: usize) -> f32 {
    HIT_HEIGHT + lanes.saturating_sub(1) as f32 * LANE_PITCH
}

/// How tall the bar is with these markers: it grows when overlapping ranges need lanes.
pub fn bar_height(markers: &[BarMarker]) -> f32 {
    height_for_lanes(assign_lanes(markers).1)
}

/// The lane of each marker's band: each range goes into the first lane where it overlaps no
/// range already there, and past [`MAX_LANES`] into the last one. Points get lane 0. Returns
/// the lanes and how many are in use (0 without ranges).
pub fn assign_lanes(markers: &[BarMarker]) -> (Vec<usize>, usize) {
    let mut lanes = vec![0; markers.len()];
    let mut ends: Vec<f32> = Vec::new();
    let mut order: Vec<usize> = (0..markers.len())
        .filter(|&i| markers[i].is_range())
        .collect();
    order.sort_by(|&a, &b| markers[a].start.total_cmp(&markers[b].start));
    for i in order {
        let marker = &markers[i];
        let lane = match ends.iter().position(|&end| end <= marker.start) {
            Some(free) => free,
            None if ends.len() < MAX_LANES => {
                ends.push(f32::MIN);
                ends.len() - 1
            }
            None => MAX_LANES - 1,
        };
        ends[lane] = ends[lane].max(marker.end);
        lanes[i] = lane;
    }
    (lanes, ends.len())
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ProgressBar<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.label
            .iter()
            .map(|(_, content)| widget::Tree::new(content))
            .collect()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        match &self.label {
            Some((_, content)) => tree.diff_children(std::slice::from_ref(content)),
            None => tree.children.clear(),
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height()))
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(self.height()), Size::ZERO))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let bar_y = bounds.y + BAR_TOP;

        // Track background
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bar_y,
                    width: bounds.width,
                    height: BAR_HEIGHT,
                },
                border: Border {
                    radius: BORDER_RADIUS.into(),
                    ..Border::default()
                },
                shadow: Shadow::default(),
                snap: true,
            },
            theme::TRACK,
        );

        // Progress fill
        let fill_width = bounds.width * self.fraction();
        if fill_width > 0.5 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bar_y,
                        width: fill_width,
                        height: BAR_HEIGHT,
                    },
                    border: Border {
                        radius: BORDER_RADIUS.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                self.fill_color.unwrap_or(theme::ACCENT),
            );
        }

        // Segment highlight, then the clip markers over it.
        // Positions are clamped to [min, max] so we never draw outside the bar.
        // Rules:
        //   only start OR only end  → single vertical marker line
        //   start < end             → filled rectangle (no rounded corners) between them
        //   start >= end            → two separate marker lines, no fill
        let span = self.max - self.min;

        if span > 0.0 {
            // Convert seconds to an x-coordinate, clamped to the bar's pixel range.
            let to_x = |secs: f32| self.x_of(bounds, secs);
            let draw_marker = |renderer: &mut Renderer, x: f32| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: x - 1.0,
                            y: bar_y,
                            width: 2.0,
                            height: BAR_HEIGHT,
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    theme::SEGMENT,
                );
            };

            match (self.segment_start, self.segment_end) {
                (Some(s), Some(e)) if s < e => {
                    // Both markers in valid order → filled range (no rounded corners) + two marker lines.
                    let x0 = to_x(s);
                    let x1 = to_x(e);
                    let w = (x1 - x0).max(2.0);
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: x0,
                                y: bar_y,
                                width: w,
                                height: BAR_HEIGHT,
                            },
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        theme::SEGMENT,
                    );
                    draw_marker(renderer, x0);
                    draw_marker(renderer, x1);
                }
                (Some(s), Some(e)) => {
                    // Both exist but start >= end → two separate marker lines, no fill.
                    draw_marker(renderer, to_x(s));
                    draw_marker(renderer, to_x(e));
                }
                (Some(s), None) => draw_marker(renderer, to_x(s)),
                (None, Some(e)) => draw_marker(renderer, to_x(e)),
                (None, None) => {}
            }
        }

        // Clip markers as pins in their colors, and a band under the bar for a range, in its
        // lane. The active pin is drawn last, over its neighbours, with its needle up to the
        // label; a range being dragged is drawn where the drag has it.
        if span > 0.0 {
            let to_x = |v: f32| self.x_of(bounds, v);
            let needle_bottom = bar_y + BAR_HEIGHT + TICK_OVERHANG;
            let quad = |bounds: Rectangle, radius: f32| renderer::Quad {
                bounds,
                border: Border {
                    radius: radius.into(),
                    ..Border::default()
                },
                shadow: Shadow::default(),
                snap: true,
            };
            let span_of = |marker: &BarMarker| match &state.drag {
                Drag::Handle {
                    guid,
                    fixed,
                    moving,
                } if marker.guid.as_deref() == Some(guid.as_str()) => {
                    (fixed.min(*moving), fixed.max(*moving))
                }
                _ => (marker.start, marker.end),
            };
            let pins = self
                .markers
                .iter()
                .zip(&self.lanes)
                .filter(|(m, _)| !m.active)
                .chain(
                    self.markers
                        .iter()
                        .zip(&self.lanes)
                        .filter(|(m, _)| m.active),
                );
            for (marker, &lane) in pins {
                let (start, end) = span_of(marker);
                let x0 = to_x(start);
                if end > start {
                    let band_y = Self::band_y(bounds, lane);
                    let x1 = to_x(end);
                    let color = if marker.active {
                        marker.color
                    } else {
                        Color {
                            a: marker.color.a * IDLE_BAND_ALPHA,
                            ..marker.color
                        }
                    };
                    renderer.fill_quad(
                        quad(
                            Rectangle {
                                x: x0,
                                y: band_y,
                                width: (x1 - x0).max(2.0),
                                height: BAND_HEIGHT,
                            },
                            0.0,
                        ),
                        color,
                    );
                    if marker.active && marker.guid.is_some() && self.on_marker_span.is_some() {
                        for x in [x0, x1] {
                            renderer.fill_quad(
                                quad(
                                    Rectangle {
                                        x: x - HANDLE_WIDTH / 2.0,
                                        y: band_y - HANDLE_OVERHANG,
                                        width: HANDLE_WIDTH,
                                        height: BAND_HEIGHT + 2.0 * HANDLE_OVERHANG,
                                    },
                                    2.0,
                                ),
                                marker.color,
                            );
                        }
                    }
                }
                let needle_top = if marker.active {
                    bounds.y + LABEL_DIP
                } else {
                    bounds.y + PIN_HEAD / 2.0
                };
                renderer.fill_quad(
                    quad(
                        Rectangle {
                            x: x0 - NEEDLE_WIDTH / 2.0,
                            y: needle_top,
                            width: NEEDLE_WIDTH,
                            height: needle_bottom - needle_top,
                        },
                        0.0,
                    ),
                    marker.color,
                );
                if !marker.active {
                    renderer.fill_quad(
                        quad(
                            Rectangle {
                                x: x0 - PIN_HEAD / 2.0,
                                y: bounds.y,
                                width: PIN_HEAD,
                                height: PIN_HEAD,
                            },
                            PIN_HEAD / 2.0,
                        ),
                        marker.color,
                    );
                }
            }
            // The range an `Alt`+drag is drawing, in the first lane.
            if let Drag::NewRange { from, to } = state.drag {
                let (x0, x1) = (to_x(from.min(to)), to_x(from.max(to)));
                renderer.fill_quad(
                    quad(
                        Rectangle {
                            x: x0,
                            y: Self::band_y(bounds, 0),
                            width: (x1 - x0).max(2.0),
                            height: BAND_HEIGHT,
                        },
                        0.0,
                    ),
                    Color::WHITE,
                );
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift = modifiers.shift();
                state.alt = modifiers.alt();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(at) = cursor.position_over(bounds) else {
                    return;
                };
                if let Some((guid, fixed, moving)) = self.handle_at(bounds, at) {
                    state.drag = Drag::Handle {
                        guid,
                        fixed,
                        moving,
                    };
                    shell.request_redraw();
                    return;
                }
                if let Some(band) = self.band_at(bounds, at) {
                    let message = if state.alt {
                        band.guid
                            .clone()
                            .zip(self.on_marker_span.as_ref())
                            .map(|(guid, f)| f(guid, band.start, band.start))
                    } else {
                        self.on_play_range.as_ref().map(|f| f(band.start, band.end))
                    };
                    if let Some(message) = message {
                        state.drag = Drag::Band;
                        shell.publish(message);
                        return;
                    }
                }
                if state.alt && self.on_new_range.is_some() {
                    if let Some(value) = self.value_from_cursor(bounds, cursor, state.shift) {
                        state.drag = Drag::NewRange {
                            from: value,
                            to: value,
                        };
                        shell.request_redraw();
                    }
                    return;
                }
                state.drag = Drag::Seek;
                if let Some(value) = self.value_from_cursor(bounds, cursor, state.shift) {
                    shell.publish((self.on_seek)(value));
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => match &mut state.drag {
                Drag::Seek => {
                    if let Some(value) = self.value_from_cursor(bounds, cursor, state.shift) {
                        shell.publish((self.on_seek)(value));
                    }
                }
                Drag::Handle { guid, moving, .. } => {
                    if let Some(value) = self.handle_value(bounds, cursor, state.shift, guid) {
                        *moving = value;
                        shell.request_redraw();
                    }
                }
                Drag::NewRange { to, .. } => {
                    if let Some(value) = self.value_from_cursor(bounds, cursor, state.shift) {
                        *to = value;
                        shell.request_redraw();
                    }
                }
                Drag::None | Drag::Band => {}
            },
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                match std::mem::take(&mut state.drag) {
                    Drag::Seek => {
                        if let Some(on_release) = &self.on_release {
                            shell.publish(on_release.clone());
                        }
                    }
                    Drag::Handle {
                        guid,
                        fixed,
                        moving,
                    } => {
                        if let Some(f) = &self.on_marker_span {
                            shell.publish(f(guid, fixed.min(moving), fixed.max(moving)));
                        }
                    }
                    Drag::NewRange { from, to } => {
                        if let Some(f) = &self.on_new_range {
                            shell.publish(f(from.min(to), from.max(to)));
                        }
                    }
                    Drag::None | Drag::Band => {}
                }
            }
            _ => {}
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let span = self.max - self.min;
        let min = self.min;
        let right_edge = self.label_right_edge;
        let (value, content) = self.label.as_mut().filter(|_| span > 0.0)?;
        let bounds = layout.bounds() + translation;
        let x = bounds.x + ((*value - min) / span).clamp(0.0, 1.0) * bounds.width;
        Some(overlay::Element::new(Box::new(LabelOverlay {
            content,
            tree: tree.children.first_mut()?,
            anchor: Point::new(x, bounds.y + LABEL_DIP),
            right_edge,
        })))
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let on_handle = matches!(state.drag, Drag::Handle { .. })
            || cursor
                .position_over(bounds)
                .is_some_and(|at| self.handle_at(bounds, at).is_some());
        if on_handle {
            mouse::Interaction::ResizingHorizontally
        } else if state.drag != Drag::None || cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ProgressBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
{
    fn from(bar: ProgressBar<'a, Message, Theme, Renderer>) -> Self {
        Self::new(bar)
    }
}

/// The label over the bar, laid out centred on `anchor` (where the active pin's needle
/// starts) with its bottom there. Centred as long as it fits: near the player's edge it moves
/// in, left of `right_edge` (the window's right edge when `None`) and right of the window's
/// left edge.
struct LabelOverlay<'a, 'b, Message, Theme, Renderer> {
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut widget::Tree,
    anchor: Point,
    right_edge: Option<f32>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for LabelOverlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, bounds),
        );
        let size = node.size();
        let right = self.right_edge.unwrap_or(bounds.width).min(bounds.width);
        let x = (self.anchor.x - size.width / 2.0)
            .min(right - LABEL_MARGIN - size.width)
            .max(LABEL_MARGIN);
        node.move_to(Point::new(x, self.anchor.y - size.height))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(start: f32) -> BarMarker {
        range(start, start)
    }

    fn range(start: f32, end: f32) -> BarMarker {
        BarMarker {
            start,
            end,
            color: Color::WHITE,
            active: false,
            guid: None,
        }
    }

    #[test]
    fn a_shift_seek_snaps_to_the_nearest_marker_within_reach() {
        let markers = [at(10.0), at(12.0)];
        // 10 px per second: 8 px is 0.8 s.
        assert_eq!(snap_to_marker(10.5, &markers, 10.0), 10.0);
        assert_eq!(snap_to_marker(11.6, &markers, 10.0), 12.0);
        assert_eq!(snap_to_marker(20.0, &markers, 10.0), 20.0);
        assert_eq!(snap_to_marker(5.0, &[], 10.0), 5.0);
    }

    #[test]
    fn overlapping_ranges_stack_into_lanes_and_the_rest_share_the_last() {
        let markers = [
            range(0.0, 10.0),
            at(2.0),
            range(5.0, 8.0),
            range(10.0, 12.0),
            range(6.0, 7.0),
            range(6.5, 9.0),
        ];
        let (lanes, count) = assign_lanes(&markers);
        // 10–12 fits after 0–10 in the first lane; 6.5–9 finds no free lane among three.
        assert_eq!(lanes, [0, 0, 1, 0, 2, 2]);
        assert_eq!(count, MAX_LANES);
        assert_eq!(assign_lanes(&[at(1.0)]), (vec![0], 0));
        assert_eq!(assign_lanes(&[range(1.0, 2.0), range(3.0, 4.0)]).1, 1);
    }

    #[test]
    fn handles_snap_to_other_ends_and_to_in_out() {
        let others = [range(10.0, 20.0), at(30.0)];
        let targets = || others.iter().flat_map(|m| [m.start, m.end]).chain([25.0]);
        assert_eq!(snap_to(19.5, targets(), 10.0), 20.0);
        assert_eq!(snap_to(25.4, targets(), 10.0), 25.0);
        assert_eq!(snap_to(27.5, targets(), 10.0), 27.5);
    }
}
