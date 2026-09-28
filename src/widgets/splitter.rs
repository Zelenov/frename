//! Draggable vertical splitter between two side-by-side panels (`docs/design/design-system.md`
//! §8.18, §13.1): a 1-px line in the middle of its hit area, brighter while hovered or dragged.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Clipboard, Shell};
use iced::mouse;
use iced::{Border, Element, Event, Length, Rectangle, Shadow, Size};

use crate::ui::tokens::{BORDER_CONTROL, BORDER_SUBTLE, LINE, SPLITTER_HIT};

/// Internal widget state for tracking drag.
#[derive(Default)]
struct State {
    is_dragging: bool,
}

/// A draggable vertical splitter bar.
///
/// Reports the cursor's absolute X position (clamped by min constraints)
/// while dragging so the parent can set the left panel width directly.
pub struct Splitter<'a, Message> {
    /// Callback that receives the clamped cursor X during drag.
    on_drag: Box<dyn Fn(f32) -> Message + 'a>,
    /// Minimum width of the left panel in pixels.
    min_left: f32,
    /// Minimum width of the right panel in pixels.
    min_right: f32,
}

impl<'a, Message> Splitter<'a, Message> {
    /// Create a new splitter.
    ///
    /// `on_drag` is called with the cursor's X position (clamped to
    /// respect minimum panel widths) while the user drags the handle.
    pub fn new(on_drag: impl Fn(f32) -> Message + 'a) -> Self {
        Self {
            on_drag: Box::new(on_drag),
            min_left: 0.0,
            min_right: 0.0,
        }
    }

    /// Set the minimum width of the left panel.
    pub fn min_left(mut self, width: f32) -> Self {
        self.min_left = width;
        self
    }

    /// Set the minimum width of the right panel.
    pub fn min_right(mut self, width: f32) -> Self {
        self.min_right = width;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Splitter<'a, Message>
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
        Size::new(Length::Fixed(SPLITTER_HIT), Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fixed(SPLITTER_HIT), Length::Fill, Size::ZERO))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        let is_active = state.is_dragging || cursor.is_over(bounds);
        let color = if is_active {
            BORDER_CONTROL
        } else {
            BORDER_SUBTLE
        };

        // The line in the middle of the hit area.
        let bar_x = bounds.x + (bounds.width - LINE) / 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bar_x,
                    y: bounds.y,
                    width: LINE,
                    height: bounds.height,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            color,
        );
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
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    state.is_dragging = true;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    if let Some(pos) = cursor.position() {
                        // Clamp so both panels respect their minimum widths
                        let max_left = viewport.width - SPLITTER_HIT - self.min_right;
                        let clamped = pos.x.clamp(self.min_left, max_left);
                        shell.publish((self.on_drag)(clamped));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
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
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Splitter<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
{
    fn from(splitter: Splitter<'a, Message>) -> Self {
        Self::new(splitter)
    }
}
