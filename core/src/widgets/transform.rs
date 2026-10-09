use std::time::Duration;

use ori::{Element, ViewId};

use crate::{
    Affine, Allocation, Context, LayoutNode, Parent, Platform, Widget, native::NativeTransform,
    widget::WidgetMut,
};

/// A [`Widget`] that transforms its contents.
pub struct TransformWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    native:   P::Transform,
    contents: W,

    transform:  Affine,
    allocation: Option<Allocation>,
}

impl<P, W> TransformWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`TransformWidget`].
    pub fn new(cx: &mut Context<P>, contents: W) -> Self {
        let native = P::Transform::build(&mut cx.platform, contents.widget_ref());

        Self {
            native,
            contents,
            transform: Affine::new(),
            allocation: None,
        }
    }

    /// Teardown returning contents.
    pub fn teardown(self, cx: &mut Context<P>) -> W {
        self.native.teardown(&mut cx.platform);
        self.contents
    }

    /// Get the contents mutably.
    pub fn contents_mut(&mut self) -> (impl Parent<P>, &mut W) {
        let parent = TransformParent {
            native: &mut self.native,
            layout: self.contents.layout_node(),
        };

        (parent, &mut self.contents)
    }

    /// Set the `transform`.
    pub fn set_transform(&mut self, cx: &mut Context<P>, transform: Affine) {
        self.transform = transform;

        if let Some(allocation) = self.allocation {
            self.native.set_content_transform(
                &mut cx.platform,
                allocation.size.width,
                allocation.size.height,
                self.transform,
            );
        }
    }
}

impl<P, W> Element for TransformWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P, W> Widget<P> for TransformWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.native.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.contents.layout_node()
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        if let Some(allocation) = cx.layout.get_allocation(self.layout_node())
            && self.allocation != Some(allocation)
        {
            self.allocation = Some(allocation);
            self.native.set_content_transform(
                &mut cx.platform,
                allocation.size.width,
                allocation.size.height,
                self.transform,
            );
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

struct TransformParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Transform,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for TransformParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.native.replace_contents(&mut cx.platform, widget);
        cx.layout.replace_node(self.layout, layout);
    }
}
