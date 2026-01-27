use crate::{
    Context, Shadow, TextSpan,
    native::{HasText, NativeText},
};

pub struct TextShadow<P>
where
    P: HasText,
{
    text: P::Text,
}

impl<P> TextShadow<P>
where
    P: HasText,
{
    pub fn new(
        cx: &mut Context<P>,
        spans: Box<[TextSpan]>,
        text: String,
    ) -> (Self, <P::Text as NativeText<P>>::Leaf) {
        let (text, leaf) = P::Text::build(&mut cx.platform, spans, text);

        (Self { text }, leaf)
    }

    pub fn teardown(self, cx: &mut Context<P>) {
        self.text.teardown(&mut cx.platform);
    }

    pub fn set_text(
        &mut self,
        spans: Box<[TextSpan]>,
        text: String,
    ) -> <P::Text as NativeText<P>>::Leaf {
        self.text.set_text(spans, text)
    }
}

impl<P> Shadow<P> for TextShadow<P>
where
    P: HasText,
{
    fn widget(&self) -> &P::Widget {
        self.text.widget()
    }

    fn layout(&mut self, _cx: &mut Context<P>, _node: taffy::NodeId) {}
}
