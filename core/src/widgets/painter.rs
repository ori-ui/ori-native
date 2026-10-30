use std::time::Duration;

use ori::{Element, ViewId};

use crate::{
    Canvas, Context, LayoutNode, LayoutStyle, Platform, Widget, WidgetMut, native::NativePainter,
};

/// A [`Widget`] that draws a [`Canvas`].
pub struct PainterWidget<P>
where
    P: Platform,
{
    native: P::Painter,
    layout: LayoutNode,
}

impl<P> PainterWidget<P>
where
    P: Platform,
{
    /// Create new [`PainterWidget`].
    pub fn new(cx: &mut Context<P>) -> Self {
        Self {
            native: P::Painter::build(&mut cx.platform),
            layout: cx.layout.add_node(&[]),
        }
    }

    /// Teardown the widget.
    pub fn teardown(self, cx: &mut Context<P>) {
        self.native.teardown(&mut cx.platform);
        cx.layout.remove_node(self.layout);
    }

    /// Set [`LayoutStyle`].
    pub fn set_layout(&mut self, cx: &mut Context<P>, layout: LayoutStyle) {
        cx.layout.set_layout(self.layout, layout);
    }

    /// Set the [`Canvas`] to draw.
    pub fn set_canvas(&mut self, cx: &mut Context<P>, canvas: Canvas) {
        self.native.set_canvas(&mut cx.platform, canvas);
    }
}

impl<P> Widget<P> for PainterWidget<P>
where
    P: Platform,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.native.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.layout
    }

    fn layout(&mut self, _cx: &mut Context<P>) {}

    fn animate(&mut self, _cx: &mut Context<P>, _dt: Duration) {}

    fn set_root(&mut self, _cx: &mut Context<P>, _root: Option<ViewId>) {}
}

impl<P> Element for PainterWidget<P>
where
    P: Platform,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}
