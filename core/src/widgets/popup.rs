use std::time::Duration;

use ori::Element;

use crate::{
    Allocation, AvailableSpace, Context, LayoutNode, Parent, Platform, PopupPosition, Size, Widget,
    WidgetMut, native::NativePopup,
};

/// A [`Widget`] that shows a popup relative to an `anchor`.
pub struct PopupWidget<P, T, U>
where
    P: Platform,
    T: Widget<P>,
    U: Widget<P>,
{
    native:   P::Popup,
    anchor:   T,
    contents: Option<U>,

    anchor_allocation:   Option<Allocation>,
    contents_allocation: Option<Allocation>,
}

impl<P, T, U> PopupWidget<P, T, U>
where
    P: Platform,
    T: Widget<P>,
    U: Widget<P>,
{
    /// Create new [`PopupWidget`].
    pub fn new(cx: &mut Context<P>, anchor: T, on_dismiss: impl Fn() + 'static) -> Self {
        let native = P::Popup::build(
            &mut cx.platform,
            anchor.widget_ref(),
            on_dismiss,
        );

        Self {
            native,
            anchor,

            contents: None,
            anchor_allocation: None,
            contents_allocation: None,
        }
    }

    /// Teardown the native widget returning anchor and contents.
    pub fn teardown(self, cx: &mut Context<P>) -> (T, Option<U>) {
        self.native.teardown(&mut cx.platform);
        (self.anchor, self.contents)
    }

    /// Open the popup.
    ///
    /// Should only be called once until [`Self::close`] is called.
    pub fn open(&mut self, cx: &mut Context<P>, contents: U) {
        debug_assert!(self.contents.is_none());

        self.native.open(&mut cx.platform, contents.widget_ref());
        self.contents = Some(contents);
    }

    /// Close the popup.
    ///
    /// Should only be called once be called once after [`Self::open`].
    pub fn close(&mut self, cx: &mut Context<P>) -> Option<U> {
        debug_assert!(self.contents.is_some());

        self.contents_allocation = None;
        self.native.close(&mut cx.platform);
        self.contents.take()
    }

    /// Set the positioning scheme of the popup.
    pub fn set_position(&mut self, cx: &mut Context<P>, position: PopupPosition) {
        self.native.set_position(&mut cx.platform, position);
    }

    /// Set whether the popup is modal.
    pub fn set_modal(&mut self, cx: &mut Context<P>, is_modal: bool) {
        self.native.set_modal(&mut cx.platform, is_modal);
    }

    /// Get a mutable reference to the anchor widget.
    pub fn anchor_mut(&mut self) -> (impl Parent<P>, &mut T) {
        let parent = AnchorParent {
            native: &mut self.native,
            layout: self.anchor.layout_node(),
        };

        (parent, &mut self.anchor)
    }

    /// Get a mutable reference to the contents widget.
    pub fn contents_mut(&mut self) -> Option<(impl Parent<P>, &mut U)> {
        let contents = self.contents.as_mut()?;

        let parent = ContentsParent {
            native: &mut self.native,
            layout: contents.layout_node(),
        };

        Some((parent, contents))
    }
}

impl<P, T, U> Element for PopupWidget<P, T, U>
where
    P: Platform,
    T: Widget<P>,
    U: Widget<P>,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P, T, U> Widget<P> for PopupWidget<P, T, U>
where
    P: Platform,
    T: Widget<P>,
    U: Widget<P>,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.native.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.anchor.layout_node()
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        if let Some(allocation) = cx.layout.get_allocation(self.anchor.layout_node())
            && self.anchor_allocation != Some(allocation)
        {
            self.anchor_allocation = Some(allocation);
            self.native.set_anchor_size(
                &mut cx.platform,
                allocation.size.width,
                allocation.size.height,
            );
        }

        self.anchor.layout(cx);

        if let Some(ref mut contents) = self.contents {
            let space = Size {
                width:  AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            };

            cx.layout.compute_layout(
                &mut cx.platform,
                contents.layout_node(),
                space,
            );

            if let Some(allocation) = cx.layout.get_allocation(contents.layout_node())
                && self.contents_allocation != Some(allocation)
            {
                self.contents_allocation = Some(allocation);

                self.native.set_popup_size(
                    &mut cx.platform,
                    allocation.size.width + allocation.margin.left + allocation.margin.right,
                    allocation.size.height + allocation.margin.top + allocation.margin.bottom,
                );

                self.native.set_content_layout(
                    &mut cx.platform,
                    allocation.x + allocation.margin.left,
                    allocation.y + allocation.margin.top,
                    allocation.size.width,
                    allocation.size.height,
                );
            }

            contents.layout(cx);
        }
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        self.anchor.animate(cx, dt);

        if let Some(ref mut contents) = self.contents {
            contents.animate(cx, dt);
        }
    }
}

struct AnchorParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Popup,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for AnchorParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.native.replace_anchor(&mut cx.platform, widget);
        cx.layout.replace_node(self.layout, layout);
    }
}

struct ContentsParent<'a, P>
where
    P: Platform,
{
    native: &'a mut P::Popup,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for ContentsParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        self.native.open(&mut cx.platform, widget);
        cx.layout.replace_node(self.layout, layout);
    }
}
