use std::{borrow::Cow, time::Duration};

use ori::{Element, ViewId};

use crate::{
    Color, Context, LayoutNode, LayoutStyle, Platform, Widget, WidgetMut, native::NativeImage,
};

/// A [`Widget`] representing an image.
pub struct ImageWidget<P>
where
    P: Platform,
{
    native: P::Image,
    layout: LayoutNode,
}

impl<P> ImageWidget<P>
where
    P: Platform,
{
    /// Create new [`ImageWidget`].
    pub fn new(cx: &mut Context<P>) -> Self {
        let native = P::Image::build(&mut cx.platform);
        let layout = cx.layout.add_node(&[]);

        Self { native, layout }
    }

    /// Teardown the widget.
    pub fn teardown(self, cx: &mut Context<P>) {
        self.native.teardown(&mut cx.platform);
        cx.layout.remove_node(self.layout);
    }

    /// Set the [`LayoutStyle`].
    pub fn set_layout(&mut self, cx: &mut Context<P>, layout: LayoutStyle) {
        cx.layout.set_layout(self.layout, layout);
    }

    /// Try loading an image from memory.
    pub fn load_data(
        &mut self,
        cx: &mut Context<P>,
        data: Cow<'static, [u8]>,
    ) -> Result<(), <P::Image as NativeImage<P>>::Error> {
        let measure = self.native.load_data(&mut cx.platform, data)?;
        cx.layout.set_measure(self.layout, measure);

        Ok(())
    }

    /// Set the `tint` of the image.
    pub fn set_tint(&mut self, cx: &mut Context<P>, tint: Option<Color>) {
        self.native.set_tint(&mut cx.platform, tint);
    }
}

impl<P> Element for ImageWidget<P>
where
    P: Platform,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P> Widget<P> for ImageWidget<P>
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
