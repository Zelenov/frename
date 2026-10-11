//! Keyboard focus for controls iced 0.14 cannot focus itself (`docs/design/design-system.md` §11):
//! a ring drawn around a button, checkbox or radio that a window's own state says is focused,
//! and the widget operations such a window needs: focus one of its own text fields (or none)
//! without touching another window's, tell which of its fields has focus, and scroll the
//! focused control into view.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::{Focusable, Outcome, Scrollable};
use iced::advanced::widget::{Id, Operation, Tree, Widget};
use iced::advanced::{self, overlay, Clipboard, Shell};
use iced::widget::scrollable::AbsoluteOffset;
use iced::{
    mouse, Background, Border, Color, Element, Event, Length, Rectangle, Size, Task, Vector,
};

use crate::ui::tokens::{ACCENT_TEXT, FOCUS_RING_GAP, RADIUS_S, RING, SPACE_S};

/// The ring reports its bounds under this id while it is drawn, so the focused control can be
/// scrolled into view. One window shows one ring at a time.
const RING_ID: &str = "focus-ring";

/// `content`, with the focus ring around it when `focused`. The ring is drawn outside the
/// content's bounds, so focusing never moves anything.
pub fn ring<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    focused: bool,
) -> Element<'a, Message> {
    Element::new(FocusRing {
        content: content.into(),
        focused,
        draw: true,
        report: true,
    })
}

/// A control that draws the focus ring as its own edge (a dropdown): nothing is drawn here, but
/// the focused control can still be scrolled into view.
pub fn edge<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    focused: bool,
) -> Element<'a, Message> {
    Element::new(FocusRing {
        content: content.into(),
        focused,
        draw: false,
        report: true,
    })
}

/// [`ring`] for a control that never scrolls (a window's button bar): showing it scrolls nothing.
pub fn fixed_ring<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    focused: bool,
) -> Element<'a, Message> {
    Element::new(FocusRing {
        content: content.into(),
        focused,
        draw: true,
        report: false,
    })
}

struct FocusRing<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    focused: bool,
    /// Draw the ring (the content may draw its own focused edge instead).
    draw: bool,
    /// Report where the focused control is, for [`scroll_into_view`].
    report: bool,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FocusRing<'_, Message, Theme, Renderer>
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
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.focused && self.report {
            operation.container(Some(&Id::new(RING_ID)), ring_bounds(layout.bounds()));
        }
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if self.focused && self.draw {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: ring_bounds(layout.bounds()),
                    border: Border {
                        color: ACCENT_TEXT,
                        width: RING,
                        radius: (RADIUS_S + FOCUS_RING_GAP).into(),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// The ring's outer edge: the gap and the ring's width outside the control.
fn ring_bounds(control: Rectangle) -> Rectangle {
    control.expand(FOCUS_RING_GAP + RING)
}

/// Focus the text field `target` among `fields` (the ids of one window's fields), or none of
/// them. Fields of other windows are left alone, unlike iced's own focus operations, which run
/// over every open window.
pub fn focus_among<T>(target: Option<&'static str>, fields: &'static [&'static str]) -> Task<T>
where
    T: Send + 'static,
{
    struct FocusAmong {
        target: Option<Id>,
        fields: Vec<Id>,
    }

    impl Operation<()> for FocusAmong {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
            operate(self);
        }

        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            let Some(id) = id else { return };
            if self.target.as_ref() == Some(id) {
                state.focus();
            } else if self.fields.contains(id) {
                state.unfocus();
            }
        }
    }

    iced::advanced::widget::operate(FocusAmong {
        target: target.map(Id::new),
        fields: fields.iter().copied().map(Id::new).collect(),
    })
    .discard()
}

/// Which of `fields` (the ids of one window's text fields) has focus, if any: a click into a
/// field moves the window's own focus there.
pub fn focused_among(fields: &'static [&'static str]) -> Task<Option<&'static str>> {
    struct FocusedAmong {
        fields: &'static [&'static str],
        found: Option<&'static str>,
    }

    impl Operation<Option<&'static str>> for FocusedAmong {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<&'static str>>)) {
            operate(self);
        }

        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            let Some(id) = id else { return };
            if state.is_focused() {
                if let Some(field) = self.fields.iter().find(|field| *id == Id::new(field)) {
                    self.found = Some(*field);
                }
            }
        }

        fn finish(&self) -> Outcome<Option<&'static str>> {
            Outcome::Some(self.found)
        }
    }

    iced::advanced::widget::operate(FocusedAmong {
        fields,
        found: None,
    })
}

