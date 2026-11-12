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
                    let at = cursor.position().and_then(|pos| {
                        drag_position(pos.x, self.min_left, self.min_right, viewport.width)
                    });
                    if let Some(x) = at {
                        shell.publish((self.on_drag)(x));
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

/// Where a drag to `x` puts the splitter in a window `width` wide: clamped so both sides keep
/// their minimum widths. `None` when the window is narrower than both minimums together (the OS
/// ignored the window's minimum size): the splitter stays where it is instead of panicking in
/// `f32::clamp` (#395).
fn drag_position(x: f32, min_left: f32, min_right: f32, width: f32) -> Option<f32> {
    let max_left = width - SPLITTER_HIT - min_right;
    (min_left <= max_left).then(|| x.clamp(min_left, max_left))
}

#[cfg(test)]
mod tests {
    use super::drag_position;
    use crate::ui::tokens::SPLITTER_HIT;

    #[test]
    fn a_drag_is_clamped_to_both_minimums() {
        let width = 1000.0;
        assert_eq!(
            drag_position(500.0, 320.0, 532.0, width),
            Some(1000.0 - SPLITTER_HIT - 532.0)
        );
        assert_eq!(drag_position(100.0, 320.0, 300.0, width), Some(320.0));
        assert_eq!(drag_position(400.0, 320.0, 300.0, width), Some(400.0));
    }

    /// Issue #395: a window narrower than both minimums together does not panic; the splitter
    /// stays where it is.
    #[test]
    fn a_window_too_narrow_for_both_minimums_moves_nothing() {
        assert_eq!(drag_position(400.0, 320.0, 532.0, 800.0), None);
    }
}
