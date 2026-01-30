use crate::{Font, LayoutLeaf, Platform, views::Newline};

pub trait HasTextInput: Platform {
    type TextInput: NativeTextInput<Self>;
}

pub trait NativeTextInput<P>: Sized
where
    P: Platform,
{
    fn widget(&self) -> &P::Widget;

    fn build(platform: &mut P) -> Self;
    fn teardown(self, platform: &mut P);

    fn set_on_change(&mut self, platform: &mut P, on_change: impl Fn(String) + 'static);
    fn set_on_submit(&mut self, platform: &mut P, on_submit: impl Fn(String) + 'static);

    fn set_newline(&mut self, platform: &mut P, newline: Newline);
    fn set_accept_tab(&mut self, platform: &mut P, accept_tab: bool);

    fn set_font(&mut self, platform: &mut P, font: Font);
    fn set_text(&mut self, platform: &mut P, text: String);
    fn set_placeholder_font(&mut self, platform: &mut P, font: Font);
    fn set_placeholder_text(&mut self, platform: &mut P, text: String);

    fn get_layout(&mut self, platform: &mut P) -> impl LayoutLeaf<P> + use<Self, P>;
}
