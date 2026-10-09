use std::{marker::PhantomData, time::Duration};

use ori::{Element, ViewId};

use crate::{Context, LayoutNode, Platform, Size, Widget, WidgetMut};

/// [`Widget`] that provides a callback for layout.
pub struct LayoutWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    contents:  W,
    on_layout: Box<dyn Fn(Size<f32>)>,

    size:   Option<Size<f32>>,
    marker: PhantomData<fn(P)>,
}

impl<P, W> LayoutWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`LayoutWidget`].
    pub fn new(contents: W, on_layout: impl Fn(Size<f32>) + 'static) -> Self {
        Self {
            contents,
            on_layout: Box::new(on_layout),
            size: None,
            marker: PhantomData,
        }
    }

    /// Teardown returning `contents`.
    pub fn teardown(self) -> W {
        self.contents
    }

    /// Get mutable reference to contents.
    pub fn contents(&mut self) -> &mut W {
        &mut self.contents
    }
}

impl<P, W> Element for LayoutWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P, W> Widget<P> for LayoutWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.contents.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.contents.layout_node()
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        if let Some(allocation) = cx.layout.get_allocation(self.layout_node())
            && self.size != Some(allocation.size)
        {
            self.size = Some(allocation.size);
            (self.on_layout)(allocation.size);
        }

        self.contents.layout(cx);
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        self.contents.animate(cx, dt);
    }

    fn set_root(&mut self, cx: &mut Context<P>, root: Option<ViewId>) {
        self.contents.set_root(cx, root);
    }
}
