use std::time::Duration;

use ori::Element;

use crate::{
    Allocation, Context, Direction, FlexStyle, LayoutNode, LayoutStyle, Overflow, Parent, Platform,
    Size, Widget, WidgetMut, native::NativeScroll,
};

/// A [`Widget`] that contains scrollable contents.
pub struct ScrollWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    native:   P::Scroll,
    layout:   LayoutNode,
    contents: W,

    scroll_allocation:  Option<Allocation>,
    content_allocation: Option<Allocation>,
}

impl<P, W> ScrollWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`ScrollWidget`].
    pub fn new(cx: &mut Context<P>, contents: W, on_scroll: impl Fn(f32, f32) + 'static) -> Self {
        let native = P::Scroll::build(
            &mut cx.platform,
            contents.widget_ref(),
            on_scroll,
        );

        let layout = cx.layout.add_node(&[contents.layout_node()]);

        Self {
            native,
            layout,
            contents,

            scroll_allocation: None,
            content_allocation: None,
        }
    }

    /// Teardown returning contents.
    pub fn teardown(self, cx: &mut Context<P>) -> W {
        self.native.teardown(&mut cx.platform);
        cx.layout.remove_node(self.layout);
        self.contents
    }

    /// Get contents mutably.
    pub fn contents_mut(&mut self) -> (impl Parent<P>, &mut W) {
        let parent = ScrollParent {
            native: &mut self.native,
            layout: self.layout,
        };

        (parent, &mut self.contents)
    }

    /// Set the [`LayoutStyle`].
    pub fn set_layout(&mut self, cx: &mut Context<P>, layout: LayoutStyle) {
        cx.layout.set_layout(self.layout, layout);
    }

    /// Set the [`Direction`] of scroll.
    pub fn set_direction(&mut self, cx: &mut Context<P>, direction: Direction) {
        self.native.set_direction(&mut cx.platform, direction);

        let overflow = match direction {
            Direction::Horizontal => Size {
                width:  Overflow::Hidden,
                height: Overflow::Visible,
            },

            Direction::Vertical => Size {
                width:  Overflow::Visible,
                height: Overflow::Hidden,
            },
        };

        cx.layout.set_overflow(self.layout, overflow);
        cx.layout.set_flex(
            self.layout,
            FlexStyle {
                direction,
                ..Default::default()
            },
        );
    }
}

impl<P, W> Element for ScrollWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P, W> Widget<P> for ScrollWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.native.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.layout
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        if let Some(allocation) = cx.layout.get_allocation(self.layout)
            && self.scroll_allocation != Some(allocation)
        {
            self.scroll_allocation = Some(allocation);
            self.native.set_content_size(
                &mut cx.platform,
                allocation.content_size.width,
                allocation.content_size.height,
            );
        }

        if let Some(allocation) = cx.layout.get_allocation(self.contents.layout_node())
            && self.content_allocation != Some(allocation)
        {
            self.content_allocation = Some(allocation);
            self.native.set_content_layout(
                &mut cx.platform,
                allocation.x,
                allocation.y,
                allocation.size.width,
                allocation.size.height,
            );
        }

        self.contents.layout(cx);
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        self.contents.animate(cx, dt);
    }
}

struct ScrollParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Scroll,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for ScrollParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.native.replace_contents(&mut cx.platform, widget);
        cx.layout.replace_child(self.layout, 0, layout);
    }
}
