//! Widget that reports its layout bounds when the cursor moves over it.
//! Used for mapping cursor position to drop index (e.g. file name panel drag).

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size};

#[derive(Default)]
struct State {
    last_bounds: Option<Rectangle>,
}

/// Fill-sized widget that reports its layout bounds when the cursor is over it.
pub struct BoundsReporter<'a, Message> {
    on_bounds: Box<dyn Fn(Rectangle) -> Message + 'a>,
}

impl<'a, Message> BoundsReporter<'a, Message> {
    pub fn new(on_bounds: impl Fn(Rectangle) -> Message + 'a) -> Self {
        Self {
            on_bounds: Box::new(on_bounds),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for BoundsReporter<'a, Message>
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
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        _event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if !cursor.is_over(bounds) {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        let changed = state
            .last_bounds
            .map(|b| {
                b.x != bounds.x || b.y != bounds.y || b.width != bounds.width || b.height != bounds.height
            })
            .unwrap_or(true);
        if changed {
            state.last_bounds = Some(bounds);
            shell.publish((self.on_bounds)(bounds));
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<BoundsReporter<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
{
    fn from(reporter: BoundsReporter<'a, Message>) -> Self {
        Self::new(reporter)
    }
}
