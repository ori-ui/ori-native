use std::time::Duration;

use ori::{Element, ViewId};

use crate::{
    Allocation, Context, LayoutNode, Parent, Platform, Widget, WidgetMut, native::NativeMeasure,
};

/// A [`Widget`] for measuring contents relative to the window.
pub struct MeasureWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    native:   P::Measure,
    contents: W,

    allocation: Option<Allocation>,
}

impl<P, W> MeasureWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`MeasureWidget`].
    pub fn new(
        cx: &mut Context<P>,
        contents: W,
        on_position_changed: impl Fn(f32, f32) + 'static,
    ) -> Self {
        let native = P::Measure::build(
            &mut cx.platform,
            contents.widget_ref(),
            on_position_changed,
        );

        Self {
            native,
            contents,
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
        let parent = MeasureParent {
            native: &mut self.native,
            layout: self.contents.layout_node(),
        };

        (parent, &mut self.contents)
    }
}

impl<P, W> Element for MeasureWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P, W> Widget<P> for MeasureWidget<P, W>
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
            self.native.set_content_size(
                &mut cx.platform,
                allocation.size.width,
                allocation.size.height,
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

struct MeasureParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Measure,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for MeasureParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.native.replace_contents(&mut cx.platform, widget);
        cx.layout.replace_node(self.layout, layout);
    }
}
