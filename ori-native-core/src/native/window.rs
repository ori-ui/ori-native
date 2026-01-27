use crate::Platform;

pub trait HasWindow: Platform {
    type Window: NativeWindow<Self>;
}

pub trait NativeWindow<P>
where
    P: Platform,
{
    fn build(platform: &mut P, contents: &P::Widget) -> Self;
    fn teardown(self, platform: &mut P);

    fn get_size(&self) -> (u32, u32);

    fn set_on_resize(&mut self, on_resize: impl Fn() + 'static);

    fn set_on_close_requested(&mut self, on_close_requested: impl Fn() + 'static);

    fn set_min_size(&mut self, width: u32, height: u32);
}
