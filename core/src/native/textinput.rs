use std::convert::Infallible;

use crate::{
    Color, Font, Measurable, Newline, Platform, Submit, TextAlign, TextInputEvent, TextWrap,
    Unsupported, platform::unsupported,
};

/// A native text input widget.
pub trait NativeTextInput<P>
where
    P: Platform,
{
    /// Build a text input widget.
    fn build(platform: &mut P, on_event: impl Fn(TextInputEvent) + 'static) -> Self;

    /// Teardown the widget.
    fn teardown(self, platform: &mut P);

    /// Get a reference to the widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Set the `newline` behaviour.
    fn set_newline(&mut self, platform: &mut P, newline: Newline);

    /// Set the `submit` behaviour.
    fn set_submit(&mut self, platform: &mut P, submit: Submit);

    /// Set whether the text input accepts and inserts tabs.
    fn set_accept_tab(&mut self, platform: &mut P, accept_tab: bool);

    /// Set the `font` of the text.
    fn set_font(&mut self, platform: &mut P, font: Font, align: TextAlign, wrap: TextWrap);

    /// Set the `text`.
    fn set_text(&mut self, platform: &mut P, text: &str);

    /// Set the `font` of the placeholder text.
    fn set_placeholder_color(&mut self, platform: &mut P, color: Color);

    /// Set the placeholder `text`.
    fn set_placeholder_text(&mut self, platform: &mut P, text: &str);

    /// Get the [`Measurable`] that measures the minimum size of the input.
    fn get_measureable(&mut self, platform: &mut P) -> impl Measurable<P>;

    /// Request that the widget become focused.
    fn request_focus(&mut self, platform: &mut P);
}

impl<P> NativeTextInput<P> for Unsupported
where
    P: Platform,
{
    fn build(_platform: &mut P, _on_event: impl Fn(TextInputEvent) + 'static) -> Self {
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

    fn set_submit(&mut self, _platform: &mut P, _submit: Submit) {
        unreachable!()
    }

    fn set_accept_tab(&mut self, _platform: &mut P, _accept_tab: bool) {
        unreachable!()
    }

    fn request_focus(&mut self, _platform: &mut P) {
        unreachable!()
    }

    fn set_font(&mut self, _platform: &mut P, _font: Font, _align: TextAlign, _wrap: TextWrap) {
        unreachable!()
    }

    fn set_text(&mut self, _platform: &mut P, _text: &str) {
        unreachable!()
    }

    fn set_placeholder_color(&mut self, _platform: &mut P, _color: Color) {
        unreachable!()
    }

    fn set_placeholder_text(&mut self, _platform: &mut P, _text: &str) {
        unreachable!()
    }

    #[allow(refining_impl_trait)]
    fn get_measureable(&mut self, _platform: &mut P) -> Infallible {
        unreachable!()
    }
}
