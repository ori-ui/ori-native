use crate::{
    Context, Font, LayoutLeaf, Shadow,
    native::{HasTextInput, NativeTextInput},
    views::Newline,
};

pub struct TextInputShadow<P>
where
    P: HasTextInput,
{
    textinput: P::TextInput,
}

impl<P> TextInputShadow<P>
where
    P: HasTextInput,
{
    pub fn new(cx: &mut Context<P>) -> Self {
        let textinput = P::TextInput::build(&mut cx.platform);

        Self { textinput }
    }

    pub fn teardown(self, cx: &mut Context<P>) {
        self.textinput.teardown(&mut cx.platform);
    }

    pub fn set_on_change(&mut self, cx: &mut Context<P>, on_change: impl Fn(String) + 'static) {
        self.textinput.set_on_change(&mut cx.platform, on_change);
    }

    pub fn set_on_submit(&mut self, cx: &mut Context<P>, on_submit: impl Fn(String) + 'static) {
        self.textinput.set_on_submit(&mut cx.platform, on_submit);
    }

    pub fn set_newline(&mut self, cx: &mut Context<P>, newline: Newline) {
        self.textinput.set_newline(&mut cx.platform, newline);
    }

    pub fn set_accept_tab(&mut self, cx: &mut Context<P>, accept_tab: bool) {
        self.textinput.set_accept_tab(&mut cx.platform, accept_tab);
    }

    pub fn set_font(&mut self, cx: &mut Context<P>, font: Font) {
        self.textinput.set_font(&mut cx.platform, font);
    }

    pub fn set_text(&mut self, cx: &mut Context<P>, text: String) {
        self.textinput.set_text(&mut cx.platform, text);
    }

    pub fn set_placeholder_font(&mut self, cx: &mut Context<P>, font: Font) {
        self.textinput.set_placeholder_font(&mut cx.platform, font);
    }

    pub fn set_placeholder_text(&mut self, cx: &mut Context<P>, text: String) {
        self.textinput.set_placeholder_text(&mut cx.platform, text);
    }

    pub fn layout(&mut self, cx: &mut Context<P>) -> impl LayoutLeaf<P> + use<P> {
        self.textinput.get_layout(&mut cx.platform)
    }
}

impl<P> Shadow<P> for TextInputShadow<P>
where
    P: HasTextInput,
{
    fn widget(&self) -> &P::Widget {
        self.textinput.widget()
    }
}