/// Scroll the scrollable `scrollable` so the focused control is in view: the ring of
/// [`ring`], or the text field `field` when the focus is in one. Nothing moves when it is in
/// view already.
pub fn scroll_into_view<T>(scrollable: &'static str, field: Option<&'static str>) -> Task<T>
where
    T: Send + 'static,
{
    /// Where the scrollable and the focused control are, once the operation has seen both.
    #[derive(Debug, Clone, Copy, Default)]
    struct Found {
        viewport: Option<Rectangle>,
        offset: f32,
        control: Option<Rectangle>,
    }

    struct Find {
        scrollable: Id,
        target: Id,
        found: Found,
    }

    impl Operation<Option<f32>> for Find {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<f32>>)) {
            operate(self);
        }

        fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
            if id == Some(&self.target) {
                self.found.control = Some(bounds);
            }
        }

        fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, _state: &mut dyn Focusable) {
            if id == Some(&self.target) {
                self.found.control = Some(bounds);
            }
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            _content_bounds: Rectangle,
            translation: Vector,
            _state: &mut dyn Scrollable,
        ) {
            if id == Some(&self.scrollable) {
                self.found.viewport = Some(bounds);
                self.found.offset = translation.y;
            }
        }

        fn finish(&self) -> Outcome<Option<f32>> {
            let Found {
                viewport: Some(viewport),
                offset,
                control: Some(control),
            } = self.found
            else {
                return Outcome::Some(None);
            };
            Outcome::Some(offset_showing(viewport, offset, control))
        }
    }

    let scrollable_id = Id::new(scrollable);
    iced::advanced::widget::operate(Find {
        scrollable: scrollable_id.clone(),
        target: Id::new(field.unwrap_or(RING_ID)),
        found: Found::default(),
    })
    .then(move |offset| match offset {
        Some(y) => iced::widget::operation::scroll_to(
            scrollable_id.clone(),
            AbsoluteOffset {
                x: None,
                y: Some(y),
            },
        ),
        None => Task::none(),
    })
}

/// The scroll offset that shows `control` (in the content's unscrolled coordinates) inside
/// `viewport`, scrolled by `offset` now, with a little room around it; `None` when it shows
/// already.
fn offset_showing(viewport: Rectangle, offset: f32, control: Rectangle) -> Option<f32> {
    let top = control.y - viewport.y - SPACE_S;
    let bottom = control.y + control.height - viewport.y + SPACE_S;
    if top < offset {
        Some(top.max(0.0))
    } else if bottom > offset + viewport.height {
        Some(bottom - viewport.height)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Point, Size};

    fn rect(y: f32, height: f32) -> Rectangle {
        Rectangle::new(Point::new(0.0, y), Size::new(100.0, height))
    }

    #[test]
    fn a_control_in_view_does_not_scroll() {
        assert_eq!(
            offset_showing(rect(50.0, 400.0), 0.0, rect(100.0, 20.0)),
            None
        );
        assert_eq!(
            offset_showing(rect(50.0, 400.0), 300.0, rect(400.0, 20.0)),
            None
        );
    }

    #[test]
    fn a_control_below_scrolls_up_to_the_bottom_edge_and_one_above_to_the_top() {
        let viewport = rect(50.0, 400.0);
        // Content y 600..620 is at 550..570 below the viewport's top: scroll so its bottom (+8)
        // meets the viewport's bottom.
        assert_eq!(
            offset_showing(viewport, 0.0, rect(600.0, 20.0)),
            Some(570.0 + SPACE_S - 400.0)
        );
        assert_eq!(
            offset_showing(viewport, 300.0, rect(100.0, 20.0)),
            Some(50.0 - SPACE_S)
        );
        assert_eq!(
            offset_showing(viewport, 300.0, rect(52.0, 20.0)),
            Some(0.0),
            "never above the top"
        );
    }
}
