//! Layers over the window (`docs/design/design-system.md` §13.3.2). Overlays of one view are
//! gathered into one group and drawn into **one** renderer layer, where iced_wgpu draws by
//! primitive type (quads, then text), not in draw order: a tooltip's box would cover another
//! overlay's box while that one's text still came out on top. An overlay that has to be above
//! another one is therefore drawn in a layer of its own and ordered by its [`Z`].
//!
//! Bottom to top: the base (panels, the timeline, the picture) and the stack layers (side list,
//! caption, notices, menus, the fullscreen video) are not overlays; then [`Z::Anchored`] (the
//! marker label over the timeline), [`Z::Tooltip`], [`Z::DragPreview`]. A new overlay picks the
//! level it belongs to.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{self, overlay, Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size, Vector};

/// The level of an overlay among the overlays; a higher one is drawn over a lower one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Z {
    /// Attached to a point of a widget, over the picture: the marker label.
    Anchored,
    /// A tooltip: over everything else that is open.
    Tooltip,
    /// What follows the pointer while something is dragged.
    DragPreview,
}

impl Z {
    /// The overlay `index` of this level (iced's default is 1.0, which no level uses).
    pub fn index(self) -> f32 {
        match self {
            Z::Anchored => 1.5,
            Z::Tooltip => 2.0,
            Z::DragPreview => 3.0,
        }
    }
}

/// `content` with the overlays it opens at level `z`, each in a layer of its own.
pub fn layered<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    z: Z,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + 'a,
{
    Element::new(Layered {
        content: content.into(),
        z,
    })
}

struct Layered<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    z: Z,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Layered<'_, Message, Theme, Renderer>
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
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let z = self.z;
        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
            .map(|inner| overlay::Element::new(Box::new(Layer { inner, z })))
    }
}

/// How far a layer reaches past the bounds of its overlay.
const LAYER_MARGIN: f32 = 16.0;

/// An overlay drawn in a renderer layer of its own, at the index of its level.
pub struct Layer<'a, Message, Theme, Renderer> {
    inner: overlay::Element<'a, Message, Theme, Renderer>,
    z: Z,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Layer<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.inner.as_overlay_mut().layout(renderer, bounds)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        // The layer covers what the overlay covers (a tooltip's group: the window), a little
        // wider for shadows. Not `Rectangle::INFINITE`: the text of a tooltip is clipped by an
        // infinite viewport, which no layer may be wider than.
        renderer.with_layer(layout.bounds().expand(LAYER_MARGIN), |renderer| {
            self.inner
                .as_overlay()
                .draw(renderer, theme, style, layout, cursor);
        });
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.inner
            .as_overlay_mut()
            .operate(layout, renderer, operation);
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
        self.inner
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, clipboard, shell);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.inner
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.inner.as_overlay_mut().overlay(layout, renderer)
    }

    fn index(&self) -> f32 {
        self.z.index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_levels_are_ordered_bottom_to_top() {
        assert!(Z::Anchored.index() < Z::Tooltip.index());
        assert!(Z::Tooltip.index() < Z::DragPreview.index());
        // iced's default index is 1.0: every level is above an overlay that sets none.
        assert!(Z::Anchored.index() > 1.0);
    }

    struct Plain;

    impl overlay::Overlay<(), (), ()> for Plain {
        fn layout(&mut self, _renderer: &(), _bounds: Size) -> layout::Node {
            layout::Node::new(Size::ZERO)
        }

        fn draw(
            &self,
            _renderer: &mut (),
            _theme: &(),
            _style: &renderer::Style,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
        ) {
        }
    }

    /// A tooltip's overlay is above the marker label's, whatever order they were gathered in.
    #[test]
    fn a_layered_overlay_takes_the_index_of_its_level() {
        let plain = || overlay::Element::new(Box::new(Plain));
        let tooltip = Layer {
            inner: plain(),
            z: Z::Tooltip,
        };
        let label = Layer {
            inner: plain(),
            z: Z::Anchored,
        };
        let unset = Plain;
        assert_eq!(overlay::Overlay::<(), (), ()>::index(&tooltip), 2.0);
        assert!(
            overlay::Overlay::<(), (), ()>::index(&label)
                < overlay::Overlay::<(), (), ()>::index(&tooltip)
        );
        assert!(
            overlay::Overlay::<(), (), ()>::index(&unset)
                < overlay::Overlay::<(), (), ()>::index(&label)
        );
    }
}
