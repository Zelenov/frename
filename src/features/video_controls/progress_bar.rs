//! Custom rectangular progress bar widget with click-to-seek and drag support

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Rectangle, Shadow, Size};

use crate::theme;

const BAR_HEIGHT: f32 = 8.0;
const HIT_HEIGHT: f32 = 24.0;
const BORDER_RADIUS: f32 = 4.0;

/// Internal widget state for tracking drag
#[derive(Default)]
struct State {
    is_dragging: bool,
}

/// A rectangular progress bar that supports click and drag to seek.
/// Works with absolute values in a range, like `Slider`.
pub struct ProgressBar<'a, Message> {
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
    /// Marker positions (in the same unit as min/max) to draw as ticks above the bar.
    markers: Vec<f32>,
}

impl<'a, Message> ProgressBar<'a, Message> {
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

    /// Set screenshot marker positions (in the same unit as the range).
    pub fn markers(mut self, positions: impl IntoIterator<Item = f32>) -> Self {
        self.markers = positions.into_iter().collect();
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

    /// Convert cursor position to a value in min..=max
    fn value_from_cursor(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<f32> {
        cursor.position().map(|pos| {
            let fraction = ((pos.x - bounds.x) / bounds.width).clamp(0.0, 1.0);
            self.min + fraction * (self.max - self.min)
        })
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ProgressBar<'a, Message>
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

        // Segment highlight + screenshot markers.
        // Positions are clamped to [min, max] so we never draw outside the bar.
        // Rules:
        //   only start OR only end  → single vertical marker line
        //   start < end             → filled rectangle (no rounded corners) between them
        //   start >= end            → two separate marker lines, no fill
        let span = self.max - self.min;

        // Screenshot markers: small vertical tick spanning the bar height.
        if span > 0.0 && !self.markers.is_empty() {
            let to_x = |v: f32| bounds.x + ((v - self.min) / span).clamp(0.0, 1.0) * bounds.width;
            for &pos in &self.markers {
                let x = to_x(pos);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle { x: x - 1.0, y: bar_y, width: 2.0, height: BAR_HEIGHT },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    theme::SCREENSHOT_MARKER,
                );
            }
        }
        if span > 0.0 {
            // Convert seconds to an x-coordinate, clamped to the bar's pixel range.
            let to_x = |secs: f32| {
                bounds.x + ((secs - self.min) / span).clamp(0.0, 1.0) * bounds.width
            };
            let draw_marker = |renderer: &mut Renderer, x: f32| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle { x: x - 1.0, y: bar_y, width: 2.0, height: BAR_HEIGHT },
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
                            bounds: Rectangle { x: x0, y: bar_y, width: w, height: BAR_HEIGHT },
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    state.is_dragging = true;
                    if let Some(value) = self.value_from_cursor(bounds, cursor) {
                        shell.publish((self.on_seek)(value));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    if let Some(value) = self.value_from_cursor(bounds, cursor) {
                        shell.publish((self.on_seek)(value));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    if let Some(on_release) = &self.on_release {
                        shell.publish(on_release.clone());
                    }
                }
            }
            _ => {}
        }
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

impl<'a, Message, Theme, Renderer> From<ProgressBar<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
{
    fn from(bar: ProgressBar<'a, Message>) -> Self {
        Self::new(bar)
    }
}
