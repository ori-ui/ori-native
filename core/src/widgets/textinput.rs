use std::time::Duration;

use ori::Element;

use crate::{
    CachedMeasurable, Context, Font, LayoutNode, LayoutStyle, Newline, Platform, TextAlign,
    TextWrap, Widget, native::NativeTextInput, widget::WidgetMut,
};

/// A [`Widget`] that handles text input.
pub struct TextInputWidget<P>
where
    P: Platform,
{
    native: P::TextInput,
    layout: LayoutNode,
}

impl<P> TextInputWidget<P>
where
    P: Platform,
{
    /// Create new [`TextInputWidget`].
    pub fn new(
        cx: &mut Context<P>,
        on_change: impl Fn(String) + 'static,
        on_submit: impl Fn(String) + 'static,
    ) -> Self {
        let native = P::TextInput::build(&mut cx.platform, on_change, on_submit);
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

    /// Set the current `text`.
    pub fn set_text(&mut self, cx: &mut Context<P>, text: String) {
        self.native.set_text(&mut cx.platform, text);
    }

    /// Set the placeholder `text`.
    pub fn set_placeholder_text(&mut self, cx: &mut Context<P>, text: String) {
        self.native.set_placeholder_text(&mut cx.platform, text);
    }

    /// Set the `font`.
    pub fn set_font(&mut self, cx: &mut Context<P>, font: Font, align: TextAlign, wrap: TextWrap) {
        self.native.set_font(&mut cx.platform, font, align, wrap);
    }

    /// Set the `font` of the placeholder text.
    pub fn set_placeholder_font(
        &mut self,
        cx: &mut Context<P>,
        font: Font,
        align: TextAlign,
        wrap: TextWrap,
    ) {
        (self.native).set_placeholder_font(&mut cx.platform, font, align, wrap);
    }

    /// Set the `newline` behaviour.
    pub fn set_newline(&mut self, cx: &mut Context<P>, newline: Newline) {
        self.native.set_newline(&mut cx.platform, newline);
    }

    /// Set whether to accept tabs.
    pub fn set_accept_tab(&mut self, cx: &mut Context<P>, accept_tab: bool) {
        self.native.set_accept_tab(&mut cx.platform, accept_tab);
    }

    /// Update the layout after changing text properties.
    pub fn update_layout(&mut self, cx: &mut Context<P>) {
        let measurable = self.native.get_measureable(&mut cx.platform);
        let cached = CachedMeasurable::new(measurable);
        cx.layout.set_measure(self.layout, cached);
    }
}

impl<P> Element for TextInputWidget<P>
where
    P: Platform,
{
    type Mut<'a>
        = WidgetMut<'a, P, Self>
    where
        Self: 'a;
}

impl<P> Widget<P> for TextInputWidget<P>
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
