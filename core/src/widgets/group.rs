use std::time::Duration;

use ori::{Element, Elements, Mut};

use crate::{
    Allocation, BorderStyle, BoxedWidget, Color, Context, Corners, FlexStyle, LayoutNode,
    LayoutStyle, Length, Overflow, Parent, Platform, Shadow, Sides, Size, Widget, WidgetMut,
    native::NativeGroup,
};

/// A utility wrapper for a [`NativeGroup`], maintaining an [`Elements`].
pub struct GroupWidget<P>
where
    P: Platform,
{
    native:       P::Group,
    layout:       LayoutNode,
    children:     Vec<Child<P>>,
    border_width: Sides<f32>,
}

struct Child<P> {
    element:    BoxedWidget<P>,
    allocation: Option<Allocation>,
}

impl<P> GroupWidget<P>
where
    P: Platform,
{
    /// Create new [`GroupWidget`].
    pub fn new(cx: &mut Context<P>) -> Self {
        Self {
            native:       P::Group::build(&mut cx.platform),
            layout:       cx.layout.add_node(&[]),
            children:     Vec::new(),
            border_width: Sides::all(0.0),
        }
    }

    /// Teardown the wrapped [`NativeGroup`].
    pub fn teardown(self, cx: &mut Context<P>) {
        self.native.teardown(&mut cx.platform);
        cx.layout.remove_node(self.layout);
    }

    /// Get the [`Elements`].
    pub fn elements(&mut self) -> impl Elements<Context<P>, BoxedWidget<P>> {
        GroupElements {
            parent:   GroupParent {
                index:  0,
                native: &mut self.native,
                layout: self.layout,
            },
            children: &mut self.children,
        }
    }

    /// Set the background `color`.
    pub fn set_background(&mut self, cx: &mut Context<P>, color: Color) {
        self.native.set_background_color(&mut cx.platform, color);
    }

    /// Set the corner radii.
    pub fn set_corners(&mut self, cx: &mut Context<P>, corners: Corners<f32>) {
        self.native.set_corners(&mut cx.platform, corners);
    }

    /// Set the `overflow` mode.
    pub fn set_overflow(&mut self, cx: &mut Context<P>, overflow: Overflow) {
        self.native.set_overflow(&mut cx.platform, overflow);
        cx.layout.set_overflow(self.layout, Size::all(overflow));
    }

    /// Set the `shadow`.
    pub fn set_shadow(&mut self, cx: &mut Context<P>, shadow: Shadow) {
        self.native.set_shadow(&mut cx.platform, shadow);
    }

    /// Set whether hardware layer is enabled.
    pub fn set_hardware_layer(&mut self, cx: &mut Context<P>, enabled: bool) {
        self.native.set_hardware_layer(&mut cx.platform, enabled);
    }

    /// Set the [`LayoutStyle`].
    pub fn set_layout(&mut self, cx: &mut Context<P>, layout: LayoutStyle) {
        cx.layout.set_layout(self.layout, layout);
    }

    /// Set the [`BorderStyle`].
    pub fn set_border(&mut self, cx: &mut Context<P>, border: BorderStyle) {
        self.native.set_border_color(&mut cx.platform, border.color);
        cx.layout.set_border(self.layout, border);
    }

    /// Set the `padding`.
    pub fn set_padding(&mut self, cx: &mut Context<P>, padding: Sides<Length>) {
        cx.layout.set_padding(self.layout, padding);
    }

    /// Set the [`FlexStyle`].
    pub fn set_flex(&mut self, cx: &mut Context<P>, flex: FlexStyle) {
        cx.layout.set_flex(self.layout, flex);
    }
}

impl<P> Widget<P> for GroupWidget<P>
where
    P: Platform,
{
    fn widget_ref(&self) -> P::WidgetRef {
        self.native.widget_ref()
    }

    fn layout_node(&self) -> LayoutNode {
        self.layout
    }

    fn layout(&mut self, cx: &mut Context<P>) {
        if let Some(allocation) = cx.layout.get_allocation(self.layout)
            && self.border_width != allocation.border
        {
            self.border_width = allocation.border;
            (self.native).set_border_width(&mut cx.platform, allocation.border);
        }

        for (index, child) in self.children.iter_mut().enumerate() {
            if let Some(allocation) = cx.layout.get_allocation(child.element.layout_node())
                && child.allocation != Some(allocation)
            {
                child.allocation = Some(allocation);
                self.native.set_child_layout(
                    &mut cx.platform,
                    index,
                    allocation.x,
                    allocation.y,
                    allocation.size.width,
                    allocation.size.height,
                );
            }

            child.element.layout(cx);
        }
    }

    fn animate(&mut self, cx: &mut Context<P>, dt: Duration) {
        for child in &mut self.children {
            child.element.animate(cx, dt);
        }
    }
}

struct GroupElements<'a, P>
where
    P: Platform,
{
    parent:   GroupParent<'a, P>,
    children: &'a mut Vec<Child<P>>,
}

impl<P> Elements<Context<P>, BoxedWidget<P>> for GroupElements<'_, P>
where
    P: Platform,
{
    fn index(&self) -> usize {
        self.parent.index
    }

    fn next(&mut self, _cx: &mut Context<P>) -> Option<Mut<'_, BoxedWidget<P>>> {
        let child = self.children.get_mut(self.parent.index)?;
        self.parent.index += 1;

        Some(WidgetMut::new(
            &mut self.parent,
            &mut child.element,
        ))
    }

    fn insert(&mut self, cx: &mut Context<P>, element: BoxedWidget<P>) {
        cx.layout.insert_child(
            self.parent.layout,
            self.parent.index,
            element.layout_node(),
        );

        self.parent.native.insert_child(
            &mut cx.platform,
            self.parent.index,
            element.widget_ref(),
        );

        let child = Child {
            element,
            allocation: None,
        };

        self.children.insert(self.parent.index, child);

        self.parent.index += 1;
    }

    fn remove(&mut self, cx: &mut Context<P>) -> Option<BoxedWidget<P>> {
        let child = self.children.remove(self.parent.index);

        (self.parent.native).remove_child(&mut cx.platform, self.parent.index);
        (cx.layout).remove_child(self.parent.layout, self.parent.index);

        Some(child.element)
    }

    fn swap(&mut self, cx: &mut Context<P>, offset: usize) {
        cx.layout.swap_children(
            self.parent.layout,
            self.parent.index,
            self.parent.index + offset,
        );

        self.parent.native.swap_children(
            &mut cx.platform,
            self.parent.index,
            self.parent.index + offset,
        );

        self.children.swap(
            self.parent.index,
            self.parent.index + offset,
        );
    }
}

impl<P> Element for GroupWidget<P>
where
    P: Platform,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

struct GroupParent<'a, P>
where
    P: Platform,
{
    index:  usize,
    native: &'a mut P::Group,
    layout: LayoutNode,
}

impl<'a, P> Parent<P> for GroupParent<'a, P>
where
    P: Platform,
{
    fn replace_child(&mut self, cx: &mut Context<P>, widget: P::WidgetRef, layout: LayoutNode) {
        // group parent is returned after the increment in next, so we need to undo that increment
        // to get the correct index
        let index = self.index - 1;

        self.native.replace_child(&mut cx.platform, index, widget);
        cx.layout.replace_child(self.layout, index, layout);
    }
}
