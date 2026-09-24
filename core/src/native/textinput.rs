use std::convert::Infallible;

use crate::{
    Font, Measurable, Newline, Platform, TextAlign, TextWrap, Unsupported, platform::unsupported,
};

/// A native text input widget.
pub trait NativeTextInput<P>
where
    P: Platform,
{
    /// Build a text input widget.
    fn build(
        platform: &mut P,
        on_change: impl Fn(String) + 'static,
        on_submit: impl Fn(String) + 'static,
    ) -> Self;

    /// Teardown the widget.
    fn teardown(self, platform: &mut P);

    /// Get a reference to the widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Set the `newline` behaviour.
    fn set_newline(&mut self, platform: &mut P, newline: Newline);

    /// Set whether text input accepts and inserts tabs.
    fn set_accept_tab(&mut self, platform: &mut P, accept_tab: bool);

    /// Set the `font` of the text.
    fn set_font(&mut self, platform: &mut P, font: Font, align: TextAlign, wrap: TextWrap);

    /// Set the `text`.
    fn set_text(&mut self, platform: &mut P, text: String);

    /// Set the `font` of the placeholder text.
    fn set_placeholder_font(
        &mut self,
        platform: &mut P,
        font: Font,
        align: TextAlign,
        wrap: TextWrap,
    );

    /// Set the placeholder `text`.
    fn set_placeholder_text(&mut self, platform: &mut P, text: String);

    /// Get the [`Measurable`] that measures the minimum size of the input.
    fn get_measureable(&mut self, platform: &mut P) -> impl Measurable<P>;
}

impl<P> NativeTextInput<P> for Unsupported
where
    P: Platform,
{
    fn build(
        _platform: &mut P,
        _on_change: impl Fn(String) + 'static,
        _on_submit: impl Fn(String) + 'static,
    ) -> Self {
        unsupported!("text input view")
    }

    fn teardown(self, _platform: &mut P) {
        unreachable!()
    }

    fn widget_ref(&self) -> P::WidgetRef {
        unreachable!()
    }

    fn set_newline(&mut self, _platform: &mut P, _newline: Newline) {
        unreachable!()
    }

    fn set_accept_tab(&mut self, _platform: &mut P, _accept_tab: bool) {
        unreachable!()
    }

    fn set_font(&mut self, _platform: &mut P, _font: Font, _align: TextAlign, _wrap: TextWrap) {
        unreachable!()
    }

    fn set_text(&mut self, _platform: &mut P, _text: String) {
        unreachable!()
    }

    fn set_placeholder_font(
        &mut self,
        _platform: &mut P,
        _font: Font,
        _align: TextAlign,
        _wrap: TextWrap,
    ) {
        unreachable!()
    }

    fn set_placeholder_text(&mut self, _platform: &mut P, _text: String) {
        unreachable!()
    }

    #[allow(refining_impl_trait)]
    fn get_measureable(&mut self, _platform: &mut P) -> Infallible {
        unreachable!()
    }
}
