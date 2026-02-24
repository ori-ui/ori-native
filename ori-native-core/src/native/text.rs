use crate::{LayoutLeaf, NativeWidget, Platform, TextSpan, Wrap};

pub trait HasText: Platform {
    type Text: NativeText<Self>;
}

pub trait NativeText<P>: NativeWidget<P> + Sized
where
    P: Platform,
{
    type Layout: LayoutLeaf<P>;

    fn build(
        platform: &mut P,
        spans: Box<[TextSpan]>,
        text: String,
        wrap: Wrap,
    ) -> (Self, Self::Layout);

    fn teardown(self, platform: &mut P);

    fn set_text(&mut self, spans: Box<[TextSpan]>, text: String, wrap: Wrap) -> Self::Layout;
}
