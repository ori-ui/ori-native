use std::time::Duration;

use ori::{Element, ViewId};

use crate::{
    Allocation, Context, Key, LayoutNode, Modifiers, Parent, Platform, PressableEvent, Widget,
    native::NativePressable, widget::WidgetMut,
};

/// A [`Widget`] that handles input.
pub struct PressableWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    native:   P::Pressable,
    contents: W,

    allocation: Option<Allocation>,
}

impl<P, W> PressableWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    /// Create new [`PressableWidget`].
    pub fn new(
        cx: &mut Context<P>,
        contents: W,
        on_event: impl Fn(PressableEvent) + 'static,
    ) -> Self {
        let pressable = P::Pressable::build(
            &mut cx.platform,
            contents.widget_ref(),
            on_event,
        );

        Self {
            native: pressable,
            contents,

            allocation: None,
        }
    }

    /// Teardown returning contents.
    pub fn teardown(self, cx: &mut Context<P>) -> W {
        self.native.teardown(&mut cx.platform);
        self.contents
    }

    /// Get contents mutably.
    pub fn contents_mut(&mut self) -> (impl Parent<P>, &mut W) {
        let parent = PressableParent {
            native: &mut self.native,
            layout: self.contents.layout_node(),
        };

        (parent, &mut self.contents)
    }

    /// Set whether the widget is scrollable.
    pub fn set_scrollable(&mut self, cx: &mut Context<P>, scrollable: bool) {
        self.native.set_scrollable(&mut cx.platform, scrollable);
    }

    /// Set whether the widget is focusable.
    pub fn set_focusable(&mut self, cx: &mut Context<P>, focusable: bool) {
        self.native.set_focusable(&mut cx.platform, focusable);
    }

    /// Set the `on_key` callback.
    pub fn set_on_key(
        &mut self,
        cx: &mut Context<P>,
        on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
        self.native.set_on_key(&mut cx.platform, on_key);
    }

    /// Request that the widget become focused.
    pub fn request_focus(&mut self, cx: &mut Context<P>) {
        self.native.request_focus(&mut cx.platform);
    }
}

impl<P, W> Widget<P> for PressableWidget<P, W>
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

impl<P, W> Element for PressableWidget<P, W>
where
    P: Platform,
    W: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

struct PressableParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Pressable,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for PressableParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.native.replace_contents(&mut cx.platform, widget);
        cx.layout.replace_node(self.layout, layout);
    }
}
