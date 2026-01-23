use ori::{BaseElement, Element};

pub trait NativeContext: BaseElement + Sized {
    type Window: NativeWindow<Self>;
    type Text: NativeText<Self>;
}

pub trait NativeWindow<C> {
    fn build(cx: &mut C) -> Self;
    fn teardown(self, cx: &mut C);
}

pub trait NativeText<C> {
    type Element: Element<C>;

    fn build(cx: &mut C, text: String) -> (Self::Element, Self);
    fn teardown(self, element: Self::Element, cx: &mut C);

    fn set_text(&mut self, element: Self::Element, cx: &mut C, text: String);
}
