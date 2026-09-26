//! Custom rectangular progress bar widget with click-to-seek and drag support. Clip markers
//! are pins in their colors: a needle through the bar with a round head above it, or, for the
//! marker the playhead is on, with its name label as the head. With Shift held, a seek snaps
//! to the nearest marker.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, overlay, renderer, Clipboard, Shell};
use iced::{keyboard, mouse};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector};

use crate::theme;

const BAR_HEIGHT: f32 = 8.0;
const HIT_HEIGHT: f32 = 24.0;
const BORDER_RADIUS: f32 = 4.0;
/// How far (px) a pin's needle reaches below the bar.
const TICK_OVERHANG: f32 = 3.0;
/// Diameter of a pin's round head, at the top of the widget.
const PIN_HEAD: f32 = 7.0;
const NEEDLE_WIDTH: f32 = 2.0;
/// Height of the band a ranged marker draws under the bar.
const BAND_HEIGHT: f32 = 3.0;
/// A Shift seek snaps to a marker this close to the cursor (px).
const SNAP_DISTANCE: f32 = 8.0;
/// How far into the widget the label (the active pin's head) reaches: its bottom sits where
/// a round head would be, and the needle runs from it.
const LABEL_DIP: f32 = 2.0;
/// Least room between the label and the edge of the window or the player.
const LABEL_MARGIN: f32 = 4.0;

/// A clip marker as the bar draws it, in the bar's unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarMarker {
    pub start: f32,
    /// Equal to `start` for a point marker.
    pub end: f32,
    pub color: Color,
    /// The playhead is on it: the label over the bar is its head, not a round one.
    pub active: bool,
}

/// Internal widget state for tracking drag
#[derive(Default)]
struct State {
    is_dragging: bool,
    /// Whether Shift is held: seeks snap to markers.
    shift: bool,
}

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
    /// Shown above the bar at a value: the name of the marker the playhead is on.
    label: Option<(f32, Element<'a, Message, Theme, Renderer>)>,
    /// Right edge (window x) the label stays left of; `None` for the window's.
    label_right_edge: Option<f32>,
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
            label: None,
            label_right_edge: None,
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

    /// Current progress as a fraction 0.0..=1.0
    fn fraction(&self) -> f32 {
        let span = self.max - self.min;
        if span > 0.0 {
            (self.value - self.min) / span
        } else {
            0.0
        }
    }

    /// Convert cursor position to a value in min..=max; with `snap`, the start of the nearest
    /// marker within [`SNAP_DISTANCE`] instead, when there is one.
    fn value_from_cursor(
        &self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        snap: bool,
    ) -> Option<f32> {
        let x = cursor.position()?.x;
        let fraction = ((x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        let value = self.min + fraction * (self.max - self.min);
        if !snap {
            return Some(value);
        }
        Some(snap_to_marker(
            value,
            &self.markers,
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
}

/// `value`, or the start of the nearest marker at most [`SNAP_DISTANCE`] px away.
fn snap_to_marker(value: f32, markers: &[BarMarker], px_per_unit: f32) -> f32 {
    markers
        .iter()
        .map(|m| (m.start, ((m.start - value) * px_per_unit).abs()))
        .filter(|(_, distance)| *distance <= SNAP_DISTANCE)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(value, |(start, _)| start)
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
        Size::new(Length::Fill, Length::Fixed(HIT_HEIGHT))
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(HIT_HEIGHT), Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let bar_y = bounds.y + (bounds.height - BAR_HEIGHT) / 2.0;

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
            let to_x =
                |secs: f32| bounds.x + ((secs - self.min) / span).clamp(0.0, 1.0) * bounds.width;
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

        // Clip markers as pins in their colors, and a band under the bar for a range. The
        // active pin is drawn last, over its neighbours, with its needle up to the label.
        if span > 0.0 {
            let to_x = |v: f32| bounds.x + ((v - self.min) / span).clamp(0.0, 1.0) * bounds.width;
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
            let pins = self
                .markers
                .iter()
                .filter(|m| !m.active)
                .chain(self.markers.iter().filter(|m| m.active));
            for marker in pins {
                let x0 = to_x(marker.start);
                if marker.end > marker.start {
                    renderer.fill_quad(
                        quad(
                            Rectangle {
                                x: x0,
                                y: bar_y + BAR_HEIGHT + 1.0,
                                width: (to_x(marker.end) - x0).max(2.0),
                                height: BAND_HEIGHT,
                            },
                            0.0,
                        ),
                        marker.color,
                    );
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
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    state.is_dragging = true;
                    if let Some(value) = self.value_from_cursor(bounds, cursor, state.shift) {
                        shell.publish((self.on_seek)(value));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    if let Some(value) = self.value_from_cursor(bounds, cursor, state.shift) {
                        shell.publish((self.on_seek)(value));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.is_dragging =>
            {
                state.is_dragging = false;
                if let Some(on_release) = &self.on_release {
                    shell.publish(on_release.clone());
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

        if state.is_dragging || cursor.is_over(layout.bounds()) {
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
        BarMarker {
            start,
            end: start,
            color: Color::WHITE,
            active: false,
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
}
