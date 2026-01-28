use crate::Platform;

pub trait HasPressable: Platform {
    type Pressable: NativePressable<Self>;
}

pub trait NativePressable<P>
where
    P: Platform,
{
    fn widget(&self) -> &P::Widget;

    fn build(plaform: &mut P, contents: &P::Widget) -> Self;
    fn teardown(self, plaform: &mut P);

    fn set_size(&mut self, width: f32, height: f32);
    fn set_on_press(&mut self, on_press: impl Fn(bool) + 'static);
    fn set_on_hover(&mut self, on_hover: impl Fn(bool) + 'static);
    fn set_on_focus(&mut self, on_focus: impl Fn(bool) + 'static);
}
