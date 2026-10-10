//! Widget that gives its content a gap below it that the content treats as its own: the pointer
//! over the gap is, for the content, on its lower edge. A navigation item keeps its highlight, its
//! hand cursor, its click and its tooltip across the gap that keeps two items apart, so moving
//! down a list never passes a frame where no item is under the pointer.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{self, overlay, Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// `content` followed by `gap` pixels that belong to it.
pub struct GapSlot<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    gap: f32,
}

impl<'a, Message, Theme, Renderer> GapSlot<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>, gap: f32) -> Self {
        Self {
            content: content.into(),
            gap,
        }
    }
}

/// The cursor the content sees: over the slot but below the content (in the gap) it is on the
/// content's last pixel row; anywhere else it is what it is.
fn content_cursor(slot: Rectangle, content: Rectangle, cursor: mouse::Cursor) -> mouse::Cursor {
    match cursor.position_over(slot) {
        Some(at) if !content.contains(at) => mouse::Cursor::Available(Point::new(
            at.x,
            (content.y + content.height - 1.0).max(content.y),
        )),
        _ => cursor,
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for GapSlot<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.content.as_widget().size().width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = content.size();
        layout::Node::with_children(Size::new(size.width, size.height + self.gap), vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if let Some(content) = layout.children().next() {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                content,
                renderer,
                operation,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let Some(content) = layout.children().next() else {
            return;
        };
        let cursor = content_cursor(layout.bounds(), content.bounds(), cursor);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(content) = layout.children().next() else {
            return mouse::Interaction::None;
        };
        let cursor = content_cursor(layout.bounds(), content.bounds(), cursor);
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(content) = layout.children().next() else {
            return;
        };
        let cursor = content_cursor(layout.bounds(), content.bounds(), cursor);
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let content = layout.children().next()?;
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            content,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<GapSlot<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
{
    fn from(slot: GapSlot<'a, Message, Theme, Renderer>) -> Self {
        Self::new(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLOT: Rectangle = Rectangle {
        x: 10.0,
        y: 100.0,
        width: 200.0,
        height: 34.0,
    };
    const CONTENT: Rectangle = Rectangle {
        x: 10.0,
        y: 100.0,
        width: 200.0,
        height: 32.0,
    };

    fn at(x: f32, y: f32) -> mouse::Cursor {
        mouse::Cursor::Available(Point::new(x, y))
    }

    #[test]
    fn the_gap_is_the_contents_last_row() {
        let seen = content_cursor(SLOT, CONTENT, at(50.0, 132.5));
        assert_eq!(seen.position_over(CONTENT), Some(Point::new(50.0, 131.0)));
        let seen = content_cursor(SLOT, CONTENT, at(50.0, 133.9));
        assert_eq!(seen.position_over(CONTENT), Some(Point::new(50.0, 131.0)));
    }

    #[test]
    fn over_the_content_the_cursor_is_unchanged() {
        assert_eq!(
            content_cursor(SLOT, CONTENT, at(50.0, 110.0)),
            at(50.0, 110.0)
        );
    }

    #[test]
    fn outside_the_slot_the_cursor_is_unchanged() {
        assert_eq!(
            content_cursor(SLOT, CONTENT, at(50.0, 134.0)),
            at(50.0, 134.0)
        );
        assert_eq!(
            content_cursor(SLOT, CONTENT, at(5.0, 133.0)),
            at(5.0, 133.0)
        );
        assert_eq!(
            content_cursor(SLOT, CONTENT, mouse::Cursor::Unavailable),
            mouse::Cursor::Unavailable
        );
    }
}
