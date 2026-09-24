use std::time::Duration;

use ori::Element;

use crate::{
    CachedMeasurable, Context, LayoutNode, LayoutStyle, Platform, TextAlign, TextSpan, TextWrap,
    Widget, native::NativeText, widget::WidgetMut,
};

/// A [`Widget`] that shows texts.
pub struct TextWidget<P>
where
    P: Platform,
{
    native: P::Text,
    layout: LayoutNode,
}

impl<P> TextWidget<P>
where
    P: Platform,
{
    /// Create new [`TextWidget`].
    pub fn new(cx: &mut Context<P>) -> Self {
        Self {
            native: P::Text::build(&mut cx.platform),
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

    /// Set the text.
    pub fn set_text(
        &mut self,
        cx: &mut Context<P>,
        spans: Box<[TextSpan]>,
        text: String,
        align: TextAlign,
        wrap: TextWrap,
    ) {
        let measurable = self.native.set_text(
            &mut cx.platform,
            spans,
            text,
            align,
            wrap,
        );

        let cached = CachedMeasurable::new(measurable);
        cx.layout.set_measure(self.layout, cached);
    }
}

impl<P> Widget<P> for TextWidget<P>
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
}

impl<P> Element for TextWidget<P>
where
    P: Platform,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}
