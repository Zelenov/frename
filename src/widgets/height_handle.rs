//! Draggable horizontal bar that resizes the panel below it: dragging up makes the panel taller.
//! A 2-px line in an 8-px hit area (design system §13.5.7), brighter under the pointer.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Clipboard, Shell};
use iced::mouse;
use iced::{Border, Element, Event, Length, Point, Rectangle, Shadow, Size};

use crate::ui::tokens::{BORDER_CONTROL, BORDER_SUBTLE, HANDLE_HIT, HANDLE_LINE};

/// Internal widget state for tracking drag.
#[derive(Default)]
struct State {
    /// Cursor position when the drag last reported, while dragging.
    last: Option<Point>,
}

/// A draggable horizontal bar above a panel whose height it changes.
///
/// Reports how many pixels the cursor moved up since the last report while dragging
/// (negative when it moved down), so the parent can grow the panel below by that much.
pub struct HeightHandle<'a, Message> {
    on_drag: Box<dyn Fn(f32) -> Message + 'a>,
}

impl<'a, Message> HeightHandle<'a, Message> {
    /// Create a handle; `on_drag` gets the pixels to grow the panel below by.
    pub fn new(on_drag: impl Fn(f32) -> Message + 'a) -> Self {
        Self {
            on_drag: Box::new(on_drag),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for HeightHandle<'a, Message>
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
        Size::new(Length::Fill, Length::Fixed(HANDLE_HIT))
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(HANDLE_HIT), Size::ZERO))
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

        let is_active = state.last.is_some() || cursor.is_over(bounds);
        let color = if is_active {
            BORDER_CONTROL
        } else {
            BORDER_SUBTLE
        };

        // A thin centered bar within the hit area.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + (bounds.height - HANDLE_LINE) / 2.0,
                    width: bounds.width,
                    height: HANDLE_LINE,
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
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_over(layout.bounds()) {
                    state.last = Some(position);
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(last) = state.last {
                    let grow = last.y - position.y;
                    if grow != 0.0 {
                        state.last = Some(*position);
                        shell.publish((self.on_drag)(grow));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.last = None;
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

        if state.last.is_some() || cursor.is_over(layout.bounds()) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<HeightHandle<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
{
    fn from(handle: HeightHandle<'a, Message>) -> Self {
        Self::new(handle)
    }
}
