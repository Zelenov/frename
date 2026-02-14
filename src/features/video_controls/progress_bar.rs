//! Custom rectangular progress bar widget with click-to-seek and drag support

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Rectangle, Shadow, Size};

const BAR_HEIGHT: f32 = 8.0;
const HIT_HEIGHT: f32 = 24.0;
const TRACK_COLOR: Color = Color::from_rgb(0.25, 0.25, 0.25);
const FILL_COLOR: Color = Color::from_rgb(0.35, 0.65, 1.0);
const BORDER_RADIUS: f32 = 4.0;

/// Internal widget state for tracking drag
#[derive(Default)]
struct State {
    is_dragging: bool,
}

/// A rectangular progress bar that supports click and drag to seek
pub struct ProgressBar<'a, Message> {
    /// Current progress as fraction 0.0 to 1.0
    progress: f32,
    /// Called with fraction 0.0..=1.0 when user clicks or drags
    on_seek: Box<dyn Fn(f32) -> Message + 'a>,
    /// Called when user releases the mouse after seeking
    on_release: Option<Message>,
}

impl<'a, Message> ProgressBar<'a, Message> {
    /// Create a new progress bar
    pub fn new(progress: f32, on_seek: impl Fn(f32) -> Message + 'a) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            on_seek: Box::new(on_seek),
            on_release: None,
        }
    }

    /// Message to emit when the user releases after seeking
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// Calculate seek fraction from cursor position
    fn fraction_from_cursor(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<f32> {
        cursor
            .position()
            .map(|pos| ((pos.x - bounds.x) / bounds.width).clamp(0.0, 1.0))
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
            TRACK_COLOR,
        );

        // Progress fill
        let fill_width = bounds.width * self.progress;
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
                FILL_COLOR,
            );
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
                    if let Some(fraction) = self.fraction_from_cursor(bounds, cursor) {
                        shell.publish((self.on_seek)(fraction));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    if let Some(fraction) = self.fraction_from_cursor(bounds, cursor) {
                        shell.publish((self.on_seek)(fraction));
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
